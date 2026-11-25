//! HUDs for volume, brightness, Wi-Fi and Bluetooth.
//!
//! A small dark glass pill appears above the work area, holds for a moment and
//! fades. Volume and brightness show a slider: a gradient fill that the
//! compositor wipes to the new level, a knob that follows it and the number.
//! Wi-Fi shows the four signal steps, Bluetooth a switch. There is no text, so
//! nothing needs translating.
//!
//! Cost: nothing while it is away. A handful of small bitmaps are drawn when it
//! appears and again when the value changes while it is still up, the
//! compositor runs every keyframe, and the click-through window is destroyed
//! when it is done. The dock only checks, every 60 ms of those two seconds,
//! whether it has finished.

use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use whirlpool_shell::icons::IconImage;
use windows::{
    UI::Composition::{ContainerVisual, Desktop::DesktopWindowTarget},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WinRT::Composition::ICompositorDesktopInterop},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SW_SHOWNOACTIVATE,
            ShowWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
            WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
        },
    },
    core::{HSTRING, Interface, h, w},
};
use windows_numerics::Vector3;

use crate::{
    charge::{Rgba, ease_out, mix3, scalar, smooth, sprite, to_image, uniform, vector},
    gfx::Gfx,
};

/// Seconds the pill stays fully visible after the last change.
const HOLD: f32 = 1.7;
/// Seconds of the fade that follows.
const FADE: f32 = 0.35;
/// Seconds the fill and the knob take to reach a new level.
const GLIDE: f32 = 0.26;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Volume {
        percent: u8,
        muted: bool,
    },
    Brightness {
        percent: u8,
    },
    /// Signal steps 1 to 4; 0 is offline.
    Wifi {
        bars: u8,
    },
    Bluetooth {
        on: bool,
    },
}

impl Kind {
    /// 0 to 1 for the kinds that have a slider.
    fn fraction(self) -> Option<f32> {
        match self {
            Self::Volume { percent, .. } | Self::Brightness { percent } => {
                Some(f32::from(percent.min(100)) / 100.0)
            }
            _ => None,
        }
    }

    fn percent(self) -> u8 {
        match self {
            Self::Volume { percent, .. } | Self::Brightness { percent } => percent.min(100),
            _ => 0,
        }
    }

    fn muted(self) -> bool {
        matches!(self, Self::Volume { muted: true, .. })
    }

    /// Same pill shape and size, so the window can stay and only the picture changes.
    fn same_layout(self, other: Self) -> bool {
        self.fraction().is_some() == other.fraction().is_some()
    }
}

/// Speaker steps (0 to 3) for a volume in percent.
pub fn speaker_steps(percent: u8) -> u8 {
    match percent {
        0 => 0,
        1..=33 => 1,
        34..=66 => 2,
        _ => 3,
    }
}

/// Pixel sizes of one HUD at one scale.
#[derive(Clone, Copy)]
struct Geometry {
    s: f32,
    /// Bitmap (and window) size, shadow margin included.
    width: f32,
    height: f32,
    pad: f32,
    /// Pill size.
    pill_w: f32,
    pill_h: f32,
    slider: bool,
}

impl Geometry {
    fn new(kind: Kind, scale: f32) -> Self {
        let slider = kind.fraction().is_some();
        let s = scale;
        let pad = 22.0 * s;
        let pill_w = if slider { 300.0 * s } else { 196.0 * s };
        let pill_h = 56.0 * s;
        Self {
            s,
            width: (pill_w + 2.0 * pad).round(),
            height: (pill_h + 2.0 * pad).round(),
            pad,
            pill_w,
            pill_h,
            slider,
        }
    }

    fn cy(self) -> f32 {
        self.height / 2.0
    }

    fn icon_x(self) -> f32 {
        self.pad + if self.slider { 32.0 } else { 40.0 } * self.s
    }

    fn num_w(self) -> f32 {
        46.0 * self.s
    }

    fn num_h(self) -> f32 {
        18.0 * self.s
    }

    fn num_x(self) -> f32 {
        self.pad + self.pill_w - 18.0 * self.s - self.num_w()
    }

    fn track_x0(self) -> f32 {
        self.pad + 62.0 * self.s
    }

    fn track_x1(self) -> f32 {
        self.num_x() - 14.0 * self.s
    }

    fn track_len(self) -> f32 {
        self.track_x1() - self.track_x0()
    }

    fn track_h(self) -> f32 {
        8.0 * self.s
    }

    fn knob_r(self) -> f32 {
        9.5 * self.s
    }
}

/* -------------------------------------------------------------- Drawing */

const ACCENT_A: [f32; 3] = [0.36, 0.74, 1.0];
const ACCENT_B: [f32; 3] = [0.82, 0.95, 1.0];
const MUTED: [f32; 3] = [0.60, 0.64, 0.70];

/// Signed distance to a pill (a rectangle with fully rounded ends) centred at
/// (`cx`, `cy`); negative inside.
fn pill_sd(x: f32, y: f32, cx: f32, cy: f32, w: f32, h: f32) -> f32 {
    let r = h / 2.0;
    let qx = (x - cx).abs() - (w / 2.0 - r);
    let dy = y - cy;
    if qx > 0.0 {
        (qx * qx + dy * dy).sqrt() - r
    } else {
        dy.abs() - r
    }
}

fn seg_dist(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((px - a.0) * abx + (py - a.1) * aby) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (a.0 + t * abx, a.1 + t * aby);
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

fn in_convex(u: f32, v: f32, points: &[(f32, f32)]) -> bool {
    let mut sign = 0.0f32;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        let cross = (b.0 - a.0) * (v - a.1) - (b.1 - a.1) * (u - a.0);
        if cross != 0.0 {
            if sign == 0.0 {
                sign = cross.signum();
            } else if sign != cross.signum() {
                return false;
            }
        }
    }
    true
}

/// A ring segment: distance from the centre between `r - t` and `r + t`.
fn ring(u: f32, v: f32, centre: (f32, f32), r: f32, t: f32) -> bool {
    ((u - centre.0).powi(2) + (v - centre.1).powi(2)).sqrt() > r - t
        && ((u - centre.0).powi(2) + (v - centre.1).powi(2)).sqrt() < r + t
}

/// Icon coverage (0 or 1; the supersampling softens it) at (`u`, `v`) in logical
/// pixels from the icon's centre, as (bright part, dim part).
fn icon(kind: Kind, u: f32, v: f32) -> (bool, bool) {
    match kind {
        Kind::Volume { percent, muted } => {
            let (u, v) = (u + 3.0, v);
            let body = (-9.0..=-5.0).contains(&u) && (-3.2..=3.2).contains(&v);
            let cone = in_convex(u, v, &[(-5.5, -3.2), (0.5, -8.0), (0.5, 8.0), (-5.5, 3.2)]);
            let mut bright = body || cone;
            let mut dim = false;
            let steps = if muted { 0 } else { speaker_steps(percent) };
            let (du, dv) = (u - 1.0, v);
            for (index, radius) in [5.2f32, 9.2, 13.2].into_iter().enumerate() {
                if du > 0.0 && dv.abs() <= du * 1.15 && ring(u, v, (1.0, 0.0), radius, 1.0) {
                    if (index as u8) < steps {
                        bright = true;
                    } else if !muted {
                        dim = true;
                    }
                }
            }
            if muted {
                let cross = seg_dist(u, v, (6.0, -4.5), (13.0, 4.5)).min(seg_dist(
                    u,
                    v,
                    (6.0, 4.5),
                    (13.0, -4.5),
                ));
                bright |= cross < 1.0;
            }
            (bright, dim)
        }
        Kind::Brightness { percent } => {
            let d = (u * u + v * v).sqrt();
            let disc = 3.0 + 2.4 * f32::from(percent.min(100)) / 100.0;
            let mut bright = d < disc;
            if (7.0..=10.5).contains(&d) {
                let a = v.atan2(u);
                let step = PI / 4.0;
                let diff = a - (a / step).round() * step;
                bright |= (d * diff.sin()).abs() < 0.95;
            }
            (bright, false)
        }
        Kind::Wifi { bars } => {
            let (u, v) = (u, v - 7.0);
            let lit = |index: u8| bars > index;
            let mut bright = false;
            let mut dim = false;
            if (u * u + v * v).sqrt() < 1.8 {
                if bars >= 1 {
                    bright = true;
                } else {
                    dim = true;
                }
            }
            for (index, radius) in [4.6f32, 8.6, 12.6].into_iter().enumerate() {
                if -v > u.abs() && ring(u, v, (0.0, 0.0), radius, 1.05) {
                    if lit(index as u8 + 1) {
                        bright = true;
                    } else {
                        dim = true;
                    }
                }
            }
            if bars == 0 {
                bright = false;
                dim |= seg_dist(u, v + 7.0, (-9.0, -9.0), (9.0, 9.0)) < 1.0;
            }
            (bright, dim)
        }
        Kind::Bluetooth { on } => {
            let lines = [
                ((-5.0, -5.0), (5.0, 5.0)),
                ((5.0, 5.0), (0.0, 10.0)),
                ((0.0, 10.0), (0.0, -10.0)),
                ((0.0, -10.0), (5.0, -5.0)),
                ((5.0, -5.0), (-5.0, 5.0)),
            ];
            let mut hit = lines.iter().any(|&(a, b)| seg_dist(u, v, a, b) < 1.0);
            if !on {
                hit |= seg_dist(u, v, (-9.0, -9.0), (9.0, 9.0)) < 1.0;
            }
            if on { (hit, false) } else { (false, hit) }
        }
    }
}

fn over(out: &mut [f32; 4], colour: [f32; 3], alpha: f32) {
    for channel in 0..3 {
        out[channel] = colour[channel] * alpha + out[channel] * (1.0 - alpha);
    }
    out[3] = alpha + out[3] * (1.0 - alpha);
}

/// The static picture: shadow, glass, icon and the groove or indicator.
fn body_at(kind: Kind, g: Geometry, x: f32, y: f32) -> Rgba {
    let mut out = [0.0f32; 4];
    let s = g.s;
    let (cx, cy) = (g.width / 2.0, g.cy());
    let sd = pill_sd(x, y, cx, cy, g.pill_w, g.pill_h);
    // Soft shadow, a little lower than the glass.
    let shadow = pill_sd(x, y - 7.0 * s, cx, cy, g.pill_w, g.pill_h);
    over(
        &mut out,
        [0.0, 0.0, 0.0],
        0.36 * smooth(18.0 * s, -4.0 * s, shadow),
    );
    let inside = smooth(0.75, -0.75, sd);
    if inside <= 0.0 {
        return out;
    }
    let top = ((y - (cy - g.pill_h / 2.0)) / g.pill_h).clamp(0.0, 1.0);
    let glass = mix3([0.13, 0.15, 0.19], [0.05, 0.06, 0.08], top);
    over(&mut out, glass, 0.90 * inside);
    // A bright hairline at the rim, strongest along the top.
    let rim = smooth(2.2 * s, 0.4 * s, -sd) * smooth(-0.3, 0.9, -sd);
    over(
        &mut out,
        [1.0, 1.0, 1.0],
        (0.10 + 0.14 * (1.0 - top)) * rim * inside,
    );

    // Icon, 1 logical pixel = `s` pixels.
    let (u, v) = ((x - g.icon_x()) / s, (y - cy) / s);
    let (bright, dim) = icon(kind, u, v);
    if dim {
        over(&mut out, [1.0, 1.0, 1.0], 0.28);
    }
    if bright {
        over(&mut out, [0.97, 0.98, 1.0], 0.97);
    }

    if g.slider {
        let tsd = pill_sd(
            x,
            y,
            (g.track_x0() + g.track_x1()) / 2.0,
            cy,
            g.track_len(),
            g.track_h(),
        );
        over(&mut out, [1.0, 1.0, 1.0], 0.16 * smooth(0.75, -0.75, tsd));
    } else {
        match kind {
            Kind::Wifi { bars } => {
                let x0 = g.pad + 96.0 * s;
                for index in 0..4u8 {
                    let height = (7.0 + 5.0 * f32::from(index)) * s;
                    let left = x0 + f32::from(index) * 9.5 * s;
                    let (bx, by) = (left + 2.75 * s, cy + 11.0 * s - height / 2.0);
                    // A vertical pill: the distance with the axes swapped.
                    let vertical = {
                        let r = 2.75 * s;
                        let qy = (y - by).abs() - (height / 2.0 - r);
                        let dx = x - bx;
                        if qy > 0.0 {
                            (qy * qy + dx * dx).sqrt() - r
                        } else {
                            dx.abs() - r
                        }
                    };
                    let cover = smooth(0.75, -0.75, vertical);
                    if bars > index {
                        over(
                            &mut out,
                            mix3(ACCENT_A, ACCENT_B, f32::from(index) / 3.0),
                            cover,
                        );
                    } else {
                        over(&mut out, [1.0, 1.0, 1.0], 0.20 * cover);
                    }
                }
            }
            Kind::Bluetooth { on } => {
                let (tw, th) = (46.0 * s, 26.0 * s);
                let tcx = g.pad + 96.0 * s + tw / 2.0;
                let sd = pill_sd(x, y, tcx, cy, tw, th);
                let cover = smooth(0.75, -0.75, sd);
                if on {
                    let t = ((x - (tcx - tw / 2.0)) / tw).clamp(0.0, 1.0);
                    over(&mut out, mix3(ACCENT_A, ACCENT_B, t), cover);
                } else {
                    over(&mut out, [1.0, 1.0, 1.0], 0.18 * cover);
                }
                let kr = 9.5 * s;
                let kx = if on {
                    tcx + tw / 2.0 - 3.5 * s - kr
                } else {
                    tcx - tw / 2.0 + 3.5 * s + kr
                };
                let d = ((x - kx).powi(2) + (y - cy).powi(2)).sqrt();
                over(
                    &mut out,
                    [0.0, 0.0, 0.0],
                    0.25 * smooth(kr + 3.0 * s, kr - 1.0, d),
                );
                over(
                    &mut out,
                    [1.0, 1.0, 1.0],
                    0.98 * smooth(kr + 0.7, kr - 0.7, d),
                );
            }
            _ => {}
        }
    }
    out
}

fn supersampled(w: u32, h: u32, mut at: impl FnMut(f32, f32) -> Rgba) -> IconImage {
    const N: u32 = 3;
    to_image(w, h, |x, y| {
        let mut sum = [0.0f32; 4];
        for sy in 0..N {
            for sx in 0..N {
                let sample = at(
                    x as f32 + (sx as f32 + 0.5) / N as f32,
                    y as f32 + (sy as f32 + 0.5) / N as f32,
                );
                for (total, value) in sum.iter_mut().zip(sample) {
                    *total += value;
                }
            }
        }
        let n = (N * N) as f32;
        [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n]
    })
}

fn body_image(kind: Kind, g: Geometry) -> IconImage {
    supersampled(g.width as u32, g.height as u32, |x, y| {
        body_at(kind, g, x, y)
    })
}

/// The bright fill, full length; the compositor clips it to the level.
fn fill_image(kind: Kind, g: Geometry) -> IconImage {
    let (w, h) = (g.track_len().round() as u32, g.track_h().round() as u32);
    let muted = kind.muted();
    supersampled(w, h, |x, y| {
        let mut out = [0.0f32; 4];
        let sd = pill_sd(x, y, w as f32 / 2.0, h as f32 / 2.0, w as f32, h as f32);
        let colour = if muted {
            MUTED
        } else {
            mix3(ACCENT_A, ACCENT_B, x / w as f32)
        };
        over(&mut out, colour, smooth(0.75, -0.75, sd));
        out
    })
}

fn knob_image(kind: Kind, g: Geometry) -> IconImage {
    let r = g.knob_r();
    let side = ((r + 5.0 * g.s) * 2.0).round() as u32;
    let c = side as f32 / 2.0;
    let colour = if kind.muted() {
        [0.86, 0.88, 0.91]
    } else {
        [1.0, 1.0, 1.0]
    };
    supersampled(side, side, |x, y| {
        let mut out = [0.0f32; 4];
        let d = ((x - c).powi(2) + (y - c - 1.5 * g.s).powi(2)).sqrt();
        over(
            &mut out,
            [0.0, 0.0, 0.0],
            0.30 * smooth(r + 4.0 * g.s, r - 2.0, d),
        );
        let d = ((x - c).powi(2) + (y - c).powi(2)).sqrt();
        over(&mut out, colour, smooth(r + 0.7, r - 0.7, d));
        out
    })
}

/// 3 x 5 digits, one row per entry, `#` for ink.
fn digit(character: char) -> [&'static str; 5] {
    match character {
        '0' => ["###", "# #", "# #", "# #", "###"],
        '1' => [" # ", "## ", " # ", " # ", "###"],
        '2' => ["###", "  #", "###", "#  ", "###"],
        '3' => ["###", "  #", "###", "  #", "###"],
        '4' => ["# #", "# #", "###", "  #", "  #"],
        '5' => ["###", "#  ", "###", "  #", "###"],
        '6' => ["###", "#  ", "###", "# #", "###"],
        '7' => ["###", "  #", "  #", "  #", "  #"],
        '8' => ["###", "# #", "###", "# #", "###"],
        _ => ["###", "# #", "###", "  #", "###"],
    }
}

fn number_image(kind: Kind, g: Geometry) -> IconImage {
    let text: Vec<char> = kind.percent().to_string().chars().collect();
    let (w, h) = (g.num_w().round() as u32, g.num_h().round() as u32);
    let cell = 3.0 * g.s;
    let columns = text.len() as f32 * 4.0 - 1.0;
    let left = w as f32 - columns * cell;
    let top = (h as f32 - 5.0 * cell) / 2.0;
    let alpha = if kind.muted() { 0.50 } else { 0.94 };
    supersampled(w, h, |x, y| {
        let mut out = [0.0f32; 4];
        let (gx, gy) = (((x - left) / cell).floor(), ((y - top) / cell).floor());
        if (0.0..columns).contains(&gx) && (0.0..5.0).contains(&gy) {
            let (index, column) = ((gx as usize) / 4, (gx as usize) % 4);
            if column < 3
                && let Some(&character) = text.get(index)
                && digit(character)[gy as usize].as_bytes()[column] == b'#'
            {
                over(&mut out, [0.96, 0.98, 1.0], alpha);
            }
        }
        out
    })
}

/* ----------------------------------------------------------------- Scene */

fn build_scene(
    gfx: &Gfx,
    kind: Kind,
    g: Geometry,
    from: f32,
    first: bool,
    animate: bool,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let out = ease_out(c)?;
    let linear: windows::UI::Composition::CompositionEasingFunction =
        c.CreateLinearEasingFunction()?.cast()?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(windows_numerics::Vector2::new(1.0, 1.0))?;
    let group = c.CreateContainerVisual()?;
    group.SetSize(windows_numerics::Vector2::new(g.width, g.height))?;
    group.SetCenterPoint(Vector3::new(g.width / 2.0, g.height / 2.0, 0.0))?;
    let body = sprite(gfx, &body_image(kind, g))?;
    group.Children()?.InsertAtTop(&body)?;

    if let Some(to) = kind.fraction() {
        let delay = if first && animate { 0.10 } else { 0.0 };
        let glide = animate && (first || (from - to).abs() > 0.004);
        let from = if first { to } else { from };
        let (x0, len) = (g.track_x0(), g.track_len());
        let top = g.cy() - g.track_h() / 2.0;

        let fill = sprite(gfx, &fill_image(kind, g))?;
        fill.SetOffset(Vector3::new(x0, top, 0.0))?;
        let clip = c.CreateInsetClip()?;
        clip.SetRightInset(len * (1.0 - to))?;
        if glide && from != to {
            clip.StartAnimation(
                h!("RightInset"),
                &scalar(
                    c,
                    &[(0.0, len * (1.0 - from)), (GLIDE, len * (1.0 - to))],
                    delay,
                    &out,
                )?,
            )?;
        }
        fill.SetClip(&clip)?;
        group.Children()?.InsertAtTop(&fill)?;

        let knob = sprite(gfx, &knob_image(kind, g))?;
        let half = knob.Size()?.X / 2.0;
        let at = |fraction: f32| Vector3::new(x0 + len * fraction - half, g.cy() - half, 0.0);
        knob.SetOffset(at(to))?;
        if glide && from != to {
            knob.StartAnimation(
                h!("Offset"),
                &vector(c, &[(0.0, at(from)), (GLIDE, at(to))], delay, &out)?,
            )?;
        }
        group.Children()?.InsertAtTop(&knob)?;

        let number = sprite(gfx, &number_image(kind, g))?;
        number.SetOffset(Vector3::new(g.num_x(), g.cy() - g.num_h() / 2.0, 0.0))?;
        group.Children()?.InsertAtTop(&number)?;
    }

    if animate {
        let opacity: &[(f32, f32)] = if first {
            &[(0.0, 0.0), (0.2, 1.0), (HOLD, 1.0), (HOLD + FADE, 0.0)]
        } else {
            &[(0.0, 1.0), (HOLD, 1.0), (HOLD + FADE, 0.0)]
        };
        group.StartAnimation(h!("Opacity"), &scalar(c, opacity, 0.0, &linear)?)?;
        if first {
            let rest = Vector3::new(0.0, 0.0, 0.0);
            let below = Vector3::new(0.0, 16.0 * g.s, 0.0);
            group.SetOffset(below)?;
            group.StartAnimation(
                h!("Offset"),
                &vector(c, &[(0.0, below), (0.4, rest)], 0.0, &out)?,
            )?;
            group.StartAnimation(
                h!("Scale"),
                &vector(c, &[(0.0, uniform(0.92)), (0.4, uniform(1.0))], 0.0, &out)?,
            )?;
        }
    }
    root.Children()?.InsertAtTop(&group)?;
    Ok(root)
}

/* ------------------------------------------------------------------ Hud */

pub struct Hud {
    hwnd: HWND,
    target: DesktopWindowTarget,
    kind: Kind,
    until: Instant,
    geometry: Geometry,
    animate: bool,
}

impl Hud {
    /// Shows `kind`; `None` (and a line in the log) if it could not be built.
    pub fn show(gfx: &Gfx, kind: Kind, animate: bool) -> Option<Self> {
        match Self::build(gfx, kind, animate) {
            Ok(hud) => Some(hud),
            Err(error) => {
                whirlpool_shell::log::line(&format!("hud: {error:#}"));
                None
            }
        }
    }

    pub fn finished(&self) -> bool {
        Instant::now() >= self.until
    }

    /// Whether `kind` can replace the picture without a new window.
    pub fn can_update(&self, kind: Kind) -> bool {
        self.kind.same_layout(kind)
    }

    /// Shows a new value in the same window: the fill glides from where it was.
    pub fn update(&mut self, gfx: &Gfx, kind: Kind) {
        let from = self.kind.fraction().unwrap_or(0.0);
        match build_scene(gfx, kind, self.geometry, from, false, self.animate) {
            Ok(root) => {
                if self.target.SetRoot(&root).is_ok() {
                    self.kind = kind;
                    self.until = deadline();
                }
            }
            Err(error) => whirlpool_shell::log::line(&format!("hud: {error:#}")),
        }
    }

    fn build(gfx: &Gfx, kind: Kind, animate: bool) -> Result<Self> {
        let monitor = whirlpool_shell::system::primary_monitor().context("no monitor")?;
        let g = Geometry::new(kind, monitor.scale.max(1.0));
        let (width, height) = (g.width as i32, g.height as i32);
        let left = (monitor.work_left + monitor.work_right) / 2 - width / 2;
        let bottom = monitor.work_bottom - (28.0 * g.s) as i32 + g.pad as i32;
        let rect = RECT {
            left,
            top: bottom - height,
            right: left + width,
            bottom,
        };
        let hwnd = create_window(rect)?;
        let built = || -> Result<DesktopWindowTarget> {
            // SAFETY: `hwnd` is the live window created above.
            let target = unsafe {
                gfx.compositor
                    .cast::<ICompositorDesktopInterop>()?
                    .CreateDesktopWindowTarget(hwnd, true)?
            };
            target.SetRoot(&build_scene(gfx, kind, g, 0.0, true, animate)?)?;
            Ok(target)
        };
        let target = match built() {
            Ok(target) => target,
            Err(error) => {
                // SAFETY: Our own window, created above.
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return Err(error);
            }
        };
        // SAFETY: Our own window; shown without taking the focus.
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        Ok(Self {
            hwnd,
            target,
            kind,
            until: deadline(),
            geometry: g,
            animate,
        })
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs_f32(HOLD + FADE + 0.05)
}

impl Drop for Hud {
    fn drop(&mut self) {
        // SAFETY: Our own window, destroyed once.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

fn create_window(rect: RECT) -> Result<HWND> {
    static CLASS: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    let class = w!("WhirlpoolHud");
    // SAFETY: The class is registered once; the window is destroyed by
    // `Hud::drop`, or by the caller if building the scene fails.
    unsafe {
        let instance = GetModuleHandleW(None)?;
        CLASS.get_or_init(|| {
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(hud_proc),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            });
        });
        // Click-through: the HUD never takes a click from what is under it.
        Ok(CreateWindowExW(
            WS_EX_LAYERED
                | WS_EX_TRANSPARENT
                | WS_EX_NOREDIRECTIONBITMAP
                | WS_EX_TOPMOST
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE,
            class,
            &HSTRING::new(),
            WS_POPUP,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            Some(instance.into()),
            None,
        )?)
    }
}

unsafe extern "system" fn hud_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: Default handling for our own window.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speaker_steps_follow_the_level() {
        assert_eq!(speaker_steps(0), 0);
        assert_eq!(speaker_steps(1), 1);
        assert_eq!(speaker_steps(50), 2);
        assert_eq!(speaker_steps(100), 3);
    }

    #[test]
    fn slider_kinds_have_a_fraction_and_the_others_do_not() {
        assert_eq!(
            Kind::Volume {
                percent: 50,
                muted: false
            }
            .fraction(),
            Some(0.5)
        );
        assert_eq!(Kind::Brightness { percent: 250 }.fraction(), Some(1.0));
        assert_eq!(Kind::Wifi { bars: 3 }.fraction(), None);
        assert!(
            Kind::Volume {
                percent: 1,
                muted: false
            }
            .same_layout(Kind::Brightness { percent: 9 })
        );
        assert!(
            !Kind::Volume {
                percent: 1,
                muted: false
            }
            .same_layout(Kind::Bluetooth { on: true })
        );
    }

    #[test]
    fn the_glass_is_opaque_in_the_middle_and_clear_at_the_corner() {
        let kind = Kind::Volume {
            percent: 40,
            muted: false,
        };
        let g = Geometry::new(kind, 1.0);
        let corner = body_at(kind, g, 1.0, 1.0);
        assert!(corner[3] < 0.01);
        let middle = body_at(kind, g, g.width / 2.0 + 20.0, g.cy() - 18.0);
        assert!(middle[3] > 0.8);
    }

    #[test]
    fn muting_changes_the_speaker_and_a_dead_wifi_is_dim() {
        let loud = Kind::Volume {
            percent: 80,
            muted: false,
        };
        let mute = Kind::Volume {
            percent: 80,
            muted: true,
        };
        let differing = (-14..=14)
            .flat_map(|v| (-14..=14).map(move |u| (u as f32, v as f32)))
            .filter(|&(u, v)| icon(loud, u, v) != icon(mute, u, v))
            .count();
        assert!(differing > 20);
        let (bright, dim) = (-14..=14)
            .flat_map(|v| (-14..=14).map(move |u| icon(Kind::Wifi { bars: 0 }, u as f32, v as f32)))
            .fold((0, 0), |(b, d), (x, y)| {
                (b + usize::from(x), d + usize::from(y))
            });
        assert_eq!(bright, 0);
        assert!(dim > 10);
    }

    #[test]
    fn the_track_fits_between_the_icon_and_the_number() {
        let g = Geometry::new(Kind::Brightness { percent: 10 }, 1.5);
        assert!(g.track_x0() > g.icon_x() + 20.0);
        assert!(g.track_len() > 100.0);
        assert!(g.track_x1() < g.num_x());
    }
}
