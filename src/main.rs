mod app;
mod file_ops;
mod ui;
mod types;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("🗂 Rust File Assistant")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust File Assistant",
        native_options,
        Box::new(|cc| Box::new(app::FileAssistantApp::new(cc))),
    )
}
