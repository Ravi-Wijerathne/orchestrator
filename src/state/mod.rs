use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{OrchestratorError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub source_path: PathBuf,
    pub hash: String,
    pub size: u64,
    pub last_synced: u64,
    pub target_drive: String,
    pub target_path: PathBuf,
    pub file_category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSync {
    pub source_path: PathBuf,
    pub file_category: String,
    pub target_drive: String,
    pub hash: String,
    pub size: u64,
    pub created_at: u64,
}

pub struct StateManager {
    db: Db,
}

impl StateManager {
    /// Create a new state manager
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db = sled::open(db_path)
            .map_err(|e| OrchestratorError::State(format!("Failed to open database: {}", e)))?;
        
        Ok(Self { db })
    }

    /// Save file state after successful sync
    pub fn save_file_state(&self, state: &FileState) -> Result<()> {
        let key = self.file_key(&state.source_path);
        let value = serde_json::to_vec(state)?;
        
        self.db.insert(key, value)?;
        self.db.flush()?;
        
        Ok(())
    }

    /// Get file state by source path
    pub fn get_file_state(&self, source_path: &Path) -> Result<Option<FileState>> {
        let key = self.file_key(source_path);
        
        if let Some(value) = self.db.get(key)? {
            let state: FileState = serde_json::from_slice(&value)?;
            return Ok(Some(state));
        }
        
        Ok(None)
    }

    /// Check if file has been synced (and hasn't changed)
    #[allow(dead_code)]
    pub fn is_file_synced(&self, source_path: &Path, current_hash: &str) -> Result<bool> {
        if let Some(state) = self.get_file_state(source_path)? {
            return Ok(state.hash == current_hash);
        }
        Ok(false)
    }

    /// Add a file to pending sync queue
    pub fn add_pending_sync(&self, pending: &PendingSync) -> Result<()> {
        let key = self.pending_key(&pending.source_path);
        let value = serde_json::to_vec(pending)?;
        
        self.db.insert(key, value)?;
        self.db.flush()?;
        
        Ok(())
    }

    /// Get all pending syncs for a specific drive
    pub fn get_pending_syncs(&self, drive_uuid: &str) -> Result<Vec<PendingSync>> {
        let prefix = format!("pending:");
        let mut pending_syncs = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item?;
            let pending: PendingSync = serde_json::from_slice(&value)?;
            
            if pending.target_drive == drive_uuid {
                pending_syncs.push(pending);
            }
        }

        Ok(pending_syncs)
    }

    /// Remove a file from pending sync queue
    pub fn remove_pending_sync(&self, source_path: &Path) -> Result<()> {
        let key = self.pending_key(source_path);
        self.db.remove(key)?;
        self.db.flush()?;
        Ok(())
    }

    /// Get all pending syncs (for all drives)
    pub fn get_all_pending_syncs(&self) -> Result<Vec<PendingSync>> {
        let prefix = format!("pending:");
        let mut pending_syncs = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item?;
            let pending: PendingSync = serde_json::from_slice(&value)?;
            pending_syncs.push(pending);
        }

        Ok(pending_syncs)
    }

    /// Get statistics about synced files
    pub fn get_sync_stats(&self) -> Result<SyncStats> {
        let mut stats = SyncStats::default();
        let prefix = "file:";

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item?;
            let state: FileState = serde_json::from_slice(&value)?;
            
            stats.total_files += 1;
            stats.total_size += state.size;
            
            *stats.by_category.entry(state.file_category.clone()).or_insert(0) += 1;
        }

        stats.pending_syncs = self.get_all_pending_syncs()?.len();

        Ok(stats)
    }

    /// Clear all state (use with caution!)
    pub fn clear_all(&self) -> Result<()> {
        self.db.clear()?;
        self.db.flush()?;
        Ok(())
    }

    /// Get all synced file states
    pub fn get_all_file_states(&self) -> Result<Vec<FileState>> {
        let prefix = "file:";
        let mut files = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item?;
            let file_state: FileState = serde_json::from_slice(&value)?;
            files.push(file_state);
        }

        Ok(files)
    }

    /// Remove a file state (for deleted files)
    pub fn remove_file_state(&self, source_path: &Path) -> Result<()> {
        let key = self.file_key(source_path);
        self.db.remove(key)?;
        self.db.flush()?;
        Ok(())
    }

    // Helper methods
    fn file_key(&self, path: &Path) -> Vec<u8> {
        format!("file:{}", path.display()).into_bytes()
    }

    fn pending_key(&self, path: &Path) -> Vec<u8> {
        format!("pending:{}", path.display()).into_bytes()
    }
}

#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_files: usize,
    pub total_size: u64,
    pub pending_syncs: usize,
    pub by_category: std::collections::HashMap<String, usize>,
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Calculate file hash using BLAKE3
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let data = std::fs::read(path.as_ref())
        .map_err(|e| OrchestratorError::State(format!("Failed to read file for hashing: {}", e)))?;
    
    let hash = blake3::hash(&data);
    Ok(hash.to_hex().to_string())
}
