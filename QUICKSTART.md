# 🚀 Quick Start Guide

## Step-by-Step Setup (5 minutes)

### 1️⃣ Build the Project

```bash
cd orchestrator
cargo build --release
```

This will take a few minutes the first time as it downloads and compiles all dependencies.

### 2️⃣ Create Configuration

```bash
# Run the compiled binary
./target/release/file-orchestrator init

# Or if installed globally:
file-orchestrator init
```

This creates a `config.toml` file.

### 3️⃣ Configure Your Source Directory

Open `config.toml` and update the source path:

```toml
[source]
path = "C:/Users/YourName/Documents/MainStorage"  # Windows
# or
path = "/home/yourname/MainStorage"               # Linux
# or  
path = "/Users/yourname/MainStorage"              # macOS
```

### 4️⃣ Connect Your USB Drives

Plug in your USB drives and check what's connected:

```bash
file-orchestrator list-connected
```

### 5️⃣ Register Your Drives

For each USB drive you want to use, register it with a category:

```bash
# For images
file-orchestrator register-drive --label "MyImageUSB" --category images

# For videos
file-orchestrator register-drive --label "MyVideoUSB" --category videos

# For music
file-orchestrator register-drive --label "MyMusicUSB" --category music
```

**Tip**: The label can be any part of the drive name you see in `list-connected`.

### 6️⃣ Verify Configuration

```bash
file-orchestrator list-drives
file-orchestrator validate
```

### 7️⃣ Test with One-Time Sync

Before running in watch mode, test with a one-time sync:

```bash
file-orchestrator sync-once
```

### 8️⃣ Run in Watch Mode

Now start the orchestrator to continuously monitor for changes:

```bash
file-orchestrator run
```

The tool will now:
- ✅ Watch your source directory for new/changed files
- ✅ Automatically classify and sync them
- ✅ Queue files when USB drives are offline
- ✅ Auto-sync when drives are reconnected

## 💡 Common Scenarios

### Scenario 1: Adding new photos while USB is connected

1. Copy photos to your source directory
2. Orchestrator detects them automatically
3. Photos are synced to ImageUSB immediately

### Scenario 2: Adding videos while USB is disconnected

1. Copy videos to your source directory
2. Orchestrator adds them to pending queue
3. When you plug in VideoUSB, they sync automatically

### Scenario 3: Checking what's queued

```bash
file-orchestrator status
```

Shows:
- Total files synced
- Pending syncs
- Statistics by category

### Scenario 4: Manual sync of pending files

```bash
file-orchestrator process-pending
```

## 🎯 Pro Tips

1. **Run as a service**: Use Windows Task Scheduler, systemd (Linux), or launchd (macOS) to run on startup

2. **Custom check intervals**: 
   ```bash
   file-orchestrator run --interval 30  # Check every 30 seconds
   ```

3. **Test specific files**:
   ```bash
   file-orchestrator sync-once --file "/path/to/test.jpg"
   ```

4. **Debug mode**:
   ```bash
   RUST_LOG=debug file-orchestrator run
   ```

## 🆘 Troubleshooting

### "Source path does not exist"
- Check the path in `config.toml`
- Use forward slashes (/) even on Windows

### "No drive configured for category"
- Run `file-orchestrator list-drives` to see registered drives
- Register the drive using `register-drive` command

### "Drive not detected"
- Run `file-orchestrator list-connected` to see available drives
- Make sure the USB is properly mounted
- Try registering with the exact path: `--path "E:/"`

## 📱 Next Steps

After setup:
1. ⭐ Add to startup to run automatically
2. 📊 Monitor with `status` command
3. 🔧 Customize file rules in `config.toml`
4. 📖 Read full README.md for advanced features

---

**Need help?** Check the main README.md or create an issue on GitHub.
