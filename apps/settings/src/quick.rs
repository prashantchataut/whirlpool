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
use whirlpool_core::tint::{self, Profile};
use whirlpool_shell::{imagery, launcher, status::AudioWatch, system};
use windows::{
    Devices::Radios::{Radio, RadioKind, RadioState},
    Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx},
};

use crate::{
    icons::{self, Icon},
    ui::{self, Palette},
};

const WIDTH: f32 = 372.0;
const HEIGHT: f32 = 352.0;
const PAD: f32 = 16.0;
const GAP: f32 = 8.0;
const RADIUS: u8 = 20;
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

/// Every colour the panel draws with. Starts from the app palette for the system theme and,
/// once the wallpaper has been read, leans toward it: the surface takes a trace of the picture's
/// average colour and the accent is the picture's dominant colour, adjusted until text on it
/// reads at 4.5:1.
#[derive(Clone, Copy)]
struct Look {
    surface: Color32,
    tile: Color32,
    tile_hover: Color32,
    border: Color32,
    rim: Color32,
    text: Color32,
    muted: Color32,
    track: Color32,
    accent: Color32,
    on_accent: Color32,
    accent_ink: Color32,
}

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn color_rgb(c: Color32) -> [u8; 3] {
    [c.r(), c.g(), c.b()]
}

/// Linear blend in egui's premultiplied space, so translucent fills keep their meaning.
fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let one = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        one(a.r(), b.r()),
        one(a.g(), b.g()),
        one(a.b(), b.b()),
        one(a.a(), b.a()),
    )
}

impl Look {
    fn base(p: &Palette, dark: bool) -> Self {
        Self {
            surface: p.window.gamma_multiply(0.96),
            tile: if dark {
                Color32::from_white_alpha(14)
            } else {
                Color32::from_white_alpha(190)
            },
            tile_hover: if dark {
                Color32::from_white_alpha(26)
            } else {
                Color32::WHITE
            },
            border: p.border,
            rim: if dark {
                Color32::from_white_alpha(22)
            } else {
                Color32::from_white_alpha(230)
            },
            text: p.text,
            muted: p.muted,
            track: p.track,
            accent: ui::ACCENT,
            on_accent: Color32::WHITE,
            accent_ink: p.accent_ink,
        }
    }

    /// The same look, leaning toward the wallpaper.
    fn with_wallpaper(mut self, profile: &Profile, dark: bool, p: &Palette) -> Self {
        let base = color_rgb(p.window);
        let wash = if dark {
            tint::mix(base, tint::mix(profile.average, [0, 0, 0], 0.72), 0.6)
        } else {
            tint::mix(
                base,
                tint::mix(profile.average, [255, 255, 255], 0.82),
                0.55,
            )
        };
        self.surface = Color32::from_rgba_unmultiplied(wash[0], wash[1], wash[2], 244);
        if let Some(vivid) = profile.vivid {
            let (fill, ink) = tint::accent_with_ink(vivid);
            self.accent = rgb(fill);
            self.on_accent = rgb(ink);
            // Text-sized accent that holds 4.5:1 on this surface.
            let surface = color_rgb(self.surface);
            let toward = if dark { [255, 255, 255] } else { [0, 0, 0] };
            let mut ink_color = fill;
            for step in 0..10u8 {
                if tint::contrast(ink_color, surface) >= 4.5 {
                    break;
                }
                ink_color = tint::mix(fill, toward, 0.1 * f32::from(step + 1));
            }
            self.accent_ink = rgb(ink_color);
        }
        self
    }

    fn lerp(self, to: Self, t: f32) -> Self {
        Self {
            surface: blend(self.surface, to.surface, t),
            tile: blend(self.tile, to.tile, t),
            tile_hover: blend(self.tile_hover, to.tile_hover, t),
            border: blend(self.border, to.border, t),
            rim: blend(self.rim, to.rim, t),
            text: blend(self.text, to.text, t),
            muted: blend(self.muted, to.muted, t),
            track: blend(self.track, to.track, t),
            accent: blend(self.accent, to.accent, t),
            on_accent: blend(self.on_accent, to.on_accent, t),
            accent_ink: blend(self.accent_ink, to.accent_ink, t),
        }
    }
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
    wallpaper_rx: Receiver<Option<Profile>>,
    profile: Option<Profile>,
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
        let (wallpaper_tx, wallpaper_rx) = mpsc::channel();
        {
            let ctx = ctx.clone();
            thread::spawn(move || {
                let profile = imagery::wallpaper_preview(0).and_then(|image| {
                    tint::profile_from_bgra(
                        image.width as usize,
                        image.height as usize,
                        &image.bgra,
                    )
                });
                let _ = wallpaper_tx.send(profile);
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
            wallpaper_rx,
            profile: None,
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

/// One toggle: a round icon, the name and its state. On, it fills with the accent.
fn tile(
    ui: &mut Ui,
    look: &Look,
    rect: Rect,
    icon: Icon,
    label: &str,
    sub: &str,
    on: bool,
) -> Response {
    let id = ui.make_persistent_id(label);
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, label));
    let ctx = ui.ctx().clone();
    let hover = ctx.animate_bool_with_time(
        id.with("hover"),
        response.hovered() || response.has_focus(),
        0.12,
    );
    let active = ctx.animate_bool_with_time(id.with("on"), on, 0.2);
    let pressed = response.is_pointer_button_down_on();
    let rect = if pressed { rect.shrink(1.5) } else { rect };

    let idle = blend(look.tile, look.tile_hover, hover);
    let fill = blend(idle, look.accent, active);
    let ink = blend(look.text, look.on_accent, active);
    let dim = blend(look.muted, look.on_accent.gamma_multiply(0.82), active);
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(14), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, blend(look.border, look.accent, active)),
        StrokeKind::Inside,
    );
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(16),
            Stroke::new(2.0, look.accent_ink),
            StrokeKind::Outside,
        );
    }
    let badge = Rect::from_center_size(
        pos2(rect.left() + 12.0 + 16.0, rect.center().y),
        vec2(32.0, 32.0),
    );
    painter.circle_filled(
        badge.center(),
        16.0,
        blend(
            look.text.gamma_multiply(0.09),
            look.on_accent.gamma_multiply(0.2),
            active,
        ),
    );
    icons::paint(painter, badge.shrink(7.0), icon, ink);
    let text_x = badge.right() + 10.0;
    painter.text(
        pos2(text_x, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        label,
        ui::heading(13.0),
        ink,
    );
    painter.text(
        pos2(text_x, rect.center().y + 9.0),
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

/// A battery drawn to scale: outline, nub, fill by charge. Amber at 20%, red at 10%.
fn battery_meter(ui: &Ui, look: &Look, rect: Rect, percent: u8, charging: bool) {
    let painter = ui.painter();
    let body = Rect::from_min_size(rect.min, vec2(rect.width() - 3.0, rect.height()));
    painter.rect_stroke(
        body,
        CornerRadius::same(4),
        Stroke::new(1.5, look.muted),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_center_size(
            pos2(rect.right() - 1.0, rect.center().y),
            vec2(2.5, rect.height() * 0.4),
        ),
        CornerRadius::same(1),
        look.muted,
    );
    let color = match percent {
        0..=10 => Color32::from_rgb(0xe5, 0x48, 0x4d),
        11..=20 => Color32::from_rgb(0xf0, 0xa2, 0x0a),
        _ => look.accent,
    };
    let inner = body.shrink(3.0);
    let mut fill = inner;
    fill.set_right(inner.left() + inner.width() * f32::from(percent.min(100)) / 100.0);
    painter.rect_filled(fill, CornerRadius::same(2), color);
    if charging {
        icons::paint(
            painter,
            Rect::from_center_size(body.center(), vec2(11.0, 11.0)),
            Icon::Bolt,
            look.text,
        );
    }
}

/// A flat text button with a hover wash; returns whether it was clicked.
fn ghost(ui: &mut Ui, look: &Look, right: f32, mid: f32, label: &str) -> (Response, f32) {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        FontId::proportional(12.5),
        look.accent_ink,
    );
    let width = galley.size().x + 22.0;
    let rect = Rect::from_min_size(pos2(right - width, mid - 15.0), vec2(width, 30.0));
    let id = ui.make_persistent_id(label);
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let hover = ui.ctx().animate_bool_with_time(
        id.with("h"),
        response.hovered() || response.has_focus(),
        0.12,
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(10),
        look.text.gamma_multiply(0.08 * hover),
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(10),
            Stroke::new(2.0, look.accent_ink),
            StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(12.5),
        look.accent_ink,
    );
    (response, rect.left() - 4.0)
}

impl eframe::App for Quick {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dark = ui.visuals().dark_mode;
        let p = Palette::of(ui);
        if let Ok(radios) = self.radios_rx.try_recv() {
            self.radios = radios;
        }
        if let Ok(profile) = self.wallpaper_rx.try_recv() {
            self.profile = profile;
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

        // Look: the system theme first, then easing toward the wallpaper once it is read.
        let plain = Look::base(&p, dark);
        let look = match &self.profile {
            Some(profile) => {
                let t =
                    ctx.animate_value_with_time(ui.make_persistent_id("wallpaper-look"), 1.0, 0.4);
                plain.lerp(plain.with_wallpaper(profile, dark, &p), t)
            }
            None => plain,
        };
        // Open: a short fade, none when the system asks for reduced motion.
        let reduced = !system::animations_enabled();
        let appear = if reduced {
            1.0
        } else {
            ctx.animate_value_with_time(ui.make_persistent_id("appear"), 1.0, 0.14)
        };
        ui.set_opacity(appear);

        let full = ui.max_rect();
        let radius = CornerRadius::same(RADIUS);
        ui.painter().rect_filled(full, radius, look.surface);
        ui.painter().rect_stroke(
            full.shrink(0.5),
            radius,
            Stroke::new(1.0, look.border),
            StrokeKind::Inside,
        );
        // A faint inner rim along the top reads as glass.
        ui.painter().rect_stroke(
            full.shrink(1.5),
            CornerRadius::same(RADIUS - 1),
            Stroke::new(1.0, look.rim),
            StrokeKind::Inside,
        );

        // Toggles: two columns, three rows.
        let tile_w = ((full.width() - 2.0 * PAD - GAP) / 2.0).floor();
        let tile_h = 58.0;
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
            &look,
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
            &look,
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
            &look,
            at(0, 1),
            Icon::Airplane,
            "Airplane mode",
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
        if tile(
            ui,
            &look,
            at(1, 1),
            Icon::Cast,
            "Cast",
            "Find displays",
            false,
        )
        .clicked()
        {
            Self::close(&ctx);
            thread::spawn(|| {
                // Let this panel go first so the Cast flyout keeps focus.
                thread::sleep(Duration::from_millis(220));
                launcher::open_cast();
            });
        }
        if tile(
            ui,
            &look,
            at(0, 2),
            Icon::Moon,
            "Night light",
            "Opens in Windows",
            false,
        )
        .clicked()
        {
            let _ = launcher::open("ms-settings:nightlight");
            Self::close(&ctx);
        }
        if tile(
            ui,
            &look,
            at(1, 2),
            Icon::Battery,
            "Battery saver",
            if saver { "On" } else { "Opens in Windows" },
            saver,
        )
        .clicked()
        {
            let _ = launcher::open("ms-settings:batterysaver");
            Self::close(&ctx);
        }

        // Volume, in its own rounded well.
        let well_top = full.top() + PAD + 3.0 * tile_h + 2.0 * GAP + 12.0;
        let well = Rect::from_min_size(
            pos2(full.left() + PAD, well_top),
            vec2(full.width() - 2.0 * PAD, 56.0),
        );
        ui.painter()
            .rect_filled(well, CornerRadius::same(14), look.tile);
        ui.painter().rect_stroke(
            well,
            CornerRadius::same(14),
            Stroke::new(1.0, look.border),
            StrokeKind::Inside,
        );
        let speaker = Rect::from_center_size(
            pos2(well.left() + 12.0 + 18.0, well.center().y),
            vec2(36.0, 36.0),
        );
        let speaker_response = ui
            .interact(speaker, ui.make_persistent_id("mute"), Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        speaker_response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Button, true, self.muted, "Mute")
        });
        let speaker_hover = ctx.animate_bool_with_time(
            ui.make_persistent_id("mute-hover"),
            speaker_response.hovered() || speaker_response.has_focus(),
            0.12,
        );
        ui.painter().circle_filled(
            speaker.center(),
            18.0,
            look.text.gamma_multiply(0.07 + 0.08 * speaker_hover),
        );
        if speaker_response.has_focus() {
            ui.painter()
                .circle_stroke(speaker.center(), 19.0, Stroke::new(2.0, look.accent_ink));
        }
        let silent = self.muted || self.volume <= 0.0;
        icons::paint(
            ui.painter(),
            speaker.shrink(8.0),
            if silent { Icon::Muted } else { Icon::Speaker },
            look.text,
        );
        if speaker_response.clicked() {
            if let Some(audio) = &mut self.audio {
                let _ = audio.toggle_mute();
            }
            self.muted = !self.muted;
        }
        let track = Rect::from_min_max(
            pos2(speaker.right() + 12.0, well.center().y - 11.0),
            pos2(well.right() - 14.0 - 34.0, well.center().y + 11.0),
        );
        let slide = ui.interact(
            track,
            ui.make_persistent_id("volume"),
            Sense::click_and_drag(),
        );
        slide.widget_info(|| {
            egui::WidgetInfo::slider(true, f64::from(self.volume * 100.0), "Volume")
        });
        let mut value = self.volume;
        if slide.dragged() || slide.clicked() {
            if let Some(pointer) = slide.interact_pointer_pos() {
                value = ((pointer.x - track.left() - 9.0) / (track.width() - 18.0)).clamp(0.0, 1.0);
            }
            self.dragging = true;
        } else {
            self.dragging = false;
        }
        if slide.has_focus() {
            let step = ctx.input(|i| {
                f32::from(i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp))
                    - f32::from(
                        i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown),
                    )
            });
            value = (value + step * 0.05).clamp(0.0, 1.0);
        }
        if (value - self.volume).abs() > 0.004 {
            self.volume = value;
            if let Some(audio) = &mut self.audio {
                let _ = audio.set_percent((value * 100.0).round() as u8);
            }
            self.muted = false;
        }
        let bar = Rect::from_center_size(track.center(), vec2(track.width(), 6.0));
        ui.painter()
            .rect_filled(bar, CornerRadius::same(3), look.track);
        let shown = if silent { 0.0 } else { self.volume };
        let knob_x = track.left() + 9.0 + (track.width() - 18.0) * shown;
        let mut filled = bar;
        filled.set_right(knob_x);
        ui.painter()
            .rect_filled(filled, CornerRadius::same(3), look.accent);
        let knob = pos2(knob_x, track.center().y);
        let knob_hover = ctx.animate_bool_with_time(
            ui.make_persistent_id("knob"),
            slide.hovered() || self.dragging || slide.has_focus(),
            0.12,
        );
        ui.painter()
            .circle_filled(knob + vec2(0.0, 1.0), 9.5, Color32::from_black_alpha(60));
        ui.painter().circle_filled(knob, 9.0, Color32::WHITE);
        if knob_hover > 0.0 {
            ui.painter().circle_stroke(
                knob,
                9.0 + 4.0 * knob_hover,
                Stroke::new(2.0, look.accent.gamma_multiply(0.5 * knob_hover)),
            );
        }
        ui.painter().text(
            pos2(well.right() - 14.0, well.center().y),
            Align2::RIGHT_CENTER,
            format!("{}", (shown * 100.0).round() as u32),
            FontId::proportional(13.0),
            look.muted,
        );

        // Footer: the battery, and the two places to go next.
        let mid = full.bottom() - PAD - 15.0;
        let meter =
            Rect::from_center_size(pos2(full.left() + PAD + 14.0 + 2.0, mid), vec2(28.0, 13.0));
        let (text, sub) = match self.battery {
            Some(b) => (
                format!("{}%", b.percent),
                if b.charging {
                    "Charging"
                } else if b.saver {
                    "Battery saver"
                } else {
                    "On battery"
                },
            ),
            None => ("Plugged in".to_owned(), ""),
        };
        if let Some(b) = self.battery {
            battery_meter(ui, &look, meter, b.percent, b.charging);
        } else {
            icons::paint(
                ui.painter(),
                Rect::from_center_size(meter.center(), vec2(18.0, 18.0)),
                Icon::Bolt,
                look.muted,
            );
        }
        let label = ui.painter().text(
            pos2(meter.right() + 10.0, mid),
            Align2::LEFT_CENTER,
            text,
            ui::heading(13.0),
            look.text,
        );
        if !sub.is_empty() {
            ui.painter().text(
                pos2(label.right() + 8.0, mid),
                Align2::LEFT_CENTER,
                sub,
                FontId::proportional(12.0),
                look.muted,
            );
        }
        let mut x = full.right() - PAD;
        for (label, action) in [("Windows settings", 1), ("Whirlpool", 0)] {
            let (response, next) = ghost(ui, &look, x, mid, label);
            x = next;
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
