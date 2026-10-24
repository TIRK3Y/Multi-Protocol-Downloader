use eframe::egui;

use crate::config::AppConfig;

#[derive(Default)]
pub struct AboutView;

impl AboutView {
    pub fn show(&mut self, ui: &mut egui::Ui, config: &AppConfig) {
        ui.vertical(|ui| {
            // Header card
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(16))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(" MPD ")
                                .size(24.0)
                                .strong()
                                .color(egui::Color32::WHITE)
                                .background_color(egui::Color32::from_rgb(41, 128, 185)),
                        );

                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.heading("Multi-Protocol Downloader");
                            ui.label(
                                egui::RichText::new("Version 1.0.0 (Release Build)")
                                    .size(12.0)
                                    .color(egui::Color32::GRAY),
                            );
                        });
                    });

                    ui.add_space(8.0);
                    ui.label(
                        "MPD is a high-performance, cross-platform media downloader desktop application and CLI \
                        written in Rust. It unifies web media stream extraction, format inspection, postprocessing, \
                        and decentralized peer-to-peer BitTorrent transfers into a single responsive interface."
                    );
                });

            ui.add_space(14.0);

            // Architecture and Core Engines card
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(16))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    ui.strong("Engine Specifications & Architecture");
                    ui.add_space(8.0);

                    egui::Grid::new("about_engine_specs_grid")
                        .num_columns(2)
                        .spacing([24.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Web Stream Extraction:");
                            ui.label("yt-dlp subprocess engine with live stdout/stderr parsing");
                            ui.end_row();

                            ui.label("Audio & Video Transcoding:");
                            ui.label("FFmpeg static engine with 1-click automated installer");
                            ui.end_row();

                            ui.label("P2P & Magnet Protocol:");
                            ui.label("librqbit v9.0 pure-Rust BitTorrent protocol engine with DHT and rustls");
                            ui.end_row();

                            ui.label("User Interface Framework:");
                            ui.label("egui v0.31 & eframe immediate-mode rendering engine");
                            ui.end_row();

                            ui.label("Asynchronous Runtime:");
                            ui.label("Tokio multi-threaded asynchronous runtime");
                            ui.end_row();

                            ui.label("Platform Targets:");
                            ui.label("Windows (x86_64), macOS (Intel & Apple Silicon), Linux (x86_64)");
                            ui.end_row();
                        });
                });

            ui.add_space(14.0);

            // Storage and Paths
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(16))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    ui.strong("Storage & Environment Locations");
                    ui.add_space(8.0);

                    egui::Grid::new("about_paths_grid")
                        .num_columns(2)
                        .spacing([24.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Default Downloads Path:");
                            ui.monospace(config.download_dir.display().to_string());
                            ui.end_row();

                            ui.label("Configuration Directory:");
                            if let Some(config_dir) = AppConfig::config_path() {
                                ui.monospace(config_dir.display().to_string());
                            } else {
                                ui.label("Standard User Directory");
                            }
                            ui.end_row();

                            ui.label("Open-Source Repository:");
                            ui.hyperlink_to("github.com/TIRK3Y/Multi-Protocol-Downloader", "https://github.com/TIRK3Y/Multi-Protocol-Downloader");
                            ui.end_row();
                        });
                });

            ui.add_space(14.0);

            // Legal & Responsibility Notice
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(16))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    ui.strong("Legal Compliance & User Responsibility Notice");
                    ui.add_space(6.0);

                    ui.label(
                        "MPD (Multi-Protocol Downloader) is distributed strictly as an open-source technical utility \
                        under the MIT License. Users are exclusively responsible for verifying that their downloads \
                        comply with all applicable terms of service, local laws, and intellectual property regulations. \
                        The project authors and maintainers disclaim all liability for any misuse of this software."
                    );

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Software License:");
                        ui.label(
                            egui::RichText::new("MIT License")
                                .strong()
                                .color(egui::Color32::from_rgb(46, 204, 113)),
                        );
                    });
                });
        });
    }
}
