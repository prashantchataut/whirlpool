//! Whirlpool settings: a native Rust window (egui) that edits the configuration
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
    whirlpool_config::migrate_legacy_folder();
    let (width, height) = window_size();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Whirlpool")
            .with_inner_size([width, height])
            .with_min_inner_size([width.min(760.0), height.min(520.0)]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Whirlpool",
        options,
        Box::new(|creation| {
            ui::install_theme(&creation.egui_ctx);
            Ok(Box::new(app::Settings::new(&creation.egui_ctx)))
        }),
    )
}

/// A window that fits the screen it opens on: about two thirds of the work
/// area on large displays, nearly all of it on small or heavily scaled ones.
fn window_size() -> (f32, f32) {
    let Some(monitor) = whirlpool_shell::system::primary_monitor() else {
        return (900.0, 640.0);
    };
    let scale = monitor.scale.max(1.0);
    let work_width = (monitor.work_right - monitor.work_left) as f32 / scale;
    let work_height = (monitor.work_bottom - monitor.work_top) as f32 / scale;
    let width = (work_width * 0.62)
        .clamp(760.0, 1040.0)
        .min(work_width - 32.0);
    let height = (work_height * 0.8)
        .clamp(520.0, 780.0)
        .min(work_height - 32.0);
    (width.max(480.0), height.max(360.0))
}
