//! Whirlpool settings: a native Rust window (egui) that edits the configuration
//! file and tells the running dock to reload it. No web view, no runtime.
//!
//! It lives inside `whirlpool.exe`: the dock starts itself again with
//! `--settings`, so there is one program to install and one icon to trust.

mod app;
mod chrome;
mod design;
mod extras;
mod glyphs;
mod hub;
mod icons;
mod launchpad;
mod notify_center;
mod overview;
mod palette;
pub mod photos;
mod preview;
mod quick;
pub mod rotate;
mod studio;
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
    with_fallback(compat, |compat| quick::run(right, anchor_y, bottom, compat))
}

/// The Launchpad: the full-screen app grid. `compat` draws it with DirectX.
pub fn run_launchpad(compat: bool) -> eframe::Result {
    with_fallback(compat, launchpad::run)
}

/// The command palette: apps, files, actions and clipboard history in one box.
pub fn run_palette(compat: bool) -> eframe::Result {
    with_fallback(compat, palette::run)
}

/// The wallpaper studio: 4K pictures from 4kwallpapers.com, applied in one click.
pub fn run_studio(compat: bool) -> eframe::Result {
    with_fallback(compat, studio::run)
}

/// `whirlpool --studio-apply ID`: downloads one studio wallpaper and applies it. Returns an exit code.
#[must_use]
pub fn studio_apply(id: &str) -> i32 {
    studio::apply_cli(id)
}

/// The window overview: every open window as a live card, full screen.
pub fn run_overview(compat: bool) -> eframe::Result {
    with_fallback(compat, overview::run)
}

/// The title of the overview window, so the dock can close a running one.
pub const OVERVIEW_TITLE: &str = overview::TITLE;

/// The title of the palette window, so the dock can close a running one.
pub const PALETTE_TITLE: &str = palette::TITLE;

/// The title of the Launchpad window, so the dock can close a running one.
pub const LAUNCHPAD_TITLE: &str = launchpad::TITLE;

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

/// The notification center: history of Whirlpool's own notifications.
pub fn run_notifications(compat: bool) -> eframe::Result {
    with_fallback(compat, notify_center::run)
}

/// The clock panel: clock and time zones, focus timer, file shelf. `tab` is `clock`, `focus` or `shelf`.
pub fn run_hub(tab: &str, compat: bool) -> eframe::Result {
    let tab = tab.to_owned();
    with_fallback(compat, move |compat| hub::run(&tab, compat))
}

/// The title of the clock panel window, so the dock can close a running one.
pub const HUB_TITLE: &str = hub::TITLE;

/// The title of the notification center window, so the dock can close a running one.
pub const NOTIFICATIONS_TITLE: &str = notify_center::TITLE;

/// Adds one entry to the notification history. `false` when history is off or it was a repeat.
pub fn push_notification(source: &str, title: &str, body: &str) -> bool {
    notify_center::push(source, title, body)
}

/// Runs a panel with OpenGL and, if that cannot start (no OpenGL 2.0: a VM, a remote session, an old
/// driver), once more with DirectX. Without this the panel's process simply exits and nothing shows.
fn with_fallback(compat: bool, run: impl Fn(bool) -> eframe::Result) -> eframe::Result {
    match run(compat) {
        Err(error) if !compat => {
            eprintln!("panel could not start with OpenGL ({error}); retrying with DirectX");
            run(true)
        }
        result => result,
    }
}
