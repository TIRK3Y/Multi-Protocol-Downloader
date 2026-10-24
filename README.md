# MPD (Multi-Protocol Downloader)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-green.svg?style=flat-square)](#cross-platform-compilation)
[![Release](https://img.shields.io/badge/Release-v1.0.0-blueviolet.svg?style=flat-square)](#)
[![Open Source](https://img.shields.io/badge/Open%20Source-100%25-brightgreen.svg?style=flat-square)](#)

MPD (Multi-Protocol Downloader) is a modern, high-performance, open-source desktop computer software and command-line utility built in Rust. It enables fast, reliable multimedia extraction and torrent stream acquisition across more than 1,000+ websites and streaming protocols (HTTP, HLS/m3u8, DASH/mpd, RTMP, and BitTorrent/Magnet).

Engineered with a native, GPU-accelerated graphical interface (eframe / egui), MPD eliminates manual terminal configuration by providing built-in 1-click automated installers for both yt-dlp and FFmpeg, alongside an embedded pure-Rust BitTorrent/DHT client engine across Windows, macOS, and Linux.

---

## User Responsibility and Legal Compliance Notice

> [!IMPORTANT]
> User Responsibility Clause:
> - MPD is an open-source technical tool developed for archiving, educational analysis, research, and authorized personal media management.
> - No Content Hosting: MPD does not host, mirror, index, or distribute any video, audio, or media content. All network connections are made directly from the user's local client to the respective remote endpoints.
> - Copyright and Compliance: Users are solely responsible for ensuring that their downloading activities respect intellectual property rights, local copyright laws, fair-use boundaries, and the applicable Terms of Service of each content provider.
> - Disclaimer of Warranty: The developers, maintainers, and contributors accept no liability for any unauthorized copying, copyright infringement, or misuse of this software. By using MPD, you agree to assume full legal responsibility for your actions.

---

## Features

- Native Desktop GUI and Responsive Layout:
  - Pure Rust desktop client powered by eframe and egui (zero WebKit or Node.js runtime bloat).
  - Collapsible and expandable left navigation sidebar with compact indicator modes.
  - Full theme engine: Dark Mode, Light Mode, and System Default Mode.
  - Professional spacing, clean typography, and zero emoji clutter.
- Dual Protocol Ingestion Engine:
  - Web and Media Streams: Download video and audio from YouTube, Vimeo, SoundCloud, TikTok, Twitter/X, Instagram, Facebook, Reddit, Twitch, and 1,000+ extractors.
  - BitTorrent and Magnet Protocol: Download files, datasets, software ISOs, and videos directly using magnet links or 40-character hex InfoHashes via native pure-Rust DHT and uTP peer networking.
- 1-Click Engine Auto-Installer (yt-dlp and FFmpeg):
  - yt-dlp: Automatically downloads, configures, and verifies the standalone engine executable if missing.
  - FFmpeg: Automatically downloads, decompresses, and activates static FFmpeg binaries on Windows, macOS, and Linux, eliminating the common "Postprocessing: ffmpeg not found" error with zero manual path setup.
- Live Telemetry and Thumbnail Inspection:
  - Asynchronously inspects title, uploader/channel, duration, formats, and renders the video thumbnail preview directly in the application.
- Quality Presets and Advanced Stream Selector:
  - Best Quality (Merged Video and Audio)
  - 1080p Full HD
  - 720p HD
  - 480p SD
  - Audio Only (MP3 Conversion)
  - Custom Format Explorer: Filter streams by Video + Audio, Audio Only, or Video Only, inspecting codec, bitrate, resolution, container, and file size.
- Real-time Download Metrics:
  - Live progress percentage bar, download speed (MB/s or KB/s), estimated time of arrival (ETA), and downloaded vs total bytes.
  - Active peer count telemetry for BitTorrent downloads.
- Real-time Cancellation:
  - Gracefully abort active downloads at any instant with clean subprocess termination and session cleanup.
- Native Operating System Integration:
  - Native folder chooser dialogs via rfd.
  - 1-click "Play / Open File" and "Show in Folder" using default system handlers via open.
  - 1-click native clipboard paste button.
- Grouped Settings Configuration:
  - Clean dropdown category selector: Appearance & Theme, Engines & Binaries, Download Preferences & Storage, and About & Legal.
- Headless Command-Line Interface (CLI):
  - Launch the full visual desktop software, or execute headless downloads directly from terminal scripts.

---

## Project Architecture

```
mpd/
|-- .github/workflows/
|   `-- release.yml             # Cross-platform release matrix (Windows, macOS, Linux)
|-- src/
|   |-- main.rs                 # CLI parser and native desktop window entrypoint
|   |-- app.rs                  # Collapsible left sidebar, theme engine, and tab shell
|   |-- config.rs               # Persistent settings, theme mode, and history manager
|   |-- engine/
|   |   |-- installer.rs        # 1-click automated downloader for yt-dlp and FFmpeg
|   |   |-- models.rs           # MediaInfo, FormatItem, QualityPreset, Progress models
|   |   |-- torrent.rs          # Embedded pure-Rust BitTorrent / Magnet session manager
|   |   |-- ytdlp.rs            # Async subprocess manager and stdout stream parser
|   |   `-- mod.rs              # Engine module declarations
|   `-- ui/
|       |-- about_view.rs       # Dedicated about page with specifications and environment details
|       |-- download_view.rs    # Dual protocol downloader (Distinct Stream and Magnet cards)
|       |-- history_view.rs     # Download log with quick file and folder launchers
|       |-- home_view.rs        # Landing workspace ready for future dashboard metrics
|       |-- icons.rs            # Lucide-style vector drawing primitives (zero emojis)
|       |-- platforms_view.rs   # 1,000+ supported sites catalog and search
|       |-- settings_view.rs    # Sub-section settings views and live engine telemetry dashboard
|       `-- mod.rs              # UI module declarations
|-- Cargo.toml                  # Cargo manifest and release optimizations
|-- CODE_OF_CONDUCT.md          # Contributor Covenant v2.1
|-- CONTRIBUTING.md             # Developer setup, coding rules, and PR guidelines
|-- LICENSE                     # MIT License
`-- SECURITY.md                 # Security and vulnerability reporting policy
```

---

## Quick Start

### 1. Prerequisites

- Rust and Cargo (version 1.75 or later)
- Git

### 2. Building and Launching

```bash
# Clone the repository
git clone https://github.com/<your-username>/mpd.git
cd mpd

# Launch the Desktop GUI application
cargo run --release
```

### 3. Headless Command-Line Interface (CLI)

MPD automatically detects whether you pass command-line arguments:

```bash
# Display help and CLI options
cargo run --release -- --help

# Download highest quality video and audio
cargo run --release -- "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Download with a specific resolution preset (1080p, 720p, 480p)
cargo run --release -- "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --preset 1080p

# Extract audio directly to MP3
cargo run --release -- "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --preset audio

# Download with a custom format ID
cargo run --release -- "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --format 137
```

---

## Cross-Platform Compilation

### Windows (MSVC)
```powershell
cargo build --release
# Standalone executable located at: target/release/mpd.exe
```

### macOS (Apple Silicon and Intel)
```bash
# Apple Silicon (M1/M2/M3/M4)
cargo build --release --target aarch64-apple-darwin

# Intel Macs (x86_64)
cargo build --release --target x86_64-apple-darwin
```

### Linux (Ubuntu / Debian / Fedora / Arch)
Install native GUI development dependencies:

```bash
# Ubuntu / Debian
sudo apt-get update
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libasound2-dev libfontconfig1-dev

# Fedora
sudo dnf install -y libxcb-devel libxkbcommon-devel alsa-lib-devel fontconfig-devel

# Build release
cargo build --release
```

Automated cross-platform binaries are published for each GitHub release via [.github/workflows/release.yml](.github/workflows/release.yml).

---

## Contributing

We welcome contributions from the open-source community:
- Review our [Contributing Guidelines](CONTRIBUTING.md) to get started with development setup and PR requirements.
- Review our [Code of Conduct](CODE_OF_CONDUCT.md) to understand our community standards.
- To report vulnerabilities responsibly, refer to our [Security Policy](SECURITY.md).

---

## License

This project is licensed under the [MIT License](LICENSE).

```
Copyright (c) 2026 MPD Contributors
```

You are free to use, modify, distribute, and integrate this software within personal, educational, or commercial projects in accordance with the terms of the MIT License.
