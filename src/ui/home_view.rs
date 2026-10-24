use eframe::egui;

pub struct HomeView;

impl Default for HomeView {
    fn default() -> Self {
        Self
    }
}

impl HomeView {
    pub fn show(&mut self, ui: &mut egui::Ui, switch_to_download: &mut bool) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            // Stylized Minimal Monogram
            ui.label(
                egui::RichText::new("MPD")
                    .size(42.0)
                    .strong()
                    .color(egui::Color32::from_rgb(52, 152, 219)),
            );

            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("Multi-Protocol Downloader")
                    .size(16.0)
                    .color(ui.visuals().text_color()),
            );

            ui.add_space(8.0);

            ui.label(
                egui::RichText::new("High-performance media stream, torrent, and protocol acquisition client.")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(140, 145, 155)),
            );

            ui.add_space(32.0);

            // Clean Canvas Card (Ready for future custom modules / dashboard additions)
            ui.group(|ui| {
                ui.set_width(ui.available_width().min(640.0));
                ui.add_space(24.0);

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Workspace Ready")
                            .size(15.0)
                            .strong()
                            .color(ui.visuals().text_color()),
                    );

                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new(
                            "This canvas is configured for upcoming dashboard metrics and queue monitors.\n\
                            Switch to the Download tab to start acquiring streams, files, or torrents.",
                        )
                        .size(12.0)
                        .color(egui::Color32::from_rgb(130, 135, 145)),
                    );

                    ui.add_space(20.0);

                    let start_btn = egui::Button::new(
                        egui::RichText::new("Open Downloader")
                            .size(14.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(41, 128, 185))
                    .min_size(egui::vec2(160.0, 34.0));

                    if ui.add(start_btn).clicked() {
                        *switch_to_download = true;
                    }
                });

                ui.add_space(24.0);
            });

            ui.add_space(32.0);

            // Feature capability tags (NO EMOJIS)
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(12.0, 6.0);
                let tags = [
                    "Web Streams & Video Platforms",
                    "BitTorrent & Magnet Links",
                    "HLS & DASH Playlists",
                    "Audio Stream Extractor",
                    "Local Engine Auto-Install",
                ];
                for tag in tags {
                    ui.label(
                        egui::RichText::new(format!("[ {} ]", tag))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 125, 135)),
                    );
                }
            });
        });
    }
}
