use eframe::egui;
use std::path::PathBuf;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::sync::watch;

use crate::config::AppConfig;
use crate::engine::installer::EngineInstaller;
use crate::engine::models::{
    DownloadProgress, DownloadStatus, FormatItem, HistoryRecord, MediaInfo, QualityPreset,
};
use crate::engine::torrent::{TorrentManager, TorrentProgress, TorrentStatus};
use crate::engine::ytdlp::YtDlpEngine;
use crate::ui::icons::{self, Icon};

pub struct DownloadView {
    // Web Media state
    pub url_input: String,
    pub is_fetching_info: bool,
    pub fetch_error: Option<String>,
    pub media_info: Option<MediaInfo>,

    pub selected_preset: QualityPreset,
    pub selected_format_id: Option<String>,
    pub format_filter_tab: usize, // 0: All, 1: Video+Audio, 2: Audio Only, 3: Video Only

    pub is_downloading: bool,
    pub current_progress: DownloadProgress,
    pub download_error: Option<String>,
    pub completed_file: Option<PathBuf>,

    download_status_rx: Option<UnboundedReceiver<DownloadStatus>>,
    cancel_tx: Option<watch::Sender<bool>>,
    info_rx: Option<tokio::sync::oneshot::Receiver<Result<MediaInfo, String>>>,

    // FFmpeg 1-click install state
    pub is_installing_ffmpeg: bool,
    pub ffmpeg_install_progress: f32,
    pub ffmpeg_install_status: String,
    ffmpeg_rx: Option<mpsc::UnboundedReceiver<(f32, String)>>,
    ffmpeg_done_rx: Option<tokio::sync::oneshot::Receiver<Result<PathBuf, String>>>,

    // Magnet / Torrent state
    pub magnet_input: String,
    pub is_torrent_downloading: bool,
    pub torrent_progress: TorrentProgress,
    pub torrent_error: Option<String>,
    pub torrent_completed_path: Option<PathBuf>,

    torrent_status_rx: Option<UnboundedReceiver<TorrentStatus>>,
    torrent_cancel_tx: Option<watch::Sender<bool>>,
}

impl Default for DownloadView {
    fn default() -> Self {
        Self {
            url_input: String::new(),
            is_fetching_info: false,
            fetch_error: None,
            media_info: None,

            selected_preset: QualityPreset::BestVideoAudio,
            selected_format_id: None,
            format_filter_tab: 0,

            is_downloading: false,
            current_progress: DownloadProgress::default(),
            download_error: None,
            completed_file: None,

            download_status_rx: None,
            cancel_tx: None,
            info_rx: None,

            is_installing_ffmpeg: false,
            ffmpeg_install_progress: 0.0,
            ffmpeg_install_status: String::new(),
            ffmpeg_rx: None,
            ffmpeg_done_rx: None,

            magnet_input: String::new(),
            is_torrent_downloading: false,
            torrent_progress: TorrentProgress::default(),
            torrent_error: None,
            torrent_completed_path: None,

            torrent_status_rx: None,
            torrent_cancel_tx: None,
        }
    }
}

impl DownloadView {
    pub fn update_async_channels(&mut self, config: &mut AppConfig, engine: &mut YtDlpEngine) {
        // Poll FFmpeg 1-click install
        if let Some(ref mut rx) = self.ffmpeg_rx {
            while let Ok((prog, status)) = rx.try_recv() {
                self.ffmpeg_install_progress = prog;
                self.ffmpeg_install_status = status;
            }
        }

        if let Some(ref mut done_rx) = self.ffmpeg_done_rx {
            if let Ok(res) = done_rx.try_recv() {
                self.is_installing_ffmpeg = false;
                match res {
                    Ok(path) => {
                        self.ffmpeg_install_status = format!("FFmpeg installed: {}", path.display());
                        *engine = YtDlpEngine::detect(config);
                        if let Some(ref err) = self.download_error {
                            if err.contains("ffmpeg not found") || err.contains("Postprocessing") {
                                self.download_error = None;
                            }
                        }
                    }
                    Err(e) => {
                        self.ffmpeg_install_status = format!("Installation failed: {e}");
                    }
                }
                self.ffmpeg_done_rx = None;
                self.ffmpeg_rx = None;
            }
        }

        // Poll metadata fetch
        if let Some(ref mut rx) = self.info_rx {
            if let Ok(res) = rx.try_recv() {
                self.is_fetching_info = false;
                match res {
                    Ok(info) => {
                        self.selected_format_id = info.formats.first().map(|f| f.format_id.clone());
                        self.media_info = Some(info);
                        self.fetch_error = None;
                    }
                    Err(e) => {
                        self.fetch_error = Some(e);
                        self.media_info = None;
                    }
                }
                self.info_rx = None;
            }
        }

        // Poll web stream download status
        if let Some(ref mut rx) = self.download_status_rx {
            while let Ok(status) = rx.try_recv() {
                match status {
                    DownloadStatus::Idle => {}
                    DownloadStatus::Extracting => {
                        self.current_progress.status_message = "Extracting media stream...".to_string();
                    }
                    DownloadStatus::Downloading(prog) => {
                        self.current_progress = prog;
                    }
                    DownloadStatus::PostProcessing(msg) => {
                        self.current_progress.status_message = format!("Processing: {msg}");
                    }
                    DownloadStatus::Completed(path) => {
                        self.is_downloading = false;
                        self.completed_file = Some(path.clone());
                        self.download_error = None;

                        let title = self
                            .media_info
                            .as_ref()
                            .map(|m| m.title.clone())
                            .unwrap_or_else(|| "Media Download".to_string());

                        let now = chrono::Local::now();
                        let record = HistoryRecord {
                            id: format!("dl_{}", now.timestamp()),
                            title,
                            url: self.url_input.clone(),
                            format_desc: self.selected_preset.label().to_string(),
                            file_path: Some(path),
                            timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                            status: "Completed".to_string(),
                        };
                        config.add_history(record);
                    }
                    DownloadStatus::Failed(err) => {
                        self.is_downloading = false;
                        self.download_error = Some(err);
                    }
                    DownloadStatus::Cancelled => {
                        self.is_downloading = false;
                        self.download_error = Some("Download was cancelled.".to_string());
                    }
                }
            }
        }

        // Poll torrent download status
        if let Some(ref mut rx) = self.torrent_status_rx {
            while let Ok(status) = rx.try_recv() {
                match status {
                    TorrentStatus::Starting => {
                        self.torrent_progress.state_message = "Starting DHT session...".to_string();
                    }
                    TorrentStatus::Downloading(prog) => {
                        self.torrent_progress = prog;
                    }
                    TorrentStatus::Completed(path) => {
                        self.is_torrent_downloading = false;
                        self.torrent_completed_path = Some(path.clone());
                        self.torrent_error = None;

                        let now = chrono::Local::now();
                        let record = HistoryRecord {
                            id: format!("torrent_{}", now.timestamp()),
                            title: self.torrent_progress.name.clone(),
                            url: self.magnet_input.clone(),
                            format_desc: "BitTorrent / Magnet".to_string(),
                            file_path: Some(path),
                            timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                            status: "Completed".to_string(),
                        };
                        config.add_history(record);
                    }
                    TorrentStatus::Failed(err) => {
                        self.is_torrent_downloading = false;
                        self.torrent_error = Some(err);
                    }
                    TorrentStatus::Cancelled => {
                        self.is_torrent_downloading = false;
                        self.torrent_error = Some("Torrent download was cancelled.".to_string());
                    }
                }
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        engine: &mut YtDlpEngine,
        runtime: &tokio::runtime::Handle,
    ) {
        self.update_async_channels(config, engine);

        ui.vertical(|ui| {
            // Section 1: Video & Media Stream Downloader Card
            self.show_media_stream_card(ui, config, engine, runtime);

            ui.add_space(18.0);

            // Section 2: BitTorrent & Magnet Downloader Card
            self.show_magnet_torrent_card(ui, config, runtime);
        });
    }

    fn show_media_stream_card(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        engine: &mut YtDlpEngine,
        runtime: &tokio::runtime::Handle,
    ) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                // Section Header
                ui.horizontal(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                    icons::paint_icon(
                        ui.painter(),
                        icon_rect,
                        Icon::Video,
                        egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(41, 128, 185)),
                    );
                    ui.add_space(6.0);
                    ui.heading("Video & Media Stream Downloader");
                });

                ui.add_space(2.0);
                ui.label(
                    "Download video, audio, and playlists from YouTube, Vimeo, Twitter/X, TikTok, Twitch, SoundCloud, and 1,000+ sites."
                );
                ui.add_space(12.0);

                // URL Input Row
                ui.horizontal(|ui| {
                    let input_resp = ui.add_sized(
                        [ui.available_width() - 250.0, 30.0],
                        egui::TextEdit::singleline(&mut self.url_input)
                            .hint_text("Enter video, audio, or playlist URL (e.g. https://www.youtube.com/watch?v=...)..."),
                    );

                    if ui.button("Paste").clicked() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                self.url_input = text.trim().to_string();
                            }
                        }
                    }

                    if ui.button("Clear").clicked() {
                        self.url_input.clear();
                        self.media_info = None;
                        self.fetch_error = None;
                    }

                    let can_fetch = !self.url_input.trim().is_empty() && !self.is_fetching_info && engine.is_available();
                    let fetch_btn = ui.add_enabled(
                        can_fetch,
                        egui::Button::new(
                            egui::RichText::new("Inspect Formats")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(41, 128, 185)),
                    );

                    if (fetch_btn.clicked() || (input_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && can_fetch {
                        self.is_fetching_info = true;
                        self.fetch_error = None;
                        self.media_info = None;
                        self.completed_file = None;

                        let url = self.url_input.trim().to_string();
                        let engine_clone = engine.clone();
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        self.info_rx = Some(rx);

                        runtime.spawn(async move {
                            let res = engine_clone.fetch_info(&url).await;
                            let _ = tx.send(res);
                        });
                    }
                });

                if self.is_fetching_info {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("Inspecting media streams & available formats...")
                                .color(egui::Color32::from_rgb(52, 152, 219)),
                        );
                    });
                }

                if let Some(ref err) = self.fetch_error {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("[Notice]").color(egui::Color32::RED).strong());
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    });
                }

                ui.add_space(10.0);

                // Media Details Card (when metadata fetched)
                if let Some(ref info) = self.media_info {
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if let Some(ref thumb_url) = info.thumbnail {
                                    ui.add(
                                        egui::Image::new(thumb_url)
                                            .max_width(180.0)
                                            .max_height(100.0)
                                            .corner_radius(4.0),
                                    );
                                }

                                ui.vertical(|ui| {
                                    ui.strong(&info.title);
                                    ui.add_space(3.0);
                                    ui.horizontal(|ui| {
                                        if let Some(ref uploader) = info.uploader {
                                            ui.label(format!("By: {uploader}"));
                                        }
                                        if let Some(dur) = info.duration {
                                            let mins = (dur as u64) / 60;
                                            let secs = (dur as u64) % 60;
                                            ui.label("|");
                                            ui.label(format!("Duration: {mins}:{secs:02}"));
                                        }
                                    });
                                });
                            });

                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(8.0);

                            // Format Inspector Table
                            ui.strong("Available Streams and Quality Options:");
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                let filter_labels = ["All Formats", "Video + Audio", "Audio Only", "Video Only"];
                                for (idx, label) in filter_labels.iter().enumerate() {
                                    if ui.selectable_label(self.format_filter_tab == idx, *label).clicked() {
                                        self.format_filter_tab = idx;
                                    }
                                }
                            });

                            ui.add_space(6.0);

                            let filtered_formats: Vec<&FormatItem> = info
                                .formats
                                .iter()
                                .filter(|f| match self.format_filter_tab {
                                    1 => f.is_video_and_audio(),
                                    2 => f.is_audio_only(),
                                    3 => f.is_video_only(),
                                    _ => true,
                                })
                                .collect();

                            egui::ScrollArea::vertical()
                                .max_height(180.0)
                                .id_salt("media_stream_formats_scroll")
                                .show(ui, |ui| {
                                    egui::Grid::new("stream_formats_grid")
                                        .num_columns(6)
                                        .spacing([14.0, 6.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            ui.strong("Select");
                                            ui.strong("ID");
                                            ui.strong("Format");
                                            ui.strong("Resolution");
                                            ui.strong("Codecs");
                                            ui.strong("Size");
                                            ui.end_row();

                                            for fmt in filtered_formats {
                                                let is_selected = self.selected_format_id.as_deref() == Some(&fmt.format_id);
                                                if ui.radio(is_selected, "").clicked() {
                                                    self.selected_format_id = Some(fmt.format_id.clone());
                                                    self.selected_preset = QualityPreset::Custom;
                                                }

                                                ui.label(&fmt.format_id);
                                                ui.label(&fmt.ext);
                                                ui.label(fmt.resolution_str());
                                                ui.label(format!("{}/{}", fmt.vcodec.as_deref().unwrap_or("none"), fmt.acodec.as_deref().unwrap_or("none")));
                                                ui.label(fmt.size_str());
                                                ui.end_row();
                                            }
                                        });
                                });
                        });

                    ui.add_space(10.0);
                }

                // Preset and Download Controls
                ui.horizontal(|ui| {
                    ui.label("Quality Preset:");
                    egui::ComboBox::from_id_salt("media_stream_preset_dropdown")
                        .selected_text(self.selected_preset.label())
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            let presets = [
                                QualityPreset::BestVideoAudio,
                                QualityPreset::Fhd1080p,
                                QualityPreset::Hd720p,
                                QualityPreset::Sd480p,
                                QualityPreset::BestAudio,
                            ];
                            for p in presets {
                                if ui.selectable_value(&mut self.selected_preset, p, p.label()).clicked() {
                                    self.selected_format_id = None;
                                }
                            }
                        });

                    let can_download = !self.url_input.trim().is_empty()
                        && !self.is_downloading
                        && engine.is_available();

                    let dl_btn = ui.add_enabled(
                        can_download,
                        egui::Button::new(
                            egui::RichText::new("Download Media")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(46, 204, 113))
                        .min_size(egui::vec2(140.0, 28.0)),
                    );

                    if dl_btn.clicked() {
                        self.start_media_download(config, engine, runtime);
                    }
                });

                // Download Progress Card
                if self.is_downloading {
                    ui.add_space(12.0);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong("Downloading Stream...");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new("Cancel").color(egui::Color32::RED)).clicked() {
                                        if let Some(ref tx) = self.cancel_tx {
                                            let _ = tx.send(true);
                                        }
                                    }
                                });
                            });

                            ui.add_space(6.0);
                            let frac = (self.current_progress.percent / 100.0).clamp(0.0, 1.0);
                            ui.add(
                                egui::ProgressBar::new(frac)
                                    .text(format!("{:.1}%", self.current_progress.percent))
                                    .animate(true),
                            );

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if !self.current_progress.speed.is_empty() {
                                    ui.label(format!("Speed: {}", self.current_progress.speed));
                                    ui.label("|");
                                }
                                if !self.current_progress.eta.is_empty() {
                                    ui.label(format!("ETA: {}", self.current_progress.eta));
                                    ui.label("|");
                                }
                                if !self.current_progress.downloaded.is_empty() {
                                    ui.label(format!(
                                        "Transferred: {} / {}",
                                        self.current_progress.downloaded, self.current_progress.total
                                    ));
                                }
                            });

                            if !self.current_progress.status_message.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.current_progress.status_message)
                                        .size(11.5)
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                            }
                        });
                }

                // Completed Banner
                if let Some(ref file_path) = self.completed_file {
                    ui.add_space(10.0);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("[Success] Download Finished.")
                                        .color(egui::Color32::from_rgb(46, 204, 113))
                                        .strong(),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Open Folder").clicked() {
                                        if let Some(parent) = file_path.parent() {
                                            let _ = open::that(parent);
                                        }
                                    }
                                    if file_path.is_file() {
                                        if ui.button("Play / Open File").clicked() {
                                            let _ = open::that(file_path);
                                        }
                                    }
                                });
                            });
                            ui.label(format!("Saved: {}", file_path.display()));
                        });
                }

                // Error Banner with FFmpeg 1-Click Resolver
                if let Some(ref err) = self.download_error {
                    ui.add_space(10.0);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("[Error]").color(egui::Color32::RED).strong());
                                ui.label(err);
                            });

                            if err.contains("ffmpeg not found") || err.contains("Postprocessing") || err.contains("--ffmpeg-location") {
                                ui.add_space(6.0);
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("Fix: FFmpeg is required to merge audio and video or extract MP3s.")
                                            .color(egui::Color32::LIGHT_BLUE),
                                    );

                                    if self.is_installing_ffmpeg {
                                        ui.add(egui::ProgressBar::new(self.ffmpeg_install_progress).show_percentage());
                                        ui.label(&self.ffmpeg_install_status);
                                    } else {
                                        let fix_btn = egui::Button::new(
                                            egui::RichText::new("Install FFmpeg Now (1-Click)")
                                                .strong()
                                                .color(egui::Color32::WHITE),
                                        )
                                        .fill(egui::Color32::from_rgb(142, 68, 173));

                                        if ui.add(fix_btn).clicked() {
                                            self.is_installing_ffmpeg = true;
                                            self.ffmpeg_install_progress = 0.05;
                                            self.ffmpeg_install_status = "Connecting to static releases...".to_string();

                                            let (tx, rx) = mpsc::unbounded_channel();
                                            let (done_tx, done_rx) = tokio::sync::oneshot::channel();

                                            self.ffmpeg_rx = Some(rx);
                                            self.ffmpeg_done_rx = Some(done_rx);

                                            runtime.spawn(async move {
                                                let res = EngineInstaller::install_ffmpeg(move |frac, msg| {
                                                    let _ = tx.send((frac, msg.to_string()));
                                                }).await;
                                                let _ = done_tx.send(res);
                                            });
                                        }
                                    }
                                });
                            }
                        });
                }
            });
    }

    fn show_magnet_torrent_card(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        runtime: &tokio::runtime::Handle,
    ) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                // Section Header
                ui.horizontal(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                    icons::paint_icon(
                        ui.painter(),
                        icon_rect,
                        Icon::Magnet,
                        egui::Stroke::new(1.8_f32, egui::Color32::from_rgb(142, 68, 173)),
                    );
                    ui.add_space(6.0);
                    ui.heading("BitTorrent & Magnet Downloader");
                });

                ui.add_space(2.0);
                ui.label(
                    "Decentralized peer-to-peer downloading via magnet:?xt=urn:btih:... links or 40-character hex InfoHashes."
                );
                ui.add_space(12.0);

                // Magnet Input Row
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [ui.available_width() - 220.0, 30.0],
                        egui::TextEdit::singleline(&mut self.magnet_input)
                            .hint_text("Enter magnet link (magnet:?xt=urn:btih:...) or 40-char InfoHash..."),
                    );

                    if ui.button("Paste").clicked() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                self.magnet_input = text.trim().to_string();
                            }
                        }
                    }

                    if ui.button("Clear").clicked() {
                        self.magnet_input.clear();
                        self.torrent_error = None;
                    }

                    let can_download = !self.magnet_input.trim().is_empty() && !self.is_torrent_downloading;
                    let dl_btn = ui.add_enabled(
                        can_download,
                        egui::Button::new(
                            egui::RichText::new("Start Download")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(142, 68, 173))
                        .min_size(egui::vec2(120.0, 28.0)),
                    );

                    if dl_btn.clicked() {
                        self.start_torrent_download(config, runtime);
                    }
                });

                // Torrent Download Progress Card
                if self.is_torrent_downloading {
                    ui.add_space(12.0);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("Torrent: {}", self.torrent_progress.name));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new("Cancel Download").color(egui::Color32::RED)).clicked() {
                                        if let Some(ref tx) = self.torrent_cancel_tx {
                                            let _ = tx.send(true);
                                        }
                                    }
                                });
                            });

                            ui.add_space(6.0);
                            let frac = (self.torrent_progress.percent / 100.0).clamp(0.0, 1.0);
                            ui.add(
                                egui::ProgressBar::new(frac)
                                    .text(format!("{:.1}%", self.torrent_progress.percent))
                                    .animate(true),
                            );

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(format!("Speed: {}", self.torrent_progress.download_speed));
                                ui.label("|");
                                ui.label(format!("Peers: {}", self.torrent_progress.peers));
                                if !self.torrent_progress.state_message.is_empty() {
                                    ui.label("|");
                                    ui.label(format!("State: {}", self.torrent_progress.state_message));
                                }
                                ui.label("|");
                                ui.label(format!(
                                    "Downloaded: {} / {}",
                                    self.torrent_progress.downloaded_bytes,
                                    self.torrent_progress.total_bytes
                                ));
                            });
                        });
                }

                // Completed Torrent Banner
                if let Some(ref path) = self.torrent_completed_path {
                    ui.add_space(10.0);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("[Success] Torrent Download Finished.")
                                        .color(egui::Color32::from_rgb(46, 204, 113))
                                        .strong(),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Open Destination Folder").clicked() {
                                        let _ = open::that(path);
                                    }
                                });
                            });
                            ui.label(format!("Saved: {}", path.display()));
                        });
                }

                // Error Torrent Banner
                if let Some(ref err) = self.torrent_error {
                    ui.add_space(10.0);
                    egui::Frame::group(ui.style())
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(6.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("[Notice]").color(egui::Color32::RED).strong());
                                ui.label(err);
                            });
                        });
                }
            });
    }

    fn start_media_download(
        &mut self,
        config: &AppConfig,
        engine: &YtDlpEngine,
        runtime: &tokio::runtime::Handle,
    ) {
        let url = self.url_input.trim().to_string();
        if url.is_empty() {
            return;
        }

        self.is_downloading = true;
        self.download_error = None;
        self.completed_file = None;
        self.current_progress = DownloadProgress::default();

        let (status_tx, status_rx) = mpsc::unbounded_channel();
        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.download_status_rx = Some(status_rx);
        self.cancel_tx = Some(cancel_tx);

        let output_dir = config.download_dir.clone();
        let preset = self.selected_preset;
        let custom_format = self.selected_format_id.clone();
        let engine_clone = engine.clone();

        let extract_audio = config.extract_audio_to_mp3;
        let embed_thumbnail = config.embed_thumbnail;
        let embed_metadata = config.embed_metadata;

        runtime.spawn(async move {
            let _ = engine_clone
                .download(
                    &url,
                    preset,
                    custom_format.as_deref(),
                    &output_dir,
                    extract_audio,
                    embed_thumbnail,
                    embed_metadata,
                    status_tx,
                    cancel_rx,
                )
                .await;
        });
    }

    fn start_torrent_download(
        &mut self,
        config: &AppConfig,
        runtime: &tokio::runtime::Handle,
    ) {
        let magnet_or_hash = self.magnet_input.trim().to_string();
        if magnet_or_hash.is_empty() {
            return;
        }

        self.is_torrent_downloading = true;
        self.torrent_error = None;
        self.torrent_completed_path = None;
        self.torrent_progress = TorrentProgress::default();

        let (status_tx, status_rx) = mpsc::unbounded_channel();
        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.torrent_status_rx = Some(status_rx);
        self.torrent_cancel_tx = Some(cancel_tx);

        let output_dir = config.download_dir.clone();

        runtime.spawn(async move {
            let _ = TorrentManager::start_magnet(&magnet_or_hash, &output_dir, status_tx, cancel_rx).await;
        });
    }
}
