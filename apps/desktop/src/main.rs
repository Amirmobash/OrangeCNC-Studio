mod app;
mod theme;

use app::OrangeCncApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([980.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "OrangeCNC Studio — Amir Mobasheraghdam",
        options,
        Box::new(|cc| Ok(Box::new(OrangeCncApp::new(cc)))),
    )
}
