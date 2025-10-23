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
            // Load emoji font
            let mut fonts = egui::FontDefinitions::default();

            // Load Noto Emoji font
            fonts.font_data.insert(
                "noto_emoji".to_owned(),
                egui::FontData::from_static(include_bytes!("../fonts/NotoEmoji-Regular.ttf")),
            );

            // Add emoji font as fallback for all font families
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("noto_emoji".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("noto_emoji".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(NamebackApp::new(cc)))
        }),
    )
}
