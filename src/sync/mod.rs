use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as async_fs;
use crate::config::Config;
use crate::classifier::{FileClassifier, FileType};
use crate::state::{StateManager, FileState, PendingSync, calculate_file_hash, current_timestamp};
use crate::drive::DriveDetector;
use crate::error::{OrchestratorError, Result};
use tracing::{info, warn, error};

pub struct SyncManager {
    config: Config,
    state: StateManager,
    drive_detector: DriveDetector,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(config: Config, state: StateManager) -> Self {
        Self {
            config,
            state,
            drive_detector: DriveDetector::new(),
        }
    }

    /// Sync a single file
    pub async fn sync_file<P: AsRef<Path>>(&mut self, source_path: P) -> Result<SyncResult> {
        let source_path = source_path.as_ref();
        
        info!("Processing file: {}", source_path.display());

        // Check if file exists
        if !source_path.exists() {
            return Err(OrchestratorError::Sync(
                format!("File does not exist: {}", source_path.display())
            ));
        }

        // Classify the file
        let file_info = FileClassifier::get_file_info(source_path)
            .map_err(|e| OrchestratorError::Sync(format!("Failed to classify file: {}", e)))?;

        if file_info.file_type == FileType::Unknown {
            warn!("Unknown file type, skipping: {}", source_path.display());
            return Ok(SyncResult::Skipped("Unknown file type".to_string()));
        }

        let category = file_info.file_type.as_str();

        // Find target drive for this category
        let (drive_uuid, drive_config) = self.config
            .find_drive_for_category(category)
            .ok_or_else(|| OrchestratorError::Sync(
                format!("No drive configured for category: {}", category)
            ))?;

        // Calculate file hash
        let hash = calculate_file_hash(source_path)
            .map_err(|e| OrchestratorError::Sync(format!("Failed to hash file: {}", e)))?;

        // Check if already synced
        if self.state.is_file_synced(source_path, &hash)? {
            info!("File already synced: {}", source_path.display());
            return Ok(SyncResult::AlreadySynced);
        }

        // Check if target drive is connected
        self.drive_detector.refresh();
        
        let drive_connected = if let Some(ref path) = drive_config.path {
            self.drive_detector.is_drive_connected(path)
        } else {
            // Try to find by label
            self.drive_detector.find_drive_by_label(&drive_config.label).is_some()
        };

        if !drive_connected {
            info!("Target drive not connected, adding to pending queue: {}", drive_config.label);
            
            let pending = PendingSync {
                source_path: source_path.to_path_buf(),
                file_category: category.to_string(),
                target_drive: drive_uuid.clone(),
                hash: hash.clone(),
                size: file_info.size,
                created_at: current_timestamp(),
            };
            
            self.state.add_pending_sync(&pending)?;
            return Ok(SyncResult::Pending(drive_config.label.clone()));
        }

        // Get target path
        let target_base = if let Some(ref path) = drive_config.path {
            path.clone()
        } else {
            self.drive_detector
                .find_drive_by_label(&drive_config.label)
                .ok_or_else(|| OrchestratorError::DriveNotFound(drive_config.label.clone()))?
                .mount_point
        };

        // Create target directory structure (preserve relative path from source)
        let relative_path = source_path
            .strip_prefix(&self.config.source.path)
            .unwrap_or(source_path);
        
        let target_path = target_base.join(category).join(relative_path);

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            async_fs::create_dir_all(parent).await
                .map_err(|e| OrchestratorError::Sync(format!("Failed to create target directory: {}", e)))?;
        }

        // Copy the file
        info!("Copying {} -> {}", source_path.display(), target_path.display());
        async_fs::copy(source_path, &target_path).await
            .map_err(|e| OrchestratorError::Sync(format!("Failed to copy file: {}", e)))?;

        // Save state
        let file_state = FileState {
            source_path: source_path.to_path_buf(),
            hash,
            size: file_info.size,
            last_synced: current_timestamp(),
            target_drive: drive_uuid.clone(),
            target_path: target_path.clone(),
            file_category: category.to_string(),
        };

        self.state.save_file_state(&file_state)?;

        // Remove from pending if it was there
        let _ = self.state.remove_pending_sync(source_path);

        info!("Successfully synced: {}", source_path.display());
        Ok(SyncResult::Synced(target_path))
    }

    /// Sync all files in the source directory
    pub async fn sync_all(&mut self) -> Result<SyncSummary> {
        let mut summary = SyncSummary::default();
        
        info!("Starting full sync from: {}", self.config.source.path.display());

        let files = self.collect_files(&self.config.source.path)?;
        
        for file in files {
            match self.sync_file(&file).await {
                Ok(SyncResult::Synced(_)) => summary.synced += 1,
                Ok(SyncResult::Pending(_)) => summary.pending += 1,
                Ok(SyncResult::AlreadySynced) => summary.already_synced += 1,
                Ok(SyncResult::Skipped(_)) => summary.skipped += 1,
                Err(e) => {
                    error!("Failed to sync {}: {}", file.display(), e);
                    summary.failed += 1;
                }
            }
        }

        Ok(summary)
    }

    /// Process pending syncs for a specific drive
    pub async fn process_pending_syncs(&mut self, drive_uuid: &str) -> Result<usize> {
        let pending_syncs = self.state.get_pending_syncs(drive_uuid)?;
        let count = pending_syncs.len();

        info!("Processing {} pending syncs for drive {}", count, drive_uuid);

        for pending in pending_syncs {
            if pending.source_path.exists() {
                match self.sync_file(&pending.source_path).await {
                    Ok(_) => info!("Synced pending file: {}", pending.source_path.display()),
                    Err(e) => error!("Failed to sync pending file: {}", e),
                }
            } else {
                warn!("Pending file no longer exists: {}", pending.source_path.display());
                let _ = self.state.remove_pending_sync(&pending.source_path);
            }
        }

        Ok(count)
    }

    /// Collect all files from a directory recursively
    fn collect_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.collect_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| OrchestratorError::Sync(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| OrchestratorError::Sync(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                self.collect_files_recursive(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Get sync statistics
    pub fn get_stats(&self) -> Result<crate::state::SyncStats> {
        self.state.get_sync_stats()
    }

    /// Check for newly connected drives and process their pending syncs
    pub async fn check_and_sync_connected_drives(&mut self) -> Result<()> {
        self.drive_detector.refresh();

        // Collect drive info first to avoid borrowing issues
        let drive_uuids: Vec<String> = self.config.drives.keys().cloned().collect();

        // Now process each drive
        for drive_uuid in drive_uuids {
            if let Some(drive_config) = self.config.drives.get(&drive_uuid).cloned() {
                let is_connected = if let Some(ref path) = drive_config.path {
                    self.drive_detector.is_drive_connected(path)
                } else {
                    self.drive_detector.find_drive_by_label(&drive_config.label).is_some()
                };

                if is_connected {
                    info!("Drive {} is connected, checking for pending syncs", drive_config.label);
                    let count = self.process_pending_syncs(&drive_uuid).await?;
                    if count > 0 {
                        info!("Processed {} pending syncs for {}", count, drive_config.label);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum SyncResult {
    Synced(PathBuf),
    Pending(String),
    AlreadySynced,
    Skipped(String),
}

#[derive(Debug, Default)]
pub struct SyncSummary {
    pub synced: usize,
    pub pending: usize,
    pub already_synced: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl SyncSummary {
    pub fn total(&self) -> usize {
        self.synced + self.pending + self.already_synced + self.skipped + self.failed
    }

    pub fn print(&self) {
        println!("\n=== Sync Summary ===");
        println!("Total files: {}", self.total());
        println!("Synced: {}", self.synced);
        println!("Already synced: {}", self.already_synced);
        println!("Pending: {}", self.pending);
        println!("Skipped: {}", self.skipped);
        println!("Failed: {}", self.failed);
        println!("====================\n");
    }
}
