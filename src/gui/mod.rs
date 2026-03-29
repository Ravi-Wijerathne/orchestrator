use eframe::egui;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::config::Config;
use crate::state::StateManager;
use crate::drive::DriveDetector;
use crate::error::Result;

// ── Color palette ──────────────────────────────────────────────────────
struct Theme;
impl Theme {
    // Background layers
    const BG_BASE:     egui::Color32 = egui::Color32::from_rgb(245, 247, 250);
    const BG_CARD:     egui::Color32 = egui::Color32::WHITE;
    const BG_SIDEBAR:  egui::Color32 = egui::Color32::from_rgb(30, 41, 59);   // slate-800
    const BG_NAV_HOVER:egui::Color32 = egui::Color32::from_rgb(51, 65, 85);   // slate-700

    // Accent
    const ACCENT:      egui::Color32 = egui::Color32::from_rgb(59, 130, 246); // blue-500
    const ACCENT_HOVER:egui::Color32 = egui::Color32::from_rgb(37, 99, 235);  // blue-600
    const ACCENT_LIGHT:egui::Color32 = egui::Color32::from_rgb(219, 234, 254);// blue-100

    // Semantic
    const SUCCESS:     egui::Color32 = egui::Color32::from_rgb(34, 197, 94);  // green-500
    const SUCCESS_BG:  egui::Color32 = egui::Color32::from_rgb(220, 252, 231);
    const DANGER:      egui::Color32 = egui::Color32::from_rgb(239, 68, 68);  // red-500
    const DANGER_BG:   egui::Color32 = egui::Color32::from_rgb(254, 226, 226);
    const WARNING:     egui::Color32 = egui::Color32::from_rgb(245, 158, 11); // amber-500
    const WARNING_BG:  egui::Color32 = egui::Color32::from_rgb(254, 243, 199);

    // Text
    const TEXT_PRIMARY:   egui::Color32 = egui::Color32::from_rgb(15, 23, 42);   // slate-900
    const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);// slate-500
    const TEXT_ON_DARK:   egui::Color32 = egui::Color32::from_rgb(226, 232, 240);// slate-200
    const TEXT_NAV_ACTIVE:egui::Color32 = egui::Color32::WHITE;

    // Borders
    const BORDER:      egui::Color32 = egui::Color32::from_rgb(226, 232, 240);  // slate-200
}

pub struct FileOrchestratorApp {
    config: Arc<Mutex<Config>>,
    state_manager: Arc<Mutex<StateManager>>,
    drive_detector: Arc<Mutex<DriveDetector>>,
    current_view: AppView,

    // Dashboard data
    pending_count: usize,
    drives_status: Vec<(String, String, bool)>,

    // Drive registration form
    new_drive_label: String,
    new_drive_category: String,
    selected_path: Option<PathBuf>,

    // Status messages
    status_message: Option<String>,
    error_message: Option<String>,

    // Drive to remove (uuid)
    drive_to_remove: Option<String>,

    // Watcher control
    watcher_running: Arc<Mutex<bool>>,
    watcher_handle: Arc<Mutex<Option<std::process::Child>>>,
    config_path: String,
    db_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppView {
    Dashboard,
    DriveManager,
    Settings,
}

impl AppView {
    fn icon(&self) -> &str {
        match self {
            AppView::Dashboard => "[DB]",
            AppView::DriveManager => "[DR]",
            AppView::Settings => "[ST]",
        }
    }
    fn label(&self) -> &str {
        match self {
            AppView::Dashboard => "Dashboard",
            AppView::DriveManager => "Drives",
            AppView::Settings => "Settings",
        }
    }
}

// ── Helper: paint a rounded card background ────────────────────────────
fn card_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(Theme::BG_CARD)
        .rounding(egui::Rounding::same(10.0))
        .stroke(egui::Stroke::new(1.0, Theme::BORDER))
        .shadow(egui::epaint::Shadow {
            extrusion: 8.0,
            color: egui::Color32::from_black_alpha(12),
        })
        .inner_margin(egui::Margin::same(20.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

fn section_heading(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(18.0));
        ui.label(
            egui::RichText::new(title)
                .size(18.0)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
    });
    ui.add_space(4.0);
    // Thin accent line under heading
    let rect = ui.available_rect_before_wrap();
    let line_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width(), 2.0),
    );
    ui.painter().rect_filled(line_rect, 1.0, Theme::ACCENT);
    ui.add_space(8.0);
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, accent: egui::Color32, bg: egui::Color32) {
    egui::Frame::none()
        .fill(bg)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(20.0))
        .show(ui, |ui| {
            ui.set_min_width(160.0);
            ui.set_min_height(70.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(12.0)
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(value)
                        .size(32.0)
                        .strong()
                        .color(accent),
                );
            });
        });
}

fn styled_button(ui: &mut egui::Ui, text: &str, fill: egui::Color32, hover: egui::Color32) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(13.0),
    )
    .fill(fill)
    .rounding(egui::Rounding::same(6.0))
    .min_size(egui::vec2(0.0, 32.0));
    let resp = ui.add(btn);
    if resp.hovered() {
        ui.painter().rect_filled(resp.rect, 6.0, hover.linear_multiply(0.15));
    }
    resp
}

fn pill_badge(ui: &mut egui::Ui, text: &str, fg: egui::Color32, bg: egui::Color32) {
    egui::Frame::none()
        .fill(bg)
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::symmetric(10.0, 3.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.0).strong().color(fg));
        });
}

impl FileOrchestratorApp {
    pub fn new(
        config: Config,
        state_manager: StateManager,
        db_path: String,
        config_path: String,
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
            drive_to_remove: None,
            watcher_running: Arc::new(Mutex::new(false)),
            watcher_handle: Arc::new(Mutex::new(None)),
            config_path,
            db_path,
        }
    }

    fn update_dashboard_stats(&mut self) {
        let config = self.config.lock().unwrap();
        let mut detector = self.drive_detector.lock().unwrap();
        detector.refresh();

        self.drives_status.clear();
        for (uuid, drive_config) in &config.drives {
            let connected = if let Some(ref path) = drive_config.path {
                detector.is_drive_connected(path)
            } else {
                detector.find_drive_by_label(&drive_config.label).is_some()
            };
            self.drives_status.push((
                uuid.clone(),
                drive_config.label.clone(),
                connected,
            ));
        }

        let state = self.state_manager.lock().unwrap();
        let mut total_pending = 0;
        for (uuid, _) in &config.drives {
            if let Ok(pending) = state.get_pending_syncs(uuid) {
                total_pending += pending.len();
            }
        }
        self.pending_count = total_pending;
    }

    // ── Dashboard ──────────────────────────────────────────────────────
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        section_heading(ui, "[DB]", "Dashboard");
        ui.add_space(8.0);

        // Stat cards row
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            stat_card(
                ui,
                "PENDING SYNCS",
                &self.pending_count.to_string(),
                Theme::ACCENT,
                Theme::ACCENT_LIGHT,
            );
            stat_card(
                ui,
                "REGISTERED DRIVES",
                &self.drives_status.len().to_string(),
                Theme::SUCCESS,
                Theme::SUCCESS_BG,
            );
        });

        ui.add_space(24.0);

        // Drive status card
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("[DR]").size(16.0));
                ui.label(
                    egui::RichText::new("Drive Status")
                        .size(15.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
            });
            ui.add_space(12.0);

            if self.drives_status.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("No drives registered. Go to ")
                            .color(Theme::TEXT_SECONDARY),
                    );
                    if ui
                        .link(egui::RichText::new("Drive Manager").color(Theme::ACCENT))
                        .clicked()
                    {
                        self.current_view = AppView::DriveManager;
                    }
                    ui.label(
                        egui::RichText::new(" to add drives.").color(Theme::TEXT_SECONDARY),
                    );
                });
            } else {
                let drives = self.drives_status.clone();
                for (i, (_uuid, label, connected)) in drives.iter().enumerate() {
                    ui.horizontal(|ui| {
                        // Status dot
                        let dot_color = if *connected { Theme::SUCCESS } else { Theme::DANGER };
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(10.0, 10.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(rect.center(), 5.0, dot_color);
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(label)
                                .size(14.0)
                                .color(Theme::TEXT_PRIMARY),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if *connected {
                                pill_badge(ui, "Connected", Theme::SUCCESS, Theme::SUCCESS_BG);
                            } else {
                                pill_badge(ui, "Disconnected", Theme::DANGER, Theme::DANGER_BG);
                            }
                        });
                    });
                    if i + 1 < drives.len() {
                        ui.add_space(4.0);
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 1.0)),
                            0.0,
                            Theme::BORDER,
                        );
                        ui.add_space(6.0);
                    }
                }
            }
        });

        ui.add_space(24.0);

        // Watcher card
        card_frame(ui, |ui| {
            let is_running = *self.watcher_running.lock().unwrap();

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("[FW]").size(16.0));
                ui.label(
                    egui::RichText::new("File Watcher")
                        .size(15.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                if is_running {
                    pill_badge(ui, "Running", Theme::SUCCESS, Theme::SUCCESS_BG);
                } else {
                    pill_badge(ui, "Stopped", Theme::DANGER, Theme::DANGER_BG);
                }
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if is_running {
                    if styled_button(ui, "Stop Watcher", Theme::DANGER, Theme::DANGER).clicked() {
                        self.stop_watcher();
                    }
                } else {
                    if styled_button(ui, "Start Watcher", Theme::SUCCESS, Theme::SUCCESS).clicked() {
                        self.start_watcher();
                    }
                }
                ui.add_space(8.0);
                if styled_button(ui, "Refresh Status", Theme::ACCENT, Theme::ACCENT_HOVER).clicked() {
                    self.update_dashboard_stats();
                    self.status_message = Some("Status refreshed".to_string());
                }
            });
        });
    }

    // ── Drive Manager ──────────────────────────────────────────────────
    fn show_drive_manager(&mut self, ui: &mut egui::Ui) {
        section_heading(ui, "[DR]", "Drive Manager");
        ui.add_space(8.0);

        // Registered drives card
        card_frame(ui, |ui| {
            ui.label(
                egui::RichText::new("Registered Drives")
                    .size(15.0)
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );
            ui.add_space(12.0);

            let config = self.config.lock().unwrap();
            if config.drives.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("[--]").size(24.0));
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("No drives registered yet")
                            .size(14.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new("Use the form below to add your first drive.")
                            .size(12.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                });
                ui.add_space(20.0);
            } else {
                let drives: Vec<_> = config
                    .drives
                    .iter()
                    .map(|(uuid, drive)| (uuid.clone(), drive.clone()))
                    .collect();
                drop(config);

                for (i, (uuid, drive_config)) in drives.iter().enumerate() {
                    egui::Frame::none()
                        .fill(Theme::BG_BASE)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                // Drive icon
                                ui.label(egui::RichText::new("[DR]").size(18.0));
                                ui.add_space(8.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&drive_config.label)
                                            .size(14.0)
                                            .strong()
                                            .color(Theme::TEXT_PRIMARY),
                                    );
                                    ui.horizontal(|ui| {
                                        pill_badge(ui, &drive_config.target, Theme::ACCENT, Theme::ACCENT_LIGHT);
                                        if let Some(path) = &drive_config.path {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(path.display().to_string())
                                                    .size(11.0)
                                                    .color(Theme::TEXT_SECONDARY),
                                            );
                                        }
                                    });
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if styled_button(ui, "Remove", Theme::DANGER, Theme::DANGER)
                                            .clicked()
                                        {
                                            self.drive_to_remove = Some(uuid.clone());
                                        }
                                    },
                                );
                            });
                        });

                    if i + 1 < drives.len() {
                        ui.add_space(6.0);
                    }
                }
            }
        });

        ui.add_space(20.0);

        // Add new drive card
        card_frame(ui, |ui| {
            ui.label(
                egui::RichText::new("+ Add New Drive")
                    .size(15.0)
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );
            ui.add_space(16.0);

            // Form grid
            egui::Grid::new("drive_form")
                .num_columns(2)
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Label")
                            .size(13.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    let te = egui::TextEdit::singleline(&mut self.new_drive_label)
                        .desired_width(280.0)
                        .hint_text("e.g. My USB Drive")
                        .margin(egui::vec2(8.0, 6.0));
                    ui.add(te);
                    ui.end_row();

                    ui.label(
                        egui::RichText::new("Category")
                            .size(13.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    egui::ComboBox::from_id_source("category")
                        .selected_text(
                            egui::RichText::new(&self.new_drive_category).size(13.0),
                        )
                        .width(280.0)
                        .show_ui(ui, |ui| {
                            for (val, label) in [
                                ("images", "Images"),
                                ("videos", "Videos"),
                                ("music", "Music"),
                                ("documents", "Documents"),
                                ("archives", "Archives"),
                            ] {
                                ui.selectable_value(
                                    &mut self.new_drive_category,
                                    val.to_string(),
                                    label,
                                );
                            }
                        });
                    ui.end_row();

                    ui.label(
                        egui::RichText::new("Path")
                            .size(13.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.horizontal(|ui| {
                        if styled_button(ui, "Browse...", Theme::ACCENT, Theme::ACCENT_HOVER)
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.selected_path = Some(path);
                            }
                        }
                        if let Some(ref path) = self.selected_path {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(path.display().to_string())
                                    .size(12.0)
                                    .color(Theme::TEXT_SECONDARY),
                            );
                        }
                    });
                    ui.end_row();
                });

            ui.add_space(16.0);

            if styled_button(ui, "Register Drive", Theme::ACCENT, Theme::ACCENT_HOVER).clicked() {
                if self.new_drive_label.is_empty() {
                    self.error_message = Some("Label cannot be empty".to_string());
                } else if self.selected_path.is_none() {
                    self.error_message = Some("Please select a drive path".to_string());
                } else {
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
                        config.save(&self.config_path)
                    };

                    if let Err(e) = save_result {
                        self.error_message = Some(format!("Failed to save config: {}", e));
                    } else {
                        self.status_message = Some(format!(
                            "Drive '{}' registered successfully",
                            self.new_drive_label
                        ));
                        self.new_drive_label.clear();
                        self.selected_path = None;
                        self.update_dashboard_stats();
                    }
                }
            }
        });

        // Handle drive removal
        if let Some(uuid) = self.drive_to_remove.take() {
            self.unregister_drive(&uuid);
        }
    }

    fn unregister_drive(&mut self, uuid: &str) {
        let mut config = self.config.lock().unwrap();

        if let Some(drive) = config.drives.remove(uuid) {
            let save_result = config.save(&self.config_path);
            drop(config);

            if let Err(e) = save_result {
                self.error_message = Some(format!("Failed to save config: {}", e));
            } else {
                let cleanup_result = {
                    let state = self.state_manager.lock().unwrap();
                    state.cleanup_drive_data(uuid)
                };

                if let Err(e) = cleanup_result {
                    self.error_message =
                        Some(format!("Warning: Failed to cleanup drive data: {}", e));
                } else {
                    self.status_message =
                        Some(format!("Drive '{}' unregistered successfully", drive.label));
                    self.update_dashboard_stats();
                }
            }
        } else {
            self.error_message = Some("Drive not found".to_string());
        }
    }

    fn start_watcher(&mut self) {
        use std::process::Command;

        let binary_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("./target/release/fo"));

        match Command::new(&binary_path)
            .arg("run")
            .arg("--interval")
            .arg("5")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => {
                *self.watcher_running.lock().unwrap() = true;
                *self.watcher_handle.lock().unwrap() = Some(child);
                self.status_message = Some("File watcher started successfully".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to start watcher: {}", e));
            }
        }
    }

    fn stop_watcher(&mut self) {
        let mut handle = self.watcher_handle.lock().unwrap();

        if let Some(mut child) = handle.take() {
            if let Err(e) = child.kill() {
                self.error_message = Some(format!("Failed to stop watcher: {}", e));
            } else {
                *self.watcher_running.lock().unwrap() = false;
                self.status_message = Some("File watcher stopped".to_string());
            }
        }
    }

    // ── Settings ───────────────────────────────────────────────────────
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        section_heading(ui, "[ST]", "Settings");
        ui.add_space(8.0);

        let source_exists = self.config.lock().unwrap().source.path.exists();
        let current_path = self.config.lock().unwrap().source.path.display().to_string();

        // Source directory card
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("[DIR]").size(16.0));
                ui.label(
                    egui::RichText::new("Source Directory")
                        .size(15.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
            });
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Path:")
                        .size(13.0)
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                if source_exists {
                    egui::Frame::none()
                        .fill(Theme::SUCCESS_BG)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&current_path)
                                    .size(13.0)
                                    .color(Theme::SUCCESS)
                                    .strong(),
                            );
                        });
                } else {
                    egui::Frame::none()
                        .fill(Theme::DANGER_BG)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("{}  (does not exist)", &current_path))
                                    .size(13.0)
                                    .color(Theme::DANGER)
                                    .strong(),
                            );
                        });
                }
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if styled_button(ui, "Change Source Path", Theme::ACCENT, Theme::ACCENT_HOVER)
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let mut config = self.config.lock().unwrap();
                        config.source.path = path;
                        if let Err(e) = config.save(&self.config_path) {
                            drop(config);
                            self.error_message = Some(format!("Failed to save config: {}", e));
                        } else {
                            let p = config.source.path.display().to_string();
                            drop(config);
                            self.status_message =
                                Some(format!("Source path updated to: {}", p));
                        }
                    }
                }

                if !source_exists {
                    ui.add_space(8.0);
                    if styled_button(ui, "Create This Directory", Theme::SUCCESS, Theme::SUCCESS)
                        .clicked()
                    {
                        let path = self.config.lock().unwrap().source.path.clone();
                        match std::fs::create_dir_all(&path) {
                            Ok(_) => {
                                self.status_message =
                                    Some(format!("Directory created: {}", path.display()));
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Failed to create directory: {}", e));
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(20.0);

        // File rules card
        let config = self.config.lock().unwrap();
        card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("[RULES]").size(16.0));
                ui.label(
                    egui::RichText::new("File Classification Rules")
                        .size(15.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
            });
            ui.add_space(12.0);

            let rules: Vec<(&str, &str, Vec<&str>)> = {
                let mut r = vec![
                    ("IMG", "Images", config.rules.images.iter().map(|s| s.as_str()).collect()),
                    ("VID", "Videos", config.rules.videos.iter().map(|s| s.as_str()).collect()),
                    ("AUD", "Music", config.rules.music.iter().map(|s| s.as_str()).collect()),
                ];
                if let Some(docs) = &config.rules.documents {
                    r.push(("DOC", "Documents", docs.iter().map(|s| s.as_str()).collect()));
                }
                if let Some(archives) = &config.rules.archives {
                    r.push(("ARC", "Archives", archives.iter().map(|s| s.as_str()).collect()));
                }
                r
            };

            egui::Grid::new("rules_grid")
                .num_columns(3)
                .spacing([12.0, 10.0])
                .striped(true)
                .show(ui, |ui| {
                    for (icon, category, extensions) in &rules {
                        ui.label(egui::RichText::new(*icon).size(16.0));
                        ui.label(
                            egui::RichText::new(*category)
                                .size(13.0)
                                .strong()
                                .color(Theme::TEXT_PRIMARY),
                        );
                        ui.horizontal_wrapped(|ui| {
                            for ext in extensions {
                                egui::Frame::none()
                                    .fill(Theme::BG_BASE)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                    .stroke(egui::Stroke::new(1.0, Theme::BORDER))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(*ext)
                                                .size(11.0)
                                                .color(Theme::TEXT_SECONDARY),
                                        );
                                    });
                            }
                        });
                        ui.end_row();
                    }
                });
        });
    }
}

impl Drop for FileOrchestratorApp {
    fn drop(&mut self) {
        let mut handle = self.watcher_handle.lock().unwrap();
        if let Some(mut child) = handle.take() {
            let _ = child.kill();
        }
    }
}

impl eframe::App for FileOrchestratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Apply global visual style ──────────────────────────────────
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Theme::BORDER);
        style.visuals.window_rounding = egui::Rounding::same(10.0);
        style.visuals.override_text_color = Some(Theme::TEXT_PRIMARY);
        ctx.set_style(style);

        // ── Left sidebar navigation ────────────────────────────────────
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .exact_width(180.0)
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_SIDEBAR)
                    .inner_margin(egui::Margin {
                        left: 0.0,
                        right: 0.0,
                        top: 0.0,
                        bottom: 0.0,
                    }),
            )
            .show(ctx, |ui| {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("File Orchestrator")
                            .size(15.0)
                            .strong()
                            .color(Theme::TEXT_NAV_ACTIVE),
                    );
                });
                ui.add_space(24.0);

                for view in [AppView::Dashboard, AppView::DriveManager, AppView::Settings] {
                    let is_active = self.current_view == view;
                    let text_color = if is_active {
                        Theme::TEXT_NAV_ACTIVE
                    } else {
                        Theme::TEXT_ON_DARK
                    };
                    let bg = if is_active {
                        Theme::ACCENT
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    let btn = egui::Frame::none()
                        .fill(bg)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(view.icon())
                                        .size(16.0)
                                        .color(text_color),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(view.label())
                                        .size(14.0)
                                        .color(text_color),
                                );
                            });
                        });

                    let resp = ui.interact(
                        btn.response.rect,
                        ui.id().with(view.label()),
                        egui::Sense::click(),
                    );
                    if resp.hovered() && !is_active {
                        ui.painter().rect_filled(
                            btn.response.rect,
                            8.0,
                            Theme::BG_NAV_HOVER,
                        );
                        // Re-draw text on top of hover bg
                        let text_rect = btn.response.rect;
                        ui.painter().text(
                            text_rect.left_center() + egui::vec2(16.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            format!("{}   {}", view.icon(), view.label()),
                            egui::FontId::proportional(14.0),
                            Theme::TEXT_ON_DARK,
                        );
                    }
                    if resp.clicked() {
                        self.current_view = view;
                        if view == AppView::Dashboard {
                            self.update_dashboard_stats();
                        }
                    }
                    ui.add_space(2.0);
                }
            });

        // ── Bottom status bar ──────────────────────────────────────────
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_CARD)
                    .stroke(egui::Stroke::new(1.0, Theme::BORDER))
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref msg) = self.status_message {
                        egui::Frame::none()
                            .fill(Theme::SUCCESS_BG)
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("OK: {}", msg))
                                            .size(12.0)
                                            .color(Theme::SUCCESS),
                                    );
                                });
                            });
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("x").size(11.0).color(Theme::TEXT_SECONDARY),
                            ).frame(false))
                            .clicked()
                        {
                            self.status_message = None;
                        }
                    }

                    if let Some(ref msg) = self.error_message {
                        egui::Frame::none()
                            .fill(Theme::DANGER_BG)
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("ERROR: {}", msg))
                                            .size(12.0)
                                            .color(Theme::DANGER),
                                    );
                                });
                            });
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("x").size(11.0).color(Theme::TEXT_SECONDARY),
                            ).frame(false))
                            .clicked()
                        {
                            self.error_message = None;
                        }
                    }
                });
            });

        // ── Central content area ───────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_BASE)
                    .inner_margin(egui::Margin::same(28.0)),
            )
            .show(ctx, |ui| {
                // Warning banner
                let source_exists = self.config.lock().unwrap().source.path.exists();
                if !source_exists {
                    egui::Frame::none()
                        .fill(Theme::WARNING_BG)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("WARNING: Source path does not exist.")
                                        .size(13.0)
                                        .color(Theme::WARNING)
                                        .strong(),
                                );
                                ui.add_space(8.0);
                                if ui
                                    .link(
                                        egui::RichText::new("Go to Settings >")
                                            .size(13.0)
                                            .color(Theme::ACCENT)
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    self.current_view = AppView::Settings;
                                }
                            });
                        });
                    ui.add_space(16.0);
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
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
    let config = Config::load_lenient(&config_path)?;
    let state_manager = StateManager::new(&db_path)?;

    let config_path_clone = config_path.clone();
    let db_path_clone = db_path.clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1050.0, 720.0])
            .with_min_inner_size([850.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "File Orchestrator",
        options,
        Box::new(move |cc| {
            // Set up custom fonts / visuals at creation time
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = Theme::BG_BASE;
            visuals.window_fill = Theme::BG_CARD;
            visuals.window_rounding = egui::Rounding::same(10.0);
            visuals.widgets.noninteractive.bg_fill = Theme::BG_CARD;
            visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Theme::TEXT_PRIMARY);
            visuals.selection.bg_fill = Theme::ACCENT_LIGHT;
            visuals.selection.stroke = egui::Stroke::new(1.0, Theme::ACCENT);
            cc.egui_ctx.set_visuals(visuals);

            Box::new(FileOrchestratorApp::new(
                config,
                state_manager,
                db_path_clone,
                config_path_clone,
            ))
        }),
    )
    .map_err(|e| crate::error::OrchestratorError::Config(format!("GUI error: {}", e)))?;

    Ok(())
}
