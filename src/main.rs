#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod engine;
mod ui;

use app::MpdApp;
use config::AppConfig;
use engine::models::QualityPreset;
use engine::ytdlp::YtDlpEngine;

fn create_settings_app_icon() -> eframe::egui::IconData {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    let cx = 15.5f32;
    let cy = 15.5f32;
    let base_r = 10.5f32;
    let tooth_h = 3.5f32;
    let inner_r = 4.5f32;
    let num_teeth = 6.0f32;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let tooth = ((angle * num_teeth).sin()).max(0.0);
            let outer_r = base_r + tooth * tooth_h;

            if dist >= inner_r && dist <= outer_r {
                rgba.extend_from_slice(&[41, 128, 185, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    eframe::egui::IconData {
        rgba,
        width,
        height,
    }
}

fn print_usage() {
    println!(
        r#"MPD (Multi-Protocol Downloader) v1.0.0
Fast, cross-platform media downloader desktop application and CLI

Usage:
  mpd                                  Launch the cross-platform desktop GUI
  mpd <URL> [OPTIONS]                  Download directly via command line

Options:
  --preset <preset>   Quality preset: best, 1080p, 720p, 480p, audio (default: best)
  --format <id>       Specific format ID from yt-dlp
  --output <dir>      Destination directory
  --audio-only        Extract audio to MP3
  --help, -h          Show this help message
"#
    );
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();

    // Check if CLI flags or a URL is passed
    if args.len() > 1 {
        let first_arg = &args[1];
        if first_arg == "--help" || first_arg == "-h" {
            print_usage();
            return Ok(());
        }

        if first_arg.starts_with("http://") || first_arg.starts_with("https://") {
            return run_cli(&args);
        }
    }

    // Launch GUI Application
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime");

    let runtime_handle = rt.handle().clone();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("MPD - Multi-Protocol Downloader")
            .with_inner_size([1000.0, 720.0])
            .with_min_inner_size([820.0, 580.0])
            .with_icon(create_settings_app_icon())
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "MPD - Multi-Protocol Downloader",
        native_options,
        Box::new(move |cc| Ok(Box::new(MpdApp::new(cc, runtime_handle)))),
    )
}

fn run_cli(args: &[String]) -> Result<(), eframe::Error> {
    let url = &args[1];
    let mut preset = QualityPreset::BestVideoAudio;
    let mut custom_format = None;
    let mut extract_audio = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--preset" if i + 1 < args.len() => {
                i += 1;
                match args[i].to_lowercase().as_str() {
                    "1080p" => preset = QualityPreset::Fhd1080p,
                    "720p" => preset = QualityPreset::Hd720p,
                    "480p" => preset = QualityPreset::Sd480p,
                    "audio" => preset = QualityPreset::BestAudio,
                    _ => preset = QualityPreset::BestVideoAudio,
                }
            }
            "--format" if i + 1 < args.len() => {
                i += 1;
                custom_format = Some(args[i].clone());
                preset = QualityPreset::Custom;
            }
            "--audio-only" => {
                extract_audio = true;
            }
            _ => {}
        }
        i += 1;
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime");

    rt.block_on(async {
        let config = AppConfig::load();
        let engine = YtDlpEngine::detect(&config);

        if !engine.is_available() {
            eprintln!("Error: yt-dlp is not installed or available on PATH.");
            eprintln!("Launch the GUI application to auto-install it with one click.");
            return;
        }

        println!("Fetching media info for: {}", url);
        match engine.fetch_info(url).await {
            Ok(info) => {
                println!("Title: {}", info.title);
                if let Some(uploader) = info.uploader.or(info.channel) {
                    println!("Uploader: {}", uploader);
                }
                println!("Starting download to {}...", config.download_dir.display());

                let (status_tx, mut status_rx) = tokio::sync::mpsc::unbounded_channel();
                let (_cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);

                tokio::spawn(async move {
                    while let Some(status) = status_rx.recv().await {
                        match status {
                            engine::models::DownloadStatus::Downloading(p) => {
                                print!("\rProgress: {:.1}% | Speed: {} | ETA: {}", p.percent, p.speed, p.eta);
                            }
                            engine::models::DownloadStatus::PostProcessing(msg) => {
                                println!("\nProcessing: {}", msg);
                            }
                            engine::models::DownloadStatus::Completed(path) => {
                                println!("\nDownload completed: {}", path.display());
                            }
                            engine::models::DownloadStatus::Failed(e) => {
                                eprintln!("\nDownload failed: {}", e);
                            }
                            _ => {}
                        }
                    }
                });

                let res = engine
                    .download(
                        url,
                        preset,
                        custom_format.as_deref(),
                        &config.download_dir,
                        extract_audio,
                        config.embed_thumbnail,
                        config.embed_metadata,
                        status_tx,
                        cancel_rx,
                    )
                    .await;

                if let Err(e) = res {
                    eprintln!("Download error: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch info: {e}");
            }
        }
    });

    Ok(())
}
