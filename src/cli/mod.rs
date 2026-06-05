use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "file-orchestrator")]
#[command(author = "Your Name")]
#[command(version = "0.1.0")]
#[command(about = "A production-grade file orchestration tool for automatic file syncing", long_about = None)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    /// Database path for state management
    #[arg(short, long, default_value = ".orchestrator.db")]
    pub db: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new configuration file
    Init {
        /// Path where to create the config file
        #[arg(short, long, default_value = "config.toml")]
        output: PathBuf,

        /// Overwrite existing config file
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Register a new USB drive
    RegisterDrive {
        /// Drive label or name
        #[arg(short, long)]
        label: String,

        /// File category this drive should handle (images, videos, music, documents, archives)
        #[arg(short, long)]
        category: String,

        /// Optional: Specific mount point/path
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// List all registered drives
    ListDrives,

    /// List all currently connected drives
    ListConnected,

    /// Perform a one-time sync of all files
    SyncOnce {
        /// Specific file to sync (optional)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Start the orchestrator in watch mode (monitors for changes)
    Run {
        /// Check interval for drive connections (seconds)
        #[arg(short, long, default_value_t = 10)]
        interval: u64,
    },

    /// Show current sync status and statistics
    Status,

    /// Process pending syncs for connected drives
    ProcessPending,

    /// Clear all sync state (WARNING: This will reset all history)
    Clear {
        /// Confirm the clear operation
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },

    /// Validate configuration file
    Validate,

    #[cfg(feature = "gui")]
    /// Launch the graphical user interface
    Gui,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
