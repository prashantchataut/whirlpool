//! Window overview: every open window as a live card on one screen (`whirlpool.exe --overview`).
//!
//! A short-lived process like the Start grid, so the dock keeps no memory for it. The previews are
//! composed by DWM into this window (the mechanism taskbar previews use): Whirlpool never reads or
//! stores the contents of another program's window. The packing and the arrow-key steps are in
//! `whirlpool_core::overview` (tested); this file draws the cards and talks to DWM.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Key, Modifiers, Rect, Sense, Stroke, StrokeKind,
    Ui, pos2, vec2,
};
use whirlpool_core::overview as pack;
use whirlpool_shell::{launcher, system, tracker};
use windows::{
    Win32::{
        Foundation::{HWND, RECT, SIZE},
        Graphics::Dwm::{
            DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION,
            DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE, DwmQueryThumbnailSourceSize,
            DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
        },
        UI::WindowsAndMessaging::FindWindowW,
    },
    core::HSTRING,
};

use crate::{glyphs, text::Texts};

pub const TITLE: &str = "Whirlpool Overview";
const ASPECT: f32 = 1.6;
const CAPTION: f32 = 38.0;
const GAP: f32 = 22.0;
const MAX_CARD: f32 = 520.0;
const REFRESH: Duration = Duration::from_millis(2000);

struct Card {
    hwnd: isize,
    title: String,
    minimized: bool,
}

/// One DWM thumbnail, released when dropped.
struct Thumb {
    handle: isize,
    source: SIZE,
    placed: Option<(RECT, u8)>,
}

impl Drop for Thumb {
    fn drop(&mut self) {
        // SAFETY: `handle` came from DwmRegisterThumbnail and is released once.
        unsafe {
            let _ = DwmUnregisterThumbnail(self.handle);
        }
    }
}

impl Thumb {
    fn update(&mut self, destination: RECT, opacity: u8) {
        if self.placed == Some((destination, opacity)) {
            return;
        }
        let properties = DWM_THUMBNAIL_PROPERTIES {
            dwFlags: DWM_TNP_RECTDESTINATION
                | DWM_TNP_VISIBLE
                | DWM_TNP_OPACITY
                | DWM_TNP_SOURCECLIENTAREAONLY,
            rcDestination: destination,
            opacity,
            fVisible: (opacity > 0).into(),
            fSourceClientAreaOnly: false.into(),
            ..Default::default()
        };
        // SAFETY: Valid handle and properties structure.
        if unsafe { DwmUpdateThumbnailProperties(self.handle, &properties) }.is_ok() {
            self.placed = Some((destination, opacity));
        }
    }
}

fn list_windows() -> Vec<Card> {
    let mut tracker = tracker::WindowTracker::default();
    tracker
        .list()
        .into_iter()
        .filter_map(|window| {
            let title = tracker::title(window.hwnd);
            // Our own panels and the dock are not windows to switch to.
            if title.trim().is_empty() || title.starts_with("Whirlpool") {
                return None;
            }
            Some(Card {
                hwnd: window.hwnd,
                title,
                minimized: window.minimized,
            })
        })
        .collect()
}

fn short(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(limit.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// The largest rectangle of the source's shape that fits `area`, centred in it.
fn fit(source: SIZE, area: Rect) -> Rect {
    let ratio = if source.cx > 0 && source.cy > 0 {
        source.cx as f32 / source.cy as f32
    } else {
        ASPECT
    };
    let (width, height) = if ratio >= area.width() / area.height().max(1.0) {
        (area.width(), area.width() / ratio)
    } else {
        (area.height() * ratio, area.height())
    };
    Rect::from_center_size(area.center(), vec2(width, height))
}

struct Overview {
    texts: Texts,
    cards: Vec<Card>,
    thumbs: HashMap<isize, Thumb>,
    /// This window, found by title once it exists; DWM draws the previews into it.
    own: Option<HWND>,
    selected: usize,
    started: Instant,
    focused_once: bool,
    refreshed: Instant,
}

impl Overview {
    fn new() -> Self {
        let config = whirlpool_config::load(&whirlpool_config::default_path())
            .ok()
            .flatten()
            .unwrap_or_default();
        Self {
            texts: Texts::for_language(config.language),
            cards: list_windows(),
            thumbs: HashMap::new(),
            own: None,
            selected: 0,
            started: Instant::now(),
            focused_once: false,
            refreshed: Instant::now(),
        }
    }

    fn close(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// Keeps the cards in step with the windows that exist now.
    fn refresh(&mut self) {
        let fresh = list_windows();
        let current = self.cards.get(self.selected).map(|card| card.hwnd);
        self.cards = fresh;
        if let Some(hwnd) = current
            && let Some(at) = self.cards.iter().position(|card| card.hwnd == hwnd)
        {
            self.selected = at;
        }
        self.selected = self.selected.min(self.cards.len().saturating_sub(1));
        let alive: Vec<isize> = self.cards.iter().map(|card| card.hwnd).collect();
        self.thumbs.retain(|hwnd, _| alive.contains(hwnd));
    }

    fn thumb(&mut self, hwnd: isize) -> Option<&mut Thumb> {
        if !self.thumbs.contains_key(&hwnd) {
            let own = self.own?;
            // SAFETY: Both handles are plain window handles; failure just means no preview.
            let handle = unsafe { DwmRegisterThumbnail(own, HWND(hwnd as *mut _)) }.ok()?;
            let source = unsafe { DwmQueryThumbnailSourceSize(handle) }.unwrap_or_default();
            self.thumbs.insert(
                hwnd,
                Thumb {
                    handle,
                    source,
                    placed: None,
                },
            );
        }
        self.thumbs.get_mut(&hwnd)
    }
}

impl eframe::App for Overview {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.02, 0.05, 0.09, 1.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.own.is_none() {
            // SAFETY: Looks a window of this process up by title.
            self.own = unsafe { FindWindowW(None, &HSTRING::from(TITLE)) }.ok();
        }
        let focused = ctx.input(|input| input.viewport().focused).unwrap_or(true);
        if focused {
            self.focused_once = true;
        } else if self.focused_once && self.started.elapsed() > Duration::from_millis(600) {
            Self::close(&ctx);
        }
        if self.refreshed.elapsed() >= REFRESH {
            self.refreshed = Instant::now();
            self.refresh();
        }
        ctx.request_repaint_after(REFRESH);

        let full = ui.max_rect();
        let ppp = ctx.pixels_per_point();
        let reduced = !system::animations_enabled();
        let appear = if reduced {
            1.0
        } else {
            ctx.animate_value_with_time(ui.make_persistent_id("appear"), 1.0, 0.2)
        };
        // Exponential ease-out from the progress of the animation: quick start, long soft landing.
        let eased = if appear >= 1.0 {
            1.0
        } else {
            (1.0 - 2f32.powf(-10.0 * appear)) / (1.0 - 2f32.powf(-10.0))
        };

        // Keys.
        let area = Rect::from_min_max(
            pos2(full.left() + 64.0, full.top() + 104.0),
            pos2(full.right() - 64.0, full.bottom() - 84.0),
        );
        let packed = pack::layout(
            self.cards.len(),
            area.width(),
            area.height(),
            ASPECT,
            CAPTION,
            GAP,
            MAX_CARD,
        );
        let (escape, enter, left, right, up, down) = ctx.input_mut(|input| {
            (
                input.consume_key(Modifiers::NONE, Key::Escape),
                input.consume_key(Modifiers::NONE, Key::Enter),
                input.consume_key(Modifiers::NONE, Key::ArrowLeft),
                input.consume_key(Modifiers::NONE, Key::ArrowRight),
                input.consume_key(Modifiers::NONE, Key::ArrowUp),
                input.consume_key(Modifiers::NONE, Key::ArrowDown),
            )
        });
        if escape {
            Self::close(&ctx);
        }
        let dx = i32::from(right) - i32::from(left);
        let dy = i32::from(down) - i32::from(up);
        if dx != 0 || dy != 0 {
            self.selected = pack::step(self.selected, self.cards.len(), packed.columns, dx, dy);
        }
        if enter && let Some(card) = self.cards.get(self.selected) {
            launcher::activate(&[card.hwnd]);
            Self::close(&ctx);
        }

        // Backdrop: deep water, lighter toward the top. A click on it closes the overview.
        let painter = ui.painter().clone();
        let mut mesh = egui::epaint::Mesh::default();
        let top = Color32::from_rgb(10, 30, 46);
        let bottom = Color32::from_rgb(3, 9, 18);
        mesh.colored_vertex(full.left_top(), top);
        mesh.colored_vertex(full.right_top(), top);
        mesh.colored_vertex(full.right_bottom(), bottom);
        mesh.colored_vertex(full.left_bottom(), bottom);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(egui::Shape::mesh(mesh));
        let background = ui.interact(full, ui.make_persistent_id("background"), Sense::click());

        painter.text(
            pos2(full.center().x, full.top() + 54.0),
            Align2::CENTER_CENTER,
            self.texts.get("overviewTitle"),
            FontId::proportional(26.0),
            Color32::from_white_alpha((235.0 * eased) as u8),
        );
        painter.text(
            pos2(full.center().x, full.bottom() - 44.0),
            Align2::CENTER_CENTER,
            self.texts.get("overviewHint"),
            FontId::proportional(14.0),
            Color32::from_white_alpha((150.0 * eased) as u8),
        );
        if self.cards.is_empty() {
            painter.text(
                full.center(),
                Align2::CENTER_CENTER,
                self.texts.get("overviewEmpty"),
                FontId::proportional(18.0),
                Color32::from_white_alpha(180),
            );
        }

        let accent = Color32::from_rgb(72, 205, 220);
        let mut activate = None;
        let mut dismiss = None;
        let mut placements: Vec<(isize, Rect, bool)> = Vec::new();
        for index in 0..self.cards.len().min(packed.slots.len()) {
            let slot = packed.slots[index];
            let (hwnd, minimized) = (self.cards[index].hwnd, self.cards[index].minimized);
            let base = Rect::from_min_size(
                area.min + vec2(slot.x, slot.y),
                vec2(slot.width, slot.height),
            );
            let response = ui.interact(base, ui.make_persistent_id(("card", hwnd)), Sense::click());
            if response.hovered() {
                self.selected = index;
            }
            let chosen = index == self.selected;
            let lift = ctx.animate_bool_with_time(
                ui.make_persistent_id(("lift", hwnd)),
                chosen,
                if reduced { 0.0 } else { 0.12 },
            );
            // The cards grow into place as the overview opens, and a little more when chosen.
            let scale = (0.9 + 0.1 * eased) * (1.0 + 0.025 * lift);
            let rect = Rect::from_center_size(base.center(), base.size() * scale);
            let alpha = eased;
            // Depth: an offset, soft-blurred shadow (a hard offset rectangle reads as a sticker).
            painter.add(
                egui::epaint::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: Color32::from_black_alpha((120.0 * alpha) as u8),
                }
                .as_shape(rect, CornerRadius::same(14)),
            );
            painter.rect_filled(
                rect,
                CornerRadius::same(14),
                Color32::from_rgba_unmultiplied(
                    255,
                    255,
                    255,
                    ((14.0 + 14.0 * lift) * alpha) as u8,
                ),
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(14),
                if chosen {
                    Stroke::new(2.0, accent)
                } else {
                    Stroke::new(1.0, Color32::from_white_alpha((34.0 * alpha) as u8))
                },
                StrokeKind::Inside,
            );
            let preview = Rect::from_min_max(
                rect.min + vec2(8.0, 8.0),
                pos2(rect.max.x - 8.0, rect.max.y - CAPTION * scale),
            );
            let caption_y = rect.max.y - CAPTION * scale * 0.5;
            let title_room = ((rect.width() - 64.0) / 8.0).max(6.0) as usize;
            painter.text(
                pos2(rect.left() + 14.0, caption_y),
                Align2::LEFT_CENTER,
                short(&self.cards[index].title, title_room),
                FontId::proportional(14.0),
                Color32::from_white_alpha((235.0 * alpha) as u8),
            );
            if minimized {
                glyphs::draw(
                    &painter,
                    "window",
                    Rect::from_center_size(preview.center(), vec2(44.0, 44.0)),
                    Color32::from_rgba_unmultiplied(150, 232, 240, 170),
                );
                painter.text(
                    pos2(preview.center().x, preview.center().y + 40.0),
                    Align2::CENTER_CENTER,
                    self.texts.get("overviewMinimized"),
                    FontId::proportional(12.0),
                    Color32::from_white_alpha(150),
                );
            }
            placements.push((hwnd, preview, !minimized));

            // The close button, drawn only on the chosen card; it sits in the caption, outside the preview.
            if chosen {
                let button =
                    Rect::from_center_size(pos2(rect.right() - 20.0, caption_y), vec2(22.0, 22.0));
                let over = ui.interact(
                    button,
                    ui.make_persistent_id(("close", hwnd)),
                    Sense::click(),
                );
                painter.circle_filled(
                    button.center(),
                    11.0,
                    if over.hovered() {
                        Color32::from_rgb(232, 96, 96)
                    } else {
                        Color32::from_white_alpha(34)
                    },
                );
                let arm = 4.0;
                let stroke = Stroke::new(1.6, Color32::WHITE);
                painter.line_segment(
                    [
                        button.center() + vec2(-arm, -arm),
                        button.center() + vec2(arm, arm),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        button.center() + vec2(-arm, arm),
                        button.center() + vec2(arm, -arm),
                    ],
                    stroke,
                );
                if over.clicked() {
                    dismiss = Some(hwnd);
                    continue;
                }
            }
            if response.clicked() {
                activate = Some(hwnd);
            }
        }

        // Previews, composed by DWM above this window's own drawing.
        let opacity = (255.0 * eased) as u8;
        for (hwnd, preview, visible) in placements {
            if !visible {
                continue;
            }
            let Some(thumb) = self.thumb(hwnd) else {
                continue;
            };
            let target = fit(thumb.source, preview);
            thumb.update(
                RECT {
                    left: ((target.left() - full.left()) * ppp).round() as i32,
                    top: ((target.top() - full.top()) * ppp).round() as i32,
                    right: ((target.right() - full.left()) * ppp).round() as i32,
                    bottom: ((target.bottom() - full.top()) * ppp).round() as i32,
                },
                opacity,
            );
        }

        if let Some(hwnd) = dismiss {
            launcher::close(&[hwnd]);
            self.cards.retain(|card| card.hwnd != hwnd);
            self.thumbs.remove(&hwnd);
            self.selected = self.selected.min(self.cards.len().saturating_sub(1));
        }
        if let Some(hwnd) = activate {
            launcher::activate(&[hwnd]);
            Self::close(&ctx);
        } else if background.clicked() {
            Self::close(&ctx);
        }
    }
}

pub fn run(compat: bool) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(TITLE)
        .with_decorations(false)
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
        Box::new(|_creation| Ok(Box::new(Overview::new()))),
    )
}
