use librqbit::api::TorrentIdOrHash;
use librqbit::{AddTorrent, AddTorrentResponse, Session, SessionOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;

#[derive(Debug, Clone, Default)]
pub struct TorrentProgress {
    pub name: String,
    pub percent: f32,
    pub download_speed: String,
    pub upload_speed: String,
    pub downloaded_bytes: String,
    pub total_bytes: String,
    pub peers: usize,
    pub state_message: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TorrentStatus {
    Starting,
    Downloading(TorrentProgress),
    Completed(PathBuf),
    Failed(String),
    Cancelled,
}

pub struct TorrentManager;

impl TorrentManager {
    pub fn format_bytes(bytes: u64) -> String {
        let mb = bytes as f64 / 1_048_576.0;
        if mb >= 1024.0 {
            format!("{:.2} GB", mb / 1024.0)
        } else {
            format!("{:.1} MB", mb)
        }
    }

    pub fn normalize_magnet(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.starts_with("magnet:?") {
            trimmed.to_string()
        } else if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            format!("magnet:?xt=urn:btih:{}", trimmed)
        } else if trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
            format!("magnet:?xt=urn:btih:{}", trimmed)
        } else {
            trimmed.to_string()
        }
    }

    pub async fn start_magnet(
        magnet_url: &str,
        output_dir: &Path,
        status_sender: UnboundedSender<TorrentStatus>,
        mut cancel_rx: watch::Receiver<bool>,
    ) -> Result<PathBuf, String> {
        let magnet = Self::normalize_magnet(magnet_url);
        let out_dir_buf = output_dir.to_path_buf();

        let _ = status_sender.send(TorrentStatus::Starting);

        let session_opts = SessionOptions::default();
        let session = Session::new_with_opts(out_dir_buf.clone(), session_opts)
            .await
            .map_err(|e| format!("Failed to initialize BitTorrent session: {e}"))?;

        let add_res = session
            .add_torrent(AddTorrent::from_url(&magnet), None)
            .await
            .map_err(|e| format!("Failed to resolve magnet metadata: {e}"))?;

        let (torrent_id, handle) = match add_res {
            AddTorrentResponse::Added(id, h) => (id, h),
            AddTorrentResponse::AlreadyManaged(id, h) => (id, h),
            AddTorrentResponse::ListOnly(_) => {
                return Err("Magnet resolved to list-only mode".to_string());
            }
        };

        let mut ticker = tokio::time::interval(Duration::from_millis(500));
        let mut final_path = out_dir_buf.clone();

        loop {
            tokio::select! {
                _ = cancel_rx.changed() => {
                    if *cancel_rx.borrow() {
                        let _ = session.delete(TorrentIdOrHash::Id(torrent_id), false).await;
                        let _ = status_sender.send(TorrentStatus::Cancelled);
                        return Err("Torrent download cancelled by user.".to_string());
                    }
                }
                _ = ticker.tick() => {
                    let stats = handle.stats();

                    let (dl_speed_str, ul_speed_str, peers) = if let Some(ref live) = stats.live {
                        let dl = live.download_speed.to_string();
                        let ul = live.upload_speed.to_string();
                        let p = live.snapshot.peer_stats.live as usize;
                        (dl, ul, p)
                    } else {
                        ("-".to_string(), "-".to_string(), 0)
                    };

                    let prog_bytes = stats.progress_bytes;
                    let total_bytes = stats.total_bytes;

                    let percent = if total_bytes > 0 {
                        ((prog_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32
                    } else {
                        0.0
                    };

                    let torrent_name = handle.name().unwrap_or_else(|| "Resolving metadata...".to_string());
                    if torrent_name != "Resolving metadata..." {
                        final_path = out_dir_buf.join(&torrent_name);
                    }

                    let state_msg = if stats.finished {
                        "Download complete, verifying files...".to_string()
                    } else if peers == 0 && prog_bytes == 0 {
                        "Connecting to DHT network & discovering peers...".to_string()
                    } else {
                        format!("Downloading from {} peers ({})", peers, dl_speed_str)
                    };

                    let progress = TorrentProgress {
                        name: torrent_name,
                        percent,
                        download_speed: dl_speed_str,
                        upload_speed: ul_speed_str,
                        downloaded_bytes: Self::format_bytes(prog_bytes),
                        total_bytes: Self::format_bytes(total_bytes),
                        peers,
                        state_message: state_msg,
                    };

                    let _ = status_sender.send(TorrentStatus::Downloading(progress));

                    if stats.finished || (total_bytes > 0 && prog_bytes >= total_bytes) {
                        let _ = status_sender.send(TorrentStatus::Completed(final_path.clone()));
                        return Ok(final_path);
                    }
                }
            }
        }
    }
}
