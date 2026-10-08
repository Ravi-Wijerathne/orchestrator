# File Orchestrator

A smart desktop file synchronization application built in Rust that monitors a storage directory and automatically synchronizes incoming files to designated USB drives based on file categories.

---

## Features

- **Welcome Onboarding Screen**: First-time user friendly landing page guiding you through initial setup.
- **Modern Graphical Interface**: Clean, responsive desktop dashboard built with `egui`.
- **Automatic File Classification**: Categorizes files (images, videos, music, documents, archives) by MIME type and extensions.
- **Real-Time Directory Watching**: Monitor source folders and automatically queue changes.
- **Drive Manager**: Detect, register, and manage USB drives directly through the visual interface.
- **Smart Queue & Syncing**: Syncs files to matching USB drives automatically once plugged in.
- **Duplicate Prevention**: Uses BLAKE3 cryptographic hashing to track files and avoid duplicate transfers.
- **Cross-Platform**: Works seamlessly on Windows, Linux, and macOS.

---

## Prerequisites

- **Rust toolchain** (1.70 or later): Install via [rustup.rs](https://rustup.rs/)
  ```bash
  rustc --version
  cargo --version
  ```
- **Windows Users**: MSVC C++ Build Tools installed.
  - Download and install [Build Tools for Visual Studio 2019 or later](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
  - During installation, select the **"Desktop development with C++"** workload to ensure `link.exe` is available.

---

## Quick Start: Building & Launching

Follow these steps to build and run File Orchestrator.

### Step 1: Clone the Repository

```bash
git clone https://github.com/Ravi-Wijerathne/orchestrator.git
cd orchestrator
```

### Step 2: Build the Application

Build the release binary with GUI support:

```bash
cargo build --release --features gui
```

> The compiled executable will be located in the `target/release/` directory:
> - **Windows:** `target\release\fo.exe`
> - **Linux / macOS:** `target/release/fo`

---

### Step 3: Launch the GUI

Start the graphical application:

- **Windows (PowerShell / CMD):**
  ```powershell
  .\target\release\fo.exe --gui
  ```
- **Linux / macOS:**
  ```bash
  ./target/release/fo --gui
  ```
- **Directly via Cargo:**
  ```bash
  cargo run --release --features gui -- --gui
  ```

---

## GUI Overview

### 1. Welcome Screen
- On first launch, the app introduces you to the workflow and helps you get started.
- Can be revisited anytime from the sidebar navigation.

### 2. Dashboard
- View active sync statistics and pending file queues.
- Monitor real-time connection status of registered drives.
- Control the background file watcher with **Start Watcher** and **Stop Watcher** buttons.

### 3. Drive Manager
- View all registered USB drives and their assigned categories.
- Register new USB drives with custom labels, category mappings (Images, Videos, Music, Documents, Archives), and folder paths via the built-in folder picker.
- Easily remove drives no longer in use.

### 4. Settings
- View and update the monitored source directory path using the interactive file browser.
- Automatically create missing directories with a single click.
- Inspect supported file extensions and categorization rules.

---

## Configuration

When the GUI starts for the first time without an existing `config.toml`, it automatically creates and loads a default configuration.

You can customize folders and settings directly within the app's **Settings** screen, or manually edit `config.toml`:

```toml
[source]
# The folder monitored for incoming files
path = "C:\\Users\\Username\\MainStorage"   # On Linux/macOS: "/home/user/MainStorage"

[rules]
images = ["jpg", "jpeg", "png", "gif", "webp", "svg"]
videos = ["mp4", "mkv", "avi", "mov", "wmv"]
music = ["mp3", "flac", "wav", "aac", "ogg"]
documents = ["pdf", "docx", "xlsx", "pptx", "txt", "md"]
archives = ["zip", "rar", "7z", "tar", "gz"]

[drives]
# Configured USB drives are stored here automatically when registered in the GUI
```

---

## Running Tests

Run the test suite to verify everything works properly:

```bash
# Run all unit and integration tests
cargo test

# Run tests with output printed to console
cargo test -- --nocapture

# Run tests for specific modules
cargo test classifier
cargo test config
cargo test state
cargo test drive
cargo test watcher
```

---

## Troubleshooting

- **GUI window fails to open:**
  Ensure the application was built with the `gui` feature enabled:
  ```bash
  cargo build --release --features gui
  ```
- **Missing Source Path warning:**
  Navigate to the **Settings** tab in the GUI and use the **Browse...** button to select an existing directory or click **Create This Directory**.
- **USB drive not detected as connected:**
  Ensure the drive is properly mounted on your system and that the path or volume label matches what was registered in the **Drive Manager**.

---

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
