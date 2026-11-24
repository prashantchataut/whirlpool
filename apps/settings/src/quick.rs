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
use whirlpool_config::{TileKind, TileSpec};
use whirlpool_core::{
    tiles,
    tint::{self, Profile},
};
use whirlpool_shell::{imagery, launcher, status::AudioWatch, system};
use windows::{
    Devices::Radios::{Radio, RadioKind, RadioState},
    Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx},
};

use crate::{
    chrome,
    design::{self, Backdrop, Direction},
    icons::{self, Icon},
    ui::{self, Palette},
};

const WIDTH: f32 = 372.0;
/// The design used unless `WHIRLPOOL_QUICK_DESIGN` (a, b or c) asks for another.
const DEFAULT_DIRECTION: Direction = Direction::A;

fn tile_h(dir: Direction) -> f32 {
    match dir {
        Direction::A => 68.0,
        Direction::B => 92.0,
        Direction::C => 100.0,
    }
}

/// Height of the volume area.
fn well_h(dir: Direction) -> f32 {
    match dir {
        Direction::A => 54.0,
        Direction::B => 56.0,
        Direction::C => 84.0,
    }
}

/// Everything below the tile grid: padding, the volume area and the battery line.
fn below_grid(dir: Direction) -> f32 {
    106.0 + well_h(dir)
}

fn grid_height(rows: usize, dir: Direction) -> f32 {
    if rows == 0 {
        0.0
    } else {
        rows as f32 * tile_h(dir) + (rows as f32 - 1.0) * GAP
    }
}

fn panel_height(rows: usize, dir: Direction) -> f32 {
    below_grid(dir) + grid_height(rows, dir)
}
const PAD: f32 = 16.0;
const GAP: f32 = 10.0;
const RADIUS: u8 = 28;
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
    dir: Direction,
    /// 0..=1 entrance progress of the item being drawn (set per tile before it is drawn).
    enter: f32,
    /// The panel's own fade-in, applied to everything drawn with a cloned painter.
    opacity: f32,
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
            dir: Direction::from_env("WHIRLPOOL_QUICK_DESIGN", DEFAULT_DIRECTION),
            enter: 1.0,
            opacity: 1.0,
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
            dir: self.dir,
            enter: self.enter,
            opacity: self.opacity,
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
    tiles: Vec<TileSpec>,
    region_done: bool,
    backdrop: Backdrop,
}

impl Quick {
    fn new(ctx: &egui::Context, tiles: Vec<TileSpec>, backdrop: Backdrop) -> Self {
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
            tiles,
            region_done: false,
            backdrop,
        }
    }

    /// One tile of the grid: draws it and does what a click on it does.
    #[allow(clippy::too_many_arguments)]
    fn tile_slot(
        &mut self,
        ui: &mut Ui,
        look: &Look,
        rect: Rect,
        kind: TileKind,
        saver: bool,
        all_off: bool,
        ctx: &egui::Context,
    ) {
        let windows_page = |uri: &str| {
            let _ = launcher::open(uri);
        };
        match kind {
            TileKind::Wifi => {
                if tile(
                    ui,
                    look,
                    rect,
                    Icon::Wifi,
                    "Wi-Fi",
                    state_text(self.radios.wifi),
                    self.radios.wifi == Some(true),
                )
                .clicked()
                    && let Some(on) = self.radios.wifi
                {
                    self.radios.wifi = Some(!on);
                    self.toggle(&[RadioKind::WiFi], !on, ctx);
                }
            }
            TileKind::Bluetooth => {
                if tile(
                    ui,
                    look,
                    rect,
                    Icon::Bluetooth,
                    "Bluetooth",
                    state_text(self.radios.bluetooth),
                    self.radios.bluetooth == Some(true),
                )
                .clicked()
                    && let Some(on) = self.radios.bluetooth
                {
                    self.radios.bluetooth = Some(!on);
                    self.toggle(&[RadioKind::Bluetooth], !on, ctx);
                }
            }
            TileKind::Airplane => {
                if tile(
                    ui,
                    look,
                    rect,
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
                        ctx,
                    );
                }
            }
            TileKind::Cast => {
                if tile(ui, look, rect, Icon::Cast, "Cast", "Find displays", false).clicked() {
                    Self::close(ctx);
                    thread::spawn(|| {
                        // Let this panel go first so the Cast flyout keeps focus.
                        thread::sleep(Duration::from_millis(220));
                        launcher::open_cast();
                    });
                }
            }
            TileKind::NightLight => {
                if tile(
                    ui,
                    look,
                    rect,
                    Icon::Moon,
                    "Night light",
                    "In Settings",
                    false,
                )
                .clicked()
                {
                    windows_page("ms-settings:nightlight");
                    Self::close(ctx);
                }
            }
            TileKind::BatterySaver => {
                if tile(
                    ui,
                    look,
                    rect,
                    Icon::Battery,
                    "Battery saver",
                    if saver { "On" } else { "In Settings" },
                    saver,
                )
                .clicked()
                {
                    windows_page("ms-settings:batterysaver");
                    Self::close(ctx);
                }
            }
            TileKind::Focus => {
                if tile(ui, look, rect, Icon::Focus, "Focus", "In Settings", false).clicked() {
                    windows_page("ms-settings:quietmomentshome");
                    Self::close(ctx);
                }
            }
            TileKind::Display => {
                if tile(
                    ui,
                    look,
                    rect,
                    Icon::Display,
                    "Display",
                    "In Settings",
                    false,
                )
                .clicked()
                {
                    windows_page("ms-settings:display");
                    Self::close(ctx);
                }
            }
            TileKind::Hotspot => {
                if tile(
                    ui,
                    look,
                    rect,
                    Icon::Hotspot,
                    "Hotspot",
                    "In Settings",
                    false,
                )
                .clicked()
                {
                    windows_page("ms-settings:network-mobilehotspot");
                    Self::close(ctx);
                }
            }
            TileKind::Sound | TileKind::Unknown => {
                if tile(ui, look, rect, Icon::Speaker, "Sound", "In Settings", false).clicked() {
                    Self::close(ctx);
                    thread::spawn(|| {
                        thread::sleep(Duration::from_millis(220));
                        launcher::open_sound();
                    });
                }
            }
        }
    }

    fn toggle(&mut self, kinds: &'static [RadioKind], on: bool, ctx: &egui::Context) {
        let (tx, ctx) = (self.radios_tx.clone(), ctx.clone());
        thread::spawn(move || {
            let _ = tx.send(set_radios(kinds, on));
            ctx.request_repaint();
        });
    }

    /// The volume control. Dragging anywhere on the track sets it; the speaker mutes. Per
    /// direction: a capsule that fills like a water level, a row of segments, or a pearl on a line.
    fn volume_row(&mut self, ui: &mut Ui, look: &Look, well: Rect, ctx: &egui::Context) {
        let dir = look.dir;
        let silent = self.muted || self.volume <= 0.0;
        let shown = if silent { 0.0 } else { self.volume };
        let (speaker, track, inset) = match dir {
            Direction::A => (
                Rect::from_center_size(pos2(well.left() + 26.0, well.center().y), vec2(36.0, 36.0)),
                well,
                well.height() / 2.0,
            ),
            Direction::B => (
                Rect::from_center_size(pos2(well.left() + 24.0, well.center().y), vec2(36.0, 36.0)),
                Rect::from_min_max(
                    pos2(well.left() + 54.0, well.top()),
                    pos2(well.right() - 46.0, well.bottom()),
                ),
                0.0,
            ),
            Direction::C => (
                Rect::from_center_size(
                    pos2(well.left() + 20.0, well.top() + 20.0),
                    vec2(36.0, 36.0),
                ),
                Rect::from_min_max(
                    pos2(well.left() + 4.0, well.bottom() - 28.0),
                    pos2(well.right() - 4.0, well.bottom() - 2.0),
                ),
                13.0,
            ),
        };

        // The slider first, the speaker on top of it, so a click on the icon mutes.
        let slide = ui.interact(
            track,
            ui.make_persistent_id("volume"),
            Sense::click_and_drag(),
        );
        slide.widget_info(|| {
            egui::WidgetInfo::slider(true, f64::from(self.volume * 100.0), "Volume")
        });
        let speaker_response = ui
            .interact(speaker, ui.make_persistent_id("mute"), Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        speaker_response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::Button, true, self.muted, "Mute")
        });
        let mut value = self.volume;
        if (slide.dragged() || slide.clicked()) && !speaker_response.hovered() {
            if let Some(pointer) = slide.interact_pointer_pos() {
                value = ((pointer.x - track.left() - inset)
                    / (track.width() - 2.0 * inset).max(1.0))
                .clamp(0.0, 1.0);
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
        if speaker_response.clicked() {
            if let Some(audio) = &mut self.audio {
                let _ = audio.toggle_mute();
            }
            self.muted = !self.muted;
        }

        let mut painter = ui.painter().clone();
        painter.set_opacity(look.opacity * design::out_expo(look.enter));
        let held = ctx.animate_bool_with_time(
            ui.make_persistent_id("held"),
            slide.hovered() || self.dragging || slide.has_focus(),
            0.12,
        );
        let speaker_hover = ctx.animate_bool_with_time(
            ui.make_persistent_id("mute-hover"),
            speaker_response.hovered() || speaker_response.has_focus(),
            0.12,
        );
        let glyph = if silent { Icon::Muted } else { Icon::Speaker };
        let percent = (shown * 100.0).round() as u32;
        match dir {
            Direction::A => {
                let h = well.height();
                let corner = CornerRadius::same((h / 2.0) as u8);
                painter.rect_filled(well, corner, Color32::from_white_alpha(26));
                painter.rect_stroke(
                    well,
                    corner,
                    Stroke::new(1.0, Color32::from_white_alpha(36)),
                    StrokeKind::Inside,
                );
                // The water level: never narrower than the capsule is tall, so the icon always
                // sits on the fill.
                let fill_w = h + (well.width() - h) * shown;
                let fill = Rect::from_min_size(well.min, vec2(fill_w, h));
                let fill_color = if silent {
                    Color32::from_white_alpha(46)
                } else {
                    blend(look.accent, Color32::WHITE, 0.10 * held)
                };
                painter.rect_filled(fill, corner, fill_color);
                if !silent {
                    // A bright edge where the water meets the air.
                    painter.line_segment(
                        [
                            pos2(fill.right() - 6.0, fill.top() + 12.0),
                            pos2(fill.right() - 6.0, fill.bottom() - 12.0),
                        ],
                        Stroke::new(2.5, look.on_accent.gamma_multiply(0.35 + 0.3 * held)),
                    );
                }
                let ink = if silent { look.text } else { look.on_accent };
                icons::paint(
                    &painter,
                    speaker.shrink(7.0 - 1.5 * speaker_hover),
                    glyph,
                    ink,
                );
                let covered = fill.right() > well.right() - 52.0;
                painter.text(
                    pos2(well.right() - 20.0, well.center().y),
                    Align2::RIGHT_CENTER,
                    percent.to_string(),
                    ui::heading(14.0),
                    if covered && !silent {
                        look.on_accent
                    } else {
                        look.text
                    },
                );
            }
            Direction::B => {
                painter.circle_filled(
                    speaker.center(),
                    18.0,
                    Color32::from_white_alpha((22.0 + 24.0 * speaker_hover) as u8),
                );
                icons::paint(&painter, speaker.shrink(8.0), glyph, look.text);
                let count = 20usize;
                let gap = 4.0;
                let seg_w = (track.width() - gap * (count as f32 - 1.0)) / count as f32;
                let lit = (shown * count as f32).ceil() as usize;
                for i in 0..count {
                    let x = track.left() + i as f32 * (seg_w + gap);
                    let tip = i + 1 == lit && lit > 0;
                    let h = if tip { 34.0 + 6.0 * held } else { 28.0 };
                    let seg = Rect::from_center_size(
                        pos2(x + seg_w / 2.0, track.center().y),
                        vec2(seg_w, h),
                    );
                    painter.rect_filled(
                        seg,
                        CornerRadius::same(3),
                        if i < lit {
                            look.accent
                        } else {
                            Color32::from_white_alpha(30)
                        },
                    );
                }
                painter.text(
                    pos2(well.right() - 4.0, well.center().y),
                    Align2::RIGHT_CENTER,
                    percent.to_string(),
                    ui::heading(14.0),
                    look.muted,
                );
            }
            Direction::C => {
                painter.circle_filled(
                    speaker.center(),
                    18.0,
                    Color32::from_white_alpha((22.0 + 24.0 * speaker_hover) as u8),
                );
                icons::paint(&painter, speaker.shrink(8.0), glyph, look.text);
                painter.text(
                    pos2(well.right() - 4.0, well.top() + 20.0),
                    Align2::RIGHT_CENTER,
                    percent.to_string(),
                    ui::heading(30.0),
                    look.text,
                );
                let bar = Rect::from_center_size(
                    pos2(track.center().x, track.center().y),
                    vec2(track.width(), 6.0),
                );
                painter.rect_filled(bar, CornerRadius::same(3), Color32::from_white_alpha(34));
                let knob_x = track.left() + inset + (track.width() - 2.0 * inset) * shown;
                let mut filled = bar;
                filled.set_right(knob_x);
                painter.rect_filled(filled, CornerRadius::same(3), look.accent);
                let knob = pos2(knob_x, track.center().y);
                painter.circle_filled(
                    knob + vec2(0.0, 3.0),
                    13.5 + 3.0 * held,
                    Color32::from_black_alpha(70),
                );
                painter.circle_filled(knob, 13.0 + 1.5 * held, Color32::WHITE);
            }
        }
    }

    fn close(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

/// One toggle. The three directions share the behaviour (a click flips it, focus shows a ring,
/// press shrinks it) and differ in how it is drawn; see `design.rs` for what `Look` carries.
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

    // Entrance: rises 14 points while fading in; the spring settles the size.
    let rise = design::out_expo(look.enter);
    let spring = design::out_back(look.enter);
    let mut painter = ui.painter().clone();
    painter.set_opacity(look.opacity * rise);
    let mut rect = rect.translate(vec2(0.0, (1.0 - rise) * 14.0));
    let grow = 0.94 + 0.06 * spring;
    rect = Rect::from_center_size(rect.center(), rect.size() * grow);
    if pressed {
        rect = rect.shrink(2.0);
    }
    let ink = blend(look.text, look.on_accent, active);
    let dim = blend(look.muted, look.on_accent.gamma_multiply(0.85), active);
    let ring = |painter: &egui::Painter, around: Rect, radius: u8| {
        if response.has_focus() {
            painter.rect_stroke(
                around.expand(2.0),
                CornerRadius::same(radius + 2),
                Stroke::new(2.0, look.accent_ink),
                StrokeKind::Outside,
            );
        }
    };
    match look.dir {
        Direction::A => {
            let corner = CornerRadius::same(20);
            let idle = blend(
                Color32::from_white_alpha(22),
                Color32::from_white_alpha(38),
                hover,
            );
            design::drop_shadow(&painter, rect, 20, 14, 5, (50.0 * active) as u8);
            painter.rect_filled(rect, corner, blend(idle, look.accent, active));
            // A lit upper edge: the tile reads as a thick piece of glass.
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(
                    1.0,
                    blend(
                        Color32::from_white_alpha(34),
                        Color32::from_white_alpha(80),
                        active,
                    ),
                ),
                StrokeKind::Inside,
            );
            ring(&painter, rect, 20);
            let badge = Rect::from_center_size(
                pos2(rect.left() + 12.0 + 18.0, rect.center().y),
                vec2(36.0, 36.0),
            );
            painter.circle_filled(
                badge.center(),
                18.0,
                blend(
                    Color32::from_white_alpha(28),
                    look.on_accent.gamma_multiply(0.16),
                    active,
                ),
            );
            icons::paint(&painter, badge.shrink(8.0), icon, ink);
            let text_x = badge.right() + 10.0;
            painter.text(
                pos2(text_x, rect.center().y - 9.0),
                Align2::LEFT_CENTER,
                label,
                ui::heading(13.5),
                ink,
            );
            painter.text(
                pos2(text_x, rect.center().y + 10.0),
                Align2::LEFT_CENTER,
                sub,
                FontId::proportional(11.5),
                dim,
            );
        }
        Direction::B => {
            let corner = CornerRadius::same(22);
            let idle = blend(
                Color32::from_black_alpha(104),
                Color32::from_black_alpha(70),
                hover,
            );
            painter.rect_filled(rect, corner, blend(idle, look.accent, active));
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(
                    1.0,
                    blend(
                        Color32::from_white_alpha(30),
                        Color32::from_white_alpha(70),
                        active,
                    ),
                ),
                StrokeKind::Inside,
            );
            ring(&painter, rect, 22);
            let chip = Rect::from_min_size(rect.min + vec2(14.0, 14.0), vec2(46.0, 46.0));
            painter.rect_filled(
                chip,
                CornerRadius::same(15),
                blend(
                    Color32::from_white_alpha(26),
                    look.on_accent.gamma_multiply(0.16),
                    active,
                ),
            );
            icons::paint(&painter, chip.shrink(11.0), icon, ink);
            painter.circle_filled(
                pos2(rect.right() - 18.0, rect.top() + 20.0),
                4.5,
                blend(Color32::from_white_alpha(60), look.on_accent, active),
            );
            painter.text(
                pos2(rect.left() + 16.0, rect.bottom() - 32.0),
                Align2::LEFT_CENTER,
                label,
                ui::heading(14.0),
                ink,
            );
            painter.text(
                pos2(rect.left() + 16.0, rect.bottom() - 15.0),
                Align2::LEFT_CENTER,
                sub,
                FontId::proportional(12.0),
                dim,
            );
        }
        Direction::C => {
            let wide = rect.width() > 220.0;
            if hover > 0.0 || wide {
                painter.rect_filled(
                    rect,
                    CornerRadius::same(26),
                    Color32::from_white_alpha(if wide {
                        (18.0 + 10.0 * hover) as u8
                    } else {
                        (14.0 * hover) as u8
                    }),
                );
            }
            ring(&painter, rect, 26);
            let (center, text_pos, align) = if wide {
                (
                    pos2(rect.left() + 14.0 + 26.0, rect.center().y),
                    pos2(rect.left() + 14.0 + 52.0 + 12.0, rect.center().y),
                    Align2::LEFT_CENTER,
                )
            } else {
                (
                    pos2(rect.center().x, rect.top() + 12.0 + 26.0),
                    pos2(rect.center().x, rect.top() + 12.0 + 52.0 + 14.0),
                    Align2::CENTER_CENTER,
                )
            };
            if active > 0.0 {
                painter.circle_filled(center, 32.0, look.accent.gamma_multiply(0.22 * active));
            }
            painter.circle_filled(
                center,
                26.0,
                blend(Color32::from_white_alpha(34), look.accent, active),
            );
            painter.circle_stroke(
                center,
                25.5,
                Stroke::new(1.0, Color32::from_white_alpha(if on { 90 } else { 40 })),
            );
            icons::paint(
                &painter,
                Rect::from_center_size(center, vec2(26.0, 26.0)),
                icon,
                blend(look.text, look.on_accent, active),
            );
            let (label_pos, sub_pos) = if wide {
                (text_pos - vec2(0.0, 9.0), text_pos + vec2(0.0, 10.0))
            } else {
                (text_pos, text_pos + vec2(0.0, 16.0))
            };
            painter.text(label_pos, align, label, ui::heading(13.0), look.text);
            painter.text(sub_pos, align, sub, FontId::proportional(11.5), look.muted);
        }
    }
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
        // The panel is always dark glass over the blurred screen, whatever the app theme is.
        let dark = true;
        let p = Palette::for_mode(true);
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
        if !self.region_done {
            self.region_done = true;
            let ppp = ctx.pixels_per_point();
            crate::chrome::round_window(
                "Whirlpool Quick Settings",
                (
                    0,
                    0,
                    (full.width() * ppp) as i32,
                    (full.height() * ppp) as i32,
                ),
                (f32::from(RADIUS) * ppp) as i32,
            );
        }
        let radius = CornerRadius::same(RADIUS);
        let painter = ui.painter().clone();
        self.backdrop.paint(
            &painter,
            full,
            Color32::from_rgba_unmultiplied(8, 20, 32, 150),
            look.surface,
        );
        // Depth: a lighter top fading into the water below.
        design::vertical_gradient(
            &painter,
            full,
            Color32::from_white_alpha(18),
            Color32::from_rgba_unmultiplied(2, 10, 18, 70),
        );
        painter.rect_stroke(
            full.shrink(0.5),
            radius,
            Stroke::new(1.0, Color32::from_white_alpha(46)),
            StrokeKind::Inside,
        );
        // A bright hairline along the top edge reads as the rim of a glass slab.
        painter.line_segment(
            [
                pos2(full.left() + f32::from(RADIUS), full.top() + 1.5),
                pos2(full.right() - f32::from(RADIUS), full.top() + 1.5),
            ],
            Stroke::new(1.0, Color32::from_white_alpha(70)),
        );
        let look = Look {
            opacity: appear,
            ..look
        };
        let elapsed = if reduced {
            1.0
        } else {
            self.started.elapsed().as_secs_f32()
        };
        if !reduced && design::entrance_running(elapsed) {
            ctx.request_repaint();
        }

        // Toggles: the user's tile grid, two columns, wide tiles span both.
        let tile_w = ((full.width() - 2.0 * PAD - GAP) / 2.0).floor();
        let tile_h = tile_h(look.dir);
        let saver = self.battery.is_some_and(|b| b.saver);
        let all_off = self.radios.wifi != Some(true)
            && self.radios.bluetooth != Some(true)
            && (self.radios.wifi.is_some() || self.radios.bluetooth.is_some());
        let spans: Vec<usize> = self.tiles.iter().map(|t| t.span()).collect();
        for slot in tiles::pack(&spans) {
            let kind = self.tiles[slot.index].kind;
            let (x, y, w, h) = tiles::slot_rect(&slot, tile_w, tile_h, GAP);
            let rect = Rect::from_min_size(full.min + vec2(PAD + x, PAD + y), vec2(w, h));
            // Tiles arrive in order, 40 ms apart.
            let item = Look {
                enter: design::stagger(elapsed, slot.index, 0.04, 0.32),
                ..look
            };
            self.tile_slot(ui, &item, rect, kind, saver, all_off, &ctx);
        }
        let grid_h = grid_height(tiles::rows(&spans), look.dir);

        // Volume.
        let well = Rect::from_min_size(
            pos2(full.left() + PAD, full.top() + PAD + grid_h + 14.0),
            vec2(full.width() - 2.0 * PAD, well_h(look.dir)),
        );
        self.volume_row(ui, &look, well, &ctx);

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
    let tiles_list = whirlpool_config::load(&whirlpool_config::default_path())
        .ok()
        .flatten()
        .map_or_else(TileSpec::defaults, |c| c.sanitized().quick_tiles);
    let spans: Vec<usize> = tiles_list.iter().map(|t| t.span()).collect();
    let dir = Direction::from_env("WHIRLPOOL_QUICK_DESIGN", DEFAULT_DIRECTION);
    let height = panel_height(tiles::rows(&spans), dir);
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
        anchor_y as f32 / scale - height - EDGE_GAP
    } else {
        anchor_y as f32 / scale + EDGE_GAP
    }
    .clamp(
        top_limit + 4.0,
        (bottom_limit - height - 4.0).max(top_limit + 4.0),
    );
    // The screen behind the panel, taken before it opens so it can be shown blurred.
    let shot = chrome::capture(
        (x * scale).round() as i32,
        (y * scale).round() as i32,
        (WIDTH * scale).round() as i32,
        (height * scale).round() as i32,
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Whirlpool Quick Settings")
            .with_decorations(false)
            .with_transparent(true)
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
        "Whirlpool Quick Settings",
        options,
        Box::new(move |creation| {
            ui::install_theme(&creation.egui_ctx);
            let backdrop = Backdrop::new(&creation.egui_ctx, shot, 2, 6);
            Ok(Box::new(Quick::new(
                &creation.egui_ctx,
                tiles_list,
                backdrop,
            )))
        }),
    )
}
