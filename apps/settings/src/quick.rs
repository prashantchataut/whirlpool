//! Quick settings: the panel the menu bar opens from its network, volume and
//! battery cluster. Wi-Fi and Bluetooth switch for real through the Windows
//! radio API; Cast, Night light and Battery saver hand over to the Windows
//! page that owns them, because Windows does not let other programs flip those.
//!
//! It is a short-lived process (`whirlpool.exe --quick-settings`) so the dock
//! keeps no memory for it. It closes when it loses focus.

use std::{
    env,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};
use whirlpool_shell::{launcher, status::AudioWatch, system};
use windows::{
    Devices::Radios::{Radio, RadioKind, RadioState},
    Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx},
};

use crate::{
    icons::{self, Icon},
    ui::{self, Palette},
};

const WIDTH: f32 = 364.0;
const HEIGHT: f32 = 340.0;
const PAD: f32 = 14.0;
const GAP: f32 = 8.0;
/// Space between the bar and the panel.
const EDGE_GAP: f32 = 8.0;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Radios {
    wifi: Option<bool>,
    bluetooth: Option<bool>,
}

fn init_com() {
    // SAFETY: Plain COM initialization of this worker thread; a repeat call
    // or a different apartment already in place is reported and ignored.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

fn read_radios() -> Radios {
    init_com();
    let mut out = Radios::default();
    let Ok(list) = Radio::GetRadiosAsync().and_then(|op| op.join()) else {
        return out;
    };
    for radio in &list {
        let on = radio.State().is_ok_and(|state| state == RadioState::On);
        match radio.Kind() {
            Ok(RadioKind::WiFi) => out.wifi = Some(out.wifi.unwrap_or(false) || on),
            Ok(RadioKind::Bluetooth) => out.bluetooth = Some(out.bluetooth.unwrap_or(false) || on),
            _ => {}
        }
    }
    out
}

fn set_radios(kinds: &[RadioKind], on: bool) -> Radios {
    init_com();
    let _ = Radio::RequestAccessAsync().and_then(|op| op.join());
    if let Ok(list) = Radio::GetRadiosAsync().and_then(|op| op.join()) {
        let state = if on { RadioState::On } else { RadioState::Off };
        for radio in &list {
            if radio.Kind().is_ok_and(|kind| kinds.contains(&kind)) {
                let _ = radio.SetStateAsync(state).and_then(|op| op.join());
            }
        }
    }
    read_radios()
}

pub struct Quick {
    radios: Radios,
    radios_rx: Receiver<Radios>,
    radios_tx: mpsc::Sender<Radios>,
    audio: Option<AudioWatch>,
    volume: f32,
    muted: bool,
    dragging: bool,
    started: Instant,
    focused_once: bool,
    last_battery: Instant,
    battery: Option<system::Battery>,
}

impl Quick {
    fn new(ctx: &egui::Context) -> Self {
        let (radios_tx, radios_rx) = mpsc::channel();
        {
            let (tx, ctx) = (radios_tx.clone(), ctx.clone());
            thread::spawn(move || {
                let _ = tx.send(read_radios());
                ctx.request_repaint();
            });
        }
        let mut audio = AudioWatch::new(0, 0);
        let (volume, muted) = audio
            .as_mut()
            .and_then(AudioWatch::volume)
            .map_or((0.5, false), |v| (f32::from(v.percent) / 100.0, v.muted));
        Self {
            radios: Radios::default(),
            radios_rx,
            radios_tx,
            audio,
            volume,
            muted,
            dragging: false,
            started: Instant::now(),
            focused_once: false,
            last_battery: Instant::now(),
            battery: system::battery(),
        }
    }

    fn toggle(&mut self, kinds: &'static [RadioKind], on: bool, ctx: &egui::Context) {
        let (tx, ctx) = (self.radios_tx.clone(), ctx.clone());
        thread::spawn(move || {
            let _ = tx.send(set_radios(kinds, on));
            ctx.request_repaint();
        });
    }

    fn close(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn tile(ui: &mut Ui, rect: Rect, icon: Icon, label: &str, sub: &str, on: bool) -> Response {
    let p = Palette::of(ui);
    let response = ui
        .interact(rect, ui.make_persistent_id(label), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let (fill, ink, dim) = if on {
        (ui::ACCENT, Color32::WHITE, Color32::from_white_alpha(205))
    } else if response.hovered() {
        (p.border, p.text, p.muted)
    } else {
        (p.card, p.text, p.muted)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(12), fill);
    if !on {
        painter.rect_stroke(
            rect,
            CornerRadius::same(12),
            Stroke::new(1.0, p.border),
            StrokeKind::Inside,
        );
    }
    icons::paint(
        painter,
        Rect::from_min_size(rect.min + vec2(12.0, 11.0), vec2(22.0, 22.0)),
        icon,
        ink,
    );
    painter.text(
        pos2(rect.left() + 13.0, rect.bottom() - 25.0),
        Align2::LEFT_CENTER,
        label,
        ui::heading(13.0),
        ink,
    );
    painter.text(
        pos2(rect.left() + 13.0, rect.bottom() - 10.0),
        Align2::LEFT_CENTER,
        sub,
        FontId::proportional(11.5),
        dim,
    );
    response
}

fn state_text(state: Option<bool>) -> &'static str {
    match state {
        Some(true) => "On",
        Some(false) => "Off",
        None => "Not available",
    }
}

impl eframe::App for Quick {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let p = Palette::of(ui);
        if let Ok(radios) = self.radios_rx.try_recv() {
            self.radios = radios;
        }
        // Close on focus loss, once the panel has been focused at all.
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or(true);
        if focused {
            self.focused_once = true;
        } else if self.focused_once || self.started.elapsed() > Duration::from_secs(2) {
            Self::close(&ctx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            Self::close(&ctx);
        }
        if self.last_battery.elapsed() > Duration::from_secs(5) {
            self.last_battery = Instant::now();
            self.battery = system::battery();
        }

        let full = ui.max_rect();
        ui.painter()
            .rect_filled(full, CornerRadius::same(16), p.window);
        ui.painter().rect_stroke(
            full.shrink(0.5),
            CornerRadius::same(16),
            Stroke::new(1.0, p.border),
            StrokeKind::Inside,
        );

        // Tiles: three columns, two rows.
        let tile_w = ((full.width() - 2.0 * PAD - 2.0 * GAP) / 3.0).floor();
        let tile_h = 76.0;
        let at = |col: usize, row: usize| {
            Rect::from_min_size(
                full.min
                    + vec2(
                        PAD + col as f32 * (tile_w + GAP),
                        PAD + row as f32 * (tile_h + GAP),
                    ),
                vec2(tile_w, tile_h),
            )
        };
        let saver = self.battery.is_some_and(|b| b.saver);
        let all_off = self.radios.wifi != Some(true)
            && self.radios.bluetooth != Some(true)
            && (self.radios.wifi.is_some() || self.radios.bluetooth.is_some());

        if tile(
            ui,
            at(0, 0),
            Icon::Wifi,
            "Wi-Fi",
            state_text(self.radios.wifi),
            self.radios.wifi == Some(true),
        )
        .clicked()
            && let Some(on) = self.radios.wifi
        {
            self.radios.wifi = Some(!on);
            self.toggle(&[RadioKind::WiFi], !on, &ctx);
        }
        if tile(
            ui,
            at(1, 0),
            Icon::Bluetooth,
            "Bluetooth",
            state_text(self.radios.bluetooth),
            self.radios.bluetooth == Some(true),
        )
        .clicked()
            && let Some(on) = self.radios.bluetooth
        {
            self.radios.bluetooth = Some(!on);
            self.toggle(&[RadioKind::Bluetooth], !on, &ctx);
        }
        if tile(
            ui,
            at(2, 0),
            Icon::Airplane,
            "Airplane",
            if all_off { "On" } else { "Off" },
            all_off,
        )
        .clicked()
        {
            let turn_on = all_off;
            self.radios = Radios {
                wifi: self.radios.wifi.map(|_| turn_on),
                bluetooth: self.radios.bluetooth.map(|_| turn_on),
            };
            self.toggle(
                &[
                    RadioKind::WiFi,
                    RadioKind::Bluetooth,
                    RadioKind::MobileBroadband,
                ],
                turn_on,
                &ctx,
            );
        }
        if tile(ui, at(0, 1), Icon::Cast, "Cast", "Find displays", false).clicked() {
            Self::close(&ctx);
            thread::spawn(|| {
                // Let this panel go first so the Cast flyout keeps focus.
                thread::sleep(Duration::from_millis(220));
                launcher::open_cast();
            });
        }
        if tile(
            ui,
            at(1, 1),
            Icon::Moon,
            "Night light",
            "Windows setting",
            false,
        )
        .clicked()
        {
            let _ = launcher::open("ms-settings:nightlight");
            Self::close(&ctx);
        }
        if tile(
            ui,
            at(2, 1),
            Icon::Battery,
            "Battery saver",
            if saver { "On" } else { "Windows setting" },
            saver,
        )
        .clicked()
        {
            let _ = launcher::open("ms-settings:batterysaver");
            Self::close(&ctx);
        }

        // Volume.
        let row_top = full.top() + PAD + 2.0 * tile_h + GAP + 22.0;
        let speaker = Rect::from_min_size(pos2(full.left() + PAD, row_top), vec2(36.0, 36.0));
        let speaker_response = ui
            .interact(speaker, ui.make_persistent_id("mute"), Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if speaker_response.hovered() {
            ui.painter().rect_filled(
                speaker,
                CornerRadius::same(10),
                p.border.gamma_multiply(0.6),
            );
        }
        icons::paint(
            ui.painter(),
            speaker.shrink(7.0),
            if self.muted || self.volume <= 0.0 {
                Icon::Muted
            } else {
                Icon::Speaker
            },
            p.text,
        );
        if speaker_response.clicked() {
            if let Some(audio) = &mut self.audio {
                let _ = audio.toggle_mute();
            }
            self.muted = !self.muted;
        }
        let track = Rect::from_min_max(
            pos2(speaker.right() + 10.0, row_top + 8.0),
            pos2(full.right() - PAD - 44.0, row_top + 28.0),
        );
        let slide = ui.interact(
            track,
            ui.make_persistent_id("volume"),
            Sense::click_and_drag(),
        );
        if slide.dragged() || slide.clicked() {
            if let Some(pointer) = slide.interact_pointer_pos() {
                let value =
                    ((pointer.x - track.left() - 9.0) / (track.width() - 18.0)).clamp(0.0, 1.0);
                if (value - self.volume).abs() > 0.004 {
                    self.volume = value;
                    if let Some(audio) = &mut self.audio {
                        let _ = audio.set_percent((value * 100.0).round() as u8);
                    }
                    self.muted = false;
                }
            }
            self.dragging = true;
        } else {
            self.dragging = false;
        }
        let bar = Rect::from_center_size(track.center(), vec2(track.width(), 6.0));
        ui.painter()
            .rect_filled(bar, CornerRadius::same(3), p.track);
        let knob_x = track.left() + 9.0 + (track.width() - 18.0) * self.volume;
        let mut filled = bar;
        filled.set_right(knob_x);
        ui.painter()
            .rect_filled(filled, CornerRadius::same(3), ui::ACCENT);
        let knob = pos2(knob_x, track.center().y);
        ui.painter()
            .circle_filled(knob + vec2(0.0, 1.0), 9.5, Color32::from_black_alpha(60));
        ui.painter().circle_filled(knob, 9.0, Color32::WHITE);
        if slide.hovered() || self.dragging {
            ui.painter().circle_stroke(
                knob,
                12.0,
                Stroke::new(2.0, ui::ACCENT.gamma_multiply(0.45)),
            );
        }
        ui.painter().text(
            pos2(full.right() - PAD, row_top + 18.0),
            Align2::RIGHT_CENTER,
            format!("{}", (self.volume * 100.0).round() as u32),
            FontId::proportional(13.0),
            p.muted,
        );

        // Footer.
        let foot_top = full.bottom() - 54.0;
        ui.painter().rect_filled(
            Rect::from_min_size(
                pos2(full.left() + 1.0, foot_top),
                vec2(full.width() - 2.0, 1.0),
            ),
            0.0,
            p.border,
        );
        let battery_text = match self.battery {
            Some(b) if b.charging => format!("{}%  ·  Charging", b.percent),
            Some(b) if b.saver => format!("{}%  ·  Battery saver", b.percent),
            Some(b) => format!("{}%", b.percent),
            None => "Plugged in".to_owned(),
        };
        let mid = foot_top + 27.0;
        icons::paint(
            ui.painter(),
            Rect::from_center_size(pos2(full.left() + PAD + 11.0, mid), vec2(22.0, 22.0)),
            if self.battery.is_some_and(|b| b.charging) {
                Icon::Bolt
            } else {
                Icon::Battery
            },
            p.muted,
        );
        ui.painter().text(
            pos2(full.left() + PAD + 28.0, mid),
            Align2::LEFT_CENTER,
            battery_text,
            FontId::proportional(13.0),
            p.text,
        );
        let mut x = full.right() - PAD;
        for (label, action) in [("Windows settings", 1), ("Whirlpool", 0)] {
            let width = 18.0 + label.len() as f32 * 6.6;
            let rect = Rect::from_min_size(pos2(x - width, mid - 15.0), vec2(width, 30.0));
            x = rect.left() - 6.0;
            let response = ui
                .interact(rect, ui.make_persistent_id(label), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if response.hovered() {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(9), p.border.gamma_multiply(0.7));
            }
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(12.5),
                p.accent_ink,
            );
            if response.clicked() {
                if action == 0 {
                    if let Ok(exe) = env::current_exe() {
                        let _ = std::process::Command::new(exe).arg("--settings").spawn();
                    }
                } else {
                    let _ = launcher::open("ms-settings:");
                }
                Self::close(&ctx);
            }
        }
        if self.dragging {
            ctx.request_repaint();
        }
    }
}

/// Opens the panel with its right edge at `right` and hanging from (or, for
/// a bar on the bottom edge, standing on) `anchor_y`; both in screen pixels.
pub fn run(right: i32, anchor_y: i32, bottom: bool, compat: bool) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let (scale, left_limit, top_limit, bottom_limit) =
        system::primary_monitor().map_or((1.0, 0.0, 0.0, 2000.0), |m| {
            let s = m.scale.max(1.0);
            (
                s,
                m.work_left as f32 / s,
                m.work_top as f32 / s,
                m.work_bottom as f32 / s,
            )
        });
    let x = (right as f32 / scale - WIDTH).max(left_limit + 8.0);
    let y = if bottom {
        anchor_y as f32 / scale - HEIGHT - EDGE_GAP
    } else {
        anchor_y as f32 / scale + EDGE_GAP
    }
    .clamp(
        top_limit + 4.0,
        (bottom_limit - HEIGHT - 4.0).max(top_limit + 4.0),
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Whirlpool Quick Settings")
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_inner_size([WIDTH, HEIGHT])
            .with_position([x, y]),
        renderer: if compat {
            eframe::Renderer::Wgpu
        } else {
            eframe::Renderer::Glow
        },
        ..Default::default()
    };
    eframe::run_native(
        "Whirlpool Quick Settings",
        options,
        Box::new(|creation| {
            ui::install_theme(&creation.egui_ctx);
            Ok(Box::new(Quick::new(&creation.egui_ctx)))
        }),
    )
}
