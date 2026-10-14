mod app;
mod editor_view;
mod palette;
mod sidebar;
mod status_bar;
mod tabs;
mod theme;

use app::TxtEditorApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("txt — High-Performance Text Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "txt",
        native_options,
        Box::new(|_cc| Ok(Box::new(TxtEditorApp::new()))),
    )
}
