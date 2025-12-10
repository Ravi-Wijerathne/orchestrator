use tauri::{
    SystemTray, SystemTrayMenu, SystemTrayMenuItem, CustomMenuItem,
    SystemTrayEvent, AppHandle, Manager,
};

pub fn create_tray_menu() -> SystemTrayMenu {
    let show = CustomMenuItem::new("show".to_string(), "Show Window");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
    let dashboard = CustomMenuItem::new("dashboard".to_string(), "Dashboard");
    let start_sync = CustomMenuItem::new("start_sync".to_string(), "Start Watching");
    let stop_sync = CustomMenuItem::new("stop_sync".to_string(), "Stop Watching");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");

    SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(dashboard)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(start_sync)
        .add_item(stop_sync)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit)
}

pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            let window = app.get_window("main").unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "dashboard" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "start_sync" => {
                    // Emit event to start watching
                    app.emit_all("start-watching", ()).unwrap();
                }
                "stop_sync" => {
                    // Emit event to stop watching
                    app.emit_all("stop-watching", ()).unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        _ => {}
    }
}
