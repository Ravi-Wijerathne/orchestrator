use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::config::Config;
use crate::state::StateManager;
use crate::drive::DriveDetector;
use crate::error::Result;

pub struct FileOrchestratorApp {
    config: Arc<Mutex<Config>>,
    state_manager: Arc<Mutex<StateManager>>,
    drive_detector: Arc<Mutex<DriveDetector>>,
    current_view: AppView,
    
    // Dashboard data
    pending_count: usize,
    drives_status: Vec<(String, String, bool)>, // (uuid, label, connected)
    
    // Drive registration form
    new_drive_label: String,
    new_drive_category: String,
    selected_path: Option<PathBuf>,
    
    // Status messages
    status_message: Option<String>,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppView {
    Dashboard,
    DriveManager,
    Settings,
}

impl FileOrchestratorApp {
    pub fn new(
        config: Config,
        state_manager: StateManager,
        _db_path: String,
    ) -> Self {
        let drive_detector = DriveDetector::new();
        
        Self {
            config: Arc::new(Mutex::new(config)),
            state_manager: Arc::new(Mutex::new(state_manager)),
            drive_detector: Arc::new(Mutex::new(drive_detector)),
            current_view: AppView::Dashboard,
            pending_count: 0,
            drives_status: Vec::new(),
            new_drive_label: String::new(),
            new_drive_category: "images".to_string(),
            selected_path: None,
            status_message: None,
            error_message: None,
        }
    }
    
    fn update_dashboard_stats(&mut self) {
        // Update drive status
        let config = self.config.lock().unwrap();
        let mut detector = self.drive_detector.lock().unwrap();
        detector.refresh();
        
        self.drives_status.clear();
        for (uuid, drive_config) in &config.drives {
            let connected = if let Some(ref path) = drive_config.path {
                detector.is_drive_connected(path)
            } else {
                false
            };
            self.drives_status.push((
                uuid.clone(),
                drive_config.label.clone(),
                connected,
            ));
        }
        
        // Count pending syncs
        let state = self.state_manager.lock().unwrap();
        let mut total_pending = 0;
        for (uuid, _) in &config.drives {
            if let Ok(pending) = state.get_pending_syncs(uuid) {
                total_pending += pending.len();
            }
        }
        self.pending_count = total_pending;
    }
    
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dashboard");
        ui.add_space(10.0);
        
        // Stats
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_min_width(200.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Pending Syncs").size(14.0));
                    ui.label(egui::RichText::new(self.pending_count.to_string()).size(24.0).strong());
                });
            });
            
            ui.group(|ui| {
                ui.set_min_width(200.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Registered Drives").size(14.0));
                    ui.label(egui::RichText::new(self.drives_status.len().to_string()).size(24.0).strong());
                });
            });
        });
        
        ui.add_space(20.0);
        
        // Drive status
        ui.heading("Drive Status");
        ui.separator();
        
        if self.drives_status.is_empty() {
            ui.label("No drives registered. Go to Drive Manager to add drives.");
        } else {
            for (_uuid, label, connected) in &self.drives_status {
                ui.horizontal(|ui| {
                    let status_text = if *connected { "[Connected]" } else { "[Disconnected]" };
                    
                    ui.label(format!("{} {}", status_text, label));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(status_text);
                    });
                });
                ui.separator();
            }
        }
        
        ui.add_space(20.0);
        
        if ui.button("Refresh Status").clicked() {
            self.update_dashboard_stats();
            self.status_message = Some("Status refreshed".to_string());
        }
    }
    
    fn show_drive_manager(&mut self, ui: &mut egui::Ui) {
        ui.heading("Drive Manager");
        ui.add_space(10.0);
        
        // Registered drives list
        ui.group(|ui| {
            ui.set_min_height(200.0);
            ui.label(egui::RichText::new("Registered Drives").strong());
            ui.separator();
            
            let config = self.config.lock().unwrap();
            
            if config.drives.is_empty() {
                ui.label("No drives registered yet.");
            } else {
                for (_uuid, drive_config) in &config.drives {
                    ui.horizontal(|ui| {
                        ui.label(format!("Drive: {}", drive_config.label));
                        ui.label(format!("Category: {}", drive_config.target));
                        if let Some(path) = &drive_config.path {
                            ui.label(format!("Path: {}", path.display()));
                        }
                    });
                    ui.separator();
                }
            }
        });
        
        ui.add_space(20.0);
        
        // Add new drive form
        ui.group(|ui| {
            ui.label(egui::RichText::new("Add New Drive").strong());
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Label:");
                ui.text_edit_singleline(&mut self.new_drive_label);
            });
            
            ui.horizontal(|ui| {
                ui.label("Category:");
                egui::ComboBox::from_id_source("category")
                    .selected_text(&self.new_drive_category)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_drive_category, "images".to_string(), "Images");
                        ui.selectable_value(&mut self.new_drive_category, "videos".to_string(), "Videos");
                        ui.selectable_value(&mut self.new_drive_category, "music".to_string(), "Music");
                        ui.selectable_value(&mut self.new_drive_category, "documents".to_string(), "Documents");
                        ui.selectable_value(&mut self.new_drive_category, "archives".to_string(), "Archives");
                    });
            });
            
            ui.horizontal(|ui| {
                if ui.button("Select Drive Path").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.selected_path = Some(path);
                    }
                }
                
                if let Some(ref path) = self.selected_path {
                    ui.label(format!("Selected: {}", path.display()));
                }
            });
            
            ui.add_space(10.0);
            
            if ui.button("Register Drive").clicked() {
                if self.new_drive_label.is_empty() {
                    self.error_message = Some("Label cannot be empty".to_string());
                } else if self.selected_path.is_none() {
                    self.error_message = Some("Please select a drive path".to_string());
                } else {
                    // Add drive to config
                    let uuid = uuid::Uuid::new_v4().to_string();
                    let new_drive = crate::config::DriveConfig {
                        label: self.new_drive_label.clone(),
                        target: self.new_drive_category.clone(),
                        path: self.selected_path.clone(),
                        last_seen: Some(chrono::Utc::now().to_rfc3339()),
                    };
                    
                    let save_result = {
                        let mut config = self.config.lock().unwrap();
                        config.drives.insert(uuid.clone(), new_drive);
                        config.save("config.toml")
                    };
                    
                    if let Err(e) = save_result {
                        self.error_message = Some(format!("Failed to save config: {}", e));
                    } else {
                        self.status_message = Some(format!("Drive '{}' registered successfully", self.new_drive_label));
                        self.new_drive_label.clear();
                        self.selected_path = None;
                        self.update_dashboard_stats();
                    }
                }
            }
        });
    }
    
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(10.0);
        
        let config = self.config.lock().unwrap();
        
        ui.group(|ui| {
            ui.label(egui::RichText::new("Source Directory").strong());
            ui.label(format!("Path: {}", config.source.path.display()));
            ui.label("Edit config.toml to change the source directory.");
        });
        
        ui.add_space(20.0);
        
        ui.group(|ui| {
            ui.label(egui::RichText::new("File Rules").strong());
            ui.separator();
            
            ui.label(format!("Images: {}", config.rules.images.join(", ")));
            ui.label(format!("Videos: {}", config.rules.videos.join(", ")));
            ui.label(format!("Music: {}", config.rules.music.join(", ")));
            
            if let Some(docs) = &config.rules.documents {
                ui.label(format!("Documents: {}", docs.join(", ")));
            }
            
            if let Some(archives) = &config.rules.archives {
                ui.label(format!("Archives: {}", archives.join(", ")));
            }
        });
    }
}

impl eframe::App for FileOrchestratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with navigation
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("File Orchestrator");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(self.current_view == AppView::Settings, "Settings").clicked() {
                        self.current_view = AppView::Settings;
                    }
                    
                    if ui.selectable_label(self.current_view == AppView::DriveManager, "Drives").clicked() {
                        self.current_view = AppView::DriveManager;
                    }
                    
                    if ui.selectable_label(self.current_view == AppView::Dashboard, "Dashboard").clicked() {
                        self.current_view = AppView::Dashboard;
                        self.update_dashboard_stats();
                    }
                });
            });
        });
        
        // Bottom panel with status messages
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(ref msg) = self.status_message {
                    ui.label(egui::RichText::new(format!("[OK] {}", msg)).color(egui::Color32::GREEN));
                    if ui.button("X").clicked() {
                        self.status_message = None;
                    }
                }
                
                if let Some(ref msg) = self.error_message {
                    ui.label(egui::RichText::new(format!("[ERROR] {}", msg)).color(egui::Color32::RED));
                    if ui.button("X").clicked() {
                        self.error_message = None;
                    }
                }
            });
        });
        
        // Central panel with main content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.current_view {
                    AppView::Dashboard => self.show_dashboard(ui),
                    AppView::DriveManager => self.show_drive_manager(ui),
                    AppView::Settings => self.show_settings(ui),
                }
            });
        });
    }
}

pub fn run_gui(config_path: String, db_path: String) -> Result<()> {
    let config = Config::load(&config_path)?;
    let state_manager = StateManager::new(&db_path)?;
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "File Orchestrator",
        options,
        Box::new(|_cc| {
            Box::new(FileOrchestratorApp::new(config, state_manager, db_path))
        }),
    ).map_err(|e| crate::error::OrchestratorError::Config(format!("GUI error: {}", e)))?;
    
    Ok(())
}
