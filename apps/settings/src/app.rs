//! The settings window: state, background work and the five pages.

use std::{
    env,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{
    self, Align, ColorImage, Layout, Rect, RichText, ScrollArea, Sense, TextureHandle,
    TextureOptions, Ui, pos2, vec2,
};
use serde_json::{Map, Value};
use whirlpool_config::{
    AnimationStyle, BarAlign, BarPosition, BarSize, BarStyle, BarTint, ChargingStyle, Config,
    DockPosition, GenieSpeed, IslandStyle, Language, StartupAnimation,
};
use whirlpool_shell::{autostart, dock_link, folder, imagery, launcher, system};

use crate::{
    icons::{self, Icon},
    photos,
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
    About,
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
    update_rx: Option<Receiver<update::Msg>>,
    update_cancel: Arc<AtomicBool>,
    rollback: Option<String>,
    reset_armed: Option<Instant>,
    banner_dismissed: bool,
    wall_thumbs: Vec<(&'static str, TextureHandle)>,
    wall_thumbs_rx: Option<Receiver<Vec<(&'static str, ColorImage)>>>,
    wall_apply_rx: Option<Receiver<(String, bool)>>,
    wall_busy: Option<&'static str>,
    wall_failed: bool,
    logo: Option<TextureHandle>,
    search: String,
    search_open: bool,
    fitted: bool,
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

/// Decodes the photo pack small, off the UI thread.
fn spawn_thumbnails(ctx: &egui::Context) -> Receiver<Vec<(&'static str, ColorImage)>> {
    let (send, receive) = mpsc::channel();
    let ctx = ctx.clone();
    thread::spawn(move || {
        let images = photos::PHOTOS
            .iter()
            .filter_map(|photo| photos::thumbnail(photo.jpeg, 480).map(|image| (photo.id, image)))
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
            update_cancel: Arc::new(AtomicBool::new(false)),
            rollback: update::rollback_target(),
            reset_armed: None,
            banner_dismissed: false,
            wall_thumbs: Vec::new(),
            wall_thumbs_rx: None,
            wall_apply_rx: None,
            wall_busy: None,
            wall_failed: false,
            logo: crate::logo_texture(ctx),
            search: String::new(),
            search_open: false,
            fitted: false,
        };
        // `WHIRLPOOL_PAGE` opens a given page (used by the capture workflow
        // and handy when testing a single page).
        if let Ok(name) = env::var("WHIRLPOOL_PAGE") {
            settings.page = match name.as_str() {
                "menubar" => Page::MenuBar,
                "personalize" => Page::Personalize,
                "windows" => Page::Windows,
                "system" => Page::System,
                "about" => Page::About,
                _ => Page::Dock,
            };
        }
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
        self.update_rx = Some(update::spawn_check(self.config.update_prereleases));
        ctx.request_repaint_after(Duration::from_millis(250));
    }

    fn start_install(&mut self, release: update::Release) {
        self.update_cancel.store(false, Ordering::Relaxed);
        self.update_state = State::Working(update::Progress::Downloading {
            done: 0,
            total: None,
        });
        self.update_rx = Some(update::spawn_install(
            release,
            Arc::clone(&self.update_cancel),
        ));
    }

    fn start_rollback(&mut self, version: &str) {
        self.update_cancel.store(false, Ordering::Relaxed);
        self.update_state = State::Working(update::Progress::Downloading {
            done: 0,
            total: None,
        });
        self.update_rx = Some(update::spawn_rollback(
            version,
            Arc::clone(&self.update_cancel),
        ));
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
            loop {
                match receive.try_recv() {
                    Ok(update::Msg::Checked(result)) => {
                        self.update_state = match result {
                            Ok(whirlpool_update::Outcome::Available(release)) => {
                                State::Available(release)
                            }
                            Ok(whirlpool_update::Outcome::UpToDate) => State::Current,
                            Err(error) => State::Failed(error.to_string()),
                        };
                        self.update_rx = None;
                        break;
                    }
                    Ok(update::Msg::Progress(progress)) => {
                        self.update_state = State::Working(progress);
                    }
                    Ok(update::Msg::Launched) => {
                        // Setup replaces this program's file: leave it free.
                        self.update_state = State::Restarting;
                        ctx.request_repaint();
                        std::process::exit(0);
                    }
                    Ok(update::Msg::Failed(why)) => {
                        self.update_state = State::Failed(why);
                        self.update_rx = None;
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        ctx.request_repaint_after(Duration::from_millis(120));
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if !matches!(self.update_state, State::Failed(_)) {
                            self.update_state = State::Failed(
                                "The update stopped unexpectedly. Try again.".to_owned(),
                            );
                        }
                        self.update_rx = None;
                        break;
                    }
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

/// Below this window width the sidebar shrinks to icons.
const COMPACT_BELOW: f32 = 900.0;
/// With this much room beside the sidebar, the live preview gets a column.
const PREVIEW_FROM: f32 = 1060.0;
const COLUMN_MAX: f32 = 780.0;
const PREVIEW_WIDTH: f32 = 364.0;

/// What the sidebar search can find: label, extra words, and the page.
const SEARCH_INDEX: &[(&str, &str, Page)] = &[
    ("Dock icon size", "dock icons size big small", Page::Dock),
    ("Magnification", "dock zoom hover grow", Page::Dock),
    (
        "Dock position",
        "dock edge left right bottom side",
        Page::Dock,
    ),
    ("Auto-hide the dock", "dock hide reveal", Page::Dock),
    (
        "Genie minimize",
        "dock genie minimise effect speed",
        Page::Dock,
    ),
    ("Running indicators", "dock dots running apps", Page::Dock),
    (
        "Menu bar on or off",
        "menu bar enable top bar",
        Page::MenuBar,
    ),
    ("Menu bar position", "top bottom edge screen", Page::MenuBar),
    (
        "Floating menu bar",
        "floating docked inset gap radius",
        Page::MenuBar,
    ),
    (
        "Menu bar height",
        "size compact regular large",
        Page::MenuBar,
    ),
    ("Menu bar auto-hide", "hide reveal edge", Page::MenuBar),
    ("Dynamic island", "island bubble notch music", Page::MenuBar),
    (
        "Clock and date",
        "clock date 24 hour 12 hour time",
        Page::MenuBar,
    ),
    (
        "Menu bar opacity",
        "transparency blur glass tint colour",
        Page::Personalize,
    ),
    (
        "Menu bar tint",
        "tint colour color theme",
        Page::Personalize,
    ),
    (
        "Animation style",
        "motion speed spring fluid snappy",
        Page::Personalize,
    ),
    ("Startup animation", "startup boot intro", Page::Personalize),
    (
        "Charging animation",
        "charging charger plug halo surge edge bloom aurora tide",
        Page::Personalize,
    ),
    (
        "Wallpapers",
        "wallpaper background photo picture",
        Page::Personalize,
    ),
    (
        "Windows behavior",
        "windows taskbar alt tab minimized",
        Page::Windows,
    ),
    ("Language", "language english spanish french", Page::System),
    (
        "Run at startup",
        "startup autostart sign in login",
        Page::System,
    ),
    ("Updates", "update version check", Page::System),
    ("Reset settings", "reset default restore", Page::System),
    ("License", "license mit author about version", Page::About),
];

fn page_icon(page: Page) -> Icon {
    match page {
        Page::Dock => Icon::Dock,
        Page::Personalize => Icon::Palette,
        Page::Windows => Icon::Window,
        Page::MenuBar => Icon::Bar,
        Page::System => Icon::System,
        Page::About => Icon::Info,
    }
}

impl Settings {
    /// Fits the window to the monitor it opened on. winit reports the monitor
    /// in the same logical points the window is sized in, so unlike a guess
    /// made before the window exists it is right at every DPI scale.
    fn fit_window(&mut self, ctx: &egui::Context) {
        if self.fitted {
            return;
        }
        let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) else {
            return;
        };
        self.fitted = true;
        // Leave room for the taskbar or the dock and for the title bar.
        let width = (monitor.x * 0.74)
            .clamp(700.0, 1240.0)
            .min(monitor.x - 40.0);
        let height = (monitor.y * 0.84)
            .clamp(460.0, 860.0)
            .min(monitor.y - 120.0);
        let size = vec2(width.max(560.0), height.max(420.0));
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(vec2(
            560.0_f32.min(size.x),
            420.0_f32.min(size.y),
        )));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos2(
            ((monitor.x - size.x) / 2.0).max(0.0),
            ((monitor.y - size.y) / 2.0 - 16.0).max(0.0),
        )));
    }

    fn page_label<'a>(&self, t: &'a Texts, page: Page) -> &'a str {
        match page {
            Page::Dock => t.get("dock"),
            Page::Personalize => t.get("personalize"),
            Page::Windows => t.get("windows"),
            Page::MenuBar => t.get("menuBar"),
            Page::System => t.get("system"),
            Page::About => t.get("about"),
        }
    }

    fn page_subtitle<'a>(&self, t: &'a Texts) -> &'a str {
        match self.page {
            Page::Dock => t.get("subtitleDock"),
            Page::Personalize => t.get("subtitlePersonalize"),
            Page::Windows => t.get("subtitleWindows"),
            Page::MenuBar => t.get("subtitleMenuBar"),
            Page::System => t.get("subtitleSystem"),
            Page::About => t.get("subtitleAbout"),
        }
    }

    fn photo_backdrop_id(&self) -> Option<&'static str> {
        let id = self.config.wallpaper.as_deref()?;
        self.wall_thumbs
            .iter()
            .map(|(known, _)| *known)
            .find(|known| *known == id)
    }
}

impl eframe::App for Settings {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        self.poll_wallpapers(&ctx);
        self.fit_window(&ctx);
        let t = Rc::clone(&self.texts);
        let palette = Palette::of(ui);
        let total = ui.max_rect().width();
        let compact = total < COMPACT_BELOW && !self.search_open;
        let nav_width = if compact { 72.0 } else { 252.0 };
        let preview = total - nav_width >= PREVIEW_FROM && self.search.trim().is_empty();

        egui::Panel::left("nav")
            .exact_size(nav_width)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(palette.sidebar)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .inner_margin(egui::Margin::symmetric(if compact { 10 } else { 14 }, 18)),
            )
            .show(ui, |ui| self.sidebar(ui, &t, palette, compact));

        if preview {
            egui::Panel::right("preview")
                .exact_size(PREVIEW_WIDTH + 28.0)
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(palette.window)
                        .inner_margin(egui::Margin {
                            left: 0,
                            right: 28,
                            top: 28,
                            bottom: 24,
                        }),
                )
                .show(ui, |ui| self.preview_column(ui, &t, palette));
        }

        let room = ui.available_width();
        let side = if preview {
            32.0
        } else {
            ((room - COLUMN_MAX) / 2.0).clamp(24.0, 400.0)
        };
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(palette.window)
                    .inner_margin(egui::Margin {
                        left: side as i8,
                        right: if preview { 28 } else { side as i8 },
                        top: 26,
                        bottom: 0,
                    }),
            )
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(COLUMN_MAX));
                self.header(ui, &t, palette);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(COLUMN_MAX));
                        self.banner(ui, &t, palette);
                        if self.search.trim().is_empty() {
                            match self.page {
                                Page::Dock => self.dock_page(ui, &t),
                                Page::Personalize => self.personalize_page(ui, &t),
                                Page::Windows => self.windows_page(ui, &t),
                                Page::MenuBar => self.menu_bar_page(ui, &t),
                                Page::System => self.system_page(ui, &t, &ctx),
                                Page::About => self.about_page(ui, &t, palette),
                            }
                        } else {
                            self.search_results(ui, &t, palette);
                        }
                        if self.save_failed {
                            ui.add_space(8.0);
                            ui.colored_label(DANGER, t.get("saveError"));
                        }
                        ui.add_space(32.0);
                    });
            });

        self.queue_changes(&ctx);
    }
}

impl Settings {
    fn header(&mut self, ui: &mut Ui, t: &Texts, palette: Palette) {
        let searching = !self.search.trim().is_empty();
        let title = if searching {
            t.get("searchResults").to_owned()
        } else {
            self.page_label(t, self.page).to_owned()
        };
        ui.label(
            RichText::new(title)
                .font(ui::heading(28.0))
                .color(palette.text),
        );
        if !searching {
            ui.add_space(2.0);
            ui.label(
                RichText::new(self.page_subtitle(t))
                    .size(13.5)
                    .color(palette.muted),
            );
        }
        ui.add_space(16.0);
    }

    fn sidebar(&mut self, ui: &mut Ui, t: &Texts, palette: Palette, compact: bool) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 11.0;
            let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
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
            if !compact {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(
                        RichText::new("Whirlpool")
                            .font(ui::heading(17.0))
                            .color(palette.text),
                    );
                    ui.label(
                        RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                            .size(11.5)
                            .color(palette.muted),
                    );
                });
            }
        });
        ui.add_space(18.0);

        // Search.
        let search_id = ui.make_persistent_id("settings-search");
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            ui.memory_mut(|memory| memory.request_focus(search_id));
        }
        if compact {
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
            if response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(10),
                    palette.border.gamma_multiply(0.55),
                );
            }
            icons::paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), vec2(20.0, 20.0)),
                Icon::Search,
                palette.muted,
            );
            if response.on_hover_text(t.get("searchSettings")).clicked() {
                self.search_open = true;
            }
        } else {
            let field = ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .id(search_id)
                    .hint_text(t.get("searchSettings"))
                    .desired_width(f32::INFINITY)
                    .margin(egui::Margin {
                        left: 34,
                        right: 10,
                        top: 9,
                        bottom: 9,
                    }),
            );
            icons::paint(
                ui.painter(),
                Rect::from_center_size(
                    field.rect.left_center() + vec2(18.0, 0.0),
                    vec2(17.0, 17.0),
                ),
                Icon::Search,
                palette.muted,
            );
            if field.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                {
                    self.search.clear();
                    self.search_open = false;
                }
            }
        }
        ui.add_space(14.0);

        let badge = matches!(self.update_state, State::Available(_));
        let groups: [(&str, &[Page]); 3] = [
            ("groupShell", &[Page::Dock, Page::MenuBar, Page::Windows]),
            ("groupLook", &[Page::Personalize]),
            ("groupApp", &[Page::System, Page::About]),
        ];
        for (group, pages) in groups {
            if !compact {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(t.get(group).to_uppercase())
                        .font(ui::heading(10.5))
                        .extra_letter_spacing(1.1)
                        .color(palette.muted),
                );
                ui.add_space(4.0);
            } else {
                ui.add_space(10.0);
            }
            for &page in pages {
                let selected = self.page == page && self.search.trim().is_empty();
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
                let round = egui::CornerRadius::same(10);
                if selected {
                    ui.painter()
                        .rect_filled(rect, round, ui::ACCENT.gamma_multiply(0.20));
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(
                            pos2(rect.left() + 2.5, rect.center().y),
                            vec2(3.0, 18.0),
                        ),
                        egui::CornerRadius::same(2),
                        palette.accent_ink,
                    );
                } else if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, round, palette.border.gamma_multiply(0.55));
                }
                let ink = if selected {
                    palette.accent_ink
                } else {
                    palette.text
                };
                let icon_center = if compact {
                    rect.center()
                } else {
                    pos2(rect.left() + 26.0, rect.center().y)
                };
                icons::paint(
                    ui.painter(),
                    Rect::from_center_size(icon_center, vec2(20.0, 20.0)),
                    page_icon(page),
                    ink,
                );
                let label = self.page_label(t, page);
                if !compact {
                    ui.painter().text(
                        pos2(rect.left() + 46.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        label,
                        if selected {
                            ui::heading(14.0)
                        } else {
                            egui::FontId::proportional(14.0)
                        },
                        ink,
                    );
                }
                if badge && page == Page::System {
                    ui.painter().circle_filled(
                        if compact {
                            rect.right_top() + vec2(-10.0, 10.0)
                        } else {
                            rect.right_center() - vec2(14.0, 0.0)
                        },
                        4.5,
                        DANGER,
                    );
                }
                let response = if compact {
                    response.on_hover_text(label)
                } else {
                    response
                };
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.page = page;
                    {
                        self.search.clear();
                        self.search_open = false;
                    }
                }
                ui.add_space(2.0);
            }
        }
    }

    fn search_results(&mut self, ui: &mut Ui, t: &Texts, palette: Palette) {
        let query = self.search.trim().to_lowercase();
        let words: Vec<&str> = query.split_whitespace().collect();
        let hits: Vec<&(&str, &str, Page)> = SEARCH_INDEX
            .iter()
            .filter(|(title, extra, page)| {
                let haystack = format!(
                    "{} {} {}",
                    title.to_lowercase(),
                    extra,
                    self.page_label(t, *page).to_lowercase()
                );
                words.iter().all(|word| haystack.contains(word))
            })
            .collect();
        if hits.is_empty() {
            ui::card(ui, None, |ui| {
                ui.add_space(22.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("{} “{}”", t.get("searchNone"), self.search.trim()))
                            .font(ui::heading(15.0))
                            .color(palette.text),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(t.get("searchNoneHint"))
                            .size(12.5)
                            .color(palette.muted),
                    );
                    ui.add_space(12.0);
                    if ui::push(ui, t.get("searchClear"), None).clicked() {
                        {
                            self.search.clear();
                            self.search_open = false;
                        }
                    }
                });
                ui.add_space(22.0);
            });
            return;
        }
        let mut go = None;
        ui::card(ui, None, |ui| {
            for (index, (title, _, page)) in hits.iter().enumerate() {
                if index > 0 {
                    ui::divider(ui);
                }
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
                if response.hovered() {
                    ui.painter().rect_filled(
                        rect.shrink2(vec2(-10.0, 3.0)),
                        egui::CornerRadius::same(8),
                        palette.border.gamma_multiply(0.45),
                    );
                }
                icons::paint(
                    ui.painter(),
                    Rect::from_center_size(rect.left_center() + vec2(12.0, 0.0), vec2(20.0, 20.0)),
                    page_icon(*page),
                    palette.accent_ink,
                );
                ui.painter().text(
                    rect.left_center() + vec2(36.0, -8.0),
                    egui::Align2::LEFT_CENTER,
                    *title,
                    egui::FontId::proportional(14.0),
                    palette.text,
                );
                ui.painter().text(
                    rect.left_center() + vec2(36.0, 9.0),
                    egui::Align2::LEFT_CENTER,
                    self.page_label(t, *page),
                    egui::FontId::proportional(12.0),
                    palette.muted,
                );
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    go = Some(*page);
                }
            }
        });
        if let Some(page) = go {
            self.page = page;
            {
                self.search.clear();
                self.search_open = false;
            }
        }
    }

    fn preview_column(&mut self, ui: &mut Ui, t: &Texts, palette: Palette) {
        ui.label(
            RichText::new(t.get("livePreview").to_uppercase())
                .font(ui::heading(10.5))
                .extra_letter_spacing(1.1)
                .color(palette.muted),
        );
        ui.add_space(8.0);
        let width = ui.available_width().min(PREVIEW_WIDTH);
        let (rect, _) =
            ui.allocate_exact_size(vec2(width, (width * 0.625).round()), Sense::hover());
        let photo = self.photo_backdrop_id().and_then(|id| {
            self.wall_thumbs
                .iter()
                .find(|(known, _)| *known == id)
                .map(|(_, texture)| texture)
        });
        let backdrop = Backdrop {
            wallpaper: photo.or(self.wallpaper.as_ref()),
            icon_size: self.config.icon_size,
        };
        ui::preview_scene(ui.painter(), rect, &backdrop, &self.config, palette.window);
        ui.add_space(14.0);
        let config = &self.config;
        let edge = match config.menu_bar_position {
            BarPosition::Top => t.get("posTop"),
            BarPosition::Bottom => t.get("posBottom"),
        };
        let style = match config.menu_bar_style {
            BarStyle::Docked => t.get("styleDocked"),
            BarStyle::Floating => t.get("styleFloating"),
        };
        let dock_edge = match config.dock_position {
            DockPosition::Bottom => t.get("posBottom"),
            DockPosition::Left => t.get("posLeft"),
            DockPosition::Right => t.get("posRight"),
        };
        for (label, value) in [
            (
                t.get("menuBar"),
                if config.menu_bar {
                    format!("{edge} · {style}")
                } else {
                    t.get("previewOff").to_owned()
                },
            ),
            (
                t.get("dock"),
                format!("{dock_edge} · {} px", config.icon_size),
            ),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(12.5).color(palette.muted));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(value).size(12.5).color(palette.text));
                });
            });
            ui.add_space(2.0);
        }
        ui.add_space(10.0);
        ui.label(
            RichText::new(t.get("previewNote"))
                .size(11.5)
                .color(palette.muted),
        );
    }

    fn about_page(&mut self, ui: &mut Ui, t: &Texts, palette: Palette) {
        ui::card(ui, None, |ui| {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
                if let Some(texture) = &self.logo {
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                ui.vertical(|ui| {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Whirlpool")
                            .font(ui::heading(22.0))
                            .color(palette.text),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Version {}  ·  {}",
                            env!("CARGO_PKG_VERSION"),
                            t.get("aboutTagline")
                        ))
                        .size(13.0)
                        .color(palette.muted),
                    );
                });
            });
            ui.add_space(10.0);
            ui.label(
                RichText::new(t.get("aboutAuthor"))
                    .size(13.5)
                    .color(palette.text),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui::push(ui, t.get("aboutSource"), None).clicked() {
                    let _ = launcher::open("https://github.com/prashantchataut/whirlpool");
                }
                if ui::push(ui, t.get("aboutIssues"), None).clicked() {
                    let _ = launcher::open("https://github.com/prashantchataut/whirlpool/issues");
                }
                if ui::push(ui, t.get("aboutPrivacy"), None).clicked() {
                    let _ = launcher::open(
                        "https://github.com/prashantchataut/whirlpool/blob/main/docs/PRIVACY.md",
                    );
                }
            });
            ui.add_space(14.0);
        });
        ui::card(ui, Some(t.get("aboutLicense")), |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(LICENSE_TEXT.trim())
                    .size(12.5)
                    .color(palette.muted),
            );
            ui.add_space(12.0);
        });
        ui::card(ui, Some(t.get("aboutPhotos")), |ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(t.get("aboutPhotosBody"))
                    .size(12.5)
                    .color(palette.muted),
            );
            ui.add_space(10.0);
        });
    }
}

const LICENSE_TEXT: &str = include_str!("../../../LICENSE");

impl Settings {
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
                    RichText::new(
                        t.with_version("updateBannerTitle", &release.version.to_string()),
                    )
                    .font(ui::heading(15.0))
                    .color(palette.text),
                );
                ui.horizontal(|ui| {
                    if ui::push(ui, t.get("updatesInstall"), Some(ui::ACCENT)).clicked() {
                        self.start_install(release.clone());
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
            let applied = photos::find(id).is_some_and(|photo| {
                whirlpool_shell::wallpaper::apply_photo(photo.id, photo.jpeg).is_ok()
            });
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
                            (AnimationStyle::Dreamy, t.get("styleDreamy")),
                            (AnimationStyle::Relaxed, t.get("styleRelaxed")),
                            (AnimationStyle::Fluid, t.get("styleFluid")),
                            (AnimationStyle::Smooth, t.get("styleSmooth")),
                            (AnimationStyle::Brisk, t.get("styleBrisk")),
                            (AnimationStyle::Snappy, t.get("styleSnappy")),
                            (AnimationStyle::Rapid, t.get("styleRapid")),
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
                            (StartupAnimation::Float, t.get("startupFloat")),
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
                                (ChargingStyle::Bloom, t.get("chargeBloom")),
                                (ChargingStyle::Aurora, t.get("chargeAurora")),
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
                            (BarTint::Violet, t.get("tintViolet")),
                            (BarTint::Mint, t.get("tintMint")),
                            (BarTint::Sunset, t.get("tintSunset")),
                            (BarTint::Slate, t.get("tintSlate")),
                            (BarTint::Sand, t.get("tintSand")),
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
                            let response = match photos::find(id) {
                                Some(photo) => response.on_hover_text(format!(
                                    "{} · pexels.com/photo/{}",
                                    photo.author, photo.pexels_id
                                )),
                                None => response,
                            };
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
                            let (name, author) = photos::find(id)
                                .map_or((*id, ""), |photo| (photo.title, photo.author));
                            ui.label(RichText::new(name).size(12.5));
                            ui.label(RichText::new(author).size(11.0).weak());
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
            let pool: Vec<&'static str> = photos::PHOTOS
                .iter()
                .map(|photo| photo.id)
                .filter(|id| Some(*id) != self.config.wallpaper.as_deref())
                .collect();
            if !pool.is_empty() {
                chosen = Some(pool[(entropy as usize) % pool.len()]);
            }
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
                    t.get("barEdge"),
                    Some(t.get("barEdgeHint")),
                    200.0,
                    |ui| {
                        ui.selectable_value(
                            &mut config.menu_bar_position,
                            BarPosition::Top,
                            t.get("barEdgeTop"),
                        );
                        ui.selectable_value(
                            &mut config.menu_bar_position,
                            BarPosition::Bottom,
                            t.get("barEdgeBottom"),
                        );
                    },
                );
                ui::divider(ui);
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
        let mut install_release: Option<update::Release> = None;
        let mut cancel_update = false;
        let mut rollback_to: Option<String> = None;
        let rollback = self.rollback.clone();
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
            if config.check_updates {
                ui::divider(ui);
                ui::toggle_row(
                    ui,
                    t.get("updateOnStartup"),
                    Some(t.get("updateOnStartupHint")),
                    &mut config.update_on_startup,
                );
                ui::divider(ui);
                ui::row(
                    ui,
                    t.get("updateEvery"),
                    Some(t.get("updateEveryHint")),
                    160.0,
                    |ui| {
                        ui::choose(
                            ui,
                            "update-every",
                            &mut config.update_check_days,
                            &[
                                (0, t.get("updateNever")),
                                (1, t.get("updateDaily")),
                                (7, t.get("updateWeekly")),
                                (30, t.get("updateMonthly")),
                            ],
                        );
                    },
                );
                ui::divider(ui);
                ui::toggle_row(
                    ui,
                    t.get("updatePrereleases"),
                    Some(t.get("updatePrereleasesHint")),
                    &mut config.update_prereleases,
                );
            }
            ui::divider(ui);
            let hint = match &update_state {
                State::Off => t.get("updatesOff").to_owned(),
                State::Checking => t.get("updatesChecking").to_owned(),
                State::Current => t.with_version("updatesCurrent", env!("CARGO_PKG_VERSION")),
                State::Available(release) => {
                    t.with_version("updatesAvailable", &release.version.to_string())
                }
                State::Working(update::Progress::Downloading { done, total }) => match total {
                    Some(total) => format!(
                        "{} {:.1} / {:.1} MB",
                        t.get("updatesDownloading"),
                        *done as f64 / 1_048_576.0,
                        *total as f64 / 1_048_576.0
                    ),
                    None => t.get("updatesDownloading").to_owned(),
                },
                State::Working(update::Progress::Verifying) => t.get("updatesVerifying").to_owned(),
                State::Restarting => t.get("updatesRestarting").to_owned(),
                State::Failed(why) => why.clone(),
            };
            ui::row(
                ui,
                &t.with_version("updatesNow", env!("CARGO_PKG_VERSION")),
                Some(&hint),
                170.0,
                |ui| match &update_state {
                    State::Available(release) => {
                        if ui::push(ui, t.get("updatesInstall"), Some(ui::ACCENT)).clicked() {
                            install_release = Some(release.clone());
                        }
                    }
                    State::Checking | State::Restarting => {
                        ui.add_enabled(false, egui::Button::new(t.get("updatesCheck")));
                    }
                    State::Working(_) => {
                        if ui::push(ui, t.get("updatesCancel"), None).clicked() {
                            cancel_update = true;
                        }
                    }
                    _ => {
                        if ui::push(ui, t.get("updatesCheck"), None).clicked() {
                            recheck = true;
                        }
                    }
                },
            );
            if let State::Working(progress) = &update_state {
                let fraction = match progress {
                    update::Progress::Downloading {
                        done,
                        total: Some(total),
                    } => *done as f32 / (*total).max(1) as f32,
                    update::Progress::Downloading { .. } => 0.0,
                    update::Progress::Verifying => 1.0,
                };
                ui.add(egui::ProgressBar::new(fraction).desired_height(6.0));
            }
            if let State::Available(release) = &update_state {
                ui::divider(ui);
                let notes = release.notes.trim();
                egui::CollapsingHeader::new(t.get("updatesNotes"))
                    .id_salt("update-notes")
                    .show(ui, |ui| {
                        ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            let shown: String = notes.chars().take(4000).collect();
                            ui.label(
                                RichText::new(if shown.is_empty() {
                                    t.get("updatesNoNotes")
                                } else {
                                    shown.as_str()
                                })
                                .size(12.5),
                            );
                        });
                    });
            }
            if let Some(previous) = &rollback {
                ui::divider(ui);
                let busy = matches!(update_state, State::Working(_) | State::Restarting);
                ui::row(
                    ui,
                    &t.with_version("updatesRollback", previous),
                    Some(t.get("updatesRollbackHint")),
                    170.0,
                    |ui| {
                        if ui
                            .add_enabled(
                                !busy,
                                egui::Button::new(t.get("updatesRollbackButton"))
                                    .min_size(vec2(0.0, 30.0)),
                            )
                            .clicked()
                        {
                            rollback_to = Some(previous.clone());
                        }
                    },
                );
            }
            ui::divider(ui);
            ui.label(
                RichText::new(t.get("updatesPrivacy"))
                    .size(12.0)
                    .color(Palette::of(ui).muted),
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
        if let Some(release) = install_release {
            self.start_install(release);
        }
        if let Some(version) = rollback_to {
            self.start_rollback(&version);
        }
        if cancel_update {
            self.update_cancel.store(true, Ordering::Relaxed);
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
