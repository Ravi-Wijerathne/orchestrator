use crate::config::Config;
use crate::state::StateManager;
use crate::sync::SyncManager;
use crate::drive::DriveDetector;
use crate::watcher::AsyncFileWatcher;
use crate::error::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

/// GUI application state
pub struct GuiState {
    pub config_path: String,
    pub db_path: String,
    pub config: Config,
    pub state_manager: StateManager,
    pub sync_manager: Option<Arc<Mutex<SyncManager>>>,
    pub watcher: Option<Arc<Mutex<AsyncFileWatcher>>>,
    pub is_watching: bool,
}

impl GuiState {
    pub fn new(config_path: String, db_path: String) -> Result<Self> {
        let config = Config::load(&PathBuf::from(&config_path))?;
        let state_manager = StateManager::new(&db_path)?;

        Ok(Self {
            config_path,
            db_path,
            config,
            state_manager,
            sync_manager: None,
            watcher: None,
            is_watching: false,
        })
    }

    pub fn reload_config(&mut self) -> Result<()> {
        self.config = Config::load(&PathBuf::from(&self.config_path))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_syncs: usize,
    pub pending_files: usize,
    pub registered_drives: usize,
    pub connected_drives: usize,
    pub total_file_types: usize,
    pub is_watching: bool,
    pub source_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub label: String,
    pub category: String,
    pub mount_point: Option<String>,
    pub is_connected: bool,
    pub total_space: Option<u64>,
    pub available_space: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingFileInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub category: String,
    pub target_drive: String,
    pub size: u64,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryEntry {
    pub id: String,
    pub source_path: String,
    pub file_name: String,
    pub target_drive: String,
    pub target_path: String,
    pub file_type: String,
    pub category: String,
    pub file_hash: String,
    pub synced_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeStats {
    pub category: String,
    pub count: usize,
    pub total_size: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub current_file: Option<String>,
    pub progress: f64,
}
