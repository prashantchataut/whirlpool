//! The clock panel: the time with other time zones, a focus timer and a file shelf, in one
//! short-lived window. Opened by `whirlpool --hub [clock|focus|shelf]`, from the bar clock or the
//! palette. Seconds tick, and the window repaints, only while it is open. The focus timer is saved
//! to a file, so it keeps counting after the window closes.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use eframe::egui::{
    self, Color32, CornerRadius, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use whirlpool_config::Config;
use whirlpool_core::{
    cliphist::Shelf,
    clock::{self, ClockWeight, HourCycle},
    pomodoro::{self, Mode, Plan, Timer},
};
use whirlpool_shell::system;

use crate::{chrome, notify_center, text::Texts, ui};

pub const TITLE: &str = "Whirlpool Clock";
const WIDTH: f32 = 384.0;
const HEIGHT: f32 = 500.0;
const RADIUS: u8 = 20;
const INK: Color32 = Color32::from_rgb(238, 246, 250);
const MUTED: Color32 = Color32::from_rgb(150, 184, 196);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Clock,
    Focus,
    Shelf,
}

fn file(name: &str) -> PathBuf {
    whirlpool_config::default_path().with_file_name(name)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

struct Hub {
    tab: Tab,
    texts: Texts,
    config: Config,
    timer: Timer,
    last: Instant,
    shelf: Shelf,
    started: Instant,
    focused_once: bool,
    flash_until: Option<Instant>,
    glass: chrome::GlassPanel,
    region_done: bool,
}

impl Hub {
    fn new(tab: Tab, backdrop: Option<whirlpool_core::glass::Image>) -> Self {
        let config = whirlpool_config::load(&whirlpool_config::default_path())
            .ok()
            .flatten()
            .unwrap_or_default();
        let now = now_ms();
        let plan = Plan::default();
        let (timer, event) = std::fs::read_to_string(file("focus.txt"))
            .ok()
            .and_then(|text| pomodoro::decode(&text, plan, now))
            .unwrap_or_else(|| (Timer::new(plan), None));
        let mut shelf = Shelf::default();
        if let Ok(text) = std::fs::read_to_string(file("shelf.txt")) {
            shelf.add(text.lines().rev().map(str::to_owned), 0);
            shelf.prune(now, 0, |p| Path::new(p).exists());
        }
        let mut hub = Self {
            tab,
            texts: Texts::for_language(config.language),
            config,
            timer,
            last: Instant::now(),
            shelf,
            started: Instant::now(),
            focused_once: false,
            flash_until: None,
            glass: chrome::GlassPanel::new(backdrop),
            region_done: false,
        };
        if event.is_some() {
            hub.finished();
        }
        hub
    }

    fn save_timer(&self) {
        if let Some(folder) = file("focus.txt").parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        let _ = std::fs::write(file("focus.txt"), pomodoro::encode(&self.timer, now_ms()));
    }

    fn save_shelf(&self) {
        let text: String = self
            .shelf
            .files
            .iter()
            .rev()
            .map(|(p, _)| format!("{p}\n"))
            .collect();
        let _ = std::fs::write(file("shelf.txt"), text);
    }

    /// A phase ended: tell the notification history and flash the panel.
    fn finished(&mut self) {
        self.flash_until = Some(Instant::now() + Duration::from_millis(1800));
        let title = self.texts.get("hubFinished").to_owned();
        let mode = self.mode_label(self.timer.mode).to_owned();
        notify_center::push("Whirlpool", &title, &mode);
        self.save_timer();
    }

    fn mode_label(&self, mode: Mode) -> &str {
        self.texts.get(match mode {
            Mode::Focus => "hubFocus",
            Mode::ShortBreak => "hubShortBreak",
            Mode::LongBreak => "hubLongBreak",
        })
    }

    fn hour_cycle(&self) -> HourCycle {
        match self.config.menu_bar_24h {
            Some(true) => HourCycle::H24,
            Some(false) => HourCycle::H12,
            None => HourCycle::System,
        }
    }

    fn clock_tab(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let now = system::local_time();
        let h24 = system::system_uses_24h();
        let cycle = self.hour_cycle();
        let seconds = self.config.clock_seconds;
        let family = if self.config.clock_font == "mono" {
            egui::FontFamily::Monospace
        } else {
            egui::FontFamily::Proportional
        };
        let size = if ClockWeight::from_id(&self.config.clock_weight) == ClockWeight::Light {
            52.0
        } else {
            56.0
        };
        ui.add_space(14.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(clock::format_time(&now, cycle, h24, seconds))
                    .font(egui::FontId::new(size, family))
                    .color(INK),
            );
            ui.label(
                RichText::new(system::date_text("dddd, d MMMM"))
                    .size(15.0)
                    .color(MUTED),
            );
        });
        ui.add_space(16.0);
        let rows = system::zone_rows(&self.config.clock_zones);
        if rows.is_empty() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(self.texts.get("hubNoZones"))
                        .size(13.0)
                        .color(MUTED),
                );
            });
        }
        for row in rows {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::hover());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(13), Color32::from_white_alpha(14));
            let day = match row.days {
                0 => self.texts.get("hubToday"),
                1 => self.texts.get("hubTomorrow"),
                -1 => self.texts.get("hubYesterday"),
                _ => "",
            };
            ui.painter().text(
                pos2(rect.left() + 14.0, rect.top() + 14.0),
                egui::Align2::LEFT_CENTER,
                &row.city,
                egui::FontId::proportional(14.5),
                INK,
            );
            ui.painter().text(
                pos2(rect.left() + 14.0, rect.top() + 31.0),
                egui::Align2::LEFT_CENTER,
                day,
                egui::FontId::proportional(11.5),
                MUTED,
            );
            ui.painter().text(
                pos2(rect.right() - 14.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                clock::format_time(&row.time, cycle, h24, false),
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
                INK,
            );
            ui.add_space(6.0);
        }
        // Seconds tick only while this window is open.
        let wait = clock::next_tick_ms(
            u32::from(now.second),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.subsec_millis()),
            seconds,
        );
        ctx.request_repaint_after(Duration::from_millis(u64::from(wait)));
    }

    fn focus_tab(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(self.mode_label(self.timer.mode))
                    .size(15.0)
                    .color(MUTED),
            );
            ui.label(
                RichText::new(self.timer.label())
                    .font(egui::FontId::new(64.0, egui::FontFamily::Monospace))
                    .color(if self.flash_until.is_some_and(|t| Instant::now() < t) {
                        ui::FOAM
                    } else {
                        INK
                    }),
            );
        });
        ui.add_space(8.0);
        let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 8.0), Sense::hover());
        ui.painter()
            .rect_filled(bar, CornerRadius::same(4), Color32::from_white_alpha(22));
        let mut fill = bar;
        fill.set_width(bar.width() * self.timer.progress());
        ui.painter()
            .rect_filled(fill, CornerRadius::same(4), ui::ACCENT);
        ui.add_space(18.0);
        let mut changed = false;
        ui.horizontal(|ui| {
            let label = if self.timer.running {
                self.texts.get("hubPause")
            } else {
                self.texts.get("hubStart")
            };
            if ui::push(ui, label, Some(ui::ACCENT)).clicked() {
                self.timer.toggle();
                self.last = Instant::now();
                changed = true;
            }
            if ui::push(ui, self.texts.get("hubReset"), None).clicked() {
                self.timer.reset();
                changed = true;
            }
            if ui::push(ui, self.texts.get("hubSkip"), None).clicked() {
                self.timer.skip();
                changed = true;
            }
        });
        ui.add_space(16.0);
        ui.label(
            RichText::new(format!(
                "{}: {} / {}",
                self.texts.get("hubSessions"),
                self.timer.done % self.timer.plan.rounds.max(1),
                self.timer.plan.rounds
            ))
            .size(13.0)
            .color(MUTED),
        );
        if changed {
            self.save_timer();
        }
        if self.timer.running {
            ctx.request_repaint_after(Duration::from_millis(1000));
        } else if self.flash_until.is_some() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn shelf_tab(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.add_space(6.0);
        ui.label(
            RichText::new(self.texts.get("hubShelfHint"))
                .size(12.5)
                .color(MUTED),
        );
        ui.add_space(8.0);
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if self.shelf.files.is_empty() {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 170.0), Sense::hover());
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(16),
                Stroke::new(
                    if hovering { 2.0 } else { 1.0 },
                    if hovering {
                        ui::FOAM
                    } else {
                        Color32::from_white_alpha(50)
                    },
                ),
                StrokeKind::Inside,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.texts.get("hubShelfEmpty"),
                egui::FontId::proportional(14.0),
                MUTED,
            );
            return;
        }
        let mut remove: Option<String> = None;
        let mut clear = false;
        ScrollArea::vertical().max_height(330.0).show(ui, |ui| {
            for (path, _) in &self.shelf.files {
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(13),
                    Color32::from_white_alpha(if response.hovered() { 26 } else { 14 }),
                );
                let name = Shelf::file_name(path);
                ui.painter().text(
                    pos2(rect.left() + 14.0, rect.top() + 16.0),
                    egui::Align2::LEFT_CENTER,
                    whirlpool_core::cliphist::preview(name, 30),
                    egui::FontId::proportional(14.0),
                    INK,
                );
                ui.painter().text(
                    pos2(rect.left() + 14.0, rect.top() + 34.0),
                    egui::Align2::LEFT_CENTER,
                    whirlpool_core::cliphist::preview(path, 44),
                    egui::FontId::proportional(11.0),
                    MUTED,
                );
                if response.hovered() {
                    let mut x = rect.right() - 12.0;
                    for (id, label) in [("rm", "hubRemove"), ("rv", "hubReveal"), ("op", "hubOpen")]
                    {
                        let text = self.texts.get(label);
                        let w = 8.0 * text.chars().count() as f32 * 0.8 + 14.0;
                        let hit = egui::Rect::from_min_size(
                            pos2(x - w, rect.center().y - 12.0),
                            vec2(w, 24.0),
                        );
                        let r = ui.interact(hit, ui.id().with((id, path)), Sense::click());
                        ui.painter().rect_filled(
                            hit,
                            CornerRadius::same(8),
                            Color32::from_white_alpha(if r.hovered() { 50 } else { 28 }),
                        );
                        ui.painter().text(
                            hit.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(11.5),
                            INK,
                        );
                        if r.clicked() {
                            match id {
                                "op" => {
                                    let _ = whirlpool_shell::launcher::open(path);
                                }
                                "rv" => {
                                    use std::os::windows::process::CommandExt;
                                    let _ = std::process::Command::new("explorer.exe")
                                        .arg(format!("/select,{path}"))
                                        .creation_flags(0x0800_0000)
                                        .spawn();
                                }
                                _ => remove = Some(path.clone()),
                            }
                        }
                        x -= w + 6.0;
                    }
                }
                ui.add_space(6.0);
            }
        });
        if ui::push(ui, self.texts.get("hubClear"), None).clicked() {
            clear = true;
        }
        if clear {
            self.shelf.files.clear();
            self.save_shelf();
        } else if let Some(path) = remove {
            self.shelf.remove(&path);
            self.save_shelf();
        }
    }
}

impl eframe::App for Hub {
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
        // Close on Escape, or when focus moves elsewhere (not while the focus timer is running).
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or(true);
        self.focused_once |= focused;
        let keep_open = self.timer.running && self.tab == Tab::Focus;
        if ctx.input(|i| i.key_pressed(egui::Key::Escape))
            || (self.focused_once
                && !focused
                && !keep_open
                && self.started.elapsed().as_millis() > 400)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // Files dropped anywhere land on the shelf.
        let dropped: Vec<String> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.as_ref().map(|p| p.display().to_string()))
                .collect()
        });
        if !dropped.is_empty() {
            self.shelf.add(dropped, now_ms());
            self.save_shelf();
            self.tab = Tab::Shelf;
        }
        // Advance the focus timer by the time since the last frame.
        let dt = self.last.elapsed();
        self.last = Instant::now();
        if self.timer.tick(dt.as_millis() as u64).is_some() {
            self.finished();
        }

        let painter = ui.painter().clone();
        let radius = CornerRadius::same(RADIUS);
        let drew_glass = self.config.glass_refraction
            && self.glass.paint(
                &ctx,
                &painter,
                full,
                full,
                f32::from(RADIUS),
                self.config.glass_strength,
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

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(full.shrink2(vec2(18.0, 16.0))),
            |ui| {
                ui.horizontal(|ui| {
                    for (tab, key) in [
                        (Tab::Clock, "hubTabClock"),
                        (Tab::Focus, "hubTabFocus"),
                        (Tab::Shelf, "hubTabShelf"),
                    ] {
                        let label = RichText::new(self.texts.get(key))
                            .size(14.5)
                            .color(if self.tab == tab { INK } else { MUTED });
                        if ui.selectable_label(self.tab == tab, label).clicked() {
                            self.tab = tab;
                        }
                    }
                });
                ui.add_space(6.0);
                match self.tab {
                    Tab::Clock => self.clock_tab(ui, &ctx),
                    Tab::Focus => self.focus_tab(ui, &ctx),
                    Tab::Shelf => self.shelf_tab(ui, &ctx),
                }
            },
        );
    }
}

pub fn run(tab: &str, compat: bool) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let tab = match tab {
        "focus" => Tab::Focus,
        "shelf" => Tab::Shelf,
        _ => Tab::Clock,
    };
    let (scale, top, bottom, right) =
        system::primary_monitor().map_or((1.0, 0.0, 1000.0, 1600.0), |m| {
            let s = m.scale.max(1.0);
            (
                s,
                m.work_top as f32 / s,
                m.work_bottom as f32 / s,
                m.work_right as f32 / s,
            )
        });
    let height = HEIGHT.min(bottom - top - 16.0).max(300.0);
    let (x, y) = (right - WIDTH - 12.0, top + 8.0);
    let backdrop = chrome::capture(
        (x * scale) as i32,
        (y * scale) as i32,
        (WIDTH * scale) as i32,
        (height * scale) as i32,
    );
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
            Ok(Box::new(Hub::new(tab, backdrop)))
        }),
    )
}
