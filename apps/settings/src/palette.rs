//! Command palette: one box for apps, files, actions and clipboard history.
//!
//! A short-lived process (`whirlpool.exe --palette`) like the Start grid, so the dock keeps no memory
//! for it. Ranking, mode prefixes and the calculator are in `whirlpool_core::palette` (tested).
//! Clipboard history is handed over on stdin by the dock when it starts this process; nothing about
//! the clipboard is ever written to disk.

use std::{
    collections::HashMap,
    io::Read,
    os::windows::process::CommandExt,
    path::Path,
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant, SystemTime},
};

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Key, Modifiers, Rect, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Ui, pos2, vec2,
};
use whirlpool_core::palette::{self as pal, Item, Kind, Mode};
use whirlpool_shell::{launcher, system};

use crate::{
    launchpad::{self, Message as PadMessage},
    text::Texts,
};

pub const TITLE: &str = "Whirlpool Search";
const WIDTH: f32 = 720.0;
const HEIGHT: f32 = 540.0;
const ROW: f32 = 52.0;
const MAX_ROWS: usize = 7;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MODES: [Mode; 5] = [
    Mode::All,
    Mode::Apps,
    Mode::Files,
    Mode::Actions,
    Mode::Clipboard,
];

enum Extra {
    Files(Vec<Item>),
    Clips(Vec<String>),
}

enum Row {
    Calc(String),
    Item(usize),
}

fn prefix(mode: Mode) -> &'static str {
    match mode {
        Mode::All => "",
        Mode::Apps => "@",
        Mode::Files => "/",
        Mode::Actions => ">",
        Mode::Clipboard => "#",
    }
}

fn scan_files() -> Vec<Item> {
    fn walk(dir: &Path, depth: u32, found: &mut Vec<(SystemTime, Item)>) {
        if depth > 2 || found.len() > 6000 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name.eq_ignore_ascii_case("desktop.ini") {
                continue;
            }
            if path.is_dir() {
                walk(&path, depth + 1, found);
            } else if let Ok(meta) = entry.metadata() {
                found.push((
                    meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    Item {
                        kind: Kind::File,
                        title: name,
                        detail: dir.to_string_lossy().into_owned(),
                        target: path.to_string_lossy().into_owned(),
                    },
                ));
            }
        }
    }
    let Some(home) = std::env::var_os("USERPROFILE") else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for folder in ["Desktop", "Documents", "Downloads"] {
        walk(&Path::new(&home).join(folder), 0, &mut found);
    }
    found.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    found.into_iter().map(|(_, item)| item).collect()
}

/// The dock writes a JSON array of clips and closes the pipe; a start without it gets none.
fn read_clips(send: Sender<Extra>, ctx: egui::Context) {
    thread::spawn(move || {
        let mut text = String::new();
        if std::io::stdin().read_to_string(&mut text).is_ok()
            && let Ok(clips) = serde_json::from_str::<Vec<String>>(&text)
        {
            let _ = send.send(Extra::Clips(clips));
            ctx.request_repaint();
        }
    });
}

fn run_action(id: &str) {
    let own = |argument: &str| {
        if let Ok(exe) = std::env::current_exe() {
            let _ = Command::new(exe)
                .arg(argument)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
    };
    let plain = |program: &str, args: &[&str]| {
        let _ = Command::new(program)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    };
    match id {
        "settings" => own("--settings"),
        "start-grid" => own("--launchpad"),
        "overlay" => own("--overlay-toggle"),
        "restore" => own("--restore"),
        "quit" => own("--quit"),
        "lock" => plain("rundll32.exe", &["user32.dll,LockWorkStation"]),
        "signout" => plain("shutdown.exe", &["/l"]),
        "task-manager" => plain("taskmgr.exe", &[]),
        "windows-settings" => plain("explorer.exe", &["ms-settings:"]),
        _ => {}
    }
}

struct Palette {
    texts: Texts,
    actions: Vec<Item>,
    apps: Vec<Item>,
    files: Vec<Item>,
    clips: Vec<Item>,
    items: Vec<Item>,
    icons: HashMap<String, TextureHandle>,
    pad: Receiver<PadMessage>,
    extra: Receiver<Extra>,
    query: String,
    rows: Vec<Row>,
    last_query: Option<String>,
    selected: usize,
    started: Instant,
    focused_once: bool,
    frames: u32,
    dirty: bool,
}

impl Palette {
    fn new(ctx: &egui::Context) -> Self {
        let language = whirlpool_config::load(&whirlpool_config::default_path())
            .ok()
            .flatten()
            .unwrap_or_default()
            .language;
        let texts = Texts::for_language(language);
        let action = |id: &str, key: &str, hint: &str| Item {
            kind: Kind::Action,
            title: texts.get(key).to_string(),
            detail: texts.get(hint).to_string(),
            target: id.to_owned(),
        };
        let actions = vec![
            action("settings", "paletteActSettings", "paletteActSettingsHint"),
            action("start-grid", "paletteActStart", "paletteActStartHint"),
            action("overlay", "paletteActOverlay", "paletteActOverlayHint"),
            action("lock", "paletteActLock", "paletteActLockHint"),
            action("signout", "paletteActSignOut", "paletteActSignOutHint"),
            action("task-manager", "paletteActTasks", "paletteActTasksHint"),
            action(
                "windows-settings",
                "paletteActWindows",
                "paletteActWindowsHint",
            ),
            action("restore", "paletteActRestore", "paletteActRestoreHint"),
            action("quit", "paletteActQuit", "paletteActQuitHint"),
        ];
        let (send, extra) = mpsc::channel();
        read_clips(send.clone(), ctx.clone());
        let files_ctx = ctx.clone();
        thread::spawn(move || {
            let _ = send.send(Extra::Files(scan_files()));
            files_ctx.request_repaint();
        });
        let mut this = Self {
            texts,
            actions,
            apps: Vec::new(),
            files: Vec::new(),
            clips: Vec::new(),
            items: Vec::new(),
            icons: HashMap::new(),
            pad: launchpad::spawn_loader(ctx.clone()),
            extra,
            query: String::new(),
            rows: Vec::new(),
            last_query: None,
            selected: 0,
            started: Instant::now(),
            focused_once: false,
            frames: 0,
            dirty: true,
        };
        this.rebuild();
        this
    }

    fn rebuild(&mut self) {
        self.items.clear();
        self.items.extend(self.actions.iter().cloned());
        self.items.extend(self.apps.iter().cloned());
        self.items.extend(self.clips.iter().cloned());
        self.items.extend(self.files.iter().cloned());
        self.dirty = true;
    }

    fn refilter(&mut self) {
        let query = pal::parse_query(&self.query);
        self.rows.clear();
        if query.mode == Mode::All
            && let Some(value) = pal::calculate(&query.text)
        {
            self.rows.push(Row::Calc(pal::format_number(value)));
        }
        for index in pal::rank(&self.items, &query, MAX_ROWS) {
            self.rows.push(Row::Item(index));
        }
        self.rows.truncate(MAX_ROWS);
        self.selected = 0;
        self.last_query = Some(self.query.clone());
        self.dirty = false;
    }

    fn set_mode(&mut self, mode: Mode) {
        let text = pal::parse_query(&self.query).text;
        self.query = format!("{}{}", prefix(mode), text);
    }

    fn cycle_mode(&mut self) {
        let now = pal::parse_query(&self.query).mode;
        let at = MODES.iter().position(|mode| *mode == now).unwrap_or(0);
        self.set_mode(MODES[(at + 1) % MODES.len()]);
    }

    fn run(&self, ctx: &egui::Context, row: usize) {
        match self.rows.get(row) {
            Some(Row::Calc(result)) => ctx.copy_text(result.clone()),
            Some(Row::Item(index)) => {
                let item = &self.items[*index];
                match item.kind {
                    Kind::Clipboard => ctx.copy_text(item.target.clone()),
                    Kind::Action => run_action(&item.target),
                    Kind::App | Kind::File => {
                        let _ = launcher::open(&item.target);
                    }
                }
            }
            None => return,
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn kind_label(&self, kind: Kind) -> &str {
        match kind {
            Kind::App => self.texts.get("paletteKindApp"),
            Kind::File => self.texts.get("paletteKindFile"),
            Kind::Action => self.texts.get("paletteKindAction"),
            Kind::Clipboard => self.texts.get("paletteKindClip"),
        }
    }

    fn draw_row(&self, ui: &Ui, rect: Rect, row: &Row, selected: bool) {
        let painter = ui.painter();
        let accent = Color32::from_rgb(72, 205, 220);
        if selected {
            painter.rect_filled(
                rect,
                CornerRadius::same(12),
                Color32::from_rgba_unmultiplied(72, 205, 220, 34),
            );
            painter.rect_filled(
                Rect::from_min_size(
                    pos2(rect.left(), rect.top() + 12.0),
                    vec2(3.0, rect.height() - 24.0),
                ),
                CornerRadius::same(2),
                accent,
            );
        }
        let icon =
            Rect::from_center_size(pos2(rect.left() + 34.0, rect.center().y), vec2(32.0, 32.0));
        let (title, detail, kind_text, texture, letter) = match row {
            Row::Calc(result) => (
                result.clone(),
                self.texts.get("paletteCalcHint").to_string(),
                self.texts.get("paletteKindCalc"),
                None,
                "=".to_owned(),
            ),
            Row::Item(index) => {
                let item = &self.items[*index];
                (
                    item.title.clone(),
                    item.detail.clone(),
                    self.kind_label(item.kind),
                    self.icons.get(&item.target),
                    item.title
                        .chars()
                        .next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_default(),
                )
            }
        };
        if let Some(texture) = texture {
            painter.image(
                texture.id(),
                icon,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.rect_filled(
                icon,
                CornerRadius::same(9),
                Color32::from_rgba_unmultiplied(255, 255, 255, 22),
            );
            painter.text(
                icon.center(),
                Align2::CENTER_CENTER,
                letter,
                FontId::proportional(16.0),
                Color32::from_white_alpha(220),
            );
        }
        painter.text(
            pos2(icon.right() + 14.0, rect.center().y - 9.0),
            Align2::LEFT_CENTER,
            clip(&title, 54),
            FontId::proportional(16.0),
            Color32::WHITE,
        );
        painter.text(
            pos2(icon.right() + 14.0, rect.center().y + 10.0),
            Align2::LEFT_CENTER,
            clip(&detail, 66),
            FontId::proportional(12.0),
            Color32::from_white_alpha(140),
        );
        painter.text(
            pos2(rect.right() - 16.0, rect.center().y),
            Align2::RIGHT_CENTER,
            kind_text,
            FontId::proportional(12.0),
            Color32::from_white_alpha(120),
        );
    }
}

fn clip(text: &str, limit: usize) -> String {
    let single: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    if single.chars().count() <= limit {
        return single;
    }
    let mut cut: String = single.chars().take(limit.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

impl eframe::App for Palette {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frames += 1;
        if self.frames < 6 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        while let Ok(message) = self.pad.try_recv() {
            match message {
                PadMessage::Apps(apps) => {
                    self.apps = apps
                        .into_iter()
                        .map(|app| Item {
                            kind: Kind::App,
                            title: app.name,
                            detail: self.texts.get("paletteKindApp").to_string(),
                            target: app.id,
                        })
                        .collect();
                    self.rebuild();
                }
                PadMessage::Icon(id, image) => {
                    let texture = ctx.load_texture(&id, image, TextureOptions::LINEAR);
                    self.icons.insert(id, texture);
                }
            }
        }
        while let Ok(message) = self.extra.try_recv() {
            match message {
                Extra::Files(files) => self.files = files,
                Extra::Clips(clips) => {
                    self.clips = clips
                        .into_iter()
                        .filter(|clip| !clip.trim().is_empty())
                        .map(|clip| Item {
                            kind: Kind::Clipboard,
                            title: clip.trim().lines().next().unwrap_or("").to_owned(),
                            detail: self.texts.get("paletteClipHint").to_string(),
                            target: clip,
                        })
                        .collect();
                }
            }
            self.rebuild();
        }
        if self.dirty || self.last_query.as_deref() != Some(self.query.as_str()) {
            self.refilter();
        }

        let focused = ctx.input(|input| input.viewport().focused).unwrap_or(true);
        if focused {
            self.focused_once = true;
        } else if self.focused_once && self.started.elapsed() > Duration::from_millis(600) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let (down, up, enter, escape, tab) = ctx.input_mut(|input| {
            (
                input.consume_key(Modifiers::NONE, Key::ArrowDown),
                input.consume_key(Modifiers::NONE, Key::ArrowUp),
                input.consume_key(Modifiers::NONE, Key::Enter),
                input.consume_key(Modifiers::NONE, Key::Escape),
                input.consume_key(Modifiers::NONE, Key::Tab),
            )
        });
        if escape {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if tab {
            self.cycle_mode();
        }
        let count = self.rows.len();
        if down && count > 0 {
            self.selected = (self.selected + 1) % count;
        }
        if up && count > 0 {
            self.selected = (self.selected + count - 1) % count;
        }
        if enter {
            self.run(&ctx, self.selected);
        }

        // Open: a short rise and fade; none when Windows asks for reduced motion.
        let reduced = !system::animations_enabled();
        let appear = if reduced {
            1.0
        } else {
            ctx.animate_value_with_time(ui.make_persistent_id("appear"), 1.0, 0.14)
        };
        ui.set_opacity(appear);
        let full = ui.max_rect();
        let rise = (1.0 - appear) * 10.0;
        let list_h = self.rows.len().max(1) as f32 * ROW;
        let panel = Rect::from_min_size(
            pos2(full.left() + 20.0, full.top() + 20.0 + rise),
            vec2(full.width() - 40.0, 118.0 + list_h + 10.0),
        );
        let painter = ui.painter().clone();
        for spread in [18.0, 10.0, 4.0] {
            painter.rect_filled(
                panel
                    .expand(spread * 0.5)
                    .translate(vec2(0.0, spread * 0.4)),
                CornerRadius::same(28),
                Color32::from_black_alpha(14),
            );
        }
        painter.rect_filled(
            panel,
            CornerRadius::same(22),
            Color32::from_rgba_unmultiplied(9, 20, 34, 238),
        );
        painter.rect_stroke(
            panel,
            CornerRadius::same(22),
            Stroke::new(1.0, Color32::from_white_alpha(30)),
            StrokeKind::Inside,
        );

        // The query box.
        let edit_rect = Rect::from_min_size(
            pos2(panel.left() + 24.0, panel.top() + 14.0),
            vec2(panel.width() - 48.0, 48.0),
        );
        let hint = self.texts.get("paletteHint").to_string();
        let edit = egui::TextEdit::singleline(&mut self.query)
            .frame(false)
            .font(FontId::proportional(22.0))
            .hint_text(egui::RichText::new(hint).color(Color32::from_white_alpha(90)))
            .desired_width(edit_rect.width())
            .text_color(Color32::WHITE);
        ui.put(edit_rect, edit).request_focus();

        // Mode chips.
        let mode_now = pal::parse_query(&self.query).mode;
        let mut x = panel.left() + 24.0;
        let chip_y = panel.top() + 68.0;
        let mut chosen = None;
        for mode in MODES {
            let label = self.texts.get(match mode {
                Mode::All => "paletteModeAll",
                Mode::Apps => "paletteModeApps",
                Mode::Files => "paletteModeFiles",
                Mode::Actions => "paletteModeActions",
                Mode::Clipboard => "paletteModeClips",
            });
            let width = 22.0 + label.chars().count() as f32 * 7.4;
            let rect = Rect::from_min_size(pos2(x, chip_y), vec2(width, 28.0));
            let response = ui.interact(rect, ui.id().with(("chip", prefix(mode))), Sense::click());
            let on = mode == mode_now;
            painter.rect_filled(
                rect,
                CornerRadius::same(14),
                if on {
                    Color32::from_rgba_unmultiplied(72, 205, 220, 52)
                } else if response.hovered() {
                    Color32::from_white_alpha(18)
                } else {
                    Color32::from_white_alpha(8)
                },
            );
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(13.0),
                if on {
                    Color32::from_rgb(150, 232, 240)
                } else {
                    Color32::from_white_alpha(170)
                },
            );
            if response.clicked() {
                chosen = Some(mode);
            }
            x += width + 8.0;
        }
        if let Some(mode) = chosen {
            self.set_mode(mode);
        }
        painter.line_segment(
            [
                pos2(panel.left() + 20.0, panel.top() + 108.0),
                pos2(panel.right() - 20.0, panel.top() + 108.0),
            ],
            Stroke::new(1.0, Color32::from_white_alpha(22)),
        );

        // Results.
        if self.rows.is_empty() {
            painter.text(
                pos2(panel.center().x, panel.top() + 108.0 + ROW / 2.0 + 4.0),
                Align2::CENTER_CENTER,
                self.texts.get("paletteEmpty"),
                FontId::proportional(14.0),
                Color32::from_white_alpha(120),
            );
        }
        let pointer = ctx.input(|input| input.pointer.hover_pos());
        let moved = ctx.input(|input| input.pointer.delta().length() > 0.0);
        let mut clicked = None;
        for (position, row) in self.rows.iter().enumerate() {
            let rect = Rect::from_min_size(
                pos2(
                    panel.left() + 10.0,
                    panel.top() + 112.0 + position as f32 * ROW,
                ),
                vec2(panel.width() - 20.0, ROW),
            );
            let response = ui.interact(rect, ui.id().with(("row", position)), Sense::click());
            if moved && pointer.is_some_and(|at| rect.contains(at)) {
                self.selected = position;
            }
            if response.clicked() {
                clicked = Some(position);
            }
            self.draw_row(ui, rect, row, position == self.selected);
        }
        if let Some(position) = clicked {
            self.run(&ctx, position);
        }
        if !reduced && appear < 1.0 {
            ctx.request_repaint();
        }
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
        .with_taskbar(false)
        .with_inner_size([WIDTH, HEIGHT]);
    if let Some(monitor) = system::primary_monitor() {
        let scale = monitor.scale.max(1.0);
        let (left, top) = (
            monitor.screen.left as f32 / scale,
            monitor.screen.top as f32 / scale,
        );
        let width = (monitor.screen.right - monitor.screen.left) as f32 / scale;
        let height = (monitor.screen.bottom - monitor.screen.top) as f32 / scale;
        viewport = viewport.with_position([
            left + (width - WIDTH) / 2.0,
            top + (height * 0.16).min(height - HEIGHT).max(0.0),
        ]);
    }
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
        Box::new(|creation| Ok(Box::new(Palette::new(&creation.egui_ctx)))),
    )
}
