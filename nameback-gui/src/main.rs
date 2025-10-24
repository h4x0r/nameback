// Disable console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use app::NamebackApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Initialize logging

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("nameback - File Renaming Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "nameback",
        native_options,
        Box::new(|cc| {
            // Load Phosphor icon font
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(NamebackApp::new(cc)))
        }),
    )
}
