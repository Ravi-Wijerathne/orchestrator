mod error;
mod config;
mod classifier;
mod state;
mod drive;
mod sync;
mod watcher;
mod cli;

#[cfg(feature = "gui")]
mod gui;

use cli::{Cli, Commands};
use config::Config;
use state::StateManager;
use sync::SyncManager;
use drive::DriveDetector;
use watcher::{AsyncFileWatcher, FileEvent};
use error::Result;

use tracing::{info, error, Level};
use tracing_subscriber;
use std::path::Path;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() -> Result<()> {
    // Check for --gui flag
    let args: Vec<String> = std::env::args().collect();
    
    #[cfg(feature = "gui")]
    {
        if args.contains(&"--gui".to_string()) {
            // Run GUI mode
            let config_path = args.iter()
                .position(|arg| arg == "--config")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "config.toml".to_string());
            
            let db_path = args.iter()
                .position(|arg| arg == "--db")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "orchestrator.db".to_string());
            
            return gui::run_gui(config_path, db_path);
        }
    }
    
    // Run CLI mode
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(run_cli())
}

async fn run_cli() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // Parse command line arguments
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Init { output, force } => {
            cmd_init(&output, force)?;
        }
        Commands::RegisterDrive { label, category, path } => {
            cmd_register_drive(&cli.config, &label, &category, path)?;
        }
        Commands::ListDrives => {
            cmd_list_drives(&cli.config)?;
        }
        Commands::ListConnected => {
            cmd_list_connected()?;
        }
        Commands::SyncOnce { file } => {
            cmd_sync_once(&cli.config, &cli.db, file).await?;
        }
        Commands::Run { interval } => {
            cmd_run(&cli.config, &cli.db, interval).await?;
        }
        Commands::Status => {
            cmd_status(&cli.config, &cli.db)?;
        }
        Commands::ProcessPending => {
            cmd_process_pending(&cli.config, &cli.db).await?;
        }
        Commands::Clear { confirm } => {
            cmd_clear(&cli.db, confirm)?;
        }
        Commands::Validate => {
            cmd_validate(&cli.config)?;
        }
    }

    Ok(())
}

/// Initialize a new configuration file
fn cmd_init(output: &Path, force: bool) -> Result<()> {
    if output.exists() && !force {
        error!("Configuration file already exists. Use --force to overwrite.");
        return Ok(());
    }

    let config = Config::default_config();
    config.save(output)?;

    println!("✓ Created configuration file: {}", output.display());
    println!("\nNext steps:");
    println!("1. Edit {} to set your source directory", output.display());
    println!("2. Register your USB drives using: file-orchestrator register-drive");
    println!("3. Run the orchestrator: file-orchestrator run");

    Ok(())
}

/// Register a new USB drive
fn cmd_register_drive(
    config_path: &Path,
    label: &str,
    category: &str,
    path: Option<std::path::PathBuf>,
) -> Result<()> {
    let mut config = Config::load(config_path)?;

    // Validate category
    let valid_categories = ["images", "videos", "music", "documents", "archives"];
    if !valid_categories.contains(&category) {
        error!("Invalid category. Must be one of: {:?}", valid_categories);
        return Ok(());
    }

    // If no path provided, try to auto-detect the drive
    let drive_path = if let Some(p) = path {
        Some(p)
    } else {
        // List connected drives and let user select
        let detector = DriveDetector::new();
        let drives = detector.get_all_drives();
        
        if drives.is_empty() {
            error!("No drives detected. Please specify path manually with --path");
            return Ok(());
        }

        println!("\n=== Available Drives ===");
        for (idx, drive) in drives.iter().enumerate() {
            println!("{}. {} - {} ({} available)", 
                idx + 1, 
                drive.name,
                drive.mount_point.display(),
                format_size(drive.available_space)
            );
        }
        println!("========================\n");

        // Prompt for selection
        println!("Which drive do you want to register as '{}'?", label);
        println!("Enter number (or press Enter to skip auto-detection): ");
        
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            None
        } else if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= drives.len() {
                Some(drives[idx - 1].mount_point.clone())
            } else {
                error!("Invalid selection");
                return Ok(());
            }
        } else {
            error!("Invalid input");
            return Ok(());
        }
    };

    // Generate a simple UUID
    let drive_uuid = uuid::Uuid::new_v4().to_string();

    // Add drive to config
    config.drives.insert(
        drive_uuid.clone(),
        config::DriveConfig {
            label: label.to_string(),
            target: category.to_string(),
            path: drive_path.clone(),
            last_seen: None,
        },
    );

    config.save(config_path)?;

    println!("✓ Registered drive:");
    println!("  Label: {}", label);
    println!("  Category: {}", category);
    println!("  UUID: {}", drive_uuid);
    if let Some(p) = drive_path {
        println!("  Path: {}", p.display());
    } else {
        println!("  Path: Not set (will be detected when connected)");
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// List all registered drives
fn cmd_list_drives(config_path: &Path) -> Result<()> {
    let config = Config::load(config_path)?;

    println!("\n=== Registered Drives ===");
    for (uuid, drive) in &config.drives {
        println!("\nUUID: {}", uuid);
        println!("  Label: {}", drive.label);
        println!("  Category: {}", drive.target);
        if let Some(ref path) = drive.path {
            println!("  Path: {}", path.display());
        }
        if let Some(ref last_seen) = drive.last_seen {
            println!("  Last Seen: {}", last_seen);
        }
    }
    println!("\n========================\n");

    Ok(())
}

/// List all currently connected drives
fn cmd_list_connected() -> Result<()> {
    let detector = DriveDetector::new();
    detector.print_drives();
    Ok(())
}

/// Perform a one-time sync
async fn cmd_sync_once(
    config_path: &Path,
    db_path: &Path,
    file: Option<std::path::PathBuf>,
) -> Result<()> {
    let config = Config::load(config_path)?;
    let state = StateManager::new(db_path)?;
    let mut sync_manager = SyncManager::new(config, state);

    if let Some(file_path) = file {
        // Sync a single file
        info!("Syncing single file: {}", file_path.display());
        match sync_manager.sync_file(&file_path).await {
            Ok(result) => {
                println!("✓ Sync result: {:?}", result);
            }
            Err(e) => {
                error!("Failed to sync file: {}", e);
            }
        }
    } else {
        // Sync all files
        info!("Starting full sync...");
        let summary = sync_manager.sync_all().await?;
        summary.print();
    }

    Ok(())
}

/// Run the orchestrator in watch mode
async fn cmd_run(config_path: &Path, db_path: &Path, interval: u64) -> Result<()> {
    let config = Config::load(config_path)?;
    let state = StateManager::new(db_path)?;
    
    // Wrap sync_manager in Arc<Mutex<>> for thread-safe sharing
    let sync_manager = Arc::new(Mutex::new(SyncManager::new(config.clone(), state)));

    info!("Starting File Orchestrator...");
    info!("Watching: {}", config.source.path.display());

    // Start file watcher
    let mut file_watcher = AsyncFileWatcher::watch(&config.source.path)?;

    // Spawn a task to check for connected drives periodically
    let sync_manager_clone = Arc::clone(&sync_manager);
    
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(interval)).await;
            
            info!("Checking for connected drives...");
            
            // Use the shared sync_manager
            let mut sm = sync_manager_clone.lock().await;
            
            if let Err(e) = sm.check_and_sync_connected_drives().await {
                error!("Error checking connected drives: {}", e);
            }
        }
    });

    // Process file events
    println!("✓ File Orchestrator is running. Press Ctrl+C to stop.");
    println!("  Watching for file changes in: {}", config.source.path.display());

    while let Some(event) = file_watcher.next_event().await {
        match event {
            FileEvent::Created(path) | FileEvent::Modified(path) => {
                info!("Detected file change: {}", path.display());
                
                let mut sm = sync_manager.lock().await;
                if let Err(e) = sm.sync_file(&path).await {
                    error!("Failed to sync file: {}", e);
                }
            }
            FileEvent::Removed(path) => {
                info!("File removed: {}", path.display());
                // Optionally handle file removals
            }
        }
    }

    Ok(())
}

/// Show current status and statistics
fn cmd_status(config_path: &Path, db_path: &Path) -> Result<()> {
    let config = Config::load(config_path)?;
    let state = StateManager::new(db_path)?;
    let sync_manager = SyncManager::new(config, state);

    let stats = sync_manager.get_stats()?;

    println!("\n=== File Orchestrator Status ===");
    println!("Total files synced: {}", stats.total_files);
    println!("Total size: {} MB", stats.total_size / 1_000_000);
    println!("Pending syncs: {}", stats.pending_syncs);
    
    println!("\nBy category:");
    for (category, count) in &stats.by_category {
        println!("  {}: {}", category, count);
    }
    println!("\n================================\n");

    Ok(())
}

/// Process pending syncs
async fn cmd_process_pending(config_path: &Path, db_path: &Path) -> Result<()> {
    let config = Config::load(config_path)?;
    let state = StateManager::new(db_path)?;
    let mut sync_manager = SyncManager::new(config, state);

    info!("Checking for connected drives and processing pending syncs...");
    sync_manager.check_and_sync_connected_drives().await?;

    println!("✓ Finished processing pending syncs");

    Ok(())
}

/// Clear all sync state
fn cmd_clear(db_path: &Path, confirm: bool) -> Result<()> {
    if !confirm {
        error!("This will delete all sync history. Use --confirm to proceed.");
        return Ok(());
    }

    let state = StateManager::new(db_path)?;
    state.clear_all()?;

    println!("✓ Cleared all sync state");

    Ok(())
}

/// Validate configuration
fn cmd_validate(config_path: &Path) -> Result<()> {
    let config = Config::load(config_path)?;

    println!("✓ Configuration is valid");
    println!("\nSource directory: {}", config.source.path.display());
    println!("Registered drives: {}", config.drives.len());

    Ok(())
}
