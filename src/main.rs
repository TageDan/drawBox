mod shapes;
mod ui;
use ui::MyApp;
mod renderer;

use eframe::egui::{self};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native("DrawBox", options, Box::new(|cc| Box::new(MyApp::new(cc))));

    Ok(())
}
