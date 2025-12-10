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

```bash
# Clone and build
git clone <repo-url>
cd orchestrator

# Quick setup using provided script
./start.sh

# Or build manually
cargo build --release

# Install globally
cargo install --path .

# Run GUI (if built with GUI feature)
fo --gui

# Or use CLI commands
fo --help
```

### GUI Setup

```bash
# Setup GUI dependencies and build
./complete-setup.sh

# Or manual setup
./setup-gui.sh
cargo build --features gui --release

# Generate icons (optional)
./generate-icons.sh
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
