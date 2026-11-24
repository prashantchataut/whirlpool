//! Notification center: a short-lived window with the history Whirlpool keeps in
//! `%APPDATA%\Whirlpool\notifications.txt`. Opened by `whirlpool --notifications`.

use std::{
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use eframe::egui::{
    self, Color32, CornerRadius, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use whirlpool_core::notify::{self, Entry};
use whirlpool_shell::system;

use crate::{chrome, text::Texts, ui};

pub const TITLE: &str = "Whirlpool Notifications";
const WIDTH: f32 = 384.0;
const HEIGHT: f32 = 540.0;
const RADIUS: u8 = 20;

fn file() -> PathBuf {
    whirlpool_config::default_path().with_file_name("notifications.txt")
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

fn load() -> Vec<Entry> {
    std::fs::read_to_string(file())
        .map(|text| notify::decode(&text))
        .unwrap_or_default()
}

fn save(list: &[Entry]) {
    if let Some(folder) = file().parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    let _ = std::fs::write(file(), notify::encode(list));
}

/// Adds one notification to the history (what `whirlpool --notify` and the dock call). Does nothing
/// when the history is turned off in Settings. Returns whether it was stored.
pub fn push(source: &str, title: &str, body: &str) -> bool {
    let on = whirlpool_config::load(&whirlpool_config::default_path())
        .ok()
        .flatten()
        .is_none_or(|config| config.notification_history);
    if !on {
        return false;
    }
    let mut list = load();
    let stored = notify::push(&mut list, now_ms(), source, title, body);
    save(&list);
    stored
}

struct Center {
    entries: Vec<Entry>,
    /// Which entries were unread when the window opened, so they keep their dot while it is open.
    was_unread: Vec<bool>,
    texts: Texts,
    started: Instant,
    focused_once: bool,
    glass: chrome::GlassPanel,
    strength: u8,
    glass_on: bool,
    region_done: bool,
}

impl Center {
    fn new(backdrop: Option<whirlpool_core::glass::Image>) -> Self {
        let config = whirlpool_config::load(&whirlpool_config::default_path())
            .ok()
            .flatten()
            .unwrap_or_default();
        let mut entries = load();
        let was_unread = entries.iter().map(|e| !e.read).collect();
        notify::mark_all_read(&mut entries);
        save(&entries);
        Self {
            entries,
            was_unread,
            texts: Texts::for_language(config.language),
            started: Instant::now(),
            focused_once: false,
            glass: chrome::GlassPanel::new(backdrop),
            strength: config.glass_strength,
            glass_on: config.glass_refraction,
            region_done: false,
        }
    }

    fn remove(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
            self.was_unread.remove(index);
            save(&self.entries);
        }
    }
}

impl eframe::App for Center {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.04, 0.09, 0.14, 1.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let full = ui.max_rect();
        let ppp = ctx.pixels_per_point();
        if !self.region_done {
            self.region_done = true;
            chrome::round_window(
                TITLE,
                (
                    0,
                    0,
                    (full.width() * ppp) as i32,
                    (full.height() * ppp) as i32,
                ),
                (f32::from(RADIUS) * ppp) as i32,
            );
        }
        // Close on Escape or when focus moves elsewhere (after the window had it once).
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or(true);
        self.focused_once |= focused;
        if ctx.input(|i| i.key_pressed(egui::Key::Escape))
            || (self.focused_once && !focused && self.started.elapsed().as_millis() > 400)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let painter = ui.painter().clone();
        let radius = CornerRadius::same(RADIUS);
        let drew_glass = self.glass_on
            && self.glass.paint(
                &ctx,
                &painter,
                full,
                full,
                f32::from(RADIUS),
                self.strength,
                Color32::from_rgba_unmultiplied(9, 20, 34, 168),
            );
        if !drew_glass {
            painter.rect_filled(full, radius, Color32::from_rgb(11, 24, 38));
        }
        painter.rect_stroke(
            full.shrink(0.5),
            radius,
            Stroke::new(1.0, Color32::from_white_alpha(34)),
            StrokeKind::Inside,
        );

        let t = &self.texts;
        let (title, clear_all) = (
            t.get("notifTitle").to_owned(),
            t.get("notifClear").to_owned(),
        );
        let now = now_ms();
        let mut dismiss: Option<usize> = None;
        let mut clear = false;
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(full.shrink2(vec2(18.0, 16.0))),
            |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&title)
                            .size(17.0)
                            .strong()
                            .color(Color32::from_rgb(238, 246, 250)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !self.entries.is_empty()
                            && ui
                                .add(egui::Button::new(
                                    RichText::new(&clear_all)
                                        .size(12.5)
                                        .color(Color32::from_rgb(160, 205, 215)),
                                ))
                                .clicked()
                        {
                            clear = true;
                        }
                    });
                });
                ui.add_space(8.0);
                if self.entries.is_empty() {
                    ui.add_space(110.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(t.get("notifEmpty"))
                                .size(15.0)
                                .color(Color32::from_rgb(220, 236, 242)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(t.get("notifEmptyHint"))
                                .size(12.5)
                                .color(Color32::from_rgb(150, 182, 194)),
                        );
                    });
                    return;
                }
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for (source, rows) in notify::group_by_source(&self.entries) {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(source.to_uppercase())
                                    .size(10.5)
                                    .color(Color32::from_rgb(120, 205, 216)),
                            );
                            for index in rows {
                                let entry = &self.entries[index];
                                let unread = self.was_unread.get(index).copied().unwrap_or(false);
                                if card(ui, entry, unread, now) {
                                    dismiss = Some(index);
                                }
                            }
                        }
                    });
            },
        );
        if clear {
            self.entries.clear();
            self.was_unread.clear();
            save(&self.entries);
        } else if let Some(index) = dismiss {
            self.remove(index);
        }
    }
}

/// One entry. Returns `true` when its dismiss button was pressed.
fn card(ui: &mut Ui, entry: &Entry, unread: bool, now: u64) -> bool {
    let width = ui.available_width();
    let body_galley = (!entry.body.is_empty()).then(|| {
        ui.painter().layout(
            entry.body.clone(),
            egui::FontId::proportional(12.5),
            Color32::from_rgb(176, 202, 212),
            width - 32.0,
        )
    });
    let height = 40.0 + body_galley.as_ref().map_or(0.0, |g| g.size().y + 4.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let hover = response.hovered();
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(13),
        Color32::from_white_alpha(if hover { 24 } else { 14 }),
    );
    if unread {
        painter.circle_filled(pos2(rect.left() + 11.0, rect.top() + 21.0), 3.5, ui::ACCENT);
    }
    let left = rect.left() + 22.0;
    let title = if entry.count > 1 {
        format!("{}  ×{}", entry.title, entry.count)
    } else {
        entry.title.clone()
    };
    painter.text(
        pos2(left, rect.top() + 13.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(13.5),
        Color32::from_rgb(238, 246, 250),
    );
    painter.text(
        pos2(rect.right() - 12.0, rect.top() + 14.0),
        egui::Align2::RIGHT_TOP,
        notify::ago(now, entry.at_ms),
        egui::FontId::proportional(11.0),
        Color32::from_rgb(130, 168, 180),
    );
    if let Some(galley) = body_galley {
        painter.galley(pos2(left, rect.top() + 34.0), galley, Color32::WHITE);
    }
    ui.add_space(5.0);
    if hover {
        let x = egui::Rect::from_center_size(
            pos2(rect.right() - 16.0, rect.bottom() - 13.0),
            vec2(20.0, 20.0),
        );
        let hit = ui.interact(
            x,
            ui.id().with(("dismiss", entry.at_ms, &entry.title)),
            Sense::click(),
        );
        let color = if hit.hovered() {
            Color32::WHITE
        } else {
            Color32::from_rgb(140, 176, 188)
        };
        let c = x.center();
        for (a, b) in [((-4.0, -4.0), (4.0, 4.0)), ((-4.0, 4.0), (4.0, -4.0))] {
            ui.painter().line_segment(
                [c + vec2(a.0, a.1), c + vec2(b.0, b.1)],
                Stroke::new(1.5, color),
            );
        }
        if hit.clicked() {
            return true;
        }
    }
    false
}

pub fn run(compat: bool) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let (scale, left, top, bottom, right) =
        system::primary_monitor().map_or((1.0, 0.0, 0.0, 1000.0, 1600.0), |m| {
            let s = m.scale.max(1.0);
            (
                s,
                m.work_left as f32 / s,
                m.work_top as f32 / s,
                m.work_bottom as f32 / s,
                m.work_right as f32 / s,
            )
        });
    let height = HEIGHT.min(bottom - top - 16.0).max(240.0);
    let (x, y) = (right - WIDTH - 12.0, top + 8.0);
    let backdrop = chrome::capture(
        (x * scale) as i32,
        (y * scale) as i32,
        (WIDTH * scale) as i32,
        (height * scale) as i32,
    );
    let _ = left;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_inner_size([WIDTH, height])
            .with_position([x, y]),
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
            Ok(Box::new(Center::new(backdrop)))
        }),
    )
}
