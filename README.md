# File Orchestrator

A smart file synchronization system built in Rust that monitors a storage directory and automatically synchronizes incoming files to designated USB drives based on file categories.

---

## Features

- **Automatic File Classification**: Automatically categorizes files (images, videos, music, documents, archives) by MIME type and extensions.
- **Real-Time Directory Watching**: Detects newly added or modified files instantly.
- **Smart Queue & Syncing**: Syncs files to matching USB drives as soon as they are plugged in.
- **Duplicate Prevention**: Uses BLAKE3 cryptographic hashing to track files and avoid duplicate copies.
- **Modern GUI & Powerful CLI**: Built with `egui` for a responsive desktop interface, plus a full-featured CLI.
- **Cross-Platform**: Works seamlessly on Windows, Linux, and macOS.

---

## Prerequisites

- **Rust toolchain** (1.70 or later): Install via [rustup.rs](https://rustup.rs/)
  ```bash
  rustc --version
  cargo --version
  ```

---

## Quick Start: Building & Launching

Follow these step-by-step instructions to get File Orchestrator running.

### Step 1: Clone the Repository

```bash
git clone https://github.com/Ravi-Wijerathne/orchestrator.git
cd orchestrator
```

### Step 2: Build the Application

You can build with GUI support (recommended) or as a lightweight CLI-only binary:

#### Option A: Build with GUI Support (Recommended)
```bash
cargo build --release --features gui
```
*This compiles the binary with both the graphical dashboard and command-line interfaces.*

#### Option B: Build CLI Only
```bash
cargo build --release
```
*This produces a smaller binary without GUI dependencies.*

> The compiled executable will be located in the `target/release/` directory:
> - **Windows:** `target\release\fo.exe`
> - **Linux / macOS:** `target/release/fo`

---

### Step 3: Configure (Optional for GUI)

The application relies on `config.toml` to define your monitored directory and USB categories.

> **Note for GUI users:** If `config.toml` does not exist, the GUI automatically starts with default settings. You can configure folders and drives directly from the **Settings** tab in the GUI!

To manually create and customize your configuration:

1. Generate a default configuration file:
   ```bash
   # Windows (PowerShell)
   .\target\release\fo.exe init

   # Linux / macOS
   ./target/release/fo init
   ```
   *Alternatively, copy the example template:*
   ```bash
   cp config.example.toml config.toml
   ```

2. Edit `config.toml` to specify your monitored folder path (see [Configuration Guide](#configuration-details)).

---

### Step 4: Launch the Application

#### 1. Launching the GUI Dashboard

You can start the GUI in either of two ways:

- **Using the compiled binary:**
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

#### 2. Launching CLI Commands

Run commands directly using the binary or via `cargo run`:

- **Check status of watcher, drives, and queue:**
  ```bash
  .\target\release\fo.exe status
  ```

- **Register a USB drive for a category:**
  ```bash
  .\target\release\fo.exe register-drive --label "MyUSB" --category images
  ```

- **List all registered drives:**
  ```bash
  .\target\release\fo.exe list-drives
  ```

- **List currently connected USB drives:**
  ```bash
  .\target\release\fo.exe list-connected
  ```

- **Start continuous background watching & syncing:**
  ```bash
  .\target\release\fo.exe run
  ```

- **Run a single sync pass without continuous watching:**
  ```bash
  .\target\release\fo.exe sync-once
  ```

- **Show all available commands and flags:**
  ```bash
  .\target\release\fo.exe --help
  ```

---

## Configuration Details

The `config.toml` file controls how File Orchestrator operates:

```toml
[source]
# The folder to monitor for new files
path = "C:\\Users\\Username\\MainStorage"   # On Linux/macOS: "/home/user/MainStorage"

[rules]
# File extension categories
images = ["jpg", "jpeg", "png", "gif", "webp", "svg"]
videos = ["mp4", "mkv", "avi", "mov", "wmv"]
music = ["mp3", "flac", "wav", "aac", "ogg"]
documents = ["pdf", "docx", "xlsx", "pptx", "txt", "md"]
archives = ["zip", "rar", "7z", "tar", "gz"]

[drives]
# Drives can be registered via CLI or GUI; they will be listed here:
# [drives."usb-uuid-here"]
# label = "Photos Backup"
# target = "images"
```

---

## Running Tests

Run the test suite to verify everything works properly:

```bash
# Run all unit and integration tests
cargo test

# Run tests with output printed to console
cargo test -- --nocapture

# Run tests for a specific module
cargo test classifier
cargo test config
cargo test state
cargo test drive
cargo test watcher
cargo test cli
```

---

## Troubleshooting

- **`Failed to read config file: The system cannot find the file specified`**:
  Run `.\target\release\fo.exe init` or copy `config.example.toml` to `config.toml`. In the GUI version, this is now handled automatically with a built-in default configuration.
- **GUI window fails to open**:
  Ensure you built with `--features gui` (e.g., `cargo build --release --features gui`).
- **USB drive not detected**:
  Verify your drive is mounted with a recognized filesystem label or path using `.\target\release\fo.exe list-connected`.

---

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
