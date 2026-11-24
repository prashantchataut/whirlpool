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

use crate::{
    chrome,
    design::{self, Backdrop, Direction},
    text::Texts,
    ui,
};

/// The design used unless `WHIRLPOOL_START_DESIGN` (a, b or c) asks for another.
const DEFAULT_DIRECTION: Direction = Direction::A;
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

/// Geometry of one design: cell size, icon size and where the search field and grid sit.
#[derive(Clone, Copy)]
struct Metrics {
    cell: f32,
    gap: f32,
    max_columns: u32,
    icon: f32,
    search_w: f32,
    search_h: f32,
}

fn metrics(direction: Direction) -> Metrics {
    match direction {
        Direction::A => Metrics {
            cell: 136.0,
            gap: 10.0,
            max_columns: 8,
            icon: 80.0,
            search_w: 580.0,
            search_h: 58.0,
        },
        Direction::B => Metrics {
            cell: 132.0,
            gap: 16.0,
            max_columns: 8,
            icon: 62.0,
            search_w: 520.0,
            search_h: 54.0,
        },
        Direction::C => Metrics {
            cell: 120.0,
            gap: 8.0,
            max_columns: 8,
            icon: 64.0,
            search_w: 400.0,
            search_h: 46.0,
        },
    }
}

/// Where everything sits for one frame, in window points.
struct Layout {
    search: Rect,
    area: pad::Rect,
    dots_y: f32,
    sheet: Option<Rect>,
}

fn layout(direction: Direction, m: &Metrics, full: Rect) -> Layout {
    let (width, height) = (full.width(), full.height());
    match direction {
        Direction::A | Direction::B => {
            let search_y = if direction == Direction::A {
                96.0
            } else {
                92.0
            };
            let top = search_y + m.search_h / 2.0 + 44.0;
            Layout {
                search: Rect::from_center_size(
                    pos2(width / 2.0, search_y),
                    vec2(m.search_w, m.search_h),
                ),
                area: pad::Rect {
                    x: width * 0.07,
                    y: top,
                    w: width * 0.86,
                    h: (height - top - 84.0).max(2.0 * m.cell),
                },
                dots_y: height - 42.0,
                sheet: None,
            }
        }
        Direction::C => {
            let sheet = Rect::from_center_size(
                pos2(width / 2.0, height / 2.0),
                vec2(
                    (width * 0.86).min(1240.0),
                    (height - 96.0).max(3.0 * m.cell),
                ),
            );
            Layout {
                search: Rect::from_min_size(
                    pos2(sheet.left() + 36.0, sheet.top() + 28.0),
                    vec2(m.search_w, m.search_h),
                ),
                area: pad::Rect {
                    x: sheet.left() - full.left() + 36.0,
                    y: sheet.top() - full.top() + 112.0,
                    w: sheet.width() - 72.0,
                    h: (sheet.height() - 112.0 - 60.0).max(2.0 * m.cell),
                },
                dots_y: sheet.bottom() - 30.0,
                sheet: Some(sheet),
            }
        }
    }
}

struct Launchpad {
    apps: Vec<App>,
    icons: HashMap<String, TextureHandle>,
    messages: Receiver<Message>,
    texts: Texts,
    direction: Direction,
    backdrop: Backdrop,
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
    /// When the tiles began arriving: the first moment there was something to show.
    revealed: Option<Instant>,
    focused_once: bool,
    frames: u32,
}

impl Launchpad {
    fn new(ctx: &egui::Context, backdrop: Backdrop) -> Self {
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
            direction: Direction::from_env("WHIRLPOOL_START_DESIGN", DEFAULT_DIRECTION),
            backdrop,
            query: String::new(),
            last_query: String::new(),
            shown: Vec::new(),
            selected: 0,
            page: 0,
            swipe: 0.0,
            scroll: 0.0,
            started: Instant::now(),
            revealed: None,
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

/// What one tile needs to be drawn this frame.
struct Slot<'a> {
    app: &'a App,
    icon: Option<&'a TextureHandle>,
    /// 0..=1: how selected it is (eased, so the highlight glides rather than jumps).
    lift: f32,
    /// 0..=1 entrance progress (1 when settled or when motion is off).
    enter: f32,
}

fn paint_icon(painter: &egui::Painter, rect: Rect, slot: &Slot<'_>) {
    match slot.icon {
        Some(texture) => {
            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            let radius = (rect.width() * 0.24) as u8;
            painter.rect_filled(
                rect,
                CornerRadius::same(radius),
                placeholder_color(&slot.app.name),
            );
            let letter = slot
                .app
                .name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                letter,
                ui::heading(rect.width() * 0.42),
                Color32::from_white_alpha(235),
            );
        }
    }
}

fn draw_slot(
    painter: &egui::Painter,
    base_opacity: f32,
    rect: Rect,
    slot: &Slot<'_>,
    m: &Metrics,
    direction: Direction,
) {
    // Entrance: rises 20 points while fading in, and the icon settles from 90% with a small
    // overshoot. Position and fade share one expo curve; the scale gets the spring.
    let rise = design::out_expo(slot.enter);
    let spring = design::out_back(slot.enter);
    let mut painter = painter.clone();
    painter.set_opacity(base_opacity * rise);
    let rect = rect.translate(vec2(0.0, (1.0 - rise) * 20.0));
    let scale = (0.9 + 0.1 * spring) * (1.0 + 0.06 * slot.lift);
    let label_color = design::mix(Color32::from_white_alpha(222), Color32::WHITE, slot.lift);
    match direction {
        Direction::A => {
            if slot.lift > 0.0 {
                let bubble = CornerRadius::same(28);
                painter.rect_filled(
                    rect,
                    bubble,
                    Color32::from_white_alpha((26.0 * slot.lift) as u8),
                );
                painter.rect_stroke(
                    rect,
                    bubble,
                    Stroke::new(1.0, Color32::from_white_alpha((64.0 * slot.lift) as u8)),
                    StrokeKind::Inside,
                );
            }
            let side = (m.icon + 14.0) * scale;
            let plate = Rect::from_center_size(
                pos2(
                    rect.center().x,
                    rect.top() + 14.0 + (m.icon + 14.0) / 2.0 - 3.0 * slot.lift,
                ),
                vec2(side, side),
            );
            design::drop_shadow(&painter, plate, 24, 20, 8, 70);
            painter.rect_filled(plate, CornerRadius::same(24), Color32::from_white_alpha(34));
            painter.rect_stroke(
                plate,
                CornerRadius::same(24),
                Stroke::new(1.0, Color32::from_white_alpha(48)),
                StrokeKind::Inside,
            );
            paint_icon(&painter, plate.shrink(side * 0.1), slot);
            painter.text(
                pos2(rect.center().x, plate.bottom() + 15.0),
                Align2::CENTER_TOP,
                short(&slot.app.name, 16),
                ui::heading(13.5),
                Color32::from_black_alpha(110),
            );
            painter.text(
                pos2(rect.center().x, plate.bottom() + 14.0),
                Align2::CENTER_TOP,
                short(&slot.app.name, 16),
                ui::heading(13.5),
                label_color,
            );
        }
        Direction::B => {
            let card = CornerRadius::same(26);
            design::drop_shadow(
                &painter,
                rect,
                26,
                (22.0 + 10.0 * slot.lift) as u8,
                10,
                (60.0 + 40.0 * slot.lift) as u8,
            );
            painter.rect_filled(
                rect,
                card,
                Color32::from_white_alpha((20.0 + 16.0 * slot.lift) as u8),
            );
            painter.rect_stroke(
                rect,
                card,
                Stroke::new(
                    1.0 + 0.5 * slot.lift,
                    design::mix(Color32::from_white_alpha(34), design::TEAL, slot.lift),
                ),
                StrokeKind::Inside,
            );
            let side = m.icon * scale;
            let icon_rect = Rect::from_center_size(
                pos2(rect.center().x, rect.top() + 18.0 + m.icon / 2.0),
                vec2(side, side),
            );
            paint_icon(&painter, icon_rect, slot);
            painter.text(
                pos2(rect.center().x, rect.bottom() - 18.0),
                Align2::CENTER_CENTER,
                short(&slot.app.name, 15),
                ui::heading(12.5),
                label_color,
            );
        }
        Direction::C => {
            if slot.lift > 0.0 {
                let bubble = CornerRadius::same(18);
                painter.rect_filled(rect, bubble, design::TEAL.gamma_multiply(0.22 * slot.lift));
                painter.rect_stroke(
                    rect,
                    bubble,
                    Stroke::new(1.5, design::TEAL.gamma_multiply(0.8 * slot.lift)),
                    StrokeKind::Inside,
                );
            }
            let side = m.icon * scale;
            let icon_rect = Rect::from_center_size(
                pos2(rect.center().x, rect.top() + 14.0 + m.icon / 2.0),
                vec2(side, side),
            );
            design::drop_shadow(&painter, icon_rect.shrink(8.0), 16, 14, 6, 56);
            paint_icon(&painter, icon_rect, slot);
            painter.text(
                pos2(rect.center().x, icon_rect.bottom() + 12.0),
                Align2::CENTER_TOP,
                short(&slot.app.name, 15),
                ui::heading(12.5),
                label_color,
            );
        }
    }
}

/// The search field's frame, magnifier and the key hint.
fn paint_search(painter: &egui::Painter, rect: Rect, direction: Direction, empty: bool) {
    let radius = match direction {
        Direction::A => (rect.height() / 2.0) as u8,
        Direction::B => 18,
        Direction::C => 14,
    };
    let corner = CornerRadius::same(radius);
    if direction != Direction::C {
        design::drop_shadow(painter, rect, radius, 26, 10, 80);
    }
    let (fill, line) = match direction {
        Direction::A => (Color32::from_white_alpha(40), Color32::from_white_alpha(70)),
        Direction::B => (Color32::from_black_alpha(96), Color32::from_white_alpha(36)),
        Direction::C => (Color32::from_white_alpha(18), Color32::from_white_alpha(34)),
    };
    painter.rect_filled(rect, corner, fill);
    painter.rect_stroke(rect, corner, Stroke::new(1.0, line), StrokeKind::Inside);
    let stroke = Stroke::new(1.9, Color32::from_white_alpha(190));
    let glass = pos2(rect.left() + 26.0, rect.center().y - 1.5);
    painter.circle_stroke(glass, 6.8, stroke);
    painter.line_segment([glass + vec2(5.0, 5.0), glass + vec2(10.5, 10.5)], stroke);
    if empty {
        let chip =
            Rect::from_center_size(pos2(rect.right() - 36.0, rect.center().y), vec2(42.0, 24.0));
        painter.rect_stroke(
            chip,
            CornerRadius::same(7),
            Stroke::new(1.0, Color32::from_white_alpha(80)),
            StrokeKind::Inside,
        );
        painter.text(
            chip.center(),
            Align2::CENTER_CENTER,
            "Esc",
            ui::heading(11.0),
            Color32::from_white_alpha(170),
        );
    }
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
                    if self.revealed.is_none() && !self.apps.is_empty() {
                        self.revealed = Some(Instant::now());
                    }
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
        let m = metrics(self.direction);
        let lay = layout(self.direction, &m, full);
        let mut area = lay.area;
        let grid = pad::fit(area.w, area.h, m.cell, m.gap, m.max_columns);
        // Centre the rows in the space they have (a little above the middle) so a short page does
        // not leave a dead band above the dots.
        let content_h = grid.rows as f32 * (m.cell + m.gap) - m.gap;
        if content_h < area.h {
            area.y += (area.h - content_h) * 0.4;
            area.h = content_h;
        }
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

        // Open: the blur fades in over 160 ms (nothing when Windows asks for reduced motion) and
        // the tiles rise in a diagonal stagger after it.
        let reduced = !system::animations_enabled();
        let appear = if reduced {
            1.0
        } else {
            ctx.animate_value_with_time(ui.make_persistent_id("appear"), 1.0, 0.16)
        };
        ui.set_opacity(appear);
        let painter = ui.painter().clone();
        self.backdrop.paint(
            &painter,
            full,
            Color32::from_rgba_unmultiplied(6, 16, 27, 150),
            Color32::from_rgba_unmultiplied(8, 16, 26, 238),
        );
        // Depth: the water darkens toward the bottom edge.
        design::vertical_gradient(
            &painter,
            full,
            Color32::from_rgba_unmultiplied(4, 24, 36, 0),
            Color32::from_rgba_unmultiplied(3, 12, 22, 120),
        );
        if let Some(sheet) = lay.sheet {
            design::drop_shadow(&painter, sheet, 34, 60, 22, 150);
            painter.rect_filled(
                sheet,
                CornerRadius::same(34),
                Color32::from_rgba_unmultiplied(10, 26, 40, 176),
            );
            painter.rect_stroke(
                sheet,
                CornerRadius::same(34),
                Stroke::new(1.0, Color32::from_white_alpha(40)),
                StrokeKind::Inside,
            );
            painter.text(
                pos2(sheet.right() - 40.0, lay.search.center().y),
                Align2::RIGHT_CENTER,
                self.texts
                    .get("launchpadCount")
                    .replace("{count}", &count.to_string()),
                ui::heading(13.0),
                Color32::from_white_alpha(150),
            );
        }

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
            pad::hit(
                grid,
                area,
                m.cell,
                m.gap,
                at.x - full.left(),
                at.y - full.top(),
            )
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
                    let at = background.interact_pointer_pos();
                    let over_search = at.is_some_and(|at| lay.search.expand(12.0).contains(at));
                    let over_dots = at.is_some_and(|at| (at.y - lay.dots_y).abs() < 16.0);
                    let over_sheet = at.is_some_and(|at| lay.sheet.is_some_and(|s| s.contains(at)));
                    if !over_search && !over_dots && !over_sheet {
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
        let since_reveal = self
            .revealed
            .map_or(0.0, |at| at.elapsed().as_secs_f32())
            .max(0.0);
        let entering =
            !reduced && self.revealed.is_some() && design::entrance_running(since_reveal);
        if entering {
            ctx.request_repaint();
        }
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
                let rect = to_rect(pad::slot_rect(grid, area, m.cell, m.gap, slot), offset)
                    .translate(full.min.to_vec2());
                let app = &self.apps[self.shown[item]];
                let selected = item == self.selected && page == self.page;
                let lift = if reduced {
                    f32::from(selected)
                } else {
                    ctx.animate_bool_with_time(ui.id().with(("lift", item)), selected, 0.14)
                };
                let order = slot as u32 / grid.columns + slot as u32 % grid.columns;
                let enter = if reduced || page != self.page {
                    1.0
                } else if self.revealed.is_none() {
                    0.0
                } else {
                    design::stagger(since_reveal, order as usize, 0.034, 0.34)
                };
                draw_slot(
                    &painter,
                    appear,
                    rect,
                    &Slot {
                        app,
                        icon: self.icons.get(&app.id),
                        lift,
                        enter,
                    },
                    &m,
                    self.direction,
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
                ui::heading(18.0),
                Color32::from_white_alpha(190),
            );
        }

        // Page dots: the current page is a longer pill, so it reads without relying on colour.
        if pages > 1 {
            let spacing = 20.0;
            let left = width / 2.0 - spacing * (pages as f32 - 1.0) / 2.0;
            for page in 0..pages {
                let center = pos2(left + spacing * page as f32, lay.dots_y);
                let hit = Rect::from_center_size(center, vec2(spacing, spacing));
                let response = ui.interact(hit, ui.id().with(("dot", page)), Sense::click());
                if response.clicked() {
                    self.go_to_page(page, grid);
                }
                let active = page == self.page;
                let dot = Rect::from_center_size(
                    center,
                    if active {
                        vec2(16.0, 7.0)
                    } else {
                        vec2(7.0, 7.0)
                    },
                );
                painter.rect_filled(
                    dot,
                    CornerRadius::same(4),
                    if active {
                        Color32::from_white_alpha(240)
                    } else {
                        Color32::from_white_alpha(96)
                    },
                );
            }
        }

        // Search: a field that always has the focus, so typing just works.
        paint_search(&painter, lay.search, self.direction, self.query.is_empty());
        let edit_rect = Rect::from_min_max(
            pos2(lay.search.left() + 50.0, lay.search.top() + 6.0),
            pos2(lay.search.right() - 74.0, lay.search.bottom() - 6.0),
        );
        let size = if self.direction == Direction::A {
            20.0
        } else {
            18.0
        };
        if self.query.is_empty() {
            // Painted here rather than as the field's hint: the hint colour follows the theme's weak
            // text, which is nearly invisible on a dark backdrop.
            painter.text(
                pos2(edit_rect.left() + 2.0, edit_rect.center().y),
                Align2::LEFT_CENTER,
                self.texts.get("launchpadSearch"),
                FontId::proportional(size),
                Color32::from_white_alpha(180),
            );
        }
        let edit = egui::TextEdit::singleline(&mut self.query)
            .font(FontId::proportional(size))
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
    // A picture of the screen, taken before the window exists, so the panel can show it blurred.
    let mut shot = None;
    let mut factor = 4;
    viewport = match system::primary_monitor() {
        Some(monitor) => {
            let scale = monitor.scale.max(1.0);
            let (w, h) = (
                monitor.screen.right - monitor.screen.left,
                monitor.screen.bottom - monitor.screen.top,
            );
            shot = chrome::capture(monitor.screen.left, monitor.screen.top, w, h);
            factor = (w / 480).max(2) as usize;
            viewport
                .with_position([
                    monitor.screen.left as f32 / scale,
                    monitor.screen.top as f32 / scale,
                ])
                .with_inner_size([w as f32 / scale, h as f32 / scale])
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
        Box::new(move |creation| {
            ui::install_theme(&creation.egui_ctx);
            let backdrop = Backdrop::new(&creation.egui_ctx, shot, factor, 5);
            Ok(Box::new(Launchpad::new(&creation.egui_ctx, backdrop)))
        }),
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
