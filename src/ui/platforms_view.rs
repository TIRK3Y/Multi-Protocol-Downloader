use eframe::egui;

pub struct PlatformsView {
    search_query: String,
}

impl Default for PlatformsView {
    fn default() -> Self {
        Self {
            search_query: String::new(),
        }
    }
}

impl PlatformsView {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Supported Platforms and Protocols");
            ui.label(
                "MPD (Multi-Protocol Downloader) is powered by the yt-dlp extraction engine, supporting over 1,000+ platforms, streaming protocols (HLS, DASH, RTMP, HTTP), and video/audio providers worldwide.",
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Search Platform:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("Clear").clicked() {
                    self.search_query.clear();
                }
            });

            ui.add_space(10.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let platforms = [
                    ("YouTube", "youtube.com / youtu.be", "Videos, Shorts, Playlists, Channels, Audio", "https://youtube.com"),
                    ("Vimeo", "vimeo.com", "HD/4K Videos, On-Demand, Showcases", "https://vimeo.com"),
                    ("SoundCloud", "soundcloud.com", "High-bitrate Audio tracks, Albums, Playlists", "https://soundcloud.com"),
                    ("TikTok", "tiktok.com", "Short-form mobile videos, Original sound", "https://tiktok.com"),
                    ("Twitter / X", "x.com / twitter.com", "Video clips, GIFs, Voice spaces", "https://x.com"),
                    ("Instagram", "instagram.com", "Reels, Posts, Stories, IGTV", "https://instagram.com"),
                    ("Facebook", "facebook.com", "Public videos, Reels, Watch", "https://facebook.com"),
                    ("Twitch", "twitch.tv", "VODs, Highlights, Clips", "https://twitch.tv"),
                    ("Reddit", "reddit.com", "Video posts (v.redd.it) with merged audio", "https://reddit.com"),
                    ("Bandcamp", "bandcamp.com", "Independent artist tracks, Full albums", "https://bandcamp.com"),
                    ("Dailymotion", "dailymotion.com", "News, Entertainment, Web video", "https://dailymotion.com"),
                    ("Bilibili", "bilibili.com", "Anime, User clips, 1080p/4K streams", "https://bilibili.com"),
                    ("VKontakte (VK)", "vk.com", "Videos, Clips, Community content", "https://vk.com"),
                    ("Flickr", "flickr.com", "HD Video clips and animations", "https://flickr.com"),
                    ("Mixcloud", "mixcloud.com", "DJ sets, Radio shows, Podcasts", "https://mixcloud.com"),
                    ("Archive.org", "archive.org", "Internet Archive historical media and public domain files", "https://archive.org"),
                ];

                let query = self.search_query.trim().to_lowercase();

                egui::Grid::new("platforms_grid")
                    .num_columns(4)
                    .spacing([20.0, 12.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Platform");
                        ui.strong("Domains");
                        ui.strong("Supported Formats");
                        ui.strong("Link");
                        ui.end_row();

                        for (name, domain, desc, url) in platforms {
                            if !query.is_empty()
                                && !name.to_lowercase().contains(&query)
                                && !domain.to_lowercase().contains(&query)
                                && !desc.to_lowercase().contains(&query)
                            {
                                continue;
                            }

                            ui.label(egui::RichText::new(name).strong());
                            ui.label(domain);
                            ui.label(desc);
                            if ui.link("Visit [Ext]").clicked() {
                                let _ = open::that(url);
                            }
                            ui.end_row();
                        }
                    });

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.label("Want to see all 1,800+ supported sites?");
                    if ui.link("Browse Full Extractors List on GitHub [Ext]").clicked() {
                        let _ = open::that("https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md");
                    }
                });
            });
        });
    }
}
