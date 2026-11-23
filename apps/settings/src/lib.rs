//! Whirlpool settings: a native Rust window (egui) that edits the configuration
//! file and tells the running dock to reload it. No web view, no runtime.
//!
//! It lives inside `whirlpool.exe`: the dock starts itself again with
//! `--settings`, so there is one program to install and one icon to trust.

mod app;
mod icons;
mod photos;
mod quick;
mod text;
mod ui;
pub mod update;

use eframe::egui;

/// Opens the settings window (OpenGL) and returns when it is closed.
pub fn run() -> eframe::Result {
    open(eframe::Renderer::Glow)
}

/// Opens the quick settings panel; see [`quick`].
pub fn run_quick(right: i32, anchor_y: i32, bottom: bool, compat: bool) -> eframe::Result {
    quick::run(right, anchor_y, bottom, compat)
}

/// The same window drawn with DirectX instead of OpenGL, for machines whose
/// graphics driver offers no usable OpenGL (virtual machines, remote desktop,
/// some older Intel drivers).
pub fn run_compat() -> eframe::Result {
    open(eframe::Renderer::Wgpu)
}

fn open(renderer: eframe::Renderer) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let (width, height) = window_size();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Whirlpool")
            .with_icon(std::sync::Arc::new(window_icon()))
            .with_inner_size([width, height])
            .with_min_inner_size([width.min(560.0), height.min(420.0)]),
        centered: true,
        renderer,
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
        .clamp(560.0, 1040.0)
        .min(work_width - 32.0);
    let height = (work_height * 0.8)
        .clamp(420.0, 780.0)
        .min(work_height - 32.0);
    (width.max(480.0), height.max(360.0))
}

const LOGO: &[u8] = include_bytes!("../../../assets/brand/whirlpool-256.png");

fn decode_logo() -> Option<(Vec<u8>, u32, u32)> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(LOGO))
        .read_info()
        .ok()?;
    let mut rgba = vec![0; reader.output_buffer_size()?];
    let frame = reader.next_frame(&mut rgba).ok()?;
    if frame.color_type != png::ColorType::Rgba || frame.bit_depth != png::BitDepth::Eight {
        return None;
    }
    rgba.truncate(frame.buffer_size());
    Some((rgba, frame.width, frame.height))
}

/// The window and taskbar icon: the real mark, not a drawing of it.
fn window_icon() -> egui::IconData {
    decode_logo().map_or_else(egui::IconData::default, |(rgba, width, height)| {
        egui::IconData {
            rgba,
            width,
            height,
        }
    })
}

/// The mark as a texture for the sidebar header.
pub(crate) fn logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let (rgba, width, height) = decode_logo()?;
    let image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba);
    Some(ctx.load_texture("whirlpool-logo", image, egui::TextureOptions::LINEAR))
}
