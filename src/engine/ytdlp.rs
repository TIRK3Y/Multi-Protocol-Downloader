use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver as WatchReceiver;

use crate::config::AppConfig;
use crate::engine::installer::EngineInstaller;
use crate::engine::models::{DownloadProgress, DownloadStatus, MediaInfo, QualityPreset};

#[derive(Debug, Clone)]
pub struct YtDlpEngine {
    pub executable_path: Option<PathBuf>,
    pub ffmpeg_path: Option<PathBuf>,
    pub version: Option<String>,
    pub ffmpeg_version: Option<String>,
}

impl YtDlpEngine {
    pub fn detect(config: &AppConfig) -> Self {
        let mut ytdlp = None;

        // 1. Check custom path in config
        if let Some(ref path) = config.custom_ytdlp_path {
            if path.exists() && path.is_file() {
                ytdlp = Some(path.clone());
            }
        }

        // 2. Check local data bin dir
        if ytdlp.is_none() {
            if let Ok(local_path) = EngineInstaller::target_bin_path() {
                if local_path.exists() && local_path.is_file() {
                    ytdlp = Some(local_path);
                }
            }
        }

        // 3. Check system PATH
        if ytdlp.is_none() {
            let cmd_name = if cfg!(target_os = "windows") {
                "yt-dlp.exe"
            } else {
                "yt-dlp"
            };

            if let Ok(output) = std::process::Command::new(cmd_name)
                .arg("--version")
                .output()
            {
                if output.status.success() {
                    ytdlp = Some(PathBuf::from(cmd_name));
                }
            }
        }

        // Check version if found
        let version = if let Some(ref bin) = ytdlp {
            std::process::Command::new(bin)
                .arg("--version")
                .output()
                .ok()
                .and_then(|out| {
                    if out.status.success() {
                        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
        } else {
            None
        };

        // Detect ffmpeg
        let mut ffmpeg = None;
        if let Some(ref path) = config.custom_ffmpeg_path {
            if path.exists() && path.is_file() {
                ffmpeg = Some(path.clone());
            }
        }

        // Local bin directory for ffmpeg
        if ffmpeg.is_none() {
            if let Ok(local_ffmpeg) = EngineInstaller::target_ffmpeg_path() {
                if local_ffmpeg.exists() && local_ffmpeg.is_file() {
                    ffmpeg = Some(local_ffmpeg);
                }
            }
        }

        // System PATH for ffmpeg
        if ffmpeg.is_none() {
            let ffmpeg_cmd = if cfg!(target_os = "windows") {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            };
            if let Ok(output) = std::process::Command::new(ffmpeg_cmd)
                .arg("-version")
                .output()
            {
                if output.status.success() {
                    ffmpeg = Some(PathBuf::from(ffmpeg_cmd));
                }
            }
        }

        let ffmpeg_version = if let Some(ref bin) = ffmpeg {
            std::process::Command::new(bin)
                .arg("-version")
                .output()
                .ok()
                .and_then(|out| {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        text.lines().next().map(|line| {
                            line.replace("ffmpeg version", "")
                                .split_whitespace()
                                .next()
                                .unwrap_or("Installed")
                                .to_string()
                        })
                    } else {
                        None
                    }
                })
        } else {
            None
        };

        Self {
            executable_path: ytdlp,
            ffmpeg_path: ffmpeg,
            version,
            ffmpeg_version,
        }
    }

    pub fn is_available(&self) -> bool {
        self.executable_path.is_some()
    }

    pub fn is_ffmpeg_available(&self) -> bool {
        self.ffmpeg_path.is_some()
    }

    pub async fn fetch_info(&self, url: &str) -> Result<MediaInfo, String> {
        let exe = self
            .executable_path
            .as_ref()
            .ok_or_else(|| "yt-dlp engine is not installed or detected.".to_string())?;

        let mut cmd = Command::new(exe);
        cmd.arg("--dump-single-json")
            .arg("--no-warnings")
            .arg("--no-playlist")
            .arg(url);

        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to execute yt-dlp: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("yt-dlp error: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: MediaInfo = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse video metadata: {e}"))?;

        Ok(info)
    }

    pub async fn download(
        &self,
        url: &str,
        preset: QualityPreset,
        custom_format_id: Option<&str>,
        output_dir: &Path,
        extract_audio: bool,
        embed_thumbnail: bool,
        embed_metadata: bool,
        status_sender: UnboundedSender<DownloadStatus>,
        mut cancel_receiver: WatchReceiver<bool>,
    ) -> Result<PathBuf, String> {
        let exe = self
            .executable_path
            .as_ref()
            .ok_or_else(|| "yt-dlp engine is not installed.".to_string())?;

        let mut cmd = Command::new(exe);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        // Format selection
        if let Some(format_id) = custom_format_id {
            if !format_id.is_empty() {
                cmd.arg("-f").arg(format_id);
            }
        } else {
            let format_arg = preset.format_arg();
            if !format_arg.is_empty() {
                cmd.arg("-f").arg(format_arg);
            }
        }

        // Audio extraction option
        if extract_audio || preset == QualityPreset::BestAudio {
            cmd.arg("-x").arg("--audio-format").arg("mp3");
        }

        // Embed options (if ffmpeg is available)
        if embed_thumbnail {
            cmd.arg("--embed-thumbnail");
        }
        if embed_metadata {
            cmd.arg("--embed-metadata");
        }

        if let Some(ref ffmpeg) = self.ffmpeg_path {
            cmd.arg("--ffmpeg-location").arg(ffmpeg);
        } else if let Ok(local_ffmpeg) = EngineInstaller::target_ffmpeg_path() {
            if local_ffmpeg.exists() {
                cmd.arg("--ffmpeg-location").arg(local_ffmpeg);
            }
        }

        // Standardized output path
        let out_template = output_dir.join("%(title)s.%(ext)s");
        cmd.arg("-o")
            .arg(out_template.to_string_lossy().to_string());

        // Print final filename after download
        cmd.arg("--print").arg("after_move:filepath");

        // Progress formatting
        cmd.arg("--newline");
        cmd.arg("--progress-template")
            .arg("DOWNLOAD_PROGRESS:%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(progress._downloaded_bytes_str)s|%(progress._total_bytes_str)s");

        cmd.arg(url);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start download process: {e}"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut final_file_path: Option<PathBuf> = None;
        let mut progress = DownloadProgress::default();

        let progress_regex = Regex::new(
            r"DOWNLOAD_PROGRESS:(?P<pct>[^|]+)\|(?P<speed>[^|]+)\|(?P<eta>[^|]+)\|(?P<dl>[^|]+)\|(?P<total>[^|]+)",
        )
        .unwrap();

        loop {
            tokio::select! {
                _ = cancel_receiver.changed() => {
                    if *cancel_receiver.borrow() {
                        let _ = child.kill().await;
                        let _ = status_sender.send(DownloadStatus::Cancelled);
                        return Err("Download cancelled by user".to_string());
                    }
                }
                line_res = stdout_reader.next_line() => {
                    match line_res {
                        Ok(Some(line)) => {
                            let trimmed = line.trim();
                            if let Some(caps) = progress_regex.captures(trimmed) {
                                let pct_str = caps.name("pct").map(|m| m.as_str()).unwrap_or("0%").trim().trim_end_matches('%');
                                let pct = pct_str.parse::<f32>().unwrap_or(0.0);
                                progress.percent = pct;
                                progress.speed = caps.name("speed").map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                progress.eta = caps.name("eta").map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                progress.downloaded = caps.name("dl").map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                progress.total = caps.name("total").map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                progress.status_message = format!("Downloading: {}% ({})", pct, progress.speed);

                                let _ = status_sender.send(DownloadStatus::Downloading(progress.clone()));
                            } else if trimmed.starts_with("[Merger]") || trimmed.starts_with("[ExtractAudio]") || trimmed.starts_with("[Fixup") {
                                let _ = status_sender.send(DownloadStatus::PostProcessing(trimmed.to_string()));
                            } else if Path::new(trimmed).is_absolute() || (trimmed.contains('.') && !trimmed.starts_with('[')) {
                                // Potential final file path printed by --print after_move:filepath
                                let p = PathBuf::from(trimmed);
                                if p.exists() || p.parent().map(|d| d.exists()).unwrap_or(false) {
                                    final_file_path = Some(p);
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                err_line = stderr_reader.next_line() => {
                    match err_line {
                        Ok(Some(line)) => {
                            let trimmed = line.trim();
                            if trimmed.contains("ERROR:") {
                                eprintln!("yt-dlp stderr: {}", trimmed);
                            }
                        }
                        Ok(None) => {},
                        Err(_) => {},
                    }
                }
            }
        }

        let exit_status = child
            .wait()
            .await
            .map_err(|e| format!("Process error: {e}"))?;

        if !exit_status.success() {
            let msg = format!("Download failed with exit code: {:?}", exit_status.code());
            let _ = status_sender.send(DownloadStatus::Failed(msg.clone()));
            return Err(msg);
        }

        let path = final_file_path.unwrap_or_else(|| output_dir.to_path_buf());
        let _ = status_sender.send(DownloadStatus::Completed(path.clone()));
        Ok(path)
    }
}
