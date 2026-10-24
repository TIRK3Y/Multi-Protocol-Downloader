use eframe::egui;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::config::{AppConfig, ThemeMode};
use crate::engine::installer::EngineInstaller;
use crate::engine::models::QualityPreset;
use crate::engine::ytdlp::YtDlpEngine;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SettingsSection {
    Appearance,
    Engines,
    Preferences,
}

impl SettingsSection {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsSection::Appearance => "Appearance & Theme",
            SettingsSection::Engines => "Engines & Postprocessing",
            SettingsSection::Preferences => "Download Preferences",
        }
    }
}

pub struct SettingsView {
    pub active_section: SettingsSection,

    pub is_installing_ytdlp: bool,
    pub ytdlp_progress: f32,
    pub ytdlp_status: String,
    ytdlp_rx: Option<mpsc::UnboundedReceiver<(f32, String)>>,
    ytdlp_done_rx: Option<tokio::sync::oneshot::Receiver<Result<PathBuf, String>>>,

    pub is_installing_ffmpeg: bool,
    pub ffmpeg_progress: f32,
    pub ffmpeg_status: String,
    ffmpeg_rx: Option<mpsc::UnboundedReceiver<(f32, String)>>,
    ffmpeg_done_rx: Option<tokio::sync::oneshot::Receiver<Result<PathBuf, String>>>,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            active_section: SettingsSection::Appearance,

            is_installing_ytdlp: false,
            ytdlp_progress: 0.0,
            ytdlp_status: String::new(),
            ytdlp_rx: None,
            ytdlp_done_rx: None,

            is_installing_ffmpeg: false,
            ffmpeg_progress: 0.0,
            ffmpeg_status: String::new(),
            ffmpeg_rx: None,
            ffmpeg_done_rx: None,
        }
    }
}

impl SettingsView {
    pub fn update_installers(&mut self, config: &mut AppConfig, engine: &mut YtDlpEngine) {
        if let Some(ref mut rx) = self.ytdlp_rx {
            while let Ok((prog, status)) = rx.try_recv() {
                self.ytdlp_progress = prog;
                self.ytdlp_status = status;
            }
        }

        if let Some(ref mut done_rx) = self.ytdlp_done_rx {
            if let Ok(res) = done_rx.try_recv() {
                self.is_installing_ytdlp = false;
                match res {
                    Ok(path) => {
                        self.ytdlp_status = format!("Installed to: {}", path.display());
                        *engine = YtDlpEngine::detect(config);
                    }
                    Err(e) => {
                        self.ytdlp_status = format!("Installation failed: {e}");
                    }
                }
                self.ytdlp_done_rx = None;
                self.ytdlp_rx = None;
            }
        }

        if let Some(ref mut rx) = self.ffmpeg_rx {
            while let Ok((prog, status)) = rx.try_recv() {
                self.ffmpeg_progress = prog;
                self.ffmpeg_status = status;
            }
        }

        if let Some(ref mut done_rx) = self.ffmpeg_done_rx {
            if let Ok(res) = done_rx.try_recv() {
                self.is_installing_ffmpeg = false;
                match res {
                    Ok(path) => {
                        self.ffmpeg_status = format!("Installed to: {}", path.display());
                        *engine = YtDlpEngine::detect(config);
                    }
                    Err(e) => {
                        self.ffmpeg_status = format!("Installation failed: {e}");
                    }
                }
                self.ffmpeg_done_rx = None;
                self.ffmpeg_rx = None;
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
        self.update_installers(config, engine);

        ui.vertical(|ui| {
            // Header showing the active section selected from the sidebar
            ui.horizontal(|ui| {
                ui.heading(self.active_section.label());
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Settings")
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });

            ui.add_space(2.0);
            match self.active_section {
                SettingsSection::Appearance => {
                    ui.label("Configure application visual theme, color scheme, and responsive layout.");
                }
                SettingsSection::Engines => {
                    ui.label("Monitor engine health status, install yt-dlp & FFmpeg, and configure binary paths.");
                }
                SettingsSection::Preferences => {
                    ui.label("Configure default download directories, extraction formats, and operational limits.");
                }
            }

            ui.add_space(14.0);

            match self.active_section {
                SettingsSection::Appearance => {
                    self.show_appearance(ui, config);
                }
                SettingsSection::Engines => {
                    self.show_engines(ui, config, engine, runtime);
                }
                SettingsSection::Preferences => {
                    self.show_preferences(ui, config);
                }
            }
        });
    }

    fn show_appearance(&mut self, ui: &mut egui::Ui, config: &mut AppConfig) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.strong("Theme and Visual Style");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Display Theme:");
                    let modes = [ThemeMode::Dark, ThemeMode::Light, ThemeMode::System];
                    for m in modes {
                        if ui.selectable_value(&mut config.theme_mode, m, m.label()).clicked() {
                            let _ = config.save();
                        }
                    }
                });

                ui.add_space(8.0);
                ui.label("Select 'Dark Mode', 'Light Mode', or 'System Default' to synchronize with operating system visuals.");

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                ui.strong("Sidebar Navigation Default");
                ui.add_space(6.0);
                if ui.checkbox(&mut config.sidebar_expanded, "Keep sidebar expanded on application startup").clicked() {
                    let _ = config.save();
                }
            });
    }

    fn show_engines(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut AppConfig,
        engine: &mut YtDlpEngine,
        runtime: &tokio::runtime::Handle,
    ) {
        // Prominent Engine Status & Telemetry Card
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.strong("Active Engines Telemetry & Health Status");
                ui.add_space(8.0);

                egui::Grid::new("engines_status_dashboard_grid")
                    .num_columns(3)
                    .spacing([24.0, 10.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Engine Name | Status Badge | Path / Version
                        ui.strong("Engine Component");
                        ui.strong("Health Status");
                        ui.strong("Location / Details");
                        ui.end_row();

                        // yt-dlp
                        ui.label("yt-dlp (Stream Extractor)");
                        if engine.is_available() {
                            ui.label(
                                egui::RichText::new("Ready")
                                    .color(egui::Color32::from_rgb(46, 204, 113))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("Missing")
                                    .color(egui::Color32::from_rgb(230, 126, 34))
                                    .strong(),
                            );
                        }
                        if let Some(ref path) = engine.executable_path {
                            ui.monospace(path.display().to_string());
                        } else {
                            ui.label("Not detected");
                        }
                        ui.end_row();

                        // FFmpeg
                        ui.label("FFmpeg (Transcoder)");
                        if engine.is_ffmpeg_available() {
                            ui.label(
                                egui::RichText::new("Ready")
                                    .color(egui::Color32::from_rgb(46, 204, 113))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("Missing")
                                    .color(egui::Color32::from_rgb(231, 76, 60))
                                    .strong(),
                            );
                        }
                        if let Some(ref path) = engine.ffmpeg_path {
                            ui.monospace(path.display().to_string());
                        } else {
                            ui.label("Not detected");
                        }
                        ui.end_row();

                        // BitTorrent Engine
                        ui.label("BitTorrent Engine (librqbit)");
                        ui.label(
                            egui::RichText::new("Active (Native)")
                                .color(egui::Color32::from_rgb(46, 204, 113))
                                .strong(),
                        );
                        ui.label("Embedded pure-Rust engine with DHT & rustls");
                        ui.end_row();
                    });
            });

        ui.add_space(14.0);

        // yt-dlp Card
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.strong("Media Extraction Engine (yt-dlp)");
                ui.add_space(6.0);

                if let Some(ref ver) = engine.version {
                    ui.label(format!("Detected Version: {}", ver));
                }

                if let Some(ref path) = engine.executable_path {
                    ui.label(format!("Path: {}", path.display()));
                }

                ui.add_space(8.0);

                if self.is_installing_ytdlp {
                    ui.add(egui::ProgressBar::new(self.ytdlp_progress).show_percentage());
                    ui.label(&self.ytdlp_status);
                } else {
                    ui.horizontal(|ui| {
                        let btn_text = if engine.is_available() {
                            "Update / Reinstall yt-dlp (1-Click)"
                        } else {
                            "Install yt-dlp Automatically (1-Click)"
                        };

                        if ui.button(egui::RichText::new(btn_text).strong()).clicked() {
                            self.is_installing_ytdlp = true;
                            self.ytdlp_progress = 0.0;
                            self.ytdlp_status = "Starting download...".to_string();

                            let (tx, rx) = mpsc::unbounded_channel();
                            let (done_tx, done_rx) = tokio::sync::oneshot::channel();
                            self.ytdlp_rx = Some(rx);
                            self.ytdlp_done_rx = Some(done_rx);

                            runtime.spawn(async move {
                                let res = EngineInstaller::install_ytdlp(move |frac, msg| {
                                    let _ = tx.send((frac, msg.to_string()));
                                }).await;
                                let _ = done_tx.send(res);
                            });
                        }

                        if ui.button("Browse Executable").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select yt-dlp Executable")
                                .pick_file()
                            {
                                config.custom_ytdlp_path = Some(path);
                                let _ = config.save();
                                *engine = YtDlpEngine::detect(config);
                            }
                        }

                        if config.custom_ytdlp_path.is_some() {
                            if ui.button("Reset Path").clicked() {
                                config.custom_ytdlp_path = None;
                                let _ = config.save();
                                *engine = YtDlpEngine::detect(config);
                            }
                        }
                    });

                    if !self.ytdlp_status.is_empty() {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&self.ytdlp_status).size(11.0));
                    }
                }
            });

        ui.add_space(14.0);

        // FFmpeg Card
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.strong("Postprocessing and Multiplexing Engine (FFmpeg)");
                ui.add_space(6.0);

                if let Some(ref path) = engine.ffmpeg_path {
                    ui.label(format!("Path: {}", path.display()));
                }

                ui.add_space(8.0);

                if self.is_installing_ffmpeg {
                    ui.add(egui::ProgressBar::new(self.ffmpeg_progress).show_percentage());
                    ui.label(&self.ffmpeg_status);
                } else {
                    ui.horizontal(|ui| {
                        let btn_text = if engine.is_ffmpeg_available() {
                            "Reinstall FFmpeg (1-Click)"
                        } else {
                            "Install FFmpeg Automatically (1-Click)"
                        };

                        if ui.button(egui::RichText::new(btn_text).strong()).clicked() {
                            self.is_installing_ffmpeg = true;
                            self.ffmpeg_progress = 0.0;
                            self.ffmpeg_status = "Starting download...".to_string();

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

                        if ui.button("Browse FFmpeg").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select FFmpeg Executable")
                                .pick_file()
                            {
                                config.custom_ffmpeg_path = Some(path);
                                let _ = config.save();
                                *engine = YtDlpEngine::detect(config);
                            }
                        }

                        if config.custom_ffmpeg_path.is_some() {
                            if ui.button("Reset Path").clicked() {
                                config.custom_ffmpeg_path = None;
                                let _ = config.save();
                                *engine = YtDlpEngine::detect(config);
                            }
                        }
                    });

                    if !self.ffmpeg_status.is_empty() {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&self.ffmpeg_status).size(11.0));
                    }
                }
            });
    }

    fn show_preferences(&mut self, ui: &mut egui::Ui, config: &mut AppConfig) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(16))
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.strong("Download Storage Directory");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.monospace(config.download_dir.display().to_string());
                    if ui.button("Change Directory").clicked() {
                        if let Some(dir) = rfd::FileDialog::new()
                            .set_directory(&config.download_dir)
                            .set_title("Select Download Directory")
                            .pick_folder()
                        {
                            config.download_dir = dir;
                            let _ = config.save();
                        }
                    }

                    if ui.button("Open Folder").clicked() {
                        let _ = open::that(&config.download_dir);
                    }
                });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                ui.strong("Default Download Quality Preset");
                ui.add_space(6.0);

                let presets = [
                    QualityPreset::BestVideoAudio,
                    QualityPreset::Fhd1080p,
                    QualityPreset::Hd720p,
                    QualityPreset::Sd480p,
                    QualityPreset::BestAudio,
                ];

                for p in presets {
                    if ui.radio_value(&mut config.default_preset, p, p.label()).clicked() {
                        let _ = config.save();
                    }
                }

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                ui.strong("Postprocessing and Metadata Options");
                ui.add_space(6.0);

                if ui.checkbox(&mut config.extract_audio_to_mp3, "Extract audio to MP3 automatically for audio presets").clicked() {
                    let _ = config.save();
                }

                if ui.checkbox(&mut config.embed_thumbnail, "Embed thumbnail artwork into downloaded media files").clicked() {
                    let _ = config.save();
                }

                if ui.checkbox(&mut config.embed_metadata, "Embed metadata tags (title, artist, album) into files").clicked() {
                    let _ = config.save();
                }
            });
    }
}
