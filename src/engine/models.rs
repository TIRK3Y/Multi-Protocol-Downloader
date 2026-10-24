use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub uploader: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub webpage_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub formats: Vec<FormatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatItem {
    pub format_id: String,
    #[serde(default)]
    pub format_note: Option<String>,
    #[serde(default)]
    pub ext: String,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub fps: Option<f64>,
    #[serde(default)]
    pub vcodec: Option<String>,
    #[serde(default)]
    pub acodec: Option<String>,
    #[serde(default)]
    pub filesize: Option<u64>,
    #[serde(default)]
    pub filesize_approx: Option<u64>,
    #[serde(default)]
    pub tbr: Option<f64>,
    #[serde(default)]
    pub asr: Option<u32>,
}

impl FormatItem {
    pub fn is_audio_only(&self) -> bool {
        let has_no_video = self
            .vcodec
            .as_deref()
            .map(|v| v == "none" || v.is_empty())
            .unwrap_or(true);
        let has_audio = self
            .acodec
            .as_deref()
            .map(|a| a != "none" && !a.is_empty())
            .unwrap_or(false);
        has_no_video && has_audio
    }

    pub fn is_video_only(&self) -> bool {
        let has_video = self
            .vcodec
            .as_deref()
            .map(|v| v != "none" && !v.is_empty())
            .unwrap_or(false);
        let has_no_audio = self
            .acodec
            .as_deref()
            .map(|a| a == "none" || a.is_empty())
            .unwrap_or(true);
        has_video && has_no_audio
    }

    pub fn is_video_and_audio(&self) -> bool {
        let has_video = self
            .vcodec
            .as_deref()
            .map(|v| v != "none" && !v.is_empty())
            .unwrap_or(false);
        let has_audio = self
            .acodec
            .as_deref()
            .map(|a| a != "none" && !a.is_empty())
            .unwrap_or(false);
        has_video && has_audio
    }

    pub fn resolution_str(&self) -> String {
        if let Some(res) = &self.resolution {
            if res != "audio only" {
                return res.clone();
            }
        }
        if let (Some(w), Some(h)) = (self.width, self.height) {
            return format!("{w}x{h}");
        }
        if let Some(h) = self.height {
            return format!("{h}p");
        }
        if self.is_audio_only() {
            return "Audio only".to_string();
        }
        "Unknown".to_string()
    }

    pub fn size_str(&self) -> String {
        let bytes = self.filesize.or(self.filesize_approx);
        if let Some(b) = bytes {
            let mb = b as f64 / 1_048_576.0;
            if mb >= 1024.0 {
                return format!("{:.2} GB", mb / 1024.0);
            }
            return format!("{:.1} MB", mb);
        }
        if let Some(tbr) = self.tbr {
            return format!("~{:.0} kbps", tbr);
        }
        "—".to_string()
    }

    pub fn summary_label(&self) -> String {
        let res = self.resolution_str();
        let ext = &self.ext;
        let note = self.format_note.as_deref().unwrap_or("");
        let size = self.size_str();

        if !note.is_empty() {
            format!("[{}] {} ({} / {}) - {}", self.format_id, res, ext, size, note)
        } else {
            format!("[{}] {} ({} / {})", self.format_id, res, ext, size)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    BestVideoAudio,
    Fhd1080p,
    Hd720p,
    Sd480p,
    BestAudio,
    Custom,
}

impl QualityPreset {
    pub fn label(&self) -> &'static str {
        match self {
            QualityPreset::BestVideoAudio => "Best Quality (Video + Audio)",
            QualityPreset::Fhd1080p => "1080p Full HD",
            QualityPreset::Hd720p => "720p HD",
            QualityPreset::Sd480p => "480p SD",
            QualityPreset::BestAudio => "Audio Only (MP3)",
            QualityPreset::Custom => "Custom Format Selection",
        }
    }

    pub fn format_arg(&self) -> &'static str {
        match self {
            QualityPreset::BestVideoAudio => "bestvideo+bestaudio/best",
            QualityPreset::Fhd1080p => "bestvideo[height<=1080]+bestaudio/best[height<=1080]/best",
            QualityPreset::Hd720p => "bestvideo[height<=720]+bestaudio/best[height<=720]/best",
            QualityPreset::Sd480p => "bestvideo[height<=480]+bestaudio/best[height<=480]/best",
            QualityPreset::BestAudio => "bestaudio/best",
            QualityPreset::Custom => "",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
    pub downloaded: String,
    pub total: String,
    pub status_message: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DownloadStatus {
    Idle,
    Extracting,
    Downloading(DownloadProgress),
    PostProcessing(String),
    Completed(PathBuf),
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub title: String,
    pub url: String,
    pub format_desc: String,
    pub file_path: Option<PathBuf>,
    pub timestamp: String,
    pub status: String,
}
