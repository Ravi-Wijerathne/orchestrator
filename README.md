# File Orchestrator

A smart file synchronization system that automatically watches a folder and syncs files to USB drives based on categories.

## Why this project?

File Orchestrator exists to solve the problem of managing and backing up files across multiple USB drives efficiently. Instead of manually copying files to different drives, this tool automatically monitors your storage folder and syncs files to the appropriate USB drives based on file type categories (images, videos, documents, etc.).

## Features

- **Automatic file classification** - Files are categorized by type (images, videos, music, documents, archives)
- **Real-time file watching** - Monitors source folder for new or modified files
- **Smart queue system** - Queues files for offline drives and syncs when they reconnect
- **Duplicate detection** - Uses BLAKE3 hashing to prevent redundant copies
- **Dual interface** - Choose between GUI or CLI modes based on your preference
- **USB drive management** - Register drives with specific categories for targeted syncing
- **One-time or continuous sync** - Run in daemon mode or perform one-time syncs

## Requirements

### Operating System
- Linux (tested on Ubuntu/Debian-based systems)
- Support for other Unix-like systems possible

### Programming Language/Runtime
- Rust 1.70 or later
- Cargo package manager

### External Tools or Libraries
- GTK3 libraries (for GUI mode)
- libnotify (for system notifications)
- Standard Unix tools

## Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/Ravi-Wijerathne/orchestrator.git
   cd orchestrator
   ```

2. **Check dependencies**
   ```bash
   ./check-deps.sh
   ```

3. **Build the project**
   ```bash
   # Build with GUI support (default)
   cargo build --release --features gui
   
   # Or build CLI-only version
   cargo build --release
   ```

4. **Install (optional)**
   ```bash
   # Copy binary to system path
   sudo cp target/release/fo /usr/local/bin/
   ```

## Usage

### First-Time Setup

Run the start script which will guide you through initial setup:
```bash
./start.sh
```

This will:
1. Build the project if needed
2. Prompt for your storage folder path
3. Launch the GUI interface

### GUI Mode

Launch the graphical interface:
```bash
# Using start script (recommended)
./start.sh

# Direct execution
./target/release/fo --gui

# Or using gui subcommand
./target/release/fo gui
```

**GUI Features:**
- **Dashboard** - View system status, start/stop the file watcher
- **Drive Manager** - Register and unregister USB drives
- **Settings** - View and modify configuration

### CLI Mode

```bash
# View all available commands
fo --help

# Initialize configuration
fo init

# Register a USB drive with a category
fo register-drive --label "MyUSB" --category images

# List all registered drives
fo list-drives

# List currently connected drives
fo list-connected

# Start the file watcher (syncs existing files first, then monitors for new files)
fo run

# Perform one-time sync
fo sync-once

# Check system status
fo status
```

## Configuration

The configuration file is located at `config.toml` in the project root. You can customize:

### Source Folder
```toml
[source]
path = "/home/user/MainStorage"
```

### File Category Rules
```toml
[rules]
images = ["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp"]
videos = ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm"]
music = ["mp3", "flac", "wav", "aac", "ogg", "m4a"]
documents = ["pdf", "doc", "docx", "txt", "odt", "rtf", "md"]
archives = ["zip", "rar", "7z", "tar", "gz", "bz2"]
```

### Example Configuration
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

## Examples

### Example 1: Register a drive for images
```bash
fo register-drive --label "PhotoBackup" --category images
```

### Example 2: Start continuous monitoring
```bash
fo run
```

### Example 3: One-time sync of all queued files
```bash
fo sync-once
```

### Example 4: Check what drives are connected
```bash
fo list-connected
```
**Sample Output:**
```
Connected USB Drives:
- PhotoBackup (images)
- VideoArchive (videos)
```

## Project Structure

```
orchestrator/
├── src/
│   ├── main.rs              # Entry point
│   ├── error.rs             # Error types
│   ├── classifier/          # File type classification
│   │   └── mod.rs
│   ├── cli/                 # CLI interface
│   │   └── mod.rs
│   ├── config/              # Configuration management
│   │   └── mod.rs
│   ├── drive/               # USB drive management
│   │   └── mod.rs
│   ├── gui/                 # GUI implementation
│   │   └── mod.rs
│   ├── state/               # State management & database
│   │   └── mod.rs
│   ├── sync/                # File synchronization logic
│   │   └── mod.rs
│   └── watcher/             # File system watcher
│       └── mod.rs
├── icons/                   # Application icons
├── Cargo.toml               # Rust dependencies
├── config.toml              # Runtime configuration
├── config.example.toml      # Example configuration
├── check-deps.sh            # Dependency checker script
├── start.sh                 # Startup script
└── README.md                # This file
```

## Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository** on GitHub
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add some amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Contribution Guidelines
- Follow Rust coding conventions
- Add tests for new features
- Update documentation as needed
- Ensure all tests pass before submitting PR

## Testing

Run the test suite:
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test classifier

# Run integration tests
cargo test --test '*'
```

## Roadmap

- [ ] **Cloud storage integration** - Support for Dropbox, Google Drive
- [ ] **Web interface** - Browser-based management UI
- [ ] **Mobile notifications** - Push notifications to phone
- [ ] **Compression options** - Automatic compression before sync
- [ ] **Encryption support** - Encrypt files on USB drives
- [ ] **Multi-platform support** - Windows and macOS compatibility
- [ ] **Advanced filtering** - Custom rules based on file size, date, etc.
- [ ] **Sync profiles** - Different sync strategies for different use cases

## Known Issues

- **Large file handling** - Very large files (>10GB) may cause memory issues
- **Network drives** - Limited support for network-mounted drives
- **Hot-plugging** - Requires manual refresh in some cases when USB is connected
- **Case sensitivity** - File extension matching is case-sensitive on Linux

## FAQ

**Q: Can I use this with network drives?**  
A: Currently, the tool is optimized for USB drives. Network drive support is limited and may not work reliably.

**Q: What happens if I disconnect a USB drive during sync?**  
A: The sync will be interrupted and the file will be added back to the queue for that drive.

**Q: Can I sync the same file to multiple drives?**  
A: Yes, register multiple drives with the same category and the file will sync to all of them.

**Q: Does it delete files from the source folder?**  
A: No, files are only copied, never moved or deleted from the source.

**Q: How does duplicate detection work?**  
A: Files are hashed using BLAKE3. If a file with the same hash exists on the target drive, it won't be copied again.

**Q: Can I customize file categories?**  
A: Yes, edit the `config.toml` file to add or modify file extensions for each category.

## License

This project is dual-licensed under:
- MIT License
- Apache License 2.0

You may choose either license for your use.

## Authors

- **Ravi Wijerathne** - *Initial work* - [Ravi-Wijerathne](https://github.com/Ravi-Wijerathne)

---

**Star this repository** if you find it useful! ⭐
