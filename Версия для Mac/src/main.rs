#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod glass;
mod inspect;
mod model;
mod motion;
mod ui;

use eframe::egui;
use ui::PlannerApp;

fn main() -> eframe::Result {
    let min_size = if inspect::Inspector::is_env_enabled() {
        [276.0, 280.0]
    } else {
        [640.0, 560.0]
    };
    eframe::run_native(
        "Мой день",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Мой день")
                .with_inner_size([1100.0, 780.0])
                .with_min_inner_size(min_size)
                .with_transparent(true)
                .with_fullsize_content_view(true),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(PlannerApp::new(cc)))),
    )
}
