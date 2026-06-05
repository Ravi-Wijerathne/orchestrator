use crate::error::{OrchestratorError, Result};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

    /// Remove all pending syncs for a specific drive
    #[allow(dead_code)]
    pub fn cleanup_drive_data(&self, drive_uuid: &str) -> Result<()> {
        let prefix = format!("pending:");
        let mut keys_to_remove = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item?;
            let pending: PendingSync = serde_json::from_slice(&value)?;

            if pending.target_drive == drive_uuid {
                keys_to_remove.push(key);
            }
        }

        // Remove all matching keys
        for key in keys_to_remove {
            self.db.remove(key)?;
        }

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

            *stats
                .by_category
                .entry(state.file_category.clone())
                .or_insert(0) += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_state_manager() -> (TempDir, StateManager) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.db");
        let state = StateManager::new(&db_path).expect("Failed to create state manager");
        (temp_dir, state)
    }

    fn create_test_file_state() -> FileState {
        FileState {
            source_path: PathBuf::from("D:/Test/file.txt"),
            hash: "test_hash_123".to_string(),
            size: 1024,
            last_synced: current_timestamp(),
            target_drive: "test-drive-uuid".to_string(),
            target_path: PathBuf::from("E:/Test/file.txt"),
            file_category: "documents".to_string(),
        }
    }

    fn create_test_pending_sync() -> PendingSync {
        PendingSync {
            source_path: PathBuf::from("D:/Test/pending.txt"),
            file_category: "documents".to_string(),
            target_drive: "test-drive-uuid".to_string(),
            hash: "pending_hash_456".to_string(),
            size: 2048,
            created_at: current_timestamp(),
        }
    }

    #[test]
    fn test_save_and_get_file_state() {
        let (_temp_dir, state) = create_test_state_manager();
        let file_state = create_test_file_state();

        state
            .save_file_state(&file_state)
            .expect("Failed to save file state");

        let retrieved = state
            .get_file_state(&file_state.source_path)
            .expect("Failed to get file state")
            .expect("File state not found");

        assert_eq!(retrieved.hash, file_state.hash);
        assert_eq!(retrieved.size, file_state.size);
    }

    #[test]
    fn test_get_nonexistent_file_state() {
        let (_temp_dir, state) = create_test_state_manager();
        let result = state
            .get_file_state(PathBuf::from("/nonexistent/path").as_path())
            .expect("Failed to get file state");
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_file_state() {
        let (_temp_dir, state) = create_test_state_manager();
        let file_state = create_test_file_state();

        state
            .save_file_state(&file_state)
            .expect("Failed to save file state");
        state
            .remove_file_state(&file_state.source_path)
            .expect("Failed to remove file state");

        let result = state
            .get_file_state(&file_state.source_path)
            .expect("Failed to get file state");
        assert!(result.is_none());
    }

    #[test]
    fn test_is_file_synced() {
        let (_temp_dir, state) = create_test_state_manager();
        let file_state = create_test_file_state();

        state
            .save_file_state(&file_state)
            .expect("Failed to save file state");

        let is_synced = state
            .is_file_synced(&file_state.source_path, &file_state.hash)
            .expect("Failed to check file sync status");
        assert!(is_synced);

        let is_synced_different = state
            .is_file_synced(&file_state.source_path, "different_hash")
            .expect("Failed to check file sync status");
        assert!(!is_synced_different);
    }

    #[test]
    fn test_add_and_get_pending_sync() {
        let (_temp_dir, state) = create_test_state_manager();
        let pending = create_test_pending_sync();

        state
            .add_pending_sync(&pending)
            .expect("Failed to add pending sync");

        let pending_list = state
            .get_pending_syncs(&pending.target_drive)
            .expect("Failed to get pending syncs");

        assert_eq!(pending_list.len(), 1);
        assert_eq!(pending_list[0].source_path, pending.source_path);
    }

    #[test]
    fn test_remove_pending_sync() {
        let (_temp_dir, state) = create_test_state_manager();
        let pending = create_test_pending_sync();

        state
            .add_pending_sync(&pending)
            .expect("Failed to add pending sync");
        state
            .remove_pending_sync(&pending.source_path)
            .expect("Failed to remove pending sync");

        let pending_list = state
            .get_pending_syncs(&pending.target_drive)
            .expect("Failed to get pending syncs");
        assert!(pending_list.is_empty());
    }

    #[test]
    fn test_get_all_pending_syncs() {
        let (_temp_dir, state) = create_test_state_manager();

        let mut pending1 = create_test_pending_sync();
        pending1.target_drive = "drive-1".to_string();

        let mut pending2 = create_test_pending_sync();
        pending2.target_drive = "drive-2".to_string();
        pending2.source_path = PathBuf::from("D:/Test/pending2.txt");

        state
            .add_pending_sync(&pending1)
            .expect("Failed to add pending 1");
        state
            .add_pending_sync(&pending2)
            .expect("Failed to add pending 2");

        let all_pending = state
            .get_all_pending_syncs()
            .expect("Failed to get all pending");
        assert_eq!(all_pending.len(), 2);
    }

    #[test]
    fn test_cleanup_drive_data() {
        let (_temp_dir, state) = create_test_state_manager();

        let mut pending1 = create_test_pending_sync();
        pending1.target_drive = "drive-to-clean".to_string();

        let mut pending2 = create_test_pending_sync();
        pending2.target_drive = "drive-to-keep".to_string();
        pending2.source_path = PathBuf::from("D:/Test/keep.txt");

        state
            .add_pending_sync(&pending1)
            .expect("Failed to add pending 1");
        state
            .add_pending_sync(&pending2)
            .expect("Failed to add pending 2");

        state
            .cleanup_drive_data("drive-to-clean")
            .expect("Failed to cleanup");

        let remaining = state
            .get_all_pending_syncs()
            .expect("Failed to get remaining");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].target_drive, "drive-to-keep");
    }

    #[test]
    fn test_get_sync_stats_empty() {
        let (_temp_dir, state) = create_test_state_manager();
        let stats = state.get_sync_stats().expect("Failed to get sync stats");

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.pending_syncs, 0);
    }

    #[test]
    fn test_get_sync_stats_with_files() {
        let (_temp_dir, state) = create_test_state_manager();

        let mut file_state1 = create_test_file_state();
        file_state1.size = 1000;
        file_state1.file_category = "images".to_string();

        let mut file_state2 = create_test_file_state();
        file_state2.source_path = PathBuf::from("D:/Test/file2.txt");
        file_state2.size = 2000;
        file_state2.file_category = "videos".to_string();

        state
            .save_file_state(&file_state1)
            .expect("Failed to save file 1");
        state
            .save_file_state(&file_state2)
            .expect("Failed to save file 2");

        let stats = state.get_sync_stats().expect("Failed to get sync stats");
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_size, 3000);
    }

    #[test]
    fn test_sync_stats_default() {
        let stats = SyncStats::default();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.pending_syncs, 0);
        assert!(stats.by_category.is_empty());
    }

    #[test]
    fn test_clear_all() {
        let (_temp_dir, state) = create_test_state_manager();

        let file_state = create_test_file_state();
        let pending = create_test_pending_sync();

        state
            .save_file_state(&file_state)
            .expect("Failed to save file state");
        state
            .add_pending_sync(&pending)
            .expect("Failed to add pending sync");

        state.clear_all().expect("Failed to clear all");

        let stats = state
            .get_sync_stats()
            .expect("Failed to get stats after clear");
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.pending_syncs, 0);
    }

    #[test]
    fn test_get_all_file_states() {
        let (_temp_dir, state) = create_test_state_manager();

        let mut file_state1 = create_test_file_state();
        file_state1.source_path = PathBuf::from("D:/Test/file1.txt");

        let mut file_state2 = create_test_file_state();
        file_state2.source_path = PathBuf::from("D:/Test/file2.txt");

        state
            .save_file_state(&file_state1)
            .expect("Failed to save file 1");
        state
            .save_file_state(&file_state2)
            .expect("Failed to save file 2");

        let all_states = state
            .get_all_file_states()
            .expect("Failed to get all file states");
        assert_eq!(all_states.len(), 2);
    }

    #[test]
    fn test_calculate_file_hash() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, b"test content").expect("Failed to write file");

        let hash = calculate_file_hash(&file_path).expect("Failed to calculate hash");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_calculate_file_hash_consistency() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, b"consistent content").expect("Failed to write file");

        let hash1 = calculate_file_hash(&file_path).expect("Failed to calculate hash 1");
        let hash2 = calculate_file_hash(&file_path).expect("Failed to calculate hash 2");

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash_different_content() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path1 = temp_dir.path().join("test1.txt");
        let file_path2 = temp_dir.path().join("test2.txt");

        std::fs::write(&file_path1, b"content 1").expect("Failed to write file 1");
        std::fs::write(&file_path2, b"content 2").expect("Failed to write file 2");

        let hash1 = calculate_file_hash(&file_path1).expect("Failed to calculate hash 1");
        let hash2 = calculate_file_hash(&file_path2).expect("Failed to calculate hash 2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash_nonexistent() {
        let result = calculate_file_hash("/nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_current_timestamp() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp();
        assert!(ts2 >= ts1);
    }
}
