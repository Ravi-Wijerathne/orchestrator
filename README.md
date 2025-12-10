# File Orchestrator

Automatic file synchronization tool for USB drives. Written in Rust.

## Features

- Automatic file classification by type
- Real-time file watching and sync
- Queue system for offline drives
- Smart duplicate detection (BLAKE3)
- GUI and CLI modes

## Quick Start

### Check Dependencies

```bash
./check-deps.sh
```

### First Run

```bash
./start.sh
```

The script will:
1. Build the application (if needed)
2. Guide you through storage folder setup
3. Launch the GUI

### Manual Setup

```bash
# Build
cargo build --features gui --release

# Initialize config
./target/release/fo init

# Edit config.toml to set your storage path

# Run GUI
./target/release/fo --gui

# Or run CLI
./target/release/fo run
```

## Usage

### GUI Mode

- **Dashboard**: View sync status and drive connections
- **Drive Manager**: Register USB drives for file categories
- **Settings**: View configuration

### CLI Mode

```bash
#For all commands
fo --help

# Register a USB drive
fo register-drive --label "MyUSB" --category images

# List drives
fo list-drives
fo list-connected

# Start watching
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

- **images**: Photos and graphics
- **videos**: Video files
- **music**: Audio files
- **documents**: PDFs, office docs
- **archives**: Compressed files

## How It Works

1. Watches your storage folder for new files
2. Classifies files by type (using magic bytes)
3. Syncs to registered USB drive for that category
4. Queues files if drive is offline
5. Auto-syncs when drive reconnects

## Troubleshooting

**Drive not detected**
```bash
fo list-connected  # See available drives
```

**Files not syncing**
```bash
fo status  # Check pending syncs
```

**Permission errors**
- Ensure read access to storage folder
- Ensure write access to USB drives

## Built With

- Rust with Tokio (async runtime)
- egui (GUI)
- sled (embedded database)
- notify (file watching)
- blake3 (hashing)
