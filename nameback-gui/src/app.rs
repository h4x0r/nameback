use eframe::egui;
use egui_phosphor::regular;
use nameback_core::{DependencyNeeds, FileAnalysis, RenameConfig, RenameEngine, RenameHistory, RenameResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
enum FileStatus {
    Pending,
    Processing(String), // Contains operation message like "Extracting metadata..."
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
    dark_mode: bool,
    security_ronin_logo: Option<egui::TextureHandle>,

    // Search state
    show_search: bool,
    search_query: String,
    search_results: Vec<usize>, // Indices of matching entries
    current_search_index: usize,
    scroll_to_index: Option<usize>, // Request to scroll to specific index

    // Pattern selection dialog
    show_pattern_dialog: bool,
    pattern_query: String,
    pattern_error: Option<String>,

    // Dependency check dialog
    show_deps_dialog: bool,
    pending_directory: Option<PathBuf>,
    missing_deps: Option<DependencyNeeds>,
    installing_deps: bool,
    install_progress: Arc<Mutex<String>>,
    install_complete: Arc<Mutex<bool>>,

    // Configuration
    config: RenameConfig,

    // History tracking
    rename_history: Option<RenameHistory>,
    show_history_dialog: bool,

    // Processing
    processing_thread: Option<std::thread::JoinHandle<Result<(), String>>>,
    rename_results: Arc<Mutex<Option<Vec<RenameResult>>>>,
    shared_file_entries: Arc<Mutex<Vec<FileEntry>>>,
}

impl NamebackApp {
    /// Creates a high-contrast light theme with better readability
    fn create_light_theme() -> egui::Visuals {
        let mut visuals = egui::Visuals::light();

        // Higher contrast text colors
        visuals.override_text_color = Some(egui::Color32::from_gray(20)); // Almost black text
        visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_gray(20);

        // Better contrast for widgets
        visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_gray(240);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(235);
        visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_gray(220);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(210);
        visuals.widgets.active.weak_bg_fill = egui::Color32::from_gray(200);
        visuals.widgets.active.bg_fill = egui::Color32::from_gray(190);

        // Stronger borders for better definition
        visuals.widgets.inactive.bg_stroke.color = egui::Color32::from_gray(180);
        visuals.widgets.hovered.bg_stroke.color = egui::Color32::from_gray(140);
        visuals.widgets.active.bg_stroke.color = egui::Color32::from_gray(100);

        // Panel backgrounds with better contrast
        visuals.panel_fill = egui::Color32::from_gray(250); // Slightly off-white
        visuals.window_fill = egui::Color32::from_gray(248);
        visuals.faint_bg_color = egui::Color32::from_gray(240);

        visuals
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Use system theme preference by default
        // egui automatically detects system dark/light mode on supported platforms
        let dark_mode = cc.egui_ctx.style().visuals.dark_mode;
        if dark_mode {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(Self::create_light_theme());
        }

        Self {
            current_directory: None,
            file_entries: Vec::new(),
            is_processing: false,
            error_message: None,
            status_message: None,
            show_about_dialog: false,
            dark_mode,
            security_ronin_logo: None,
            show_search: false,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            scroll_to_index: None,
            show_pattern_dialog: false,
            pattern_query: String::new(),
            pattern_error: None,
            show_deps_dialog: false,
            pending_directory: None,
            missing_deps: None,
            installing_deps: false,
            install_progress: Arc::new(Mutex::new(String::new())),
            install_complete: Arc::new(Mutex::new(false)),
            config: RenameConfig::default(),
            rename_history: None,
            show_history_dialog: false,
            processing_thread: None,
            rename_results: Arc::new(Mutex::new(None)),
            shared_file_entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn select_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.current_directory = Some(path.clone());

            // Check dependencies for this directory
            match nameback_core::detect_needed_dependencies(&path) {
                Ok(needs) => {
                    log::debug!("Dependency check results - required missing: {}, optional missing: {}",
                        needs.has_required_missing(),
                        !needs.missing_optional.is_empty());

                    if needs.has_required_missing() || !needs.missing_optional.is_empty() {
                        log::info!("Showing dependency dialog");
                        self.show_deps_dialog = true;
                        self.pending_directory = Some(path);
                        self.missing_deps = Some(needs);
                    } else {
                        log::info!("All required dependencies available, starting analysis");
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
        self.status_message = Some("Scanning directory...".to_string());
        self.file_entries.clear();

        let config = self.config.clone();
        let file_entries = Arc::new(Mutex::new(Vec::new()));
        let file_entries_clone = Arc::clone(&file_entries);

        // Spawn thread to scan directory and analyze files progressively
        self.processing_thread = Some(std::thread::spawn(move || {
            use walkdir::WalkDir;

            // First, scan directory to get list of files
            let mut files = Vec::new();
            for entry in WalkDir::new(&path)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| {
                    if config.skip_hidden {
                        !e.file_name()
                            .to_str()
                            .map(|s| s.starts_with('.'))
                            .unwrap_or(false)
                    } else {
                        true
                    }
                })
            {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to access entry: {}", e);
                    }
                }
            }

            // Create placeholder entries for all files
            let mut entries_lock = file_entries_clone.lock().unwrap();
            for file_path in &files {
                let original_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                entries_lock.push(FileEntry {
                    analysis: FileAnalysis {
                        original_path: file_path.clone(),
                        original_name: original_name.clone(),
                        proposed_name: None, // Will be filled in progressively
                        file_category: nameback_core::FileCategory::Unknown,
                    },
                    selected: true,
                    status: FileStatus::Pending,
                });
            }
            drop(entries_lock);

            // Set all files to Processing status
            {
                let mut entries_lock = file_entries_clone.lock().unwrap();
                for entry in entries_lock.iter_mut() {
                    entry.status = FileStatus::Processing("Analyzing...".to_string());
                }
            }

            // Now analyze each file and update progressively
            let engine = RenameEngine::new(config);
            for analysis in engine.analyze_directory(&path).map_err(|e| e.to_string())? {
                // Find and update the matching entry
                let mut entries_lock = file_entries_clone.lock().unwrap();
                if let Some(entry) = entries_lock
                    .iter_mut()
                    .find(|e| e.analysis.original_path == analysis.original_path)
                {
                    // Update analysis result
                    entry.analysis = analysis.clone();

                    // Update status based on whether we got a proposed name
                    if analysis.proposed_name.is_some() {
                        entry.status = FileStatus::Pending; // Ready for rename
                    } else {
                        entry.status = FileStatus::Error("No suitable metadata found".to_string());
                    }
                }
            }

            Ok(())
        }));

        // Store reference for UI updates
        self.shared_file_entries = file_entries;
    }

    fn check_analysis_complete(&mut self) {
        // Update file_entries from shared state (progressive updates)
        {
            let shared = self.shared_file_entries.lock().unwrap();
            self.file_entries = shared.clone();
        }

        // Update status message with progress
        let total = self.file_entries.len();
        let analyzed = self
            .file_entries
            .iter()
            .filter(|e| e.analysis.proposed_name.is_some() || e.analysis.file_category != nameback_core::FileCategory::Unknown)
            .count();

        if self.is_processing && total > 0 {
            self.status_message = Some(format!(
                "Analyzing... {}/{} files processed",
                analyzed, total
            ));
        }

        // Check if background thread is complete
        if let Some(thread) = self.processing_thread.take() {
            if thread.is_finished() {
                match thread.join() {
                    Ok(Ok(())) => {
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

    fn invert_selection(&mut self) {
        for entry in &mut self.file_entries {
            // Only invert if the entry has a proposed name (can be selected)
            if entry.analysis.proposed_name.is_some() {
                entry.selected = !entry.selected;
            }
        }
    }

    fn select_by_pattern(&mut self) {
        use regex::Regex;

        // Clear any previous error
        self.pattern_error = None;

        // Try to compile the regex
        let re = match Regex::new(&self.pattern_query) {
            Ok(r) => r,
            Err(e) => {
                self.pattern_error = Some(format!("Invalid regex: {}", e));
                return;
            }
        };

        // Select files matching the pattern
        for entry in &mut self.file_entries {
            if entry.analysis.proposed_name.is_some() {
                // Test against original filename
                if re.is_match(&entry.analysis.original_name) {
                    entry.selected = true;
                }
                // Also test against proposed filename
                else if let Some(proposed) = &entry.analysis.proposed_name {
                    if re.is_match(proposed) {
                        entry.selected = true;
                    }
                }
            }
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

    fn perform_search(&mut self) {
        self.search_results.clear();
        self.current_search_index = 0;

        if self.search_query.is_empty() {
            return;
        }

        let query_lower = self.search_query.to_lowercase();

        for (index, entry) in self.file_entries.iter().enumerate() {
            // Search in original filename
            if entry.analysis.original_name.to_lowercase().contains(&query_lower) {
                self.search_results.push(index);
                continue;
            }

            // Search in proposed filename
            if let Some(proposed) = &entry.analysis.proposed_name {
                if proposed.to_lowercase().contains(&query_lower) {
                    self.search_results.push(index);
                }
            }
        }

        // Scroll to first result if any found
        if !self.search_results.is_empty() {
            self.scroll_to_index = Some(self.search_results[0]);
        }
    }

    fn find_next(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
            // Request scroll to current match
            if let Some(&index) = self.search_results.get(self.current_search_index) {
                self.scroll_to_index = Some(index);
            }
        }
    }

    fn find_previous(&mut self) {
        if !self.search_results.is_empty() {
            if self.current_search_index == 0 {
                self.current_search_index = self.search_results.len() - 1;
            } else {
                self.current_search_index -= 1;
            }
            // Request scroll to current match
            if let Some(&index) = self.search_results.get(self.current_search_index) {
                self.scroll_to_index = Some(index);
            }
        }
    }

    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("{} Search:", regular::MAGNIFYING_GLASS));

            let response = ui.text_edit_singleline(&mut self.search_query);
            if response.changed() {
                self.perform_search();
            }

            // Up arrow - previous result
            if ui.button(format!("{}", regular::CARET_UP)).clicked() {
                self.find_previous();
            }

            // Down arrow - next result
            if ui.button(format!("{}", regular::CARET_DOWN)).clicked() {
                self.find_next();
            }

            // Show match count
            if !self.search_results.is_empty() {
                ui.label(format!(
                    "{}/{} matches",
                    self.current_search_index + 1,
                    self.search_results.len()
                ));
            } else if !self.search_query.is_empty() {
                ui.colored_label(egui::Color32::GRAY, "No matches");
            }

            // Close search button
            if ui.button(format!("{}", regular::X)).clicked() {
                self.show_search = false;
                self.search_query.clear();
                self.search_results.clear();
            }
        });
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
            if ui.button(format!("{} Invert Selection", regular::SWAP)).clicked() {
                self.invert_selection();
            }
            if ui.button(format!("{} Select by Pattern", regular::FUNNEL)).clicked() {
                self.show_pattern_dialog = true;
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
        let scroll_to_index = self.scroll_to_index.take(); // Take the scroll request

        let scroll_area = egui::ScrollArea::vertical();
        scroll_area.show(ui, |ui| {
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
                    for (index, entry) in self.file_entries.iter_mut().enumerate() {
                        // Highlight if this is a search result
                        let is_current_match = !self.search_results.is_empty()
                            && self.search_results.get(self.current_search_index) == Some(&index);
                        let is_match = self.search_results.contains(&index);

                        // Checkbox
                        let has_proposed_name = entry.analysis.proposed_name.is_some();
                        let checkbox_response = ui.add_enabled(has_proposed_name, egui::Checkbox::new(&mut entry.selected, ""));

                        // Scroll to this row if requested
                        if scroll_to_index == Some(index) {
                            checkbox_response.scroll_to_me(Some(egui::Align::Center));
                        }

                        // Original filename with search highlighting using scope
                        if is_current_match || is_match {
                            let bg_color = if is_current_match {
                                egui::Color32::from_rgb(100, 150, 255)
                            } else {
                                egui::Color32::from_rgb(150, 180, 255)
                            };

                            egui::Frame::none()
                                .fill(bg_color)
                                .inner_margin(2.0)
                                .show(ui, |ui| {
                                    ui.strong(&entry.analysis.original_name);
                                });
                        } else {
                            ui.label(&entry.analysis.original_name);
                        }

                        // Arrow
                        ui.label("→");

                        // New filename with color coding
                        // Use theme-aware blue color - darker blue for light mode, lighter for dark mode
                        let blue_color = if self.dark_mode {
                            egui::Color32::LIGHT_BLUE // Light blue works well in dark mode
                        } else {
                            egui::Color32::from_rgb(0, 90, 181) // Darker blue for light mode (WCAG AA compliant)
                        };

                        match &entry.status {
                            FileStatus::Pending => {
                                if let Some(new_name) = &entry.analysis.proposed_name {
                                    ui.colored_label(blue_color, new_name.as_str());
                                } else {
                                    ui.colored_label(
                                        egui::Color32::GRAY,
                                        "(analyzing...)"
                                    );
                                }
                            }
                            FileStatus::Processing(msg) => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.colored_label(blue_color, msg.as_str());
                                });
                            }
                            FileStatus::Renamed => {
                                ui.colored_label(egui::Color32::GREEN, "✓ Renamed");
                            }
                            FileStatus::Error(e) => {
                                ui.colored_label(egui::Color32::RED, e.as_str());
                            }
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

    fn render_about_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);

            // Load Security Ronin logo if not already loaded
            if self.security_ronin_logo.is_none() {
                let logo_bytes = include_bytes!("../assets/security-ronin-logo.png");
                if let Ok(image) = image::load_from_memory(logo_bytes) {
                    let size = [image.width() as usize, image.height() as usize];
                    let image_buffer = image.to_rgba8();
                    let pixels = image_buffer.as_flat_samples();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        pixels.as_slice(),
                    );
                    self.security_ronin_logo = Some(ctx.load_texture(
                        "security_ronin_logo",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }

            // Display Security Ronin logo
            if let Some(logo) = &self.security_ronin_logo {
                ui.add(egui::Image::new(logo).max_width(200.0));
                ui.add_space(15.0);
            }

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

            // Security Ronin branding
            ui.label("A Security Ronin production");
            ui.hyperlink_to(
                "https://securityronin.com",
                "https://securityronin.com"
            );
            ui.add_space(20.0);

            if ui.button("Close").clicked() {
                self.show_about_dialog = false;
            }
            ui.add_space(10.0);
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

        // Top panel with theme toggle
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("nameback");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Theme toggle button
                    let theme_icon = if self.dark_mode {
                        regular::SUN // Show sun icon when in dark mode (click to go light)
                    } else {
                        regular::MOON // Show moon icon when in light mode (click to go dark)
                    };

                    if ui.button(format!("{}", theme_icon)).clicked() {
                        self.dark_mode = !self.dark_mode;
                        ctx.set_visuals(if self.dark_mode {
                            egui::Visuals::dark()
                        } else {
                            Self::create_light_theme()
                        });
                    }
                });
            });
        });

        // Handle Ctrl+F hotkey
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.command) {
            self.show_search = !self.show_search;
        }

        // Handle search navigation when search is active
        if self.show_search {
            // Escape to close search
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_search = false;
                self.search_query.clear();
                self.search_results.clear();
            }

            // Enter or F3 for next match
            if ctx.input(|i| {
                (i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                    || (i.key_pressed(egui::Key::F3) && !i.modifiers.shift)
            }) {
                self.find_next();
            }

            // Shift+Enter or Shift+F3 for previous match
            if ctx.input(|i| {
                (i.key_pressed(egui::Key::Enter) && i.modifiers.shift)
                    || (i.key_pressed(egui::Key::F3) && i.modifiers.shift)
            }) {
                self.find_previous();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Control buttons
            self.render_controls(ui);
            ui.add_space(10.0);

            // Search bar
            if self.show_search {
                self.render_search_bar(ui);
                ui.add_space(10.0);
            }

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

        // Pattern selection dialog
        if self.show_pattern_dialog {
            egui::Window::new("Select by Pattern")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Enter a regex pattern to select matching files:");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("Pattern:");
                        ui.text_edit_singleline(&mut self.pattern_query);
                    });

                    if let Some(error) = &self.pattern_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }

                    ui.add_space(10.0);
                    ui.label("Examples:");
                    ui.label("  .*\\.jpg$  - Match all .jpg files");
                    ui.label("  ^IMG.*  - Match files starting with IMG");
                    ui.label("  .*20(23|24).*  - Match files containing 2023 or 2024");

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Select Matching").clicked() {
                            self.select_by_pattern();
                            if self.pattern_error.is_none() {
                                self.show_pattern_dialog = false;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_pattern_dialog = false;
                            self.pattern_error = None;
                        }
                    });
                });
        }

        // About dialog
        if self.show_about_dialog {
            egui::Window::new("About nameback")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    // Add gray background in dark mode for better logo contrast
                    if self.dark_mode {
                        egui::Frame::none()
                            .fill(egui::Color32::from_gray(60))
                            .inner_margin(15.0)
                            .show(ui, |ui| {
                                self.render_about_content(ui, ctx);
                            });
                    } else {
                        self.render_about_content(ui, ctx);
                    }
                });
        }
    }
}
