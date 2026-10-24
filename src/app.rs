use eframe::egui;

use crate::config::{AppConfig, ThemeMode};
use crate::engine::ytdlp::YtDlpEngine;
use crate::ui::icons::{self, Icon};
use crate::ui::{
    AboutView, DownloadView, HistoryView, HomeView, PlatformsView, SettingsSection, SettingsView,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NavigationTab {
    Home,
    Download,
    History,
    Platforms,
    Settings(SettingsSection),
    About,
}

pub struct MpdApp {
    pub config: AppConfig,
    pub engine: YtDlpEngine,
    pub current_tab: NavigationTab,
    pub settings_menu_expanded: bool,

    pub home_view: HomeView,
    pub download_view: DownloadView,
    pub history_view: HistoryView,
    pub platforms_view: PlatformsView,
    pub settings_view: SettingsView,
    pub about_view: AboutView,

    pub runtime: tokio::runtime::Handle,
}

impl MpdApp {
    pub fn new(cc: &eframe::CreationContext<'_>, runtime: tokio::runtime::Handle) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let config = AppConfig::load();
        let engine = YtDlpEngine::detect(&config);

        let app = Self {
            config,
            engine,
            current_tab: NavigationTab::Home,
            settings_menu_expanded: false,
            home_view: HomeView::default(),
            download_view: DownloadView::default(),
            history_view: HistoryView::default(),
            platforms_view: PlatformsView::default(),
            settings_view: SettingsView::default(),
            about_view: AboutView::default(),
            runtime,
        };

        app.apply_theme(&cc.egui_ctx);
        app
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        let is_dark = match self.config.theme_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => {
                ctx.system_theme().map(|t| t == egui::Theme::Dark).unwrap_or(true)
            }
        };

        let mut visuals = if is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.menu_corner_radius = 4.0.into();
        visuals.window_corner_radius = 8.0.into();
        visuals.widgets.noninteractive.corner_radius = 4.0.into();
        visuals.widgets.inactive.corner_radius = 4.0.into();
        visuals.widgets.hovered.corner_radius = 4.0.into();
        visuals.widgets.active.corner_radius = 4.0.into();

        ctx.set_visuals(visuals);
    }
}

impl eframe::App for MpdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);

        // Left Collapsible Sidebar Panel
        egui::SidePanel::left("mpd_left_sidebar")
            .resizable(false)
            .exact_width(if self.config.sidebar_expanded { 220.0 } else { 62.0 })
            .show(ctx, |ui| {
                ui.add_space(10.0);

                // Top Header Branding: MPD on top, Downloader directly under it
                if self.config.sidebar_expanded {
                    ui.horizontal(|ui| {
                        // Styled Logo Box with Settings Emblem
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(41, 128, 185));
                        icons::paint_icon(
                            ui.painter(),
                            rect,
                            Icon::Settings,
                            egui::Stroke::new(1.8_f32, egui::Color32::WHITE),
                        );

                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("MPD")
                                    .size(16.0)
                                    .strong()
                                    .color(ui.visuals().strong_text_color()),
                            );
                            ui.label(
                                egui::RichText::new("Downloader")
                                    .size(11.5)
                                    .color(egui::Color32::GRAY),
                            );
                        });
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(41, 128, 185));
                        icons::paint_icon(
                            ui.painter(),
                            rect,
                            Icon::Settings,
                            egui::Stroke::new(1.8_f32, egui::Color32::WHITE),
                        );
                    });
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                // Sidebar Navigation Links
                // 1. Home
                let is_home = self.current_tab == NavigationTab::Home;
                if icons::render_sidebar_item(
                    ui,
                    Icon::Home,
                    "Home",
                    is_home,
                    self.config.sidebar_expanded,
                    false,
                    false,
                    false,
                )
                .clicked()
                {
                    self.current_tab = NavigationTab::Home;
                }
                ui.add_space(3.0);

                // 2. Download
                let is_download = self.current_tab == NavigationTab::Download;
                if icons::render_sidebar_item(
                    ui,
                    Icon::Download,
                    "Download",
                    is_download,
                    self.config.sidebar_expanded,
                    false,
                    false,
                    false,
                )
                .clicked()
                {
                    self.current_tab = NavigationTab::Download;
                }
                ui.add_space(3.0);

                // 3. History
                let is_history = self.current_tab == NavigationTab::History;
                let history_label = if !self.config.history.is_empty() {
                    format!("History ({})", self.config.history.len())
                } else {
                    "History".to_string()
                };

                if icons::render_sidebar_item(
                    ui,
                    Icon::History,
                    &history_label,
                    is_history,
                    self.config.sidebar_expanded,
                    false,
                    false,
                    false,
                )
                .clicked()
                {
                    self.current_tab = NavigationTab::History;
                }
                ui.add_space(3.0);

                // 4. Platforms
                let is_platforms = self.current_tab == NavigationTab::Platforms;
                if icons::render_sidebar_item(
                    ui,
                    Icon::Platforms,
                    "Platforms",
                    is_platforms,
                    self.config.sidebar_expanded,
                    false,
                    false,
                    false,
                )
                .clicked()
                {
                    self.current_tab = NavigationTab::Platforms;
                }
                ui.add_space(3.0);

                // 5. Settings (Dropdown inside menu like web applications)
                let is_any_settings = matches!(self.current_tab, NavigationTab::Settings(_));

                if self.config.sidebar_expanded {
                    let settings_header_resp = icons::render_sidebar_item(
                        ui,
                        Icon::Settings,
                        "Settings",
                        is_any_settings,
                        true,
                        false,
                        true,
                        self.settings_menu_expanded,
                    );

                    if settings_header_resp.clicked() {
                        self.settings_menu_expanded = !self.settings_menu_expanded;
                        if !is_any_settings {
                            self.current_tab = NavigationTab::Settings(SettingsSection::Appearance);
                        }
                    }

                    // Nested sub-items inside left sidebar menu (webapps style)
                    if self.settings_menu_expanded {
                        ui.add_space(2.0);

                        // Sub 1: Appearance & Theme
                        let is_app_sub = self.current_tab == NavigationTab::Settings(SettingsSection::Appearance);
                        if icons::render_sidebar_item(
                            ui,
                            Icon::Platforms,
                            "Appearance",
                            is_app_sub,
                            true,
                            true,
                            false,
                            false,
                        )
                        .clicked()
                        {
                            self.current_tab = NavigationTab::Settings(SettingsSection::Appearance);
                        }
                        ui.add_space(1.0);

                        // Sub 2: Engines & Postprocessing
                        let is_eng_sub = self.current_tab == NavigationTab::Settings(SettingsSection::Engines);
                        if icons::render_sidebar_item(
                            ui,
                            Icon::Settings,
                            "Engines & Tools",
                            is_eng_sub,
                            true,
                            true,
                            false,
                            false,
                        )
                        .clicked()
                        {
                            self.current_tab = NavigationTab::Settings(SettingsSection::Engines);
                        }
                        ui.add_space(1.0);

                        // Sub 3: Download Preferences
                        let is_pref_sub = self.current_tab == NavigationTab::Settings(SettingsSection::Preferences);
                        if icons::render_sidebar_item(
                            ui,
                            Icon::Download,
                            "Preferences",
                            is_pref_sub,
                            true,
                            true,
                            false,
                            false,
                        )
                        .clicked()
                        {
                            self.current_tab = NavigationTab::Settings(SettingsSection::Preferences);
                        }
                    }
                } else {
                    // Collapsed mode: single settings icon
                    if icons::render_sidebar_item(
                        ui,
                        Icon::Settings,
                        "Settings",
                        is_any_settings,
                        false,
                        false,
                        false,
                        false,
                    )
                    .clicked()
                    {
                        self.current_tab = NavigationTab::Settings(SettingsSection::Appearance);
                    }
                }
                ui.add_space(3.0);

                // 6. About Page (Dedicated Product Details)
                let is_about = self.current_tab == NavigationTab::About;
                if icons::render_sidebar_item(
                    ui,
                    Icon::About,
                    "About",
                    is_about,
                    self.config.sidebar_expanded,
                    false,
                    false,
                    false,
                )
                .clicked()
                {
                    self.current_tab = NavigationTab::About;
                }

                // Sidebar Footer: Version above the bottom-left expand/collapse button
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(8.0);

                    // Expand / Collapse icon button at bottom-left
                    if self.config.sidebar_expanded {
                        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
                        let is_hovered = response.hovered();
                        if is_hovered {
                            ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
                        }
                        let icon_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + 8.0, rect.center().y - 8.0),
                            egui::vec2(16.0, 16.0),
                        );
                        let stroke_color = if is_hovered {
                            ui.visuals().widgets.hovered.fg_stroke.color
                        } else {
                            ui.visuals().text_color()
                        };
                        icons::paint_icon(ui.painter(), icon_rect, Icon::ChevronLeft, egui::Stroke::new(1.6_f32, stroke_color));
                        ui.painter().text(
                            egui::pos2(rect.min.x + 30.0, rect.center().y - 7.0),
                            egui::Align2::LEFT_TOP,
                            "Collapse",
                            egui::FontId::proportional(12.5),
                            stroke_color,
                        );
                        if response.on_hover_text("Collapse sidebar (<)").clicked() {
                            self.config.sidebar_expanded = false;
                            let _ = self.config.save();
                        }
                    } else {
                        let (rect, response) = ui.allocate_exact_size(egui::vec2(38.0, 32.0), egui::Sense::click());
                        let is_hovered = response.hovered();
                        if is_hovered {
                            ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
                        }
                        let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(18.0, 18.0));
                        let stroke_color = if is_hovered {
                            ui.visuals().widgets.hovered.fg_stroke.color
                        } else {
                            ui.visuals().text_color()
                        };
                        icons::paint_icon(ui.painter(), icon_rect, Icon::ChevronRight, egui::Stroke::new(1.6_f32, stroke_color));
                        if response.on_hover_text("Expand sidebar (>)").clicked() {
                            self.config.sidebar_expanded = true;
                            let _ = self.config.save();
                        }
                    }

                    ui.add_space(6.0);

                    // Version displayed directly above the collapse/expand button
                    if self.config.sidebar_expanded {
                        ui.label(
                            egui::RichText::new("v1.0.0")
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("v1.0")
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                    }

                    ui.add_space(4.0);
                    ui.separator();
                });
            });

        // Main Content Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    let mut switch_to_dl = false;

                    match self.current_tab {
                        NavigationTab::Home => {
                            self.home_view.show(ui, &mut switch_to_dl);
                            if switch_to_dl {
                                self.current_tab = NavigationTab::Download;
                            }
                        }
                        NavigationTab::Download => {
                            self.download_view.show(
                                ui,
                                &mut self.config,
                                &mut self.engine,
                                &self.runtime,
                            );
                        }
                        NavigationTab::History => {
                            self.history_view.show(ui, &mut self.config);
                        }
                        NavigationTab::Platforms => {
                            self.platforms_view.show(ui);
                        }
                        NavigationTab::Settings(section) => {
                            self.settings_view.active_section = section;
                            self.settings_view.show(
                                ui,
                                &mut self.config,
                                &mut self.engine,
                                &self.runtime,
                            );
                        }
                        NavigationTab::About => {
                            self.about_view.show(ui, &self.config);
                        }
                    }
                });
        });

        // Request continuous repaint while downloading or installing to animate progress smoothly
        if self.download_view.is_downloading
            || self.download_view.is_torrent_downloading
            || self.download_view.is_fetching_info
            || self.download_view.is_installing_ffmpeg
            || self.settings_view.is_installing_ytdlp
            || self.settings_view.is_installing_ffmpeg
        {
            ctx.request_repaint();
        }
    }
}
