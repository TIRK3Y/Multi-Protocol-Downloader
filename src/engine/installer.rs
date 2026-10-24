use flate2::read::GzDecoder;
use futures_util::StreamExt;
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use crate::config::AppConfig;

pub struct EngineInstaller;

impl EngineInstaller {
    pub fn get_ytdlp_download_url() -> (&'static str, &'static str) {
        #[cfg(target_os = "windows")]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
                "yt-dlp.exe",
            )
        }
        #[cfg(target_os = "macos")]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos",
                "yt-dlp",
            )
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux",
                "yt-dlp",
            )
        }
    }

    pub fn get_ffmpeg_download_url() -> (&'static str, &'static str) {
        #[cfg(target_os = "windows")]
        {
            (
                "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-win32-x64.gz",
                "ffmpeg.exe",
            )
        }
        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            {
                (
                    "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-arm64.gz",
                    "ffmpeg",
                )
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                (
                    "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-x64.gz",
                    "ffmpeg",
                )
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            #[cfg(target_arch = "aarch64")]
            {
                (
                    "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-linux-arm64.gz",
                    "ffmpeg",
                )
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                (
                    "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-linux-x64.gz",
                    "ffmpeg",
                )
            }
        }
    }

    pub fn target_bin_path() -> Result<PathBuf, String> {
        let bin_dir = AppConfig::bin_dir()
            .ok_or_else(|| "Could not determine local app data directory".to_string())?;
        fs::create_dir_all(&bin_dir).map_err(|e| format!("Failed to create bin dir: {e}"))?;
        let (_, filename) = Self::get_ytdlp_download_url();
        Ok(bin_dir.join(filename))
    }

    pub fn target_ffmpeg_path() -> Result<PathBuf, String> {
        let bin_dir = AppConfig::bin_dir()
            .ok_or_else(|| "Could not determine local app data directory".to_string())?;
        fs::create_dir_all(&bin_dir).map_err(|e| format!("Failed to create bin dir: {e}"))?;
        let (_, filename) = Self::get_ffmpeg_download_url();
        Ok(bin_dir.join(filename))
    }

    pub async fn install_ytdlp<F>(progress_cb: F) -> Result<PathBuf, String>
    where
        F: Fn(f32, &str) + Send + 'static,
    {
        let (url, _) = Self::get_ytdlp_download_url();
        let target_path = Self::target_bin_path()?;

        progress_cb(0.05, "Connecting to yt-dlp GitHub Releases...");

        let client = reqwest::Client::builder()
            .user_agent("MPD-Desktop/1.0.0")
            .build()
            .map_err(|e| format!("HTTP Client error: {e}"))?;

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed with status: {}", resp.status()));
        }

        let total_size = resp.content_length().unwrap_or(0);
        let temp_path = target_path.with_extension("tmp");

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| format!("Could not create temp file: {e}"))?;

        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Write error: {e}"))?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let frac = (downloaded as f32 / total_size as f32).clamp(0.0, 1.0);
                let msg = format!(
                    "Downloading yt-dlp: {:.1} MB / {:.1} MB",
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0
                );
                progress_cb(frac, &msg);
            } else {
                let msg = format!(
                    "Downloading yt-dlp: {:.1} MB",
                    downloaded as f64 / 1_048_576.0
                );
                progress_cb(0.5, &msg);
            }
        }

        file.flush().await.map_err(|e| format!("Flush error: {e}"))?;
        drop(file);

        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }
        fs::rename(&temp_path, &target_path).map_err(|e| format!("Rename error: {e}"))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&target_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&target_path, perms);
            }
        }

        progress_cb(1.0, "yt-dlp installation complete!");
        Ok(target_path)
    }

    pub async fn install_ffmpeg<F>(progress_cb: F) -> Result<PathBuf, String>
    where
        F: Fn(f32, &str) + Send + 'static,
    {
        let (url, _) = Self::get_ffmpeg_download_url();
        let target_path = Self::target_ffmpeg_path()?;

        progress_cb(0.05, "Connecting to FFmpeg static releases...");

        let client = reqwest::Client::builder()
            .user_agent("MPD-Desktop/1.0.0")
            .build()
            .map_err(|e| format!("HTTP Client error: {e}"))?;

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed with status: {}", resp.status()));
        }

        let total_size = resp.content_length().unwrap_or(0);
        let gz_temp_path = target_path.with_extension("gz");

        let mut file = tokio::fs::File::create(&gz_temp_path)
            .await
            .map_err(|e| format!("Could not create temp file: {e}"))?;

        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Write error: {e}"))?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let frac = (downloaded as f32 / total_size as f32).clamp(0.0, 0.90);
                let msg = format!(
                    "Downloading FFmpeg: {:.1} MB / {:.1} MB",
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0
                );
                progress_cb(frac, &msg);
            } else {
                let msg = format!(
                    "Downloading FFmpeg: {:.1} MB",
                    downloaded as f64 / 1_048_576.0
                );
                progress_cb(0.5, &msg);
            }
        }

        file.flush().await.map_err(|e| format!("Flush error: {e}"))?;
        drop(file);

        progress_cb(0.92, "Decompressing FFmpeg binary...");

        // Decompress the .gz archive in a blocking task
        let gz_path_clone = gz_temp_path.clone();
        let target_path_clone = target_path.clone();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let gz_file = fs::File::open(&gz_path_clone)
                .map_err(|e| format!("Failed to open downloaded archive: {e}"))?;
            let mut decoder = GzDecoder::new(gz_file);

            let temp_extracted = target_path_clone.with_extension("extracted_tmp");
            let mut out_file = fs::File::create(&temp_extracted)
                .map_err(|e| format!("Failed to create destination file: {e}"))?;

            std::io::copy(&mut decoder, &mut out_file)
                .map_err(|e| format!("Failed during decompression: {e}"))?;

            drop(out_file);
            let _ = fs::remove_file(&gz_path_clone);

            if target_path_clone.exists() {
                let _ = fs::remove_file(&target_path_clone);
            }
            fs::rename(&temp_extracted, &target_path_clone)
                .map_err(|e| format!("Failed to place executable: {e}"))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(&target_path_clone) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&target_path_clone, perms);
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| format!("Worker thread error: {e}"))??;

        progress_cb(1.0, "FFmpeg installation complete!");
        Ok(target_path)
    }
}
