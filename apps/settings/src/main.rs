//! Muelle settings: a native Rust window (egui) that edits the configuration
//! file and tells the running dock to reload it. No web view, no runtime.
//!
//! The dock starts this program with `--update` once a day. Updates are
//! released as plain executables, so that mode has nothing to install and
//! exits at once; the window offers a manual check instead.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod text;
mod ui;
mod update;

use eframe::egui;

fn main() -> eframe::Result {
    if std::env::args().any(|argument| argument == "--update") {
        return Ok(());
    }
    muelle_config::migrate_legacy_folder();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Muelle")
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([760.0, 520.0]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Muelle",
        options,
        Box::new(|creation| {
            ui::install_theme(&creation.egui_ctx);
            Ok(Box::new(app::Settings::new(&creation.egui_ctx)))
        }),
    )
}
