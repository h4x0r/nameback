use eframe::egui;
use egui_phosphor::regular;
use nameback_core::{DependencyNeeds, FileAnalysis, RenameConfig, RenameEngine, RenameResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
enum FileStatus {
    Pending,
    Renamed,
    Error(String),
}

#[derive(Debug, Clone)]
struct FileEntry {
    analysis: FileAnalysis,
    selected: bool,
    status: FileStatus,
}

pub struct NamebackApp {
    // Directory state
    current_directory: Option<PathBuf>,

    // File entries
    file_entries: Vec<FileEntry>,

    // UI state
    is_processing: bool,
    error_message: Option<String>,
    status_message: Option<String>,
    show_about_dialog: bool,

    // Dependency check dialog
    show_deps_dialog: bool,
    pending_directory: Option<PathBuf>,
    missing_deps: Option<DependencyNeeds>,
    installing_deps: bool,
    install_progress: Arc<Mutex<String>>,
    install_complete: Arc<Mutex<bool>>,

    // Configuration
    config: RenameConfig,

    // Processing
    processing_thread: Option<std::thread::JoinHandle<Result<Vec<FileAnalysis>, String>>>,
    rename_results: Arc<Mutex<Option<Vec<RenameResult>>>>,
}

impl NamebackApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_directory: None,
            file_entries: Vec::new(),
            is_processing: false,
            error_message: None,
            status_message: None,
            show_about_dialog: false,
            show_deps_dialog: false,
            pending_directory: None,
            missing_deps: None,
            installing_deps: false,
            install_progress: Arc::new(Mutex::new(String::new())),
            install_complete: Arc::new(Mutex::new(false)),
            config: RenameConfig::default(),
            processing_thread: None,
            rename_results: Arc::new(Mutex::new(None)),
        }
    }

    fn select_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.current_directory = Some(path.clone());

            // Check dependencies for this directory
            match nameback_core::detect_needed_dependencies(&path) {
                Ok(needs) => {
                    if needs.has_required_missing() || !needs.missing_optional.is_empty() {
                        self.show_deps_dialog = true;
                        self.pending_directory = Some(path);
                        self.missing_deps = Some(needs);
                    } else {
                        self.start_analysis(path);
                    }
                }
                Err(e) => {
                    log::warn!("Dependency check failed: {}", e);
                    self.start_analysis(path); // Proceed anyway
                }
            }
        }
    }

    fn start_analysis(&mut self, path: PathBuf) {
        self.is_processing = true;
        self.error_message = None;
        self.status_message = Some("Analyzing directory...".to_string());
        self.file_entries.clear();

        let config = self.config.clone();

        self.processing_thread = Some(std::thread::spawn(move || {
            let engine = RenameEngine::new(config);
            engine
                .analyze_directory(&path)
                .map_err(|e| e.to_string())
        }));
    }

    fn check_analysis_complete(&mut self) {
        if let Some(thread) = self.processing_thread.take() {
            if thread.is_finished() {
                match thread.join() {
                    Ok(Ok(analyses)) => {
                        self.file_entries = analyses
                            .into_iter()
                            .map(|analysis| FileEntry {
                                analysis,
                                selected: true, // Select by default
                                status: FileStatus::Pending,
                            })
                            .collect();

                        let total = self.file_entries.len();
                        let renameable = self
                            .file_entries
                            .iter()
                            .filter(|e| e.analysis.proposed_name.is_some())
                            .count();

                        self.status_message = Some(format!(
                            "Found {} files ({} can be renamed)",
                            total, renameable
                        ));
                        self.is_processing = false;
                    }
                    Ok(Err(e)) => {
                        self.error_message = Some(format!("Analysis failed: {}", e));
                        self.is_processing = false;
                    }
                    Err(_) => {
                        self.error_message = Some("Analysis thread panicked".to_string());
                        self.is_processing = false;
                    }
                }
            } else {
                // Put it back if not finished
                self.processing_thread = Some(thread);
            }
        }
    }

    fn execute_renames(&mut self) {
        let selected_analyses: Vec<FileAnalysis> = self
            .file_entries
            .iter()
            .filter(|e| e.selected && e.analysis.proposed_name.is_some())
            .map(|e| e.analysis.clone())
            .collect();

        if selected_analyses.is_empty() {
            self.error_message = Some("No files selected for renaming".to_string());
            return;
        }

        self.is_processing = true;
        self.status_message = Some(format!("Renaming {} files...", selected_analyses.len()));

        let config = self.config.clone();
        let rename_results = Arc::clone(&self.rename_results);

        std::thread::spawn(move || {
            let engine = RenameEngine::new(config);
            let results = engine.rename_files(&selected_analyses, false);

            let mut results_lock = rename_results.lock().unwrap();
            *results_lock = Some(results);
        });
    }

    fn check_rename_complete(&mut self) {
        let mut results_lock = self.rename_results.lock().unwrap();
        if let Some(results) = results_lock.take() {
            // Update file entry statuses
            for result in &results {
                if let Some(entry) = self
                    .file_entries
                    .iter_mut()
                    .find(|e| e.analysis.original_path == result.original_path)
                {
                    entry.status = if result.success {
                        FileStatus::Renamed
                    } else {
                        FileStatus::Error(
                            result.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
                        )
                    };
                }
            }

            let successful = results.iter().filter(|r| r.success).count();
            let failed = results.iter().filter(|r| !r.success).count();

            self.status_message = Some(format!(
                "Rename complete! {} successful, {} failed",
                successful, failed
            ));
            self.is_processing = false;
        }
    }

    fn select_all(&mut self) {
        for entry in &mut self.file_entries {
            if entry.analysis.proposed_name.is_some() {
                entry.selected = true;
            }
        }
    }

    fn deselect_all(&mut self) {
        for entry in &mut self.file_entries {
            entry.selected = false;
        }
    }

    fn install_dependencies(&mut self) {
        self.installing_deps = true;

        let progress = Arc::clone(&self.install_progress);
        let complete = Arc::clone(&self.install_complete);

        std::thread::spawn(move || {
            let result = nameback_core::install_dependencies_with_progress(Some(Box::new(
                move |msg: &str, pct: u8| {
                    let mut prog = progress.lock().unwrap();
                    *prog = format!("{} ({}%)", msg, pct);
                },
            )));

            let mut comp = complete.lock().unwrap();
            *comp = result.is_ok();
        });
    }

    fn check_install_complete(&mut self) {
        let complete = self.install_complete.lock().unwrap();
        if *complete {
            drop(complete); // Release lock before modifying self

            self.installing_deps = false;
            self.show_deps_dialog = false;

            // Reset completion flag
            *self.install_complete.lock().unwrap() = false;

            // Start analysis with the pending directory
            if let Some(path) = self.pending_directory.take() {
                self.start_analysis(path);
            }

            self.missing_deps = None;
        }
    }

    fn render_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Directory selection
            if ui.button(format!("{} Select Directory", regular::FOLDER)).clicked() {
                self.select_directory();
            }

            ui.separator();

            // Refresh button
            if let Some(dir) = &self.current_directory {
                if ui.button(format!("{} Refresh", regular::ARROW_CLOCKWISE)).clicked() && !self.is_processing {
                    self.start_analysis(dir.clone());
                }
            }

            ui.separator();

            // Select/Deselect buttons
            if ui.button(format!("{} Select All", regular::CHECK_SQUARE)).clicked() {
                self.select_all();
            }
            if ui.button(format!("{} Deselect All", regular::SQUARE)).clicked() {
                self.deselect_all();
            }

            ui.separator();

            // Rename button
            let selected_count = self
                .file_entries
                .iter()
                .filter(|e| e.selected && e.analysis.proposed_name.is_some())
                .count();

            let rename_button = egui::Button::new(format!("{} Rename {} Files", regular::LIGHTNING, selected_count));
            let can_rename = selected_count > 0 && !self.is_processing;

            if ui
                .add_enabled(can_rename, rename_button)
                .on_hover_text("Rename selected files")
                .clicked()
            {
                self.execute_renames();
            }

            ui.separator();

            // About button
            if ui.button(format!("{} About", regular::INFO)).clicked() {
                self.show_about_dialog = true;
            }
        });
    }

    fn render_dual_panes(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("file_grid")
                .num_columns(4)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header row
                    ui.label("");
                    ui.strong("Original Filename");
                    ui.label("→");
                    ui.strong("New Filename");
                    ui.end_row();

                    // File rows
                    for entry in self.file_entries.iter_mut() {
                        // Checkbox
                        let has_proposed_name = entry.analysis.proposed_name.is_some();
                        ui.add_enabled(has_proposed_name, egui::Checkbox::new(&mut entry.selected, ""));

                        // Original filename
                        ui.label(&entry.analysis.original_name);

                        // Arrow
                        ui.label("→");

                        // New filename with color coding
                        if let Some(new_name) = &entry.analysis.proposed_name {
                            let (text, color) = match &entry.status {
                                FileStatus::Pending => (new_name.as_str(), egui::Color32::LIGHT_BLUE),
                                FileStatus::Renamed => ("✓ Renamed", egui::Color32::GREEN),
                                FileStatus::Error(e) => {
                                    (e.as_str(), egui::Color32::RED)
                                }
                            };
                            ui.colored_label(color, text);
                        } else {
                            ui.colored_label(
                                egui::Color32::GRAY,
                                "(no suitable metadata)"
                            );
                        }

                        ui.end_row();
                    }
                });
        });
    }

    fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if let Some(dir) = &self.current_directory {
                ui.label(format!("{} {}", regular::FOLDER_OPEN, dir.display()));
                ui.separator();
            }

            let total = self.file_entries.len();
            let renameable = self
                .file_entries
                .iter()
                .filter(|e| e.analysis.proposed_name.is_some())
                .count();
            let selected = self
                .file_entries
                .iter()
                .filter(|e| e.selected && e.analysis.proposed_name.is_some())
                .count();
            let renamed = self
                .file_entries
                .iter()
                .filter(|e| matches!(e.status, FileStatus::Renamed))
                .count();

            ui.label(format!("{} Total: {}", regular::FILE, total));
            ui.separator();
            ui.label(format!("{} Renameable: {}", regular::CHECK_CIRCLE, renameable));
            ui.separator();
            ui.label(format!("{} Selected: {}", regular::CHECK_SQUARE, selected));
            ui.separator();
            ui.label(format!("{} Renamed: {}", regular::CHECK, renamed));

            if self.is_processing {
                ui.separator();
                ui.spinner();
                if let Some(msg) = &self.status_message {
                    ui.label(msg);
                }
            }
        });
    }
}

impl eframe::App for NamebackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for background task completion
        if self.is_processing {
            self.check_analysis_complete();
            self.check_rename_complete();
            ctx.request_repaint(); // Keep refreshing while processing
        }

        // Check for dependency installation completion
        if self.installing_deps {
            self.check_install_complete();
            ctx.request_repaint(); // Keep refreshing during installation
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("nameback v{} - File Renaming Tool", env!("CARGO_PKG_VERSION")));
            ui.add_space(10.0);

            // Control buttons
            self.render_controls(ui);
            ui.add_space(10.0);

            // Error message
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("{} {}", regular::X_CIRCLE, error));
                ui.add_space(10.0);
            }

            // Status message
            if let Some(status) = &self.status_message {
                if !self.is_processing {
                    ui.colored_label(egui::Color32::GREEN, format!("{} {}", regular::INFO, status));
                    ui.add_space(10.0);
                }
            }

            // Dual-pane file list
            if !self.file_entries.is_empty() {
                self.render_dual_panes(ui);
            } else if !self.is_processing {
                ui.centered_and_justified(|ui| {
                    ui.heading("Select a directory to begin");
                });
            }
        });

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });

        // Dependency check modal dialog
        if self.show_deps_dialog {
            egui::Window::new("Dependencies Required")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(ref needs) = self.missing_deps {
                        if !needs.missing_required.is_empty() {
                            ui.heading(format!("{} Required Dependencies Missing", regular::WARNING));
                            ui.add_space(10.0);
                            ui.label("The following required dependencies are not installed:");
                            ui.add_space(5.0);

                            for dep in &needs.missing_required {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}", regular::X_CIRCLE));
                                    ui.strong(dep.name());
                                    ui.label("-");
                                    ui.label(dep.description());
                                });
                            }
                            ui.add_space(10.0);
                        }

                        if !needs.missing_optional.is_empty() {
                            if needs.missing_required.is_empty() {
                                ui.heading("Optional Dependencies Missing");
                            } else {
                                ui.heading("Optional Dependencies Also Missing");
                            }
                            ui.add_space(10.0);
                            ui.label("The following optional dependencies would improve functionality:");
                            ui.add_space(5.0);

                            for dep in &needs.missing_optional {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}", regular::WARNING));
                                    ui.strong(dep.name());
                                    ui.label("-");
                                    ui.label(dep.description());
                                });
                            }
                            ui.add_space(10.0);
                        }

                        if self.installing_deps {
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.spinner();
                                let progress = self.install_progress.lock().unwrap();
                                ui.label(progress.as_str());
                            });
                            ui.add_space(10.0);
                        } else {
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.button(format!("{} Install Dependencies", regular::DOWNLOAD_SIMPLE)).clicked() {
                                    self.install_dependencies();
                                }

                                if ui.button(format!("{} Skip", regular::SKIP_FORWARD)).clicked() {
                                    self.show_deps_dialog = false;
                                    if let Some(path) = self.pending_directory.take() {
                                        self.start_analysis(path);
                                    }
                                    self.missing_deps = None;
                                }

                                if ui.button(format!("{} Cancel", regular::X)).clicked() {
                                    self.show_deps_dialog = false;
                                    self.pending_directory = None;
                                    self.missing_deps = None;
                                }
                            });
                        }
                    }
                });
        }

        // About dialog
        if self.show_about_dialog {
            egui::Window::new("About nameback")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading(format!("nameback v{}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(20.0);

                        ui.label("Copyright (c) 2025 Albert Hui");
                        ui.hyperlink_to("albert@securityronin.com", "mailto:albert@securityronin.com");
                        ui.add_space(10.0);

                        ui.label("License: MIT");
                        ui.hyperlink_to(
                            "https://github.com/h4x0r/nameback",
                            "https://github.com/h4x0r/nameback"
                        );
                        ui.add_space(20.0);

                        if ui.button("Close").clicked() {
                            self.show_about_dialog = false;
                        }
                        ui.add_space(10.0);
                    });
                });
        }
    }
}
