# File Orchestrator

Smart file synchronization system that automatically watches a folder and syncs files to USB drives based on categories.

## Features

- Automatic file classification by type (images, videos, music, documents, archives)
- Real-time file watching with smart queue system
- Duplicate detection using BLAKE3 hashing
- GUI and CLI interfaces
- USB drive management with category-based syncing

## Requirements

- Rust 1.70 or later
- GTK3 libraries (for GUI mode)
- libnotify (for notifications)

## Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/Ravi-Wijerathne/orchestrator.git
   cd orchestrator
   ```

2. **Check dependencies**
   ```bash
   ./check-deps.sh          # Linux/macOS
   python check-deps.py     # Windows
   ```

3. **Build the project**
   ```bash
   cargo build --release --features gui
   ```

4. **Run first-time setup**
   ```bash
   ./start.sh          # Linux/macOS
   python start.py     # Windows
   ```
   This will prompt for your storage folder path and create the configuration file.

## Usage

### GUI Mode
```bash
./target/release/fo --gui
```

### CLI Mode
```bash
# Initialize configuration
fo init

# Register a USB drive
fo register-drive --label "MyUSB" --category images

# List registered drives
fo list-drives

# Start file watcher
fo run

# One-time sync
fo sync-once
```

## Configuration

Edit `config.toml` to customize:

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

## License

Dual-licensed under MIT and Apache License 2.0
