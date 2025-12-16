# File Orchestrator

Automatic file synchronization to USB drives. Watches a folder and syncs files to USB drives by category.

## Features

- Automatic file classification by type
- Real-time file watching
- Queue system for offline drives
- Duplicate detection (BLAKE3 hashing)
- GUI and CLI modes

## Quick Start

### Prerequisites Check

```bash
./check-deps.sh
```

### First Run

```bash
./start.sh
```

This will:
1. Build if needed
2. Prompt for storage folder path
3. Launch GUI

## Usage

### GUI Mode

Start GUI and use the interface:
- **Dashboard**: View status, Start/Stop watcher
- **Drive Manager**: Register/Unregister USB drives
- **Settings**: View configuration

Launch GUI using one of these methods:
```bash
# Method 1: Using start script (recommended)
./start.sh

# Method 2: Using --gui flag
./target/release/fo --gui

# Method 3: Using gui subcommand
./target/release/fo gui
```

### CLI Mode

```bash
#For view all Commands
fo --help

# Initialize config
fo init

# Register USB drive
fo register-drive --label "MyUSB" --category images

# List drives
fo list-drives
fo list-connected

# Start watcher (syncs existing files first, then watches for new files)
fo run

# One-time sync
fo sync-once

# Check status
fo status
```

## Configuration

Edit `config.toml`:

```toml
[source]
path = "/home/user/MainStorage"

[rules]
images = ["jpg", "png", "gif"]
videos = ["mp4", "mkv", "avi"]
music = ["mp3", "flac", "wav"]
documents = ["pdf", "docx", "txt"]
archives = ["zip", "rar", "7z"]
```

## File Categories

- **images**: jpg, png, gif, etc.
- **videos**: mp4, mkv, avi, etc.
- **music**: mp3, flac, wav, etc.
- **documents**: pdf, docx, txt, etc.
- **archives**: zip, rar, 7z, etc.

## How It Works

1. Watcher starts and syncs all existing files
2. Monitors folder for new/modified files
3. Classifies files by type (magic bytes)
4. Syncs to correct USB drive by category
5. Queues if drive offline, syncs when reconnected

## Troubleshooting

**Drive not detected:**
```bash
fo list-connected
```

**Files not syncing:**
```bash
fo status  # Check pending syncs
```

**Permission errors:**
- Ensure read access to storage folder
- Ensure write access to USB drives

## Manual Build

```bash
# CLI only
cargo build --release

# With GUI
cargo build --features gui --release
```

## Built With

Rust, Tokio, egui, sled, notify, blake3
