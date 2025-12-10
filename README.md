# File Orchestrator

Automatic file synchronization across multiple storage devices with intelligent classification.

## Features

- **Intelligent File Classification**: Detects file types using magic bytes (MIME types)
- **Automatic Syncing**: Real-time file watching and synchronization
- **Pending Queue**: Queues files when drives are offline, syncs when reconnected
- **GUI & CLI**: Graphical interface or command-line
- **Smart State Management**: BLAKE3 hashing to avoid duplicates
- **Cross-Platform**: Windows, Linux, macOS

## Prerequisites

- Rust 1.70+
- For GUI: Node.js 18+ and npm

## Installation

### Option 1: CLI Only (Recommended for beginners)

```bash
# Clone and setup
git clone https://github.com/Ravi-Wijerathne/orchestrator.git
cd orchestrator

# Run the setup script - handles everything automatically
./start-cli.sh

# The script will:
# - Check dependencies
# - Build the project
# - Offer to install globally
# After installation, use 'fo' command anywhere
```

### Option 2: GUI Version (Full featured)

```bash
# Clone the repository
git clone https://github.com/Ravi-Wijerathne/orchestrator.git
cd orchestrator

# Run complete GUI setup - one command does it all
./complete-setup.sh

# This script will:
# - Install Node.js dependencies
# - Generate icons
# - Build the project with GUI features
# - Create the binary in target/release/

# Launch the GUI anytime with:
./start-gui.sh
```

### Option 3: Manual Build

```bash
# CLI only
cargo build --release

# GUI version
./setup-gui.sh              # Install frontend dependencies
./generate-icons.sh         # Generate app icons (optional)
cargo build --features gui --release

# Install globally (optional)
cargo install --path .
```

## Quick Start

```bash
# 1. Initialize
fo init

# 2. Edit config.toml and set your source directory
[source]
path = "/path/to/your/storage"

# 3. Register drives
fo register-drive --label "MyUSB" --category images

# 4. Start watching
fo run
```

## Configuration

Edit `config.toml`:

```toml
[source]
path = "/path/to/MainStorage"

[rules]
images = ["jpg", "jpeg", "png", "gif"]
videos = ["mp4", "avi", "mov", "mkv"]
music  = ["mp3", "wav", "flac", "aac"]
documents = ["pdf", "doc", "docx", "txt"]
archives = ["zip", "rar", "7z", "tar"]
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Create configuration file |
| `register-drive` | Register a USB drive |
| `list-drives` | List registered drives |
| `list-connected` | List connected drives |
| `sync-once` | One-time sync |
| `run` | Start watch mode |
| `status` | Show statistics |
| `process-pending` | Process queued syncs |
| `validate` | Validate configuration |
| `clear` | Clear sync history |

## How It Works

1. Monitors source directory for file changes
2. Classifies files by type using magic bytes
3. Copies to corresponding USB drive if connected
4. Queues files if drive is offline
5. Auto-syncs when drive reconnects

## License

MIT License
