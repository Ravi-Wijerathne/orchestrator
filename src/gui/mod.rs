pub mod commands;
pub mod state;
pub mod tray;

use crate::error::Result;
use tauri::{Manager, SystemTray};
use std::sync::Arc;
use tokio::sync::Mutex;

pub use commands::*;
pub use state::*;
pub use tray::*;

/// Initialize and run the Tauri GUI application
pub async fn run_gui(config_path: String, db_path: String) -> Result<()> {
    let system_tray = SystemTray::new().with_menu(create_tray_menu());

    tauri::Builder::default()
        .setup(move |app| {
            let config_path = config_path.clone();
            let db_path = db_path.clone();
            
            // Initialize the GUI state
            let gui_state = GuiState::new(config_path, db_path)?;
            app.manage(Arc::new(Mutex::new(gui_state)));

            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            // Dashboard commands
            get_dashboard_stats,
            get_sync_status,
            
            // Drive commands
            get_drives,
            get_connected_drives,
            register_drive_cmd,
            remove_drive,
            
            // Sync commands
            sync_file_cmd,
            sync_pending_cmd,
            get_pending_files,
            
            // Config commands
            get_config,
            update_config,
            validate_config_cmd,
            
            // History commands
            get_sync_history,
            clear_history,
            
            // Statistics commands
            get_file_type_stats,
            
            // Control commands
            start_watching,
            stop_watching,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| crate::error::OrchestrationError::ConfigError(e.to_string()))?;

    Ok(())
}
