// Disable console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use app::NamebackApp;
use eframe::egui;

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../assets/nameback.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();

    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

/// Ensure common tool paths are in PATH for dependency detection
/// This fixes the issue where GUI launched from Finder doesn't inherit shell PATH
/// On Windows, fixes issue where GUI launched from MSI doesn't inherit updated system PATH
fn setup_path() {
    use std::env;

    let current_path = env::var("PATH").unwrap_or_default();

    // Common installation locations for dependencies
    let additional_paths = if cfg!(target_os = "macos") {
        vec![
            "/opt/homebrew/bin",      // Apple Silicon Homebrew
            "/usr/local/bin",          // Intel Homebrew
            "/opt/local/bin",          // MacPorts
        ]
    } else if cfg!(target_os = "linux") {
        vec![
            "/usr/local/bin",
            "/usr/bin",
        ]
    } else if cfg!(windows) {
        // On Windows, prioritize bundled MSI dependencies, then fall back to package managers
        let mut paths = Vec::new();

        // Bundled MSI installer dependencies (HIGHEST PRIORITY)
        // Installed to C:\Program Files\nameback\deps\{tool_name}
        if let Ok(programfiles) = env::var("PROGRAMFILES") {
            paths.push(format!("{}\\nameback\\deps\\exiftool", programfiles));
            paths.push(format!("{}\\nameback\\deps\\tesseract", programfiles));
            paths.push(format!("{}\\nameback\\deps\\ffmpeg", programfiles));
            paths.push(format!("{}\\nameback\\deps\\imagemagick", programfiles));
        }

        // Scoop user-level package manager (fallback)
        if let Ok(userprofile) = env::var("USERPROFILE") {
            let scoop_shims = format!("{}\\scoop\\shims", userprofile);
            paths.push(scoop_shims);

            // ImageMagick app directory (Scoop installs here)
            let imagemagick_app = format!("{}\\scoop\\apps\\imagemagick\\current", userprofile);
            paths.push(imagemagick_app);
        }

        // Chocolatey system-level package manager (fallback)
        if let Ok(programdata) = env::var("PROGRAMDATA") {
            let choco_bin = format!("{}\\chocolatey\\bin", programdata);
            paths.push(choco_bin);
        }

        paths
    } else {
        vec![]
    };

    // Build new PATH with additional directories prepended
    if cfg!(windows) {
        // Windows uses semicolons
        let new_path = if additional_paths.is_empty() {
            current_path
        } else {
            format!("{};{}", additional_paths.join(";"), current_path)
        };
        log::debug!("Enhanced PATH for dependency detection: {}", new_path);
        env::set_var("PATH", new_path);
    } else {
        // Unix uses colons
        let mut path_components: Vec<&str> = additional_paths.iter().map(|s| s.as_str()).collect();
        path_components.push(&current_path);
        let new_path = path_components.join(":");
        log::debug!("Enhanced PATH for dependency detection: {}", new_path);
        env::set_var("PATH", new_path);
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init(); // Initialize logging
    setup_path(); // Ensure dependencies can be found

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("nameback")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "nameback",
        native_options,
        Box::new(|cc| {
            // Load fonts with Unicode support (including Hebrew, Arabic, CJK)
            let mut fonts = egui::FontDefinitions::default();

            // Add Phosphor icons
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

            // Enable emoji/Unicode font fallback for Hebrew, Arabic, CJK, etc.
            // egui's "emoji-icon-font" includes comprehensive Unicode coverage
            fonts.families.insert(
                egui::FontFamily::Proportional,
                vec![
                    "Hack".to_owned(),           // Default proportional font
                    "phosphor".to_owned(),       // Phosphor icons
                    "emoji-icon-font".to_owned(), // Unicode fallback (Hebrew, Arabic, CJK, etc.)
                ],
            );

            fonts.families.insert(
                egui::FontFamily::Monospace,
                vec![
                    "Hack".to_owned(),           // Default monospace font
                    "phosphor".to_owned(),       // Icons
                    "emoji-icon-font".to_owned(), // Unicode fallback
                ],
            );

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(NamebackApp::new(cc)))
        }),
    )
}
