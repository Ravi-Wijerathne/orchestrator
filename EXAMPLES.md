# 📚 Usage Examples

## Example 1: Basic Setup

```bash
# Step 1: Build the project
cargo build --release

# Step 2: Create default configuration
./target/release/file-orchestrator init

# Step 3: List connected drives to see available drives
./target/release/file-orchestrator list-connected

# Output example:
# === Connected Drives ===
# 
# Drive: C:\
#   Mount Point: C:\
#   Total Space: 500 GB
#   Available: 250 GB
#   File System: NTFS
#   Removable: false
#   Drive ID: drive-1a2b3c4d5e6f
#
# Drive: E:\
#   Mount Point: E:\
#   Total Space: 64 GB
#   Available: 60 GB
#   File System: FAT32
#   Removable: true
#   Drive ID: drive-9f8e7d6c5b4a

# Step 4: Register your USB drives
./target/release/file-orchestrator register-drive --label "E:" --category images
./target/release/file-orchestrator register-drive --label "USB_Videos" --category videos
./target/release/file-orchestrator register-drive --label "MUSIC_USB" --category music
```

## Example 2: Configure Source Directory

Edit `config.toml`:

```toml
[source]
path = "C:/Users/YourName/Documents/MediaLibrary"

[rules]
images = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"]
videos = ["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm"]
music = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma"]
documents = ["pdf", "doc", "docx", "txt", "xlsx", "pptx"]
archives = ["zip", "rar", "7z", "tar", "gz"]

[drives]
# Your registered drives will appear here
```

## Example 3: One-Time Sync

```bash
# Sync all files once
./target/release/file-orchestrator sync-once

# Output:
# [INFO] Processing file: C:/Users/YourName/Documents/MediaLibrary/photo.jpg
# [INFO] Copying C:/Users/.../photo.jpg -> E:/images/photo.jpg
# [INFO] Successfully synced: C:/Users/.../photo.jpg
# 
# === Sync Summary ===
# Total files: 150
# Synced: 120
# Already synced: 20
# Pending: 5
# Skipped: 3
# Failed: 2
# ====================

# Sync a specific file
./target/release/file-orchestrator sync-once --file "C:/Users/YourName/Documents/MediaLibrary/video.mp4"
```

## Example 4: Watch Mode (Continuous Monitoring)

```bash
# Start the orchestrator in watch mode
./target/release/file-orchestrator run

# Output:
# [INFO] Starting File Orchestrator...
# [INFO] Watching: C:/Users/YourName/Documents/MediaLibrary
# ✓ File Orchestrator is running. Press Ctrl+C to stop.
#   Watching for file changes in: C:/Users/YourName/Documents/MediaLibrary
# [INFO] File created: C:/Users/YourName/Documents/MediaLibrary/newphoto.jpg
# [INFO] Copying C:/Users/.../newphoto.jpg -> E:/images/newphoto.jpg
# [INFO] Successfully synced: C:/Users/.../newphoto.jpg

# With custom drive check interval (check every 30 seconds)
./target/release/file-orchestrator run --interval 30
```

## Example 5: Check Status

```bash
./target/release/file-orchestrator status

# Output:
# === File Orchestrator Status ===
# Total files synced: 1250
# Total size: 25600 MB
# Pending syncs: 15
# 
# By category:
#   images: 450
#   videos: 230
#   music: 520
#   documents: 50
# 
# ================================
```

## Example 6: Process Pending Syncs

```bash
# Scenario: You've been working offline, and now you plug in your USB drives
./target/release/file-orchestrator process-pending

# Output:
# [INFO] Checking for connected drives and processing pending syncs...
# [INFO] Drive ImageUSB is connected, checking for pending syncs
# [INFO] Processing 15 pending syncs for drive 550e8400-e29b-41d4-a716-446655440000
# [INFO] Synced pending file: C:/Users/.../photo1.jpg
# [INFO] Synced pending file: C:/Users/.../photo2.jpg
# ...
# [INFO] Processed 15 pending syncs for ImageUSB
# ✓ Finished processing pending syncs
```

## Example 7: List Registered Drives

```bash
./target/release/file-orchestrator list-drives

# Output:
# === Registered Drives ===
# 
# UUID: 550e8400-e29b-41d4-a716-446655440000
#   Label: ImageUSB
#   Category: images
#   Last Seen: 2025-11-09 14:30:00
# 
# UUID: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
#   Label: VideoUSB
#   Category: videos
#   
# UUID: 6ba7b814-9dad-11d1-80b4-00c04fd430c8
#   Label: MusicUSB
#   Category: music
# 
# ========================
```

## Example 8: Validate Configuration

```bash
./target/release/file-orchestrator validate

# Output if valid:
# ✓ Configuration is valid
# 
# Source directory: C:/Users/YourName/Documents/MediaLibrary
# Registered drives: 3

# Output if invalid:
# Error: Configuration error: Source path does not exist: C:/InvalidPath
```

## Example 9: Clear Sync History (Dangerous!)

```bash
# This will delete all sync history and pending syncs
./target/release/file-orchestrator clear --confirm

# Output:
# ✓ Cleared all sync state

# Without --confirm:
./target/release/file-orchestrator clear
# Output:
# Error: This will delete all sync history. Use --confirm to proceed.
```

## Example 10: Debug Mode

```bash
# Run with verbose logging for troubleshooting
$env:RUST_LOG="debug"
./target/release/file-orchestrator run

# On Linux/macOS:
RUST_LOG=debug ./target/release/file-orchestrator run

# Output includes detailed debug information:
# [DEBUG] Loading configuration from: config.toml
# [DEBUG] Opening state database at: .orchestrator.db
# [DEBUG] Checking if drive is connected: E:\
# [DEBUG] File hash: 1a2b3c4d5e6f7g8h9i0j...
# ...
```

## Example 11: Real-World Workflow

```bash
# Morning: Start the orchestrator
./target/release/file-orchestrator run

# Throughout the day:
# - Take photos with your camera
# - Copy them to C:/Users/YourName/Documents/MediaLibrary
# - Orchestrator automatically syncs them to ImageUSB

# Afternoon: Unplug ImageUSB to bring photos somewhere
# - Orchestrator detects USB removal
# - New photos get queued in pending syncs

# Evening: Plug ImageUSB back in
# - Orchestrator detects USB reconnection
# - Automatically syncs all pending photos

# Anytime: Check status
./target/release/file-orchestrator status
```

## Example 12: Running as a Background Service (Windows)

Create a batch file `start-orchestrator.bat`:

```batch
@echo off
cd C:\path\to\orchestrator
target\release\file-orchestrator.exe run
```

Then:
1. Open Task Scheduler
2. Create Basic Task
3. Name: "File Orchestrator"
4. Trigger: "When I log on"
5. Action: "Start a program"
6. Program: `C:\path\to\orchestrator\start-orchestrator.bat`

## Example 13: Custom File Rules

Edit `config.toml` to add custom file types:

```toml
[rules]
images = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "heic", "raw", "cr2", "nef"]
videos = ["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm", "m4v", "mpg", "mpeg", "3gp", "mts"]
music = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "alac", "ape"]
documents = ["pdf", "doc", "docx", "txt", "rtf", "odt", "xlsx", "xls", "pptx", "ppt", "csv", "md"]
archives = ["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "dmg"]
```

## Example 14: Multiple Source Folders (Advanced)

Currently, the tool supports one source directory. For multiple sources, you can:

**Option 1: Use symbolic links**
```powershell
# Windows
mklink /D C:\MediaLibrary\Photos "D:\Camera"
mklink /D C:\MediaLibrary\Videos "E:\Videos"
```

**Option 2: Run multiple instances**
```bash
# Instance 1 for Photos
./target/release/file-orchestrator run --config photos.toml

# Instance 2 for Videos (in another terminal)
./target/release/file-orchestrator run --config videos.toml
```

## Tips & Best Practices

1. **Always validate**: Run `validate` after editing config
2. **Test first**: Use `sync-once` before `run`
3. **Monitor logs**: Check terminal output for errors
4. **Regular backups**: The tool doesn't delete source files
5. **Unique labels**: Use distinct drive labels for easy identification
6. **Path format**: Use forward slashes `/` even on Windows

---

For more information, see the main [README.md](README.md)
