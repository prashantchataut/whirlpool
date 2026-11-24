//! Launchpad: the full-screen app grid the dock's Start button and the hotkey open.
//!
//! It is a short-lived process (`whirlpool.exe --launchpad`) like Quick Settings, so the dock keeps no
//! memory for it. Running the command again closes it, which makes the hotkey a toggle.
//!
//! Apps come from the Start Menu shortcuts (instant) and then `Get-StartApps` (Store apps), so the
//! grid shows something at once and fills in. Icons load on a worker thread. Grid arithmetic, paging,
//! search ranking and keyboard navigation are in `whirlpool_core::launchpad`, where they are tested.

use std::{
    collections::{HashMap, HashSet},
    env,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{
    self, Align2, Color32, ColorImage, CornerRadius, FontId, Key, Modifiers, Rect, Sense, Stroke,
    StrokeKind, TextureHandle, TextureOptions, Ui, pos2, vec2,
};
use whirlpool_core::launchpad::{self as pad, Nav};
use whirlpool_shell::{icons, launcher, system};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};

use crate::text::Texts;

/// Edge of one grid cell (icon and label) and the space between cells, in logical pixels.
const CELL: f32 = 116.0;
const GAP: f32 = 14.0;
const MAX_COLUMNS: u32 = 9;
const ICON: f32 = 64.0;
const SEARCH_TOP: f32 = 84.0;
const GRID_TOP: f32 = 168.0;
const GRID_BOTTOM_MARGIN: f32 = 84.0;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// The window title the dock looks for to close a running Launchpad.
pub const TITLE: &str = "Whirlpool Launchpad";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct App {
    pub(crate) name: String,
    /// What to hand to the shell: a `.lnk` path or `shell:AppsFolder\<AppUserModelID>`.
    pub(crate) id: String,
}

pub(crate) enum Message {
    Apps(Vec<App>),
    Icon(String, ColorImage),
}

/// Start Menu entries that are not programs people launch.
fn wanted(name: &str) -> bool {
    let lower = name.to_lowercase();
    ![
        "uninstall",
        "readme",
        "release notes",
        "license",
        "documentation",
        "manual",
        "website",
    ]
    .iter()
    .any(|word| lower.contains(word))
}

fn walk(dir: &Path, depth: u32, found: &mut Vec<App>) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, depth + 1, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("lnk"))
            && let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
            && wanted(name)
        {
            found.push(App {
                name: name.to_owned(),
                id: path.to_string_lossy().into_owned(),
            });
        }
    }
}

fn sorted_unique(mut apps: Vec<App>) -> Vec<App> {
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps.dedup_by_key(|app| app.name.to_lowercase());
    apps
}

fn scan_shortcuts() -> Vec<App> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for (variable, tail) in [
        ("ProgramData", r"Microsoft\Windows\Start Menu\Programs"),
        ("APPDATA", r"Microsoft\Windows\Start Menu\Programs"),
    ] {
        if let Some(base) = env::var_os(variable) {
            roots.push(PathBuf::from(base).join(tail));
        }
    }
    let mut found = Vec::new();
    for root in roots {
        walk(&root, 0, &mut found);
    }
    sorted_unique(found)
}

/// `Name<TAB>AppID` lines from `Get-StartApps`.
fn parse_start_apps(text: &str) -> Vec<App> {
    text.lines()
        .filter_map(|line| {
            let (name, id) = line.trim().split_once('\t')?;
            (!name.trim().is_empty() && !id.trim().is_empty() && wanted(name)).then(|| App {
                name: name.trim().to_owned(),
                id: format!("shell:AppsFolder\\{}", id.trim()),
            })
        })
        .collect()
}

fn start_apps() -> Vec<App> {
    let script = "[Console]::OutputEncoding=[Text.Encoding]::UTF8; \
                  Get-StartApps | ForEach-Object { $_.Name + \"`t\" + $_.AppID }";
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|output| parse_start_apps(&String::from_utf8_lossy(&output.stdout)))
        .unwrap_or_default()
}

fn load_icons(
    apps: &[App],
    loaded: &mut HashSet<String>,
    send: &Sender<Message>,
    ctx: &egui::Context,
) {
    for (count, app) in apps.iter().enumerate() {
        if !loaded.insert(app.id.clone()) {
            continue;
        }
        if let Ok(image) = icons::load_icon(&app.id, 128) {
            let mut rgba = image.bgra;
            for pixel in rgba.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
            let image = ColorImage::from_rgba_premultiplied(
                [image.width as usize, image.height as usize],
                &rgba,
            );
            if send.send(Message::Icon(app.id.clone(), image)).is_err() {
                return;
            }
        }
        if count % 6 == 5 {
            ctx.request_repaint();
        }
    }
    ctx.request_repaint();
}

pub(crate) fn spawn_loader(ctx: egui::Context) -> Receiver<Message> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        // SAFETY: Initialising COM for this thread; an apartment already in place is ignored.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let mut apps = scan_shortcuts();
        if send.send(Message::Apps(apps.clone())).is_err() {
            return;
        }
        ctx.request_repaint();
        let mut loaded = HashSet::new();
        load_icons(&apps, &mut loaded, &send, &ctx);

        let known: HashSet<String> = apps.iter().map(|app| app.name.to_lowercase()).collect();
        let extra: Vec<App> = start_apps()
            .into_iter()
            .filter(|app| !known.contains(&app.name.to_lowercase()))
            .collect();
        if !extra.is_empty() {
            apps.extend(extra);
            let apps = sorted_unique(apps);
            if send.send(Message::Apps(apps.clone())).is_err() {
                return;
            }
            ctx.request_repaint();
            load_icons(&apps, &mut loaded, &send, &ctx);
        }
    });
    receive
}

struct Launchpad {
    apps: Vec<App>,
    icons: HashMap<String, TextureHandle>,
    messages: Receiver<Message>,
    texts: Texts,
    query: String,
    last_query: String,
    /// Indices into `apps`, best match first.
    shown: Vec<usize>,
    /// Position in `shown`.
    selected: usize,
    page: usize,
    swipe: f32,
    scroll: f32,
    started: Instant,
    focused_once: bool,
    frames: u32,
}

impl Launchpad {
    fn new(ctx: &egui::Context) -> Self {
        let language = whirlpool_config::load(&whirlpool_config::default_path())
            .ok()
            .flatten()
            .unwrap_or_default()
            .language;
        Self {
            apps: Vec::new(),
            icons: HashMap::new(),
            messages: spawn_loader(ctx.clone()),
            texts: Texts::for_language(language),
            query: String::new(),
            last_query: String::new(),
            shown: Vec::new(),
            selected: 0,
            page: 0,
            swipe: 0.0,
            scroll: 0.0,
            started: Instant::now(),
            focused_once: false,
            frames: 0,
        }
    }

    fn close(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn refilter(&mut self) {
        let names: Vec<&str> = self.apps.iter().map(|app| app.name.as_str()).collect();
        self.shown = pad::search(&names, &self.query);
        self.selected = 0;
        self.page = 0;
        self.last_query = self.query.clone();
    }

    fn launch(&self, ctx: &egui::Context, position: usize) {
        if let Some(&index) = self.shown.get(position) {
            let id = self.apps[index].id.clone();
            let _ = launcher::open(&id);
            Self::close(ctx);
        }
    }

    fn go_to_page(&mut self, page: usize, grid: pad::Grid) {
        self.page = page;
        self.selected = (page * grid.per_page()).min(self.shown.len().saturating_sub(1));
    }
}

fn to_rect(rect: pad::Rect, dx: f32) -> Rect {
    Rect::from_min_size(pos2(rect.x + dx, rect.y), vec2(rect.w, rect.h))
}

/// A label that fits the cell: cut with an ellipsis rather than overflowing into its neighbour.
fn short(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        name.to_owned()
    } else {
        let kept: String = name.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", kept.trim_end())
    }
}

/// A steady colour per app name, for the tile shown until its icon arrives (or when it has none).
fn placeholder_color(name: &str) -> Color32 {
    let hash = name.bytes().fold(2166136261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16777619)
    });
    let channel = |shift: u32| 70 + ((hash >> shift) & 0x7f) as u8 / 2;
    Color32::from_rgb(channel(0), channel(8), channel(16))
}

fn draw_slot(
    painter: &egui::Painter,
    rect: Rect,
    app: &App,
    icon: Option<&TextureHandle>,
    selected: bool,
) {
    if selected {
        painter.rect_filled(rect, CornerRadius::same(18), Color32::from_white_alpha(30));
        painter.rect_stroke(
            rect,
            CornerRadius::same(18),
            Stroke::new(1.5, Color32::from_rgb(0x9f, 0xf0, 0xe6)),
            StrokeKind::Inside,
        );
    }
    let icon_rect = Rect::from_center_size(
        pos2(rect.center().x, rect.top() + 12.0 + ICON / 2.0),
        vec2(ICON, ICON),
    );
    match icon {
        Some(texture) => {
            painter.image(
                texture.id(),
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            painter.rect_filled(
                icon_rect,
                CornerRadius::same(15),
                placeholder_color(&app.name),
            );
            let letter = app
                .name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            painter.text(
                icon_rect.center(),
                Align2::CENTER_CENTER,
                letter,
                FontId::proportional(28.0),
                Color32::from_white_alpha(230),
            );
        }
    }
    painter.text(
        pos2(rect.center().x, icon_rect.bottom() + 10.0),
        Align2::CENTER_TOP,
        short(&app.name, 15),
        FontId::proportional(13.0),
        Color32::from_white_alpha(if selected { 255 } else { 225 }),
    );
}

impl eframe::App for Launchpad {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frames += 1;
        if self.frames < 6 {
            // A window started by a hotkey is not automatically in front.
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        let mut apps_changed = false;
        while let Ok(message) = self.messages.try_recv() {
            match message {
                Message::Apps(apps) => {
                    self.apps = apps;
                    apps_changed = true;
                }
                Message::Icon(id, image) => {
                    let texture = ctx.load_texture(&id, image, TextureOptions::LINEAR);
                    self.icons.insert(id, texture);
                }
            }
        }
        if apps_changed || self.query != self.last_query {
            self.refilter();
        }

        // Close when focus goes elsewhere, once the window has had it.
        let focused = ctx.input(|input| input.viewport().focused).unwrap_or(true);
        if focused {
            self.focused_once = true;
        } else if self.focused_once && self.started.elapsed() > Duration::from_millis(600) {
            Self::close(&ctx);
        }

        let full = ui.max_rect();
        let (width, height) = (full.width(), full.height());
        let area = pad::Rect {
            x: width * 0.07,
            y: GRID_TOP,
            w: width * 0.86,
            h: (height - GRID_TOP - GRID_BOTTOM_MARGIN).max(2.0 * CELL),
        };
        let grid = pad::fit(area.w, area.h, CELL, GAP, MAX_COLUMNS);
        let count = self.shown.len();
        let pages = pad::page_count(count, grid);
        self.page = self.page.min(pages - 1);

        // Keys. The search box has the focus the whole time, so the keys that move the selection are
        // taken before it sees them.
        let mut nav = None;
        let (mut enter, mut escape) = (false, false);
        ctx.input_mut(|input| {
            for (key, direction) in [
                (Key::ArrowLeft, Nav::Left),
                (Key::ArrowRight, Nav::Right),
                (Key::ArrowUp, Nav::Up),
                (Key::ArrowDown, Nav::Down),
                (Key::Home, Nav::Home),
                (Key::End, Nav::End),
                (Key::PageUp, Nav::PageUp),
                (Key::PageDown, Nav::PageDown),
            ] {
                if input.consume_key(Modifiers::NONE, key) {
                    nav = Some(direction);
                }
            }
            enter = input.consume_key(Modifiers::NONE, Key::Enter);
            escape = input.consume_key(Modifiers::NONE, Key::Escape);
        });
        if let Some(direction) = nav {
            self.selected = pad::navigate(self.selected, count, grid, direction);
            self.page = pad::page_of(self.selected, grid);
        }
        if escape {
            if self.query.is_empty() {
                Self::close(&ctx);
            } else {
                self.query.clear();
            }
        }
        if enter {
            self.launch(&ctx, self.selected);
        }
        // The wheel turns pages.
        let wheel = ctx.input(|input| input.smooth_scroll_delta.y + input.smooth_scroll_delta.x);
        if wheel == 0.0 {
            self.scroll = 0.0;
        } else {
            self.scroll += wheel;
            if self.scroll <= -40.0 && self.page + 1 < pages {
                self.go_to_page(self.page + 1, grid);
                self.scroll = 0.0;
            } else if self.scroll >= 40.0 && self.page > 0 {
                self.go_to_page(self.page - 1, grid);
                self.scroll = 0.0;
            }
        }

        // Open: a short fade, none when Windows asks for reduced motion.
        let reduced = !system::animations_enabled();
        let appear = if reduced {
            1.0
        } else {
            ctx.animate_value_with_time(ui.make_persistent_id("appear"), 1.0, 0.16)
        };
        ui.set_opacity(appear);
        ui.painter().rect_filled(
            full,
            CornerRadius::ZERO,
            Color32::from_rgba_unmultiplied(8, 12, 20, 214),
        );

        // Background: swipe to turn a page, click on nothing to close.
        let background = ui.interact(full, ui.id().with("background"), Sense::click_and_drag());
        if background.dragged() {
            self.swipe += background.drag_delta().x;
        }
        if background.drag_stopped() {
            let to = pad::swiped_page(self.page, pages, self.swipe, width);
            if to != self.page {
                self.go_to_page(to, grid);
            }
            self.swipe = 0.0;
        }

        // Hover moves the selection, but only when the pointer actually moved.
        let pointer = ctx.input(|input| input.pointer.hover_pos());
        let moved = ctx.input(|input| input.pointer.delta().length() > 0.0);
        let hovered = pointer.and_then(|at| {
            pad::hit(grid, area, CELL, GAP, at.x - full.left(), at.y - full.top())
                .map(|slot| self.page * grid.per_page() + slot)
                .filter(|&position| position < count)
        });
        if moved && let Some(position) = hovered {
            self.selected = position;
        }
        if background.clicked() && !background.dragged() {
            match hovered {
                Some(position) => self.launch(&ctx, position),
                None => {
                    let over_search = background.interact_pointer_pos().is_some_and(|at| {
                        at.y < GRID_TOP - 20.0 && (at.x - width / 2.0).abs() < 230.0
                    });
                    let over_dots = background
                        .interact_pointer_pos()
                        .is_some_and(|at| at.y > height - GRID_BOTTOM_MARGIN + 20.0);
                    if !over_search && !over_dots {
                        Self::close(&ctx);
                    }
                }
            }
        }

        // The grid. Pages slide: the one leaving and the one arriving are both drawn.
        let position = if reduced {
            self.page as f32
        } else {
            ctx.animate_value_with_time(ui.id().with("page"), self.page as f32, 0.22)
        };
        let painter = ui.painter().clone();
        let first = position.floor().max(0.0) as usize;
        for page in [first, first + 1] {
            if page >= pages {
                continue;
            }
            let offset = (page as f32 - position) * width;
            if offset.abs() >= width {
                continue;
            }
            for (slot, item) in pad::page_range(page, count, grid).enumerate() {
                let rect = to_rect(pad::slot_rect(grid, area, CELL, GAP, slot), offset)
                    .translate(full.min.to_vec2());
                let app = &self.apps[self.shown[item]];
                draw_slot(
                    &painter,
                    rect,
                    app,
                    self.icons.get(&app.id),
                    item == self.selected && page == self.page,
                );
            }
        }
        if count == 0 {
            let message = if self.apps.is_empty() {
                self.texts.get("launchpadLoading").to_owned()
            } else {
                self.texts
                    .get("launchpadEmpty")
                    .replace("{query}", self.query.trim())
            };
            painter.text(
                pos2(width / 2.0, height / 2.0),
                Align2::CENTER_CENTER,
                message,
                FontId::proportional(18.0),
                Color32::from_white_alpha(170),
            );
        }

        // Page dots.
        if pages > 1 {
            let spacing = 22.0;
            let left = width / 2.0 - spacing * (pages as f32 - 1.0) / 2.0;
            for page in 0..pages {
                let center = pos2(
                    left + spacing * page as f32,
                    height - GRID_BOTTOM_MARGIN / 2.0,
                );
                let hit = Rect::from_center_size(center, vec2(spacing, spacing));
                let response = ui.interact(hit, ui.id().with(("dot", page)), Sense::click());
                if response.clicked() {
                    self.go_to_page(page, grid);
                }
                let active = page == self.page;
                painter.circle_filled(
                    center,
                    if active { 4.5 } else { 3.5 },
                    Color32::from_white_alpha(if active { 235 } else { 90 }),
                );
            }
        }

        // Search: a pill at the top that always has the focus, so typing just works.
        let search = Rect::from_center_size(pos2(width / 2.0, SEARCH_TOP), vec2(440.0, 46.0));
        painter.rect_filled(
            search,
            CornerRadius::same(23),
            Color32::from_white_alpha(26),
        );
        painter.rect_stroke(
            search,
            CornerRadius::same(23),
            Stroke::new(1.0, Color32::from_white_alpha(52)),
            StrokeKind::Inside,
        );
        let glass = pos2(search.left() + 25.0, search.center().y - 1.0);
        painter.circle_stroke(glass, 6.5, Stroke::new(1.8, Color32::from_white_alpha(170)));
        painter.line_segment(
            [glass + vec2(4.8, 4.8), glass + vec2(10.0, 10.0)],
            Stroke::new(1.8, Color32::from_white_alpha(170)),
        );
        let edit_rect = Rect::from_min_max(
            pos2(search.left() + 48.0, search.top() + 6.0),
            pos2(search.right() - 18.0, search.bottom() - 6.0),
        );
        if self.query.is_empty() {
            // Painted here rather than as the field's hint: the hint colour follows the theme's weak
            // text, which is nearly invisible on this dark backdrop.
            painter.text(
                pos2(edit_rect.left() + 2.0, edit_rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.texts.get("launchpadSearch"),
                FontId::proportional(18.0),
                Color32::from_white_alpha(165),
            );
        }
        let edit = egui::TextEdit::singleline(&mut self.query)
            .font(FontId::proportional(18.0))
            .frame(egui::Frame::NONE)
            .desired_width(edit_rect.width())
            .text_color(Color32::WHITE);
        let response = ui.put(edit_rect, edit);
        response.request_focus();
    }
}

pub fn run(compat: bool) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(TITLE)
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_always_on_top()
        .with_taskbar(false);
    viewport = match system::primary_monitor() {
        Some(monitor) => {
            let scale = monitor.scale.max(1.0);
            viewport
                .with_position([
                    monitor.screen.left as f32 / scale,
                    monitor.screen.top as f32 / scale,
                ])
                .with_inner_size([
                    (monitor.screen.right - monitor.screen.left) as f32 / scale,
                    (monitor.screen.bottom - monitor.screen.top) as f32 / scale,
                ])
        }
        None => viewport.with_fullscreen(true),
    };
    let options = eframe::NativeOptions {
        viewport,
        renderer: if compat {
            eframe::Renderer::Wgpu
        } else {
            eframe::Renderer::Glow
        },
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(|creation| Ok(Box::new(Launchpad::new(&creation.egui_ctx)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_menu_noise_is_left_out() {
        assert!(wanted("Visual Studio Code"));
        assert!(!wanted("Uninstall Foo"));
        assert!(!wanted("Foo Documentation"));
        assert!(!wanted("README"));
    }

    #[test]
    fn start_apps_output_becomes_launchable_ids() {
        let apps = parse_start_apps(
            "Calculator\tMicrosoft.WindowsCalculator_8wekyb3d8bbwe!App\r\n\r\nbroken line\nNotepad\t{1AC14E77}\\notepad.exe\n",
        );
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].name, "Calculator");
        assert_eq!(
            apps[0].id,
            "shell:AppsFolder\\Microsoft.WindowsCalculator_8wekyb3d8bbwe!App"
        );
    }

    #[test]
    fn duplicates_collapse_and_sort_case_insensitively() {
        let app = |name: &str| App {
            name: name.to_owned(),
            id: format!("id-{name}"),
        };
        let apps = sorted_unique(vec![app("zed"), app("Alpha"), app("alpha"), app("Mid")]);
        let names: Vec<&str> = apps.iter().map(|app| app.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "Mid", "zed"]);
    }

    #[test]
    fn long_names_are_cut_with_an_ellipsis() {
        assert_eq!(short("Notepad", 15), "Notepad");
        assert_eq!(
            short("Microsoft Visual Studio Installer", 15),
            "Microsoft Visu…"
        );
    }
}
