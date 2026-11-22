//! The settings window: state, background work and the five pages.

use std::{
    env,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{
    self, Align, ColorImage, Layout, RichText, ScrollArea, Sense, TextureHandle, TextureOptions,
    Ui, pos2, vec2,
};
use serde_json::{Map, Value};
use whirlpool_config::{
    AnimationStyle, BarAlign, BarSize, BarStyle, BarTint, ChargingStyle, Config, DockPosition,
    GenieSpeed, IslandStyle, Language, StartupAnimation,
};
use whirlpool_core::wallpaper;
use whirlpool_shell::{autostart, dock_link, folder, imagery, launcher, system};

use crate::{
    text::Texts,
    ui::{self, Backdrop, Choice, DANGER, Palette},
    update::{self, State},
};

/// Dragging a slider saves at most this often; the dock redraws on each save.
const SAVE_DELAY: Duration = Duration::from_millis(60);
/// How long the reset button stays armed after the first click.
const RESET_ARMED: Duration = Duration::from_secs(4);
/// Roster keys belong to the dock; Settings never writes them.
const DOCK_OWNED: [&str; 5] = ["version", "pinned", "groups", "recent", "taskbarPins"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Dock,
    Personalize,
    Windows,
    MenuBar,
    System,
}

pub struct Settings {
    texts: Rc<Texts>,
    config: Config,
    saved: Config,
    page: Page,
    pending: Map<String, Value>,
    pending_since: Option<Instant>,
    writer: Sender<Map<String, Value>>,
    write_results: Receiver<bool>,
    save_failed: bool,
    autostart: bool,
    autostart_rx: Receiver<bool>,
    autostart_tx: Sender<bool>,
    animations_on: bool,
    animations_checked: Instant,
    wallpaper_rx: Receiver<Option<ColorImage>>,
    wallpaper: Option<TextureHandle>,
    folder_rx: Option<Receiver<Option<String>>>,
    update_state: State,
    update_rx: Option<Receiver<Option<update::Release>>>,
    reset_armed: Option<Instant>,
    banner_dismissed: bool,
    wall_thumbs: Vec<(&'static str, TextureHandle)>,
    wall_thumbs_rx: Option<Receiver<Vec<(&'static str, ColorImage)>>>,
    wall_apply_rx: Option<Receiver<(String, bool)>>,
    wall_busy: Option<&'static str>,
    wall_failed: bool,
    logo: Option<TextureHandle>,
}

fn dock_exe() -> PathBuf {
    env::current_exe()
        .map(|path| path.with_file_name("whirlpool.exe"))
        .unwrap_or_else(|_| PathBuf::from("whirlpool.exe"))
}

fn load_config() -> Config {
    whirlpool_config::load(&whirlpool_config::default_path())
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn as_map(config: &Config) -> Map<String, Value> {
    match serde_json::to_value(config) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

/// One thread writes every patch in order and merges the ones that queued up.
fn spawn_writer() -> (Sender<Map<String, Value>>, Receiver<bool>) {
    let (send, receive) = mpsc::channel::<Map<String, Value>>();
    let (done_send, done_receive) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(mut patch) = receive.recv() {
            while let Ok(more) = receive.try_recv() {
                patch.extend(more);
            }
            let ok = whirlpool_config::update_preferences(&whirlpool_config::default_path(), patch)
                .is_ok();
            if ok {
                dock_link::notify_config_changed();
            }
            if done_send.send(ok).is_err() {
                break;
            }
        }
    });
    (send, done_receive)
}

/// Draws every wallpaper preset small, off the UI thread.
fn spawn_thumbnails(ctx: &egui::Context) -> Receiver<Vec<(&'static str, ColorImage)>> {
    let (send, receive) = mpsc::channel();
    let ctx = ctx.clone();
    thread::spawn(move || {
        let images = wallpaper::PRESETS
            .iter()
            .map(|preset| {
                let raster = wallpaper::render(preset, 240, 150);
                let size = [raster.width as usize, raster.height as usize];
                (preset.id, ColorImage::from_rgb(size, &raster.rgb))
            })
            .collect();
        let _ = send.send(images);
        ctx.request_repaint();
    });
    receive
}

fn spawn_wallpaper(ctx: &egui::Context) -> Receiver<Option<ColorImage>> {
    let (send, receive) = mpsc::channel();
    let ctx = ctx.clone();
    thread::spawn(move || {
        let image = imagery::wallpaper_preview(0).map(|image| {
            let mut rgba = image.bgra;
            for pixel in rgba.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
            ColorImage::from_rgba_premultiplied(
                [image.width as usize, image.height as usize],
                &rgba,
            )
        });
        let _ = send.send(image);
        ctx.request_repaint();
    });
    receive
}

impl Settings {
    pub fn new(ctx: &egui::Context) -> Self {
        let config = load_config();
        let (writer, write_results) = spawn_writer();
        let (autostart_tx, autostart_rx) = mpsc::channel();
        {
            let (tx, ctx) = (autostart_tx.clone(), ctx.clone());
            thread::spawn(move || {
                let dock = dock_exe();
                let enabled = autostart::enabled_for(&dock.to_string_lossy())
                    .unwrap_or_else(|_| autostart::is_enabled());
                let _ = tx.send(enabled);
                ctx.request_repaint();
            });
        }
        let mut settings = Self {
            texts: Rc::new(Texts::for_language(config.language)),
            saved: config.clone(),
            update_state: State::Off,
            config,
            page: Page::Dock,
            pending: Map::new(),
            pending_since: None,
            writer,
            write_results,
            save_failed: false,
            autostart: false,
            autostart_rx,
            autostart_tx,
            animations_on: true,
            animations_checked: Instant::now(),
            wallpaper_rx: spawn_wallpaper(ctx),
            wallpaper: None,
            folder_rx: None,
            update_rx: None,
            reset_armed: None,
            banner_dismissed: false,
            wall_thumbs: Vec::new(),
            wall_thumbs_rx: None,
            wall_apply_rx: None,
            wall_busy: None,
            wall_failed: false,
            logo: crate::logo_texture(ctx),
        };
        settings.animations_on = system::animations_enabled();
        settings.recheck_updates(ctx);
        settings
    }

    fn recheck_updates(&mut self, ctx: &egui::Context) {
        if !self.config.check_updates {
            self.update_state = State::Off;
            self.update_rx = None;
            return;
        }
        self.update_state = State::Checking;
        self.update_rx = Some(update::check());
        ctx.request_repaint_after(Duration::from_millis(250));
    }

    /// Collects what the background threads have finished.
    fn poll(&mut self, ctx: &egui::Context) {
        if let Ok(value) = self.autostart_rx.try_recv() {
            self.autostart = value;
        }
        while let Ok(ok) = self.write_results.try_recv() {
            self.save_failed = !ok;
        }
        if let Ok(image) = self.wallpaper_rx.try_recv() {
            self.wallpaper =
                image.map(|image| ctx.load_texture("wallpaper", image, TextureOptions::LINEAR));
        }
        if let Some(receive) = &self.folder_rx
            && let Ok(choice) = receive.try_recv()
        {
            self.folder_rx = None;
            if let Some(path) = choice {
                self.config.shortcuts_folder = Some(path);
            }
        }
        if let Some(receive) = &self.update_rx {
            match receive.try_recv() {
                Ok(found) => {
                    self.update_state = match found {
                        Some(release) if update::is_newer(&release.version) => {
                            State::Available(release)
                        }
                        Some(_) => State::Current,
                        None => State::Failed,
                    };
                    self.update_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(250));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.update_state = State::Failed;
                    self.update_rx = None;
                }
            }
        }
        if self.animations_checked.elapsed() > Duration::from_millis(1500) {
            self.animations_checked = Instant::now();
            self.animations_on = system::animations_enabled();
        }
        if let Some(armed) = self.reset_armed
            && armed.elapsed() > RESET_ARMED
        {
            self.reset_armed = None;
        }
        ctx.request_repaint_after(Duration::from_millis(1500));
    }

    /// Turns whatever the user changed this frame into a debounced patch.
    fn queue_changes(&mut self, ctx: &egui::Context) {
        // The magnified size can never be smaller than the icon.
        self.config.magnified_size = self.config.magnified_size.max(self.config.icon_size);
        if self.config != self.saved {
            let (new, old) = (as_map(&self.config), as_map(&self.saved));
            for (key, value) in new {
                if !DOCK_OWNED.contains(&key.as_str()) && old.get(&key) != Some(&value) {
                    self.pending.insert(key, value);
                }
            }
            if self.config.language != self.saved.language {
                self.texts = Rc::new(Texts::for_language(self.config.language));
            }
            if self.config.check_updates != self.saved.check_updates {
                self.recheck_updates(ctx);
            }
            self.saved = self.config.clone();
            self.pending_since = Some(Instant::now());
        }
        if let Some(since) = self.pending_since {
            if since.elapsed() >= SAVE_DELAY {
                let patch = std::mem::take(&mut self.pending);
                self.pending_since = None;
                if !patch.is_empty() && self.writer.send(patch).is_err() {
                    self.save_failed = true;
                }
            } else {
                ctx.request_repaint_after(SAVE_DELAY);
            }
        }
    }

    fn choose_folder(&mut self, ctx: &egui::Context) {
        if self.folder_rx.is_some() {
            return;
        }
        let (send, receive) = mpsc::channel();
        let current = self.config.shortcuts_folder.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let _ = send.send(folder::pick(0, current.as_deref()));
            ctx.request_repaint();
        });
        self.folder_rx = Some(receive);
    }

    fn set_autostart(&mut self, enabled: bool, ctx: &egui::Context) {
        self.autostart = enabled;
        let (tx, ctx) = (self.autostart_tx.clone(), ctx.clone());
        thread::spawn(move || {
            let dock = dock_exe();
            let target = enabled.then(|| dock.to_string_lossy().into_owned());
            let result = autostart::set(target.as_deref());
            // Report what Windows actually ended up with.
            let actual = if result.is_ok() {
                enabled
            } else {
                autostart::is_enabled()
            };
            let _ = tx.send(actual);
            ctx.request_repaint();
        });
    }

    fn reset(&mut self) {
        let keep = self.config.clone();
        self.config = Config {
            version: keep.version,
            pinned: keep.pinned,
            groups: keep.groups,
            recent: keep.recent,
            taskbar_pins: keep.taskbar_pins,
            ..Config::default()
        };
        self.reset_armed = None;
    }
}

impl eframe::App for Settings {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        self.poll_wallpapers(&ctx);
        let t = Rc::clone(&self.texts);
        let palette = Palette::of(ui);

        egui::Panel::left("nav")
            .exact_size(210.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(palette.sidebar)
                    .inner_margin(egui::Margin::symmetric(14, 18)),
            )
            .show(ui, |ui| self.sidebar(ui, &t, palette));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(palette.window)
                    .inner_margin(egui::Margin::symmetric(28, 20)),
            )
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.page_title(&t))
                        .font(ui::heading(26.0))
                        .color(palette.text),
                );
                ui.add_space(8.0);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(680.0);
                        self.banner(ui, &t, palette);
                        match self.page {
                            Page::Dock => self.dock_page(ui, &t),
                            Page::Personalize => self.personalize_page(ui, &t),
                            Page::Windows => self.windows_page(ui, &t),
                            Page::MenuBar => self.menu_bar_page(ui, &t),
                            Page::System => self.system_page(ui, &t, &ctx),
                        }
                        if self.save_failed {
                            ui.add_space(8.0);
                            ui.colored_label(DANGER, t.get("saveError"));
                        }
                        ui.add_space(24.0);
                    });
            });

        self.queue_changes(&ctx);
    }
}

impl Settings {
    fn page_title<'a>(&self, t: &'a Texts) -> &'a str {
        match self.page {
            Page::Dock => t.get("dock"),
            Page::Personalize => t.get("personalize"),
            Page::Windows => t.get("windows"),
            Page::MenuBar => t.get("menuBar"),
            Page::System => t.get("system"),
        }
    }

    fn sidebar(&mut self, ui: &mut Ui, t: &Texts, palette: Palette) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 11.0;
            let (rect, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
            match &self.logo {
                Some(texture) => {
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                None => ui::logo(ui.painter(), rect),
            }
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(
                    RichText::new("Whirlpool")
                        .font(ui::heading(17.0))
                        .color(palette.text),
                );
                ui.label(
                    RichText::new("dock + menu bar")
                        .size(11.5)
                        .color(palette.muted),
                );
            });
        });
        ui.add_space(22.0);
        let pages = [
            (Page::Dock, t.get("dock")),
            (Page::Personalize, t.get("personalize")),
            (Page::Windows, t.get("windows")),
            (Page::MenuBar, t.get("menuBar")),
            (Page::System, t.get("system")),
        ];
        let badge = matches!(self.update_state, State::Available(_));
        for (page, label) in pages {
            let selected = self.page == page;
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
            let round = egui::CornerRadius::same(10);
            if selected {
                ui.painter()
                    .rect_filled(rect, round, ui::ACCENT.gamma_multiply(0.20));
                // The tide mark: a short bar on the leading edge.
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(
                        pos2(rect.left() + 3.0, rect.center().y),
                        vec2(3.0, 18.0),
                    ),
                    egui::CornerRadius::same(2),
                    palette.accent_ink,
                );
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(rect, round, palette.border.gamma_multiply(0.55));
            }
            let (color, font) = if selected {
                (palette.accent_ink, ui::heading(14.0))
            } else {
                (palette.text, egui::FontId::proportional(14.0))
            };
            ui.painter().text(
                rect.left_center() + vec2(16.0, 0.0),
                egui::Align2::LEFT_CENTER,
                label,
                font,
                color,
            );
            if badge && page == Page::System {
                ui.painter()
                    .circle_filled(rect.right_center() - vec2(16.0, 0.0), 4.5, DANGER);
            }
            if response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                self.page = page;
            }
            ui.add_space(2.0);
        }
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.label(
                RichText::new(format!(
                    "v{}  ·  100% native Rust",
                    env!("CARGO_PKG_VERSION")
                ))
                .size(11.5)
                .color(palette.muted),
            );
        });
    }

    fn banner(&mut self, ui: &mut Ui, t: &Texts, palette: Palette) {
        let State::Available(release) = self.update_state.clone() else {
            return;
        };
        if self.banner_dismissed {
            return;
        }
        egui::Frame::new()
            .fill(ui::ACCENT.gamma_multiply(0.18))
            .stroke(egui::Stroke::new(1.0, ui::ACCENT))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(t.with_version("updateBannerTitle", &release.version))
                        .font(ui::heading(15.0))
                        .color(palette.text),
                );
                ui.horizontal(|ui| {
                    if ui::push(ui, t.get("updatesInstall"), Some(ui::ACCENT)).clicked() {
                        let _ = launcher::open(&release.page);
                    }
                    if ui::push(ui, t.get("updateLater"), None).clicked() {
                        self.banner_dismissed = true;
                    }
                });
            });
        ui.add_space(10.0);
    }

    fn dock_page(&mut self, ui: &mut Ui, t: &Texts) {
        let (left, bottom, right) = (
            t.get("dockPositionLeft"),
            t.get("dockPositionBottom"),
            t.get("dockPositionRight"),
        );
        let backdrop = Backdrop {
            wallpaper: self.wallpaper.as_ref(),
            icon_size: self.config.icon_size,
        };
        let config = &mut self.config;
        ui::card(ui, Some(t.get("groupAppearance")), |ui| {
            ui.add_space(10.0);
            ui.label(RichText::new(t.get("dockPosition")).size(14.0));
            let options = [
                Choice {
                    value: DockPosition::Left,
                    label: left,
                    detail: None,
                    preview: ui::preview_dock,
                },
                Choice {
                    value: DockPosition::Bottom,
                    label: bottom,
                    detail: None,
                    preview: ui::preview_dock,
                },
                Choice {
                    value: DockPosition::Right,
                    label: right,
                    detail: None,
                    preview: ui::preview_dock,
                },
            ];
            ui::visual_choices(ui, &mut config.dock_position, &backdrop, &options);
            ui::divider(ui);
            ui::row(ui, t.get("size"), None, 280.0, |ui| {
                ui.add(
                    egui::Slider::new(
                        &mut config.icon_size,
                        whirlpool_config::ICON_SIZE_RANGE.0..=whirlpool_config::ICON_SIZE_RANGE.1,
                    )
                    .suffix(" px"),
                );
            });
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("magnification"),
                Some(t.get("magnificationHint")),
                &mut config.magnification,
            );
            if config.magnification {
                ui::divider(ui);
                let low = config.icon_size;
                ui::row(ui, t.get("magnifiedSize"), None, 280.0, |ui| {
                    ui.add(
                        egui::Slider::new(
                            &mut config.magnified_size,
                            low..=whirlpool_config::MAX_MAGNIFIED_SIZE,
                        )
                        .suffix(" px"),
                    );
                });
            }
        });
        ui::card(ui, Some(t.get("groupBehavior")), |ui| {
            ui::toggle_row(
                ui,
                t.get("autoHide"),
                Some(t.get("autoHideHint")),
                &mut config.auto_hide,
            );
            ui::divider(ui);
            ui::toggle_row(ui, t.get("bounceOnOpen"), None, &mut config.bounce_on_open);
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("mascot"),
                Some(t.get("mascotHint")),
                &mut config.mascot,
            );
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("showIndicators"),
                None,
                &mut config.show_indicators,
            );
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("showRecents"),
                Some(t.get("showRecentsHint")),
                &mut config.show_recents,
            );
        });
        let mut pick = false;
        ui::card(ui, None, |ui| {
            let hint = config
                .shortcuts_folder
                .clone()
                .unwrap_or_else(|| t.get("shortcutsFolderHint").to_owned());
            ui::row(ui, t.get("shortcutsFolder"), Some(&hint), 220.0, |ui| {
                if ui::push(ui, t.get("shortcutsFolderChoose"), None).clicked() {
                    pick = true;
                }
                if config.shortcuts_folder.is_some()
                    && ui::push(ui, t.get("shortcutsFolderRemove"), None).clicked()
                {
                    config.shortcuts_folder = None;
                }
            });
        });
        if pick {
            let ctx = ui.ctx().clone();
            self.choose_folder(&ctx);
        }
    }

    fn poll_wallpapers(&mut self, ctx: &egui::Context) {
        if let Some(receive) = &self.wall_thumbs_rx
            && let Ok(images) = receive.try_recv()
        {
            self.wall_thumbs = images
                .into_iter()
                .map(|(id, image)| {
                    let texture =
                        ctx.load_texture(format!("wallpaper-{id}"), image, TextureOptions::LINEAR);
                    (id, texture)
                })
                .collect();
            self.wall_thumbs_rx = None;
        }
        if let Some(receive) = &self.wall_apply_rx
            && let Ok((id, applied)) = receive.try_recv()
        {
            self.wall_apply_rx = None;
            self.wall_busy = None;
            if applied {
                self.config.wallpaper = Some(id);
            } else {
                self.wall_failed = true;
            }
        }
    }

    fn apply_wallpaper(&mut self, id: &'static str, ctx: &egui::Context) {
        if self.wall_busy.is_some() {
            return;
        }
        self.wall_busy = Some(id);
        self.wall_failed = false;
        let (send, receive) = mpsc::channel();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let applied = whirlpool_shell::wallpaper::apply(id).is_ok();
            let _ = send.send((id.to_owned(), applied));
            ctx.request_repaint();
        });
        self.wall_apply_rx = Some(receive);
    }

    fn personalize_page(&mut self, ui: &mut Ui, t: &Texts) {
        let ctx = ui.ctx().clone();
        if self.wall_thumbs.is_empty() && self.wall_thumbs_rx.is_none() {
            self.wall_thumbs_rx = Some(spawn_thumbnails(&ctx));
        }
        let config = &mut self.config;
        ui::card(ui, Some(t.get("groupAnimations")), |ui| {
            ui::row(
                ui,
                t.get("animationStyle"),
                Some(t.get("animationStyleHint")),
                300.0,
                |ui| {
                    ui::choose(
                        ui,
                        "animation-style",
                        &mut config.animation_style,
                        &[
                            (AnimationStyle::Relaxed, t.get("styleRelaxed")),
                            (AnimationStyle::Fluid, t.get("styleFluid")),
                            (AnimationStyle::Smooth, t.get("styleSmooth")),
                            (AnimationStyle::Brisk, t.get("styleBrisk")),
                            (AnimationStyle::Snappy, t.get("styleSnappy")),
                        ],
                    );
                },
            );
            ui::divider(ui);
            ui::row(
                ui,
                t.get("startupAnimation"),
                Some(t.get("startupAnimationHint")),
                340.0,
                |ui| {
                    ui::choose(
                        ui,
                        "startup-animation",
                        &mut config.startup_animation,
                        &[
                            (StartupAnimation::Off, t.get("startupOff")),
                            (StartupAnimation::Drift, t.get("startupDrift")),
                            (StartupAnimation::Glide, t.get("startupGlide")),
                            (StartupAnimation::Rise, t.get("startupRise")),
                            (StartupAnimation::Swift, t.get("startupSwift")),
                            (StartupAnimation::Dart, t.get("startupDart")),
                        ],
                    );
                },
            );
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("chargingAnimation"),
                Some(t.get("chargingAnimationHint")),
                &mut config.charging_animation,
            );
            if config.charging_animation {
                ui::divider(ui);
                ui::row(
                    ui,
                    t.get("chargingStyle"),
                    Some(t.get("chargingStyleHint")),
                    200.0,
                    |ui| {
                        ui::choose(
                            ui,
                            "charging-style",
                            &mut config.charging_style,
                            &[
                                (ChargingStyle::Surge, t.get("chargeSurge")),
                                (ChargingStyle::Halo, t.get("chargeHalo")),
                                (ChargingStyle::Edge, t.get("chargeEdge")),
                                (ChargingStyle::Tide, t.get("chargeTide")),
                            ],
                        );
                    },
                );
            }
        });
        ui::card(ui, Some(t.get("groupMenuBarLook")), |ui| {
            ui::row(ui, t.get("menuBarOpacity"), None, 280.0, |ui| {
                ui.add(
                    egui::Slider::new(
                        &mut config.menu_bar_opacity,
                        whirlpool_config::MENU_BAR_OPACITY_RANGE.0
                            ..=whirlpool_config::MENU_BAR_OPACITY_RANGE.1,
                    )
                    .suffix(" %"),
                );
            });
            ui::divider(ui);
            ui::divider(ui);
            ui::row(
                ui,
                t.get("barTint"),
                Some(t.get("barTintHint")),
                200.0,
                |ui| {
                    ui::choose(
                        ui,
                        "bar-tint",
                        &mut config.menu_bar_tint,
                        &[
                            (BarTint::Ocean, t.get("tintOcean")),
                            (BarTint::Graphite, t.get("tintGraphite")),
                            (BarTint::Midnight, t.get("tintMidnight")),
                            (BarTint::Forest, t.get("tintForest")),
                            (BarTint::Amber, t.get("tintAmber")),
                            (BarTint::Rose, t.get("tintRose")),
                        ],
                    );
                },
            );
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("barBorder"),
                Some(t.get("barBorderHint")),
                &mut config.menu_bar_border,
            );
            ui::divider(ui);
            ui::toggle_row(ui, t.get("menuBarDate"), None, &mut config.menu_bar_date);
            ui::divider(ui);
            ui::row(ui, t.get("menuBarClock"), None, 280.0, |ui| {
                ui.selectable_value(&mut config.menu_bar_24h, None, t.get("clockAuto"));
                ui.selectable_value(&mut config.menu_bar_24h, Some(true), t.get("clock24"));
                ui.selectable_value(&mut config.menu_bar_24h, Some(false), t.get("clock12"));
            });
        });

        let mut chosen: Option<&'static str> = None;
        let mut shuffle = false;
        let busy = self.wall_busy;
        let current = config.wallpaper.clone();
        let thumbs = &self.wall_thumbs;
        let failed = self.wall_failed;
        ui::card(ui, Some(t.get("groupWallpapers")), |ui| {
            let hint = if failed {
                t.get("wallpaperFailed")
            } else if busy.is_some() {
                t.get("wallpaperApplying")
            } else {
                t.get("wallpapersHint")
            };
            ui::row(ui, t.get("wallpaperShuffle"), Some(hint), 140.0, |ui| {
                if ui::push(ui, t.get("shuffle"), None).clicked() {
                    shuffle = true;
                }
            });
            ui::divider(ui);
            ui.add_space(6.0);
            if thumbs.is_empty() {
                ui.label(t.get("wallpapersLoading"));
            }
            let columns = 3usize;
            let gap = 12.0;
            let width = ((ui.available_width() - gap * (columns as f32 - 1.0)) / columns as f32)
                .floor()
                .max(80.0);
            let height = (width * 0.625).round();
            egui::Grid::new("wallpapers")
                .spacing([gap, gap])
                .show(ui, |ui| {
                    for (index, (id, texture)) in thumbs.iter().enumerate() {
                        let selected = current.as_deref() == Some(*id);
                        ui.vertical(|ui| {
                            let (rect, response) =
                                ui.allocate_exact_size(vec2(width, height), Sense::click());
                            ui.painter().image(
                                texture.id(),
                                rect,
                                egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                egui::Color32::WHITE,
                            );
                            if selected || response.hovered() {
                                ui.painter().rect_stroke(
                                    rect,
                                    egui::CornerRadius::same(4),
                                    egui::Stroke::new(
                                        if selected { 3.0 } else { 1.5 },
                                        if selected { ui::ACCENT } else { ui::FOAM },
                                    ),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            let name = wallpaper::find(id).map_or(*id, |preset| preset.name);
                            ui.label(RichText::new(name).size(12.5));
                            if response.clicked() && busy.is_none() {
                                chosen = Some(*id);
                            }
                        });
                        if (index + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
        if shuffle {
            let entropy = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(7, |elapsed| elapsed.as_nanos() as u64);
            chosen = Some(wallpaper::shuffle(self.config.wallpaper.as_deref(), entropy).id);
        }
        if let Some(id) = chosen {
            self.apply_wallpaper(id, &ctx);
        }
    }

    fn windows_page(&mut self, ui: &mut Ui, t: &Texts) {
        let animations_on = self.animations_on;
        let backdrop = Backdrop {
            wallpaper: self.wallpaper.as_ref(),
            icon_size: self.config.icon_size,
        };
        let (normal, fast, faster) = (
            t.get("genieSpeedNormal"),
            t.get("genieSpeedFast"),
            t.get("genieSpeedFaster"),
        );
        let config = &mut self.config;
        let mut open_animation_settings = false;
        ui::card(ui, Some(t.get("groupMinimize")), |ui| {
            ui::toggle_row(
                ui,
                t.get("genie"),
                Some(t.get("genieHint")),
                &mut config.genie,
            );
            if config.genie {
                ui::divider(ui);
                ui.add_space(10.0);
                ui.label(RichText::new(t.get("genieSpeed")).size(14.0));
                ui.label(
                    RichText::new(t.get("genieSpeedHint"))
                        .size(12.0)
                        .color(Palette::of(ui).muted),
                );
                let options = [
                    Choice {
                        value: GenieSpeed::Normal,
                        label: normal,
                        detail: Some("0.5 s".into()),
                        preview: ui::preview_speed,
                    },
                    Choice {
                        value: GenieSpeed::Fast,
                        label: fast,
                        detail: Some("0.35 s".into()),
                        preview: ui::preview_speed,
                    },
                    Choice {
                        value: GenieSpeed::Faster,
                        label: faster,
                        detail: Some("0.25 s".into()),
                        preview: ui::preview_speed,
                    },
                ];
                ui::visual_choices(ui, &mut config.genie_speed, &backdrop, &options);
                if !animations_on {
                    ui::divider(ui);
                    ui::row(
                        ui,
                        t.get("genieOff"),
                        Some(t.get("genieOffHint")),
                        190.0,
                        |ui| {
                            if ui::push(ui, t.get("genieOffAction"), None).clicked() {
                                open_animation_settings = true;
                            }
                        },
                    );
                }
            }
            ui::divider(ui);
            ui::toggle_row(
                ui,
                t.get("showMinimized"),
                Some(t.get("showMinimizedHint")),
                &mut config.show_minimized,
            );
        });
        if open_animation_settings {
            let page = if system::is_windows_11() {
                "ms-settings:easeofaccess-visualeffects"
            } else {
                "ms-settings:easeofaccess-display"
            };
            let _ = launcher::open(page);
        }
    }

    fn menu_bar_page(&mut self, ui: &mut Ui, t: &Texts) {
        let backdrop = Backdrop {
            wallpaper: self.wallpaper.as_ref(),
            icon_size: self.config.icon_size,
        };
        let (notch, bubble) = (t.get("islandStyleNotch"), t.get("islandStyleBubble"));
        let config = &mut self.config;
        ui::card(ui, None, |ui| {
            ui::toggle_row(
                ui,
                t.get("menuBarShow"),
                Some(t.get("menuBarHint")),
                &mut config.menu_bar,
            );
            if config.menu_bar {
                ui::divider(ui);
                ui::toggle_row(
                    ui,
                    t.get("menuBarAutoHide"),
                    Some(t.get("menuBarAutoHideHint")),
                    &mut config.menu_bar_auto_hide,
                );
                ui::divider(ui);
                ui::toggle_row(
                    ui,
                    t.get("menuBarMusic"),
                    Some(t.get("menuBarMusicHint")),
                    &mut config.menu_bar_music,
                );
                ui::divider(ui);
                ui::toggle_row(
                    ui,
                    t.get("aiUsage"),
                    Some(t.get("aiUsageHint")),
                    &mut config.ai_usage,
                );
                if config.menu_bar_music || config.ai_usage {
                    ui::divider(ui);
                    ui.add_space(10.0);
                    ui.label(RichText::new(t.get("islandStyle")).size(14.0));
                    ui.label(
                        RichText::new(t.get("islandStyleHint"))
                            .size(12.0)
                            .color(Palette::of(ui).muted),
                    );
                    let options = [
                        Choice {
                            value: IslandStyle::Notch,
                            label: notch,
                            detail: None,
                            preview: ui::preview_island,
                        },
                        Choice {
                            value: IslandStyle::Bubble,
                            label: bubble,
                            detail: None,
                            preview: ui::preview_island,
                        },
                    ];
                    ui::visual_choices(ui, &mut config.menu_bar_island_style, &backdrop, &options);
                }
                ui::divider(ui);
                ui::toggle_row(
                    ui,
                    t.get("accessoryBattery"),
                    Some(t.get("accessoryBatteryHint")),
                    &mut config.accessory_battery,
                );
            }
        });
        if config.menu_bar {
            ui::card(ui, Some(t.get("menuBarLook")), |ui| {
                ui::row(
                    ui,
                    t.get("barStyle"),
                    Some(t.get("barStyleHint")),
                    230.0,
                    |ui| {
                        ui.selectable_value(
                            &mut config.menu_bar_style,
                            BarStyle::Docked,
                            t.get("barDocked"),
                        );
                        ui.selectable_value(
                            &mut config.menu_bar_style,
                            BarStyle::Floating,
                            t.get("barFloating"),
                        );
                    },
                );
                ui::divider(ui);
                ui::row(
                    ui,
                    t.get("barSize"),
                    Some(t.get("barSizeHint")),
                    280.0,
                    |ui| {
                        ui.selectable_value(
                            &mut config.menu_bar_size,
                            BarSize::Compact,
                            t.get("barCompact"),
                        );
                        ui.selectable_value(
                            &mut config.menu_bar_size,
                            BarSize::Regular,
                            t.get("barRegular"),
                        );
                        ui.selectable_value(
                            &mut config.menu_bar_size,
                            BarSize::Large,
                            t.get("barLarge"),
                        );
                    },
                );
                if config.menu_bar_style == BarStyle::Floating {
                    ui::divider(ui);
                    ui::row(
                        ui,
                        t.get("barInset"),
                        Some(t.get("barInsetHint")),
                        230.0,
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut config.menu_bar_inset, 0..=160)
                                    .suffix(" px"),
                            );
                        },
                    );
                    ui::divider(ui);
                    ui::row(
                        ui,
                        t.get("barGap"),
                        Some(t.get("barGapHint")),
                        230.0,
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut config.menu_bar_gap, 0..=32).suffix(" px"),
                            );
                        },
                    );
                    ui::divider(ui);
                    ui::row(ui, t.get("barRadius"), None, 230.0, |ui| {
                        ui.add(
                            egui::Slider::new(&mut config.menu_bar_radius, 0..=24).suffix(" px"),
                        );
                    });
                    ui::divider(ui);
                    ui::row(
                        ui,
                        t.get("barWidth"),
                        Some(t.get("barWidthHint")),
                        230.0,
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut config.menu_bar_width, 40..=100)
                                    .suffix(" %"),
                            );
                        },
                    );
                    if config.menu_bar_width < 100 {
                        ui::divider(ui);
                        ui::row(ui, t.get("barAlign"), None, 230.0, |ui| {
                            ui::choose(
                                ui,
                                "bar-align",
                                &mut config.menu_bar_align,
                                &[
                                    (BarAlign::Left, t.get("alignLeft")),
                                    (BarAlign::Center, t.get("alignCenter")),
                                    (BarAlign::Right, t.get("alignRight")),
                                ],
                            );
                        });
                    }
                }
            });
            ui::card(ui, Some(t.get("menuBarItems")), |ui| {
                ui::toggle_row(
                    ui,
                    t.get("menuBarBluetooth"),
                    Some(t.get("menuBarBluetoothHint")),
                    &mut config.menu_bar_bluetooth,
                );
                for (label, hint, field) in [
                    (
                        "menuBarBatteryPercent",
                        "menuBarBatteryPercentHint",
                        &mut config.menu_bar_battery_percent,
                    ),
                    (
                        "menuBarSearch",
                        "menuBarSearchHint",
                        &mut config.menu_bar_search,
                    ),
                    (
                        "menuBarTaskView",
                        "menuBarTaskViewHint",
                        &mut config.menu_bar_task_view,
                    ),
                    (
                        "menuBarWidgets",
                        "menuBarWidgetsHint",
                        &mut config.menu_bar_widgets,
                    ),
                ] {
                    ui::divider(ui);
                    let mut value = field.unwrap_or(true);
                    ui::toggle_row(ui, t.get(label), Some(t.get(hint)), &mut value);
                    if value != field.unwrap_or(true) {
                        *field = Some(value);
                    }
                }
            });
        }
    }

    fn system_page(&mut self, ui: &mut Ui, t: &Texts, ctx: &egui::Context) {
        let mut autostart = self.autostart;
        let mut recheck = false;
        let mut reset_clicked = false;
        let mut open_repo = false;
        let update_state = self.update_state.clone();
        let reset_armed = self.reset_armed.is_some();
        let config = &mut self.config;
        ui::card(ui, None, |ui| {
            ui::toggle_row(ui, t.get("autostart"), None, &mut autostart);
            ui::divider(ui);
            ui::row(ui, t.get("language"), None, 200.0, |ui| {
                let label = match config.language {
                    Language::Auto => t.get("languageAuto"),
                    Language::Es => "Español",
                    Language::En => "English",
                    Language::Fr => "Français",
                };
                egui::ComboBox::from_id_salt("language")
                    .selected_text(label)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut config.language,
                            Language::Auto,
                            t.get("languageAuto"),
                        );
                        ui.selectable_value(&mut config.language, Language::En, "English");
                        ui.selectable_value(&mut config.language, Language::Es, "Español");
                        ui.selectable_value(&mut config.language, Language::Fr, "Français");
                    });
            });
        });
        ui::card(ui, Some(t.get("updates")), |ui| {
            ui::toggle_row(
                ui,
                t.get("checkUpdates"),
                Some(t.get("checkUpdatesHint")),
                &mut config.check_updates,
            );
            ui::divider(ui);
            let hint = match &update_state {
                State::Off => t.get("updatesOff").to_owned(),
                State::Checking => t.get("updatesChecking").to_owned(),
                State::Current => t.with_version("updatesCurrent", env!("CARGO_PKG_VERSION")),
                State::Available(release) => t.with_version("updatesAvailable", &release.version),
                State::Failed => t.get("updatesError").to_owned(),
            };
            ui::row(
                ui,
                t.get("updatesNow"),
                Some(&hint),
                140.0,
                |ui| match &update_state {
                    State::Available(release) => {
                        if ui::push(ui, t.get("updatesInstall"), Some(ui::ACCENT)).clicked() {
                            let _ = launcher::open(&release.page);
                        }
                    }
                    State::Checking => {
                        ui.add_enabled(false, egui::Button::new(t.get("updatesCheck")));
                    }
                    _ => {
                        if ui::push(ui, t.get("updatesCheck"), None).clicked() {
                            recheck = true;
                        }
                    }
                },
            );
        });
        ui::card(ui, None, |ui| {
            let hint = if reset_armed {
                t.get("resetConfirmHint")
            } else {
                t.get("resetHint")
            };
            ui::row(ui, t.get("reset"), Some(hint), 150.0, |ui| {
                let (label, tone) = if reset_armed {
                    (t.get("resetConfirm"), Some(DANGER))
                } else {
                    (t.get("resetButton"), None)
                };
                if ui::push(ui, label, tone).clicked() {
                    reset_clicked = true;
                }
            });
            ui::divider(ui);
            ui::row(
                ui,
                t.get("githubTitle"),
                Some(update::REPOSITORY),
                180.0,
                |ui| {
                    if ui::push(ui, t.get("githubStar"), None).clicked() {
                        open_repo = true;
                    }
                },
            );
        });
        if autostart != self.autostart {
            self.set_autostart(autostart, ctx);
        }
        if recheck {
            self.recheck_updates(ctx);
        }
        if open_repo {
            let _ = launcher::open(update::REPOSITORY);
        }
        if reset_clicked {
            if self.reset_armed.is_some() {
                self.reset();
            } else {
                self.reset_armed = Some(Instant::now());
            }
        }
    }
}
