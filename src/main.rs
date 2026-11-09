#![allow(dead_code)]

mod app;
mod core;
mod logging;
mod network;
mod persistence;
mod platform;
mod ui;

fn main() -> eframe::Result {
    logging::init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "start aplikace");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 700.0])
            .with_min_inner_size([700.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AlfoPlus Configurator",
        options,
        Box::new(|cc| Ok(Box::new(app::Application::new(cc)))),
    )
}
