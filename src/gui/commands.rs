use super::state::*;
use crate::config::Config;
use crate::drive::DriveDetector;
use crate::sync::SyncManager;
use crate::watcher::{AsyncFileWatcher, FileEvent};
use crate::error::Result;
use tauri::{State, Manager, AppHandle};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use std::collections::HashMap;

// ============================================================================
// Dashboard Commands
// ============================================================================

#[tauri::command]
pub async fn get_dashboard_stats(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<DashboardStats, String> {
    let gui_state = state.lock().await;
    
    let total_syncs = gui_state.state_manager.get_sync_count()
        .map_err(|e| e.to_string())?;
    
    let pending_files = gui_state.state_manager.get_pending_count()
        .map_err(|e| e.to_string())?;
    
    let registered_drives = gui_state.config.drives.len();
    
    let detector = DriveDetector::new();
    let connected_drives_list = detector.detect_drives(&gui_state.config)
        .map_err(|e| e.to_string())?;
    let connected_drives = connected_drives_list.len();
    
    let file_types = gui_state.state_manager.get_file_type_counts()
        .map_err(|e| e.to_string())?;
    let total_file_types = file_types.len();
    
    Ok(DashboardStats {
        total_syncs,
        pending_files,
        registered_drives,
        connected_drives,
        total_file_types,
        is_watching: gui_state.is_watching,
        source_directory: gui_state.config.source_dir.clone(),
    })
}

#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<SyncStatus, String> {
    let gui_state = state.lock().await;
    
    Ok(SyncStatus {
        is_syncing: gui_state.sync_manager.is_some(),
        current_file: None,
        progress: 0.0,
    })
}

// ============================================================================
// Drive Commands
// ============================================================================

#[tauri::command]
pub async fn get_drives(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<Vec<DriveInfo>, String> {
    let gui_state = state.lock().await;
    let detector = DriveDetector::new();
    let connected = detector.detect_drives(&gui_state.config)
        .map_err(|e| e.to_string())?;
    
    let mut drives = Vec::new();
    
    for drive in &gui_state.config.drives {
        let is_connected = connected.iter().any(|d| d.label == drive.label);
        let mount_info = connected.iter().find(|d| d.label == drive.label);
        
        drives.push(DriveInfo {
            label: drive.label.clone(),
            category: drive.category.clone(),
            mount_point: mount_info.map(|d| d.mount_point.display().to_string()),
            is_connected,
            total_space: mount_info.and_then(|d| d.total_space),
            available_space: mount_info.and_then(|d| d.available_space),
        });
    }
    
    Ok(drives)
}

#[tauri::command]
pub async fn get_connected_drives() -> std::result::Result<Vec<String>, String> {
    let detector = DriveDetector::new();
    let drives = detector.list_all_drives().map_err(|e| e.to_string())?;
    Ok(drives)
}

#[tauri::command]
pub async fn register_drive_cmd(
    state: State<'_, Arc<Mutex<GuiState>>>,
    label: String,
    category: String,
    path: Option<String>,
) -> std::result::Result<(), String> {
    let mut gui_state = state.lock().await;
    
    let drive_config = crate::config::DriveConfig {
        label: label.clone(),
        category: category.clone(),
        mount_point: path.map(PathBuf::from),
    };
    
    gui_state.config.drives.push(drive_config);
    gui_state.config.save(&PathBuf::from(&gui_state.config_path))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn remove_drive(
    state: State<'_, Arc<Mutex<GuiState>>>,
    label: String,
) -> std::result::Result<(), String> {
    let mut gui_state = state.lock().await;
    
    gui_state.config.drives.retain(|d| d.label != label);
    gui_state.config.save(&PathBuf::from(&gui_state.config_path))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// ============================================================================
// Sync Commands
// ============================================================================

#[tauri::command]
pub async fn sync_file_cmd(
    state: State<'_, Arc<Mutex<GuiState>>>,
    file_path: String,
    app_handle: AppHandle,
) -> std::result::Result<String, String> {
    let gui_state = state.lock().await;
    
    let path = PathBuf::from(file_path);
    let sync_manager = SyncManager::new(
        gui_state.config.clone(),
        gui_state.state_manager.clone(),
    );
    
    let result = sync_manager.sync_file(&path).await
        .map_err(|e| e.to_string())?;
    
    // Send notification
    let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
        .title("File Synced")
        .body(&format!("Successfully synced: {}", path.file_name().unwrap().to_string_lossy()))
        .show();
    
    Ok(result)
}

#[tauri::command]
pub async fn sync_pending_cmd(
    state: State<'_, Arc<Mutex<GuiState>>>,
    app_handle: AppHandle,
) -> std::result::Result<usize, String> {
    let gui_state = state.lock().await;
    
    let sync_manager = SyncManager::new(
        gui_state.config.clone(),
        gui_state.state_manager.clone(),
    );
    
    let count = sync_manager.process_pending_syncs().await
        .map_err(|e| e.to_string())?;
    
    // Send notification
    let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
        .title("Pending Files Synced")
        .body(&format!("Successfully synced {} pending files", count))
        .show();
    
    Ok(count)
}

#[tauri::command]
pub async fn get_pending_files(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<Vec<PendingFileInfo>, String> {
    let gui_state = state.lock().await;
    
    let pending = gui_state.state_manager.get_pending_syncs()
        .map_err(|e| e.to_string())?;
    
    let mut files = Vec::new();
    
    for (file_path, (category, target_drive)) in pending {
        let path = PathBuf::from(&file_path);
        let metadata = std::fs::metadata(&path).ok();
        let size = metadata.map(|m| m.len()).unwrap_or(0);
        
        let file_type = if let Some(ext) = path.extension() {
            ext.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        };
        
        files.push(PendingFileInfo {
            file_path: file_path.clone(),
            file_name: path.file_name().unwrap().to_string_lossy().to_string(),
            file_type,
            category,
            target_drive,
            size,
            added_at: chrono::Utc::now().to_rfc3339(),
        });
    }
    
    Ok(files)
}

// ============================================================================
// Configuration Commands
// ============================================================================

#[tauri::command]
pub async fn get_config(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<Config, String> {
    let gui_state = state.lock().await;
    Ok(gui_state.config.clone())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, Arc<Mutex<GuiState>>>,
    config: Config,
) -> std::result::Result<(), String> {
    let mut gui_state = state.lock().await;
    
    config.save(&PathBuf::from(&gui_state.config_path))
        .map_err(|e| e.to_string())?;
    
    gui_state.config = config;
    
    Ok(())
}

#[tauri::command]
pub async fn validate_config_cmd(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<Vec<String>, String> {
    let gui_state = state.lock().await;
    
    match gui_state.config.validate() {
        Ok(_) => Ok(vec![]),
        Err(e) => Ok(vec![e.to_string()]),
    }
}

// ============================================================================
// History Commands
// ============================================================================

#[tauri::command]
pub async fn get_sync_history(
    state: State<'_, Arc<Mutex<GuiState>>>,
    limit: Option<usize>,
) -> std::result::Result<Vec<SyncHistoryEntry>, String> {
    let gui_state = state.lock().await;
    
    let history = gui_state.state_manager.get_sync_history(limit.unwrap_or(100))
        .map_err(|e| e.to_string())?;
    
    let mut entries = Vec::new();
    
    for record in history {
        entries.push(SyncHistoryEntry {
            id: record.id,
            source_path: record.source_path,
            file_name: PathBuf::from(&record.source_path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            target_drive: record.target_drive,
            target_path: record.target_path,
            file_type: record.file_type,
            category: record.category,
            file_hash: record.file_hash,
            synced_at: record.synced_at,
            status: "completed".to_string(),
        });
    }
    
    Ok(entries)
}

#[tauri::command]
pub async fn clear_history(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<(), String> {
    let gui_state = state.lock().await;
    gui_state.state_manager.clear_all()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Statistics Commands
// ============================================================================

#[tauri::command]
pub async fn get_file_type_stats(
    state: State<'_, Arc<Mutex<GuiState>>>,
) -> std::result::Result<Vec<FileTypeStats>, String> {
    let gui_state = state.lock().await;
    
    let type_counts = gui_state.state_manager.get_file_type_counts()
        .map_err(|e| e.to_string())?;
    
    let total: usize = type_counts.values().sum();
    
    let mut stats = Vec::new();
    
    for (category, count) in type_counts {
        let percentage = if total > 0 {
            (count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        stats.push(FileTypeStats {
            category,
            count,
            total_size: 0, // We'd need to track this separately
            percentage,
        });
    }
    
    stats.sort_by(|a, b| b.count.cmp(&a.count));
    
    Ok(stats)
}

// ============================================================================
// Control Commands
// ============================================================================

#[tauri::command]
pub async fn start_watching(
    state: State<'_, Arc<Mutex<GuiState>>>,
    app_handle: AppHandle,
) -> std::result::Result<(), String> {
    let mut gui_state = state.lock().await;
    
    if gui_state.is_watching {
        return Ok(());
    }
    
    let source_dir = PathBuf::from(&gui_state.config.source_dir);
    let sync_manager = Arc::new(Mutex::new(SyncManager::new(
        gui_state.config.clone(),
        gui_state.state_manager.clone(),
    )));
    
    let watcher = AsyncFileWatcher::new(source_dir.clone())
        .map_err(|e| e.to_string())?;
    
    gui_state.sync_manager = Some(sync_manager.clone());
    gui_state.watcher = Some(Arc::new(Mutex::new(watcher)));
    gui_state.is_watching = true;
    
    // Spawn watcher task
    let watcher_clone = gui_state.watcher.as_ref().unwrap().clone();
    let sync_manager_clone = sync_manager.clone();
    let app_handle_clone = app_handle.clone();
    
    tokio::spawn(async move {
        let mut watcher = watcher_clone.lock().await;
        
        loop {
            match watcher.next_event().await {
                Ok(event) => {
                    if let FileEvent::Created(path) | FileEvent::Modified(path) = event {
                        let sync_manager = sync_manager_clone.lock().await;
                        
                        match sync_manager.sync_file(&path).await {
                            Ok(msg) => {
                                let _ = tauri::api::notification::Notification::new(&app_handle_clone.config().tauri.bundle.identifier)
                                    .title("File Synced")
                                    .body(&msg)
                                    .show();
                                
                                let _ = app_handle_clone.emit_all("file-synced", &path.to_string_lossy().to_string());
                            }
                            Err(e) => {
                                eprintln!("Sync error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watcher error: {}", e);
                    break;
                }
            }
        }
    });
    
    let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
        .title("File Watching Started")
        .body(&format!("Monitoring: {}", source_dir.display()))
        .show();
    
    Ok(())
}

#[tauri::command]
pub async fn stop_watching(
    state: State<'_, Arc<Mutex<GuiState>>>,
    app_handle: AppHandle,
) -> std::result::Result<(), String> {
    let mut gui_state = state.lock().await;
    
    gui_state.is_watching = false;
    gui_state.watcher = None;
    gui_state.sync_manager = None;
    
    let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
        .title("File Watching Stopped")
        .body("File monitoring has been stopped")
        .show();
    
    Ok(())
}
