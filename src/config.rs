use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::engine::models::{HistoryRecord, QualityPreset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

impl ThemeMode {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark Mode",
            ThemeMode::Light => "Light Mode",
            ThemeMode::System => "System Default",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub download_dir: PathBuf,
    pub custom_ytdlp_path: Option<PathBuf>,
    pub custom_ffmpeg_path: Option<PathBuf>,
    pub default_preset: QualityPreset,
    pub extract_audio_to_mp3: bool,
    pub embed_thumbnail: bool,
    pub embed_metadata: bool,
    pub theme_mode: ThemeMode,
    pub sidebar_expanded: bool,
    pub history: Vec<HistoryRecord>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let download_dir = directories::UserDirs::new()
            .and_then(|u| u.download_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("downloads"));

        Self {
            download_dir,
            custom_ytdlp_path: None,
            custom_ffmpeg_path: None,
            default_preset: QualityPreset::BestVideoAudio,
            extract_audio_to_mp3: false,
            embed_thumbnail: true,
            embed_metadata: true,
            theme_mode: ThemeMode::Dark,
            sidebar_expanded: true,
            history: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("org", "MPD", "MPD")?;
        let config_dir = proj_dirs.config_dir();
        Some(config_dir.join("config.json"))
    }

    pub fn bin_dir() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("org", "MPD", "MPD")?;
        let data_dir = proj_dirs.data_local_dir();
        Some(data_dir.join("bin"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(self)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    pub fn add_history(&mut self, record: HistoryRecord) {
        // Keep up to 100 recent items
        self.history.retain(|h| h.id != record.id);
        self.history.insert(0, record);
        if self.history.len() > 100 {
            self.history.truncate(100);
        }
        let _ = self.save();
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        let _ = self.save();
    }
}
