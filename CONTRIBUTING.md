# Contributing to MPD (Multi-Protocol Downloader)

Thank you for your interest in contributing to MPD (Multi-Protocol Downloader)! As a fully open-source, community-driven project, we welcome contributions of all kinds: bug reports, documentation improvements, feature enhancements, and code contributions.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Submitting Pull Requests](#submitting-pull-requests)
- [Development Setup](#development-setup)
- [Coding Guidelines](#coding-guidelines)
- [Legal and Responsibility](#legal-and-responsibility)

---

## Code of Conduct

All contributors and community participants are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it to understand the standards of behavior we expect in our community.

---

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to ensure the problem has not already been reported.

When filing a bug report, include:
- A clear, descriptive title.
- Steps to reproduce the behavior.
- Expected behavior vs. actual behavior.
- Operating system and version (Windows 11, macOS Sequoia, Ubuntu 24.04, etc.).
- Any relevant logs or terminal output.

### Suggesting Enhancements

Feature requests are welcome! When opening an enhancement suggestion:
- Use a clear and descriptive title.
- Provide a detailed explanation of the proposed feature and its use case.
- Explain why this enhancement would be useful to the broader MPD community.

### Submitting Pull Requests

1. Fork the repository on GitHub.
2. Clone your fork locally:
   ```bash
   git clone https://github.com/<your-username>/mpd.git
   cd mpd
   ```
3. Create a topic branch:
   ```bash
   git checkout -b feature/my-new-feature
   ```
4. Make your changes cleanly with clear commit messages.
5. Verify formatting and checks:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   cargo build --release
   ```
6. Push to your fork and submit a Pull Request against the main branch.

---

## Development Setup

### Prerequisites

- Rust and Cargo (version 1.75 or later)
- Git

### Linux Build Dependencies (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libasound2-dev libfontconfig1-dev
```

### Running Locally

```bash
# Run in Desktop GUI mode
cargo run

# Run in CLI mode
cargo run -- "https://example.com/video" --preset 1080p
```

---

## Coding Guidelines

- Clean Architecture: Follow the established module separation (engine, ui, config, app).
- Error Handling: Use idiomatic Result<T, E> types without unwrap in user-facing UI flows.
- Asynchronous Execution: Long-running network or disk operations must be dispatched asynchronously to the Tokio runtime to keep the GUI responsive at all times.
- Cross-Platform Compatibility: Avoid platform-specific assumptions or hardcoded path separators; use std::path::PathBuf and standard OS directories.
- Zero Emojis: Do not use emoji characters anywhere in user-facing strings, logs, or documentation.

---

## Legal and Responsibility

By contributing to MPD, you agree that your contributions will be licensed under the project's [MIT License](LICENSE). All contributions must respect intellectual property rights and be original work or appropriately credited under compatible permissive licenses.
