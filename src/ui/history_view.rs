use eframe::egui;

use crate::config::AppConfig;

pub struct HistoryView {
    filter_query: String,
}

impl Default for HistoryView {
    fn default() -> Self {
        Self {
            filter_query: String::new(),
        }
    }
}

impl HistoryView {
    pub fn show(&mut self, ui: &mut egui::Ui, config: &mut AppConfig) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Download History");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear History").clicked() {
                        config.clear_history();
                    }
                });
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.filter_query);
                if ui.button("Clear").clicked() {
                    self.filter_query.clear();
                }
            });

            ui.add_space(8.0);

            if config.history.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("No downloads recorded yet.")
                            .color(egui::Color32::GRAY)
                            .size(15.0),
                    );
                    ui.label("Files and streams downloaded through MPD will appear here.");
                });
                return;
            }

            let query = self.filter_query.trim().to_lowercase();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut index_to_remove = None;

                for (idx, record) in config.history.iter().enumerate() {
                    if !query.is_empty()
                        && !record.title.to_lowercase().contains(&query)
                        && !record.url.to_lowercase().contains(&query)
                        && !record.format_desc.to_lowercase().contains(&query)
                    {
                        continue;
                    }

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&record.title);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&record.timestamp)
                                            .size(11.0)
                                            .color(egui::Color32::GRAY),
                                    );
                                    ui.label("|");
                                    ui.label(
                                        egui::RichText::new(&record.format_desc)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(52, 152, 219)),
                                    );
                                });
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("X").on_hover_text("Remove entry from history").clicked() {
                                    index_to_remove = Some(idx);
                                }

                                if let Some(ref path) = record.file_path {
                                    if path.exists() {
                                        if ui.button("Open Folder").clicked() {
                                            if path.is_dir() {
                                                let _ = open::that(path);
                                            } else if let Some(parent) = path.parent() {
                                                let _ = open::that(parent);
                                            }
                                        }
                                        if path.is_file() {
                                            if ui.button("Play / Open").clicked() {
                                                let _ = open::that(path);
                                            }
                                        }
                                    } else {
                                        ui.label(
                                            egui::RichText::new("(File moved or deleted)")
                                                .size(11.0)
                                                .color(egui::Color32::DARK_GRAY),
                                        );
                                    }
                                }

                                if !record.url.is_empty() && ui.small_button("Link").on_hover_text("Open URL or magnet in browser").clicked() {
                                    let _ = open::that(&record.url);
                                }
                            });
                        });
                    });
                    ui.add_space(4.0);
                }

                if let Some(idx) = index_to_remove {
                    config.history.remove(idx);
                    let _ = config.save();
                }
            });
        });
    }
}
