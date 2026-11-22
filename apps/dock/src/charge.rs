//! The charging animations. Four styles, one quiet default:
//!
//! - **Surge**: a small glass medallion drops in under the menu bar with the
//!   battery ring, a bolt and the level, while two thin rings of water spread.
//! - **Halo**: a slim glass capsule with a bolt, the level and a thin meter.
//! - **Edge**: the edges of the screen glow like liquid glass, green when the
//!   battery is healthy, yellow in the middle and orange when it is low.
//! - **Tide**: a line of light sweeps along the top edge up to the level.
//!
//! They play when the charger is plugged in, never at startup. The glows are
//! drawn at a quarter of the resolution and stretched by the compositor: they
//! are blurry on purpose, and a 4K screen costs a few hundred kilobytes.
//!
//! Cost: nothing while it is away. Three bitmaps are drawn once, the
//! compositor runs every keyframe, and the click-through window is destroyed
//! when the animation ends. The dock only checks, every 50 ms of those few
//! seconds, whether it has finished.

use std::{
    f32::consts::TAU,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use whirlpool_shell::icons::IconImage;
use windows::{
    Foundation::TimeSpan,
    UI::Composition::{
        CompositionBitmapInterpolationMode, CompositionEasingFunction, CompositionStretch,
        Compositor, ContainerVisual, Desktop::DesktopWindowTarget, ScalarKeyFrameAnimation,
        SpriteVisual, Vector3KeyFrameAnimation,
    },
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
use windows_numerics::{Vector2, Vector3};

use crate::gfx::Gfx;

use whirlpool_config::ChargingStyle;

pub struct Charge {
    hwnd: HWND,
    _target: DesktopWindowTarget,
    started: Instant,
    /// Seconds from the first frame to the window's destruction.
    total: f32,
}

impl Charge {
    /// Plays the animation of `style` for a battery at `percent`.
    pub fn play(gfx: &Gfx, percent: u8, style: ChargingStyle) -> Option<Self> {
        match Self::build(gfx, percent, style) {
            Ok(charge) => Some(charge),
            Err(error) => {
                whirlpool_shell::log::line(&format!("carga: {error:#}"));
                None
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.started.elapsed() >= Duration::from_secs_f32(self.total)
    }

    fn build(gfx: &Gfx, percent: u8, style: ChargingStyle) -> Result<Self> {
        let monitor = whirlpool_shell::system::primary_monitor().context("sin monitor")?;
        let scale = monitor.scale.max(1.0);
        let centre_x = (monitor.work_left + monitor.work_right) / 2;
        let medal_px = (112.0 * scale).round() as u32;
        let ring_px = (medal_px as f32 * 2.3).round() as u32;
        let (rect, total) = match style {
            ChargingStyle::Edge => (monitor.screen, 3.6),
            ChargingStyle::Tide => (
                RECT {
                    bottom: monitor.screen.top + (64.0 * scale) as i32,
                    ..monitor.screen
                },
                3.4,
            ),
            ChargingStyle::Bloom => (
                RECT {
                    top: monitor.screen.bottom - (280.0 * scale) as i32,
                    ..monitor.screen
                },
                3.8,
            ),
            ChargingStyle::Aurora => (
                RECT {
                    bottom: monitor.screen.top + (300.0 * scale) as i32,
                    ..monitor.screen
                },
                4.4,
            ),
            ChargingStyle::Halo => {
                let (width, height) = ((220.0 * scale) as i32, (96.0 * scale) as i32);
                let top = monitor.work_top + (4.0 * scale) as i32;
                (
                    RECT {
                        left: centre_x - width / 2,
                        top,
                        right: centre_x + width / 2,
                        bottom: top + height,
                    },
                    3.4,
                )
            }
            ChargingStyle::Surge => {
                let side = (medal_px as f32 * 2.5).round() as i32;
                let centre_y = monitor.work_top + (medal_px as f32 * 0.5 + 24.0 * scale) as i32;
                (
                    RECT {
                        left: centre_x - side / 2,
                        top: centre_y - side / 2,
                        right: centre_x + side / 2,
                        bottom: centre_y + side / 2,
                    },
                    3.6,
                )
            }
        };
        let (width, height) = (
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        );

        let hwnd = create_window(rect)?;
        let built = || -> Result<DesktopWindowTarget> {
            // SAFETY: `hwnd` is the live window created above.
            let target = unsafe {
                gfx.compositor
                    .cast::<ICompositorDesktopInterop>()?
                    .CreateDesktopWindowTarget(hwnd, true)?
            };
            let root = match style {
                ChargingStyle::Surge => choreograph(gfx, width, medal_px, ring_px, percent, scale)?,
                ChargingStyle::Halo => capsule_scene(gfx, width, height, percent, scale)?,
                ChargingStyle::Edge => edge_scene(gfx, width, height, percent, scale)?,
                ChargingStyle::Tide => tide_scene(gfx, width, height, percent, scale)?,
                ChargingStyle::Bloom => bloom_scene(gfx, width, height, percent, scale)?,
                ChargingStyle::Aurora => aurora_scene(gfx, width, height, percent, scale)?,
            };
            target.SetRoot(&root)?;
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
            _target: target,
            started: Instant::now(),
            total,
        })
    }
}

impl Drop for Charge {
    fn drop(&mut self) {
        // SAFETY: Our own window, destroyed once.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/* ------------------------------------------------------------ Animation */

fn span(seconds: f32) -> TimeSpan {
    TimeSpan {
        Duration: (f64::from(seconds) * 1e7) as i64,
    }
}

fn scalar(
    c: &Compositor,
    keys: &[(f32, f32)],
    delay: f32,
    ease: &CompositionEasingFunction,
) -> Result<ScalarKeyFrameAnimation> {
    let end = keys.last().map_or(1.0, |key| key.0).max(0.01);
    let animation = c.CreateScalarKeyFrameAnimation()?;
    animation.SetDuration(span(end))?;
    animation.SetDelayTime(span(delay))?;
    for &(time, value) in keys {
        animation.InsertKeyFrameWithEasingFunction(time / end, value, ease)?;
    }
    Ok(animation)
}

fn vector(
    c: &Compositor,
    keys: &[(f32, Vector3)],
    delay: f32,
    ease: &CompositionEasingFunction,
) -> Result<Vector3KeyFrameAnimation> {
    let end = keys.last().map_or(1.0, |key| key.0).max(0.01);
    let animation = c.CreateVector3KeyFrameAnimation()?;
    animation.SetDuration(span(end))?;
    animation.SetDelayTime(span(delay))?;
    for &(time, value) in keys {
        animation.InsertKeyFrameWithEasingFunction(time / end, value, ease)?;
    }
    Ok(animation)
}

fn uniform(value: f32) -> Vector3 {
    Vector3::new(value, value, 1.0)
}

fn sprite(gfx: &Gfx, image: &IconImage) -> Result<SpriteVisual> {
    let c = &gfx.compositor;
    let brush = c.CreateSurfaceBrushWithSurface(&gfx.image_surface(image)?)?;
    brush.SetStretch(CompositionStretch::None)?;
    brush.SetBitmapInterpolationMode(CompositionBitmapInterpolationMode::Linear)?;
    let sprite = c.CreateSpriteVisual()?;
    sprite.SetBrush(&brush)?;
    sprite.SetSize(Vector2::new(image.width as f32, image.height as f32))?;
    Ok(sprite)
}

fn choreograph(
    gfx: &Gfx,
    side: f32,
    medal_px: u32,
    ring_px: u32,
    percent: u8,
    scale: f32,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let out: CompositionEasingFunction = c
        .CreateCubicBezierEasingFunction(Vector2::new(0.2, 0.9), Vector2::new(0.3, 1.0))?
        .cast()?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let mid = side / 2.0;

    // The glow behind everything.
    let glow_px = (medal_px as f32 * 1.7).round() as u32;
    let glow = sprite(gfx, &glow_image(glow_px))?;
    glow.SetOffset(Vector3::new(
        mid - glow_px as f32 / 2.0,
        mid - glow_px as f32 / 2.0,
        0.0,
    ))?;
    glow.SetCenterPoint(Vector3::new(
        glow_px as f32 / 2.0,
        glow_px as f32 / 2.0,
        0.0,
    ))?;
    glow.SetOpacity(0.0)?;
    glow.StartAnimation(
        h!("Opacity"),
        &scalar(
            c,
            &[(0.0, 0.0), (0.5, 0.55), (2.4, 0.32), (3.5, 0.0)],
            0.0,
            &linear,
        )?,
    )?;
    glow.StartAnimation(
        h!("Scale"),
        &vector(
            c,
            &[
                (0.0, uniform(0.6)),
                (0.9, uniform(1.12)),
                (3.5, uniform(1.0)),
            ],
            0.0,
            &out,
        )?,
    )?;
    root.Children()?.InsertAtTop(&glow)?;

    // Rings of water: two thin ones, one after another.
    {
        let image = ring_image(ring_px);
        for index in 0..2 {
            let ring = sprite(gfx, &image)?;
            let half = ring_px as f32 / 2.0;
            ring.SetOffset(Vector3::new(mid - half, mid - half, 0.0))?;
            ring.SetCenterPoint(Vector3::new(half, half, 0.0))?;
            ring.SetOpacity(0.0)?;
            ring.SetScale(uniform(0.42))?;
            let delay = 0.3 + index as f32 * 0.55;
            ring.StartAnimation(
                h!("Scale"),
                &vector(c, &[(0.0, uniform(0.42)), (1.9, uniform(1.0))], delay, &out)?,
            )?;
            ring.StartAnimation(
                h!("Opacity"),
                &scalar(c, &[(0.0, 0.0), (0.12, 0.42), (1.9, 0.0)], delay, &linear)?,
            )?;
            root.Children()?.InsertAtTop(&ring)?;
        }
    }

    // The medallion: drops in, settles, holds, then rises away.
    let medal = sprite(gfx, &medal_image(medal_px, percent))?;
    let half = medal_px as f32 / 2.0;
    let rest = Vector3::new(mid - half, mid - half, 0.0);
    let low = Vector3::new(rest.X, rest.Y + 18.0 * scale, 0.0);
    let high = Vector3::new(rest.X, rest.Y - 10.0 * scale, 0.0);
    medal.SetOffset(low)?;
    medal.SetCenterPoint(Vector3::new(half, half, 0.0))?;
    medal.SetOpacity(0.0)?;
    medal.SetScale(uniform(0.55))?;
    medal.StartAnimation(
        h!("Offset"),
        &vector(
            c,
            &[(0.0, low), (0.55, rest), (2.95, rest), (3.6, high)],
            0.0,
            &out,
        )?,
    )?;
    medal.StartAnimation(
        h!("Scale"),
        &vector(
            c,
            &[
                (0.0, uniform(0.55)),
                (0.45, uniform(1.06)),
                (0.75, uniform(1.0)),
                (2.95, uniform(1.0)),
                (3.6, uniform(0.9)),
            ],
            0.0,
            &out,
        )?,
    )?;
    medal.StartAnimation(
        h!("Opacity"),
        &scalar(
            c,
            &[(0.0, 0.0), (0.25, 1.0), (2.95, 1.0), (3.55, 0.0)],
            0.0,
            &linear,
        )?,
    )?;
    root.Children()?.InsertAtTop(&medal)?;
    Ok(root)
}

/* ------------------------------------------------------------- Bitmaps */

type Rgba = [f32; 4];

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 3 x 5 digits and the percent sign, one row per entry, `#` for ink.
fn glyph(character: char) -> [&'static str; 5] {
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
        '9' => ["###", "# #", "###", "  #", "###"],
        _ => ["# #", "  #", " # ", "#  ", "# #"],
    }
}

fn inside_bolt(u: f32, v: f32) -> bool {
    const BOLT: [(f32, f32); 6] = [
        (0.08, -0.52),
        (-0.24, 0.06),
        (-0.04, 0.06),
        (-0.12, 0.44),
        (0.24, -0.12),
        (0.04, -0.12),
    ];
    let mut inside = false;
    let mut previous = BOLT[BOLT.len() - 1];
    for &point in &BOLT {
        if (point.1 > v) != (previous.1 > v)
            && u < (previous.0 - point.0) * (v - point.1) / (previous.1 - point.1) + point.0
        {
            inside = !inside;
        }
        previous = point;
    }
    inside
}

/// The medallion at (`u`, `v`) in -1..1: premultiplied colour.
fn medal_at(u: f32, v: f32, fraction: f32, text: &[char]) -> Rgba {
    let r = (u * u + v * v).sqrt();
    let mut out = [0.0f32; 4];
    let mut paint = |color: [f32; 3], alpha: f32| {
        for channel in 0..3 {
            out[channel] = color[channel] * alpha + out[channel] * (1.0 - alpha);
        }
        out[3] = alpha + out[3] * (1.0 - alpha);
    };
    if r < 0.80 {
        let light = (0.5 - 0.35 * (u + v)).clamp(0.0, 1.0);
        let mut body = mix3([0.02, 0.10, 0.14], [0.07, 0.26, 0.33], light * 0.8);
        let gloss = smooth(0.62, 0.0, ((u + 0.15).powi(2) + (v + 0.6).powi(2)).sqrt()) * 0.16;
        body = mix3(body, [1.0, 1.0, 1.0], gloss);
        paint(body, 0.95);
    }
    if (0.80..0.83).contains(&r) {
        paint([1.0, 1.0, 1.0], 0.10);
    }
    if (0.86..0.97).contains(&r) {
        paint([1.0, 1.0, 1.0], 0.14);
        let turn = u.atan2(-v).rem_euclid(TAU) / TAU;
        if turn <= fraction {
            let t = turn / fraction.max(0.01);
            let color = if t < 0.6 {
                mix3([0.09, 0.63, 0.77], [0.47, 0.94, 0.94], t / 0.6)
            } else {
                mix3([0.47, 0.94, 0.94], [0.86, 1.0, 0.98], (t - 0.6) / 0.4)
            };
            paint(color, 1.0);
        }
    }
    if r < 0.8 {
        if inside_bolt(u, v + 0.14) {
            paint(
                mix3(
                    [1.0, 1.0, 1.0],
                    [0.74, 0.98, 0.96],
                    ((v + 0.66) / 0.96).clamp(0.0, 1.0),
                ),
                1.0,
            );
        }
        let cell = 0.05;
        let columns = text.len() as f32 * 4.0 - 1.0;
        let (x0, y0) = (-columns * cell / 2.0, 0.38);
        let (gx, gy) = (((u - x0) / cell).floor(), ((v - y0) / cell).floor());
        if (0.0..columns).contains(&gx) && (0.0..5.0).contains(&gy) {
            let (index, column) = ((gx as usize) / 4, (gx as usize) % 4);
            if column < 3
                && let Some(&character) = text.get(index)
                && glyph(character)[gy as usize].as_bytes()[column] == b'#'
            {
                paint([0.88, 0.98, 0.97], 1.0);
            }
        }
    }
    out
}

fn to_image(width: u32, height: u32, mut sample: impl FnMut(u32, u32) -> Rgba) -> IconImage {
    let mut bgra = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let [r, g, b, a] = sample(x, y);
            bgra.extend_from_slice(&[
                (b.clamp(0.0, 1.0) * 255.0).round() as u8,
                (g.clamp(0.0, 1.0) * 255.0).round() as u8,
                (r.clamp(0.0, 1.0) * 255.0).round() as u8,
                (a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ]);
        }
    }
    IconImage {
        width,
        height,
        bgra,
    }
}

fn medal_image(px: u32, percent: u8) -> IconImage {
    let fraction = f32::from(percent.min(100)) / 100.0;
    let text: Vec<char> = format!("{}%", percent.min(100)).chars().collect();
    const SAMPLES: u32 = 3;
    to_image(px, px, |x, y| {
        let mut sum = [0.0f32; 4];
        for sy in 0..SAMPLES {
            for sx in 0..SAMPLES {
                let u = ((x as f32 + (sx as f32 + 0.5) / SAMPLES as f32) / px as f32) * 2.0 - 1.0;
                let v = ((y as f32 + (sy as f32 + 0.5) / SAMPLES as f32) / px as f32) * 2.0 - 1.0;
                let sample = medal_at(u, v, fraction, &text);
                for (total, value) in sum.iter_mut().zip(sample) {
                    *total += value;
                }
            }
        }
        let n = (SAMPLES * SAMPLES) as f32;
        [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n]
    })
}

fn ring_image(px: u32) -> IconImage {
    to_image(px, px, |x, y| {
        let u = (x as f32 + 0.5) / px as f32 * 2.0 - 1.0;
        let v = (y as f32 + 0.5) / px as f32 * 2.0 - 1.0;
        let r = (u * u + v * v).sqrt();
        let line = (-((r - 0.94) / 0.03).powi(2)).exp();
        let wash = 0.07 * smooth(0.94, 0.3, r) * (r < 0.97) as u8 as f32;
        let a = (line * 0.9 + wash).min(1.0);
        [0.59 * a, 0.94 * a, 0.92 * a, a]
    })
}

fn glow_image(px: u32) -> IconImage {
    to_image(px, px, |x, y| {
        let u = (x as f32 + 0.5) / px as f32 * 2.0 - 1.0;
        let v = (y as f32 + 0.5) / px as f32 * 2.0 - 1.0;
        let r2 = u * u + v * v;
        let a = if r2 < 1.0 {
            0.55 * (-r2 / 0.16).exp() * (1.0 - r2 * r2)
        } else {
            0.0
        };
        [0.24 * a, 0.86 * a, 0.90 * a, a]
    })
}

/* -------------------------------------------------- Battery colours */

/// Green when the battery is healthy, yellow in the middle, orange when low.
fn level_color(percent: u8) -> [f32; 3] {
    match percent {
        0..=20 => [1.0, 0.50, 0.12],
        21..=59 => [1.0, 0.80, 0.18],
        _ => [0.28, 0.90, 0.48],
    }
}

/// A sprite of `image` stretched to `width` x `height` (for soft bitmaps
/// drawn at a fraction of their size).
fn stretched(gfx: &Gfx, image: &IconImage, width: f32, height: f32) -> Result<SpriteVisual> {
    let c = &gfx.compositor;
    let brush = c.CreateSurfaceBrushWithSurface(&gfx.image_surface(image)?)?;
    brush.SetStretch(CompositionStretch::Fill)?;
    brush.SetBitmapInterpolationMode(CompositionBitmapInterpolationMode::Linear)?;
    let sprite = c.CreateSpriteVisual()?;
    sprite.SetBrush(&brush)?;
    sprite.SetSize(Vector2::new(width, height))?;
    Ok(sprite)
}

fn ease_out(c: &Compositor) -> Result<CompositionEasingFunction> {
    Ok(
        c.CreateCubicBezierEasingFunction(Vector2::new(0.2, 0.9), Vector2::new(0.3, 1.0))?
            .cast()?,
    )
}

/* ------------------------------------------------------------ Capsule */

fn capsule_scene(
    gfx: &Gfx,
    width: f32,
    height: f32,
    percent: u8,
    scale: f32,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let out = ease_out(c)?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;

    let pill_w = (132.0 * scale).round() as u32;
    let pill_h = (38.0 * scale).round() as u32;
    let pill = sprite(gfx, &capsule_image(pill_w, pill_h, percent))?;
    let rest = Vector3::new(
        (width - pill_w as f32) / 2.0,
        (height - pill_h as f32) / 2.0,
        0.0,
    );
    let above = Vector3::new(rest.X, rest.Y - 22.0 * scale, 0.0);
    pill.SetOffset(above)?;
    pill.SetCenterPoint(Vector3::new(pill_w as f32 / 2.0, pill_h as f32 / 2.0, 0.0))?;
    pill.SetOpacity(0.0)?;
    pill.StartAnimation(
        h!("Offset"),
        &vector(
            c,
            &[(0.0, above), (0.7, rest), (2.7, rest), (3.3, above)],
            0.0,
            &out,
        )?,
    )?;
    pill.StartAnimation(
        h!("Scale"),
        &vector(
            c,
            &[
                (0.0, uniform(0.9)),
                (0.7, uniform(1.0)),
                (2.7, uniform(1.0)),
                (3.3, uniform(0.96)),
            ],
            0.0,
            &out,
        )?,
    )?;
    pill.StartAnimation(
        h!("Opacity"),
        &scalar(
            c,
            &[(0.0, 0.0), (0.4, 1.0), (2.7, 1.0), (3.3, 0.0)],
            0.0,
            &linear,
        )?,
    )?;
    root.Children()?.InsertAtTop(&pill)?;
    Ok(root)
}

/// The capsule at (`x`, `y`) pixels: premultiplied colour. Deliberately
/// small and quiet: dark glass, a soft dot in the battery's colour, the level.
fn capsule_at(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    _fraction: f32,
    text: &[char],
    color: [f32; 3],
) -> Rgba {
    let mut out = [0.0f32; 4];
    let mut paint = |c: [f32; 3], alpha: f32| {
        for channel in 0..3 {
            out[channel] = c[channel] * alpha + out[channel] * (1.0 - alpha);
        }
        out[3] = alpha + out[3] * (1.0 - alpha);
    };
    // Signed distance to the pill: negative inside.
    let radius = h / 2.0;
    let (px, py) = ((x - w / 2.0).abs() - (w / 2.0 - radius), y - h / 2.0);
    let sd = if px > 0.0 {
        (px * px + py * py).sqrt() - radius
    } else {
        py.abs() - radius
    };
    let edge = 0.75;
    let inside = smooth(edge, -edge, sd);
    if inside <= 0.0 {
        return out;
    }
    let sheen = (1.0 - (y / h)).clamp(0.0, 1.0);
    let body = mix3([0.04, 0.05, 0.07], [0.10, 0.12, 0.15], sheen * 0.6);
    paint(body, 0.86 * inside);
    // A hairline just inside the edge.
    let rim = smooth(2.0, 0.5, -sd) * smooth(-0.2, 0.9, -sd);
    paint([1.0, 1.0, 1.0], 0.14 * rim * inside.max(0.4));

    // The dot and the level, centred together.
    let cell = h * 0.064;
    let columns = text.len() as f32 * 4.0 - 1.0;
    let dot = h * 0.22;
    let gap = h * 0.30;
    let group = dot + gap + columns * cell;
    let left = (w - group) / 2.0;
    let (cx, cy) = (left + dot / 2.0, h / 2.0);
    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    // Halo of the dot, then the dot.
    paint(color, 0.22 * smooth(h * 0.40, dot * 0.5, d));
    paint(
        mix3(color, [1.0, 1.0, 1.0], 0.25),
        smooth(dot * 0.5 + 0.7, dot * 0.5 - 0.7, d),
    );

    let (x0, y0) = (left + dot + gap, h * 0.5 - 2.5 * cell);
    let (gx, gy) = (((x - x0) / cell).floor(), ((y - y0) / cell).floor());
    if (0.0..columns).contains(&gx) && (0.0..5.0).contains(&gy) {
        let (index, column) = ((gx as usize) / 4, (gx as usize) % 4);
        if column < 3
            && let Some(&character) = text.get(index)
            && glyph(character)[gy as usize].as_bytes()[column] == b'#'
        {
            paint([0.95, 0.97, 0.98], 0.96);
        }
    }
    out
}

fn capsule_image(w: u32, h: u32, percent: u8) -> IconImage {
    let fraction = f32::from(percent.min(100)) / 100.0;
    let text: Vec<char> = format!("{}%", percent.min(100)).chars().collect();
    let color = level_color(percent);
    const SAMPLES: u32 = 3;
    to_image(w, h, |x, y| {
        let mut sum = [0.0f32; 4];
        for sy in 0..SAMPLES {
            for sx in 0..SAMPLES {
                let px = x as f32 + (sx as f32 + 0.5) / SAMPLES as f32;
                let py = y as f32 + (sy as f32 + 0.5) / SAMPLES as f32;
                let sample = capsule_at(px, py, w as f32, h as f32, fraction, &text, color);
                for (total, value) in sum.iter_mut().zip(sample) {
                    *total += value;
                }
            }
        }
        let n = (SAMPLES * SAMPLES) as f32;
        [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n]
    })
}

/* --------------------------------------------------------------- Edge */

#[derive(Clone, Copy)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// A band of liquid light along one edge, `long` x `depth` pixels (swapped
/// for the sides). The inner boundary wobbles like the surface of water, a
/// second faint band sits inside it like a refraction, and a soft bright rim
/// runs along the very edge.
fn edge_pixel(side: Side, along: f32, depth: f32, color: [f32; 3], seed: f32) -> Rgba {
    let wobble = 0.10 * (along * TAU * 2.0 + seed).sin()
        + 0.06 * (along * TAU * 5.0 + seed * 1.7).sin()
        + 0.03 * (along * TAU * 11.0 + seed * 0.4).sin();
    let _ = side;
    let d = (depth + wobble * 0.55).max(0.0);
    let body = (1.0 - smooth(0.0, 1.0, d)).powi(2) * 0.50;
    let rim = (-depth / 0.035).exp() * 0.85;
    let band = 0.16 * (-((d - 0.38) / 0.07).powi(2)).exp();
    let alpha = (body + rim + band).clamp(0.0, 1.0);
    let white = (rim * 0.55 + band * 0.4).min(0.75);
    let rgb = mix3(color, [1.0, 1.0, 1.0], white);
    [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha]
}

fn edge_image(side: Side, long: u32, depth: u32, color: [f32; 3], seed: f32) -> IconImage {
    let (w, h) = match side {
        Side::Top | Side::Bottom => (long, depth),
        Side::Left | Side::Right => (depth, long),
    };
    to_image(w, h, |x, y| {
        let (along, d) = match side {
            Side::Top => (x as f32 / w as f32, (y as f32 + 0.5) / h as f32),
            Side::Bottom => (x as f32 / w as f32, 1.0 - (y as f32 + 0.5) / h as f32),
            Side::Left => (y as f32 / h as f32, (x as f32 + 0.5) / w as f32),
            Side::Right => (y as f32 / h as f32, 1.0 - (x as f32 + 0.5) / w as f32),
        };
        edge_pixel(side, along, d, color, seed)
    })
}

fn edge_scene(
    gfx: &Gfx,
    width: f32,
    height: f32,
    percent: u8,
    scale: f32,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let out = ease_out(c)?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let color = level_color(percent);
    let depth = (170.0 * scale).min(width.min(height) * 0.3).round();
    // Quarter resolution: the glow is soft, the compositor does the blur.
    let quarter = |value: f32| ((value / 4.0).ceil() as u32).max(4);
    let sides = [
        (Side::Top, 0.3f32),
        (Side::Bottom, 2.1),
        (Side::Left, 4.0),
        (Side::Right, 5.4),
    ];
    for (index, &(side, seed)) in sides.iter().enumerate() {
        let (long, w, h, rest, away) = match side {
            Side::Top => (
                width,
                width,
                depth,
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -depth * 0.55, 0.0),
            ),
            Side::Bottom => (
                width,
                width,
                depth,
                Vector3::new(0.0, height - depth, 0.0),
                Vector3::new(0.0, height - depth * 0.45, 0.0),
            ),
            Side::Left => (
                height,
                depth,
                height,
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(-depth * 0.55, 0.0, 0.0),
            ),
            Side::Right => (
                height,
                depth,
                height,
                Vector3::new(width - depth, 0.0, 0.0),
                Vector3::new(width - depth * 0.45, 0.0, 0.0),
            ),
        };
        let image = edge_image(side, quarter(long), quarter(depth), color, seed);
        let strip = stretched(gfx, &image, w, h)?;
        strip.SetOffset(away)?;
        strip.SetOpacity(0.0)?;
        let delay = index as f32 * 0.06;
        strip.StartAnimation(
            h!("Offset"),
            &vector(c, &[(0.0, away), (1.0, rest)], delay, &out)?,
        )?;
        // Fade in, breathe, fade away.
        strip.StartAnimation(
            h!("Opacity"),
            &scalar(
                c,
                &[
                    (0.0, 0.0),
                    (0.8, 1.0),
                    (1.6, 0.72),
                    (2.4, 1.0),
                    (3.5 - delay, 0.0),
                ],
                delay,
                &linear,
            )?,
        )?;
        root.Children()?.InsertAtTop(&strip)?;
    }
    Ok(root)
}

/* --------------------------------------------------------------- Tide */

fn tide_image(w: u32, h: u32, color: [f32; 3]) -> IconImage {
    to_image(w, h, |x, y| {
        let along = (x as f32 + 0.5) / w as f32;
        let down = y as f32 / h as f32;
        // A comet: faint tail, bright head. A hot core line, soft bloom below.
        let tail = smooth(0.0, 1.0, along).powf(1.6);
        let core = (-(down / 0.05).powi(2)).exp();
        let bloom = (-down / 0.30).exp() * 0.40;
        let alpha = ((core * 0.95 + bloom) * (0.15 + 0.85 * tail)).clamp(0.0, 1.0);
        let white = (core * 0.6 * tail).min(0.8);
        let rgb = mix3(color, [1.0, 1.0, 1.0], white);
        [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha]
    })
}

fn dot_image(px: u32, color: [f32; 3]) -> IconImage {
    to_image(px, px, |x, y| {
        let u = (x as f32 + 0.5) / px as f32 * 2.0 - 1.0;
        let v = (y as f32 + 0.5) / px as f32 * 2.0 - 1.0;
        let r2 = u * u + v * v;
        let a = if r2 < 1.0 {
            (-r2 / 0.07).exp() * (1.0 - r2)
        } else {
            0.0
        };
        let rgb = mix3(color, [1.0, 1.0, 1.0], 0.5);
        [rgb[0] * a, rgb[1] * a, rgb[2] * a, a]
    })
}

fn tide_scene(
    gfx: &Gfx,
    width: f32,
    height: f32,
    percent: u8,
    scale: f32,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let out = ease_out(c)?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let color = level_color(percent);
    let fraction = (f32::from(percent.min(100)) / 100.0).max(0.04);
    let bar_w = (width * fraction).round();
    let bar_h = height.min((46.0 * scale).round());

    // The faint track: where the line will go.
    let track = stretched(gfx, &tide_image(64, 16, color), width, bar_h)?;
    track.SetOpacity(0.0)?;
    track.StartAnimation(
        h!("Opacity"),
        &scalar(
            c,
            &[(0.0, 0.0), (0.4, 0.14), (2.4, 0.14), (3.2, 0.0)],
            0.0,
            &linear,
        )?,
    )?;
    root.Children()?.InsertAtTop(&track)?;

    // The line itself grows from the left up to the level.
    let line = stretched(
        gfx,
        &tide_image(((bar_w / 4.0).ceil() as u32).clamp(16, 1024), 16, color),
        bar_w,
        bar_h,
    )?;
    line.SetCenterPoint(Vector3::new(0.0, 0.0, 0.0))?;
    line.SetOpacity(0.0)?;
    line.SetScale(Vector3::new(0.0, 1.0, 1.0))?;
    line.StartAnimation(
        h!("Scale"),
        &vector(
            c,
            &[
                (0.0, Vector3::new(0.0, 1.0, 1.0)),
                (1.5, Vector3::new(1.0, 1.0, 1.0)),
            ],
            0.15,
            &out,
        )?,
    )?;
    line.StartAnimation(
        h!("Opacity"),
        &scalar(
            c,
            &[(0.0, 0.0), (0.3, 1.0), (2.6, 1.0), (3.3, 0.0)],
            0.0,
            &linear,
        )?,
    )?;
    root.Children()?.InsertAtTop(&line)?;

    // The head: a soft light riding the end of the line.
    let head_px = (56.0 * scale).round() as u32;
    let head = stretched(
        gfx,
        &dot_image(head_px / 2, color),
        head_px as f32,
        head_px as f32,
    )?;
    let half = head_px as f32 / 2.0;
    let start = Vector3::new(-half, -half + 3.0 * scale, 0.0);
    let end = Vector3::new(bar_w - half, start.Y, 0.0);
    head.SetOffset(start)?;
    head.SetOpacity(0.0)?;
    head.StartAnimation(
        h!("Offset"),
        &vector(c, &[(0.0, start), (1.5, end)], 0.15, &out)?,
    )?;
    head.StartAnimation(
        h!("Opacity"),
        &scalar(
            c,
            &[(0.0, 0.0), (0.3, 0.95), (2.0, 0.7), (3.2, 0.0)],
            0.0,
            &linear,
        )?,
    )?;
    root.Children()?.InsertAtTop(&head)?;
    Ok(root)
}

/* -------------------------------------------------------------- Bloom */

/// A wide soft glow, brightest at the bottom centre. `u` runs -1..1 across,
/// `v` 0..1 from top to bottom.
fn bloom_image(w: u32, h: u32, color: [f32; 3]) -> IconImage {
    to_image(w, h, |x, y| {
        let u = (x as f32 + 0.5) / w as f32 * 2.0 - 1.0;
        let v = (y as f32 + 0.5) / h as f32;
        let across = (-(u * u) / 0.30).exp();
        let rise = v.powf(2.4);
        let a = (0.85 * across * rise).clamp(0.0, 1.0);
        let c = mix3(color, [1.0, 1.0, 1.0], 0.35 * v * across);
        [c[0] * a, c[1] * a, c[2] * a, a]
    })
}

fn bloom_scene(
    gfx: &Gfx,
    width: f32,
    height: f32,
    percent: u8,
    scale: f32,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let out = ease_out(c)?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let color = level_color(percent);
    let _ = scale;

    // Broad glow, then a tighter and brighter core that follows it.
    for (fraction, tall, delay, peak) in
        [(1.0f32, 1.0f32, 0.0f32, 0.80f32), (0.42, 0.62, 0.25, 0.95)]
    {
        let (w, h) = (width * fraction, height * tall);
        let glow = stretched(gfx, &bloom_image(128, 48, color), w, h)?;
        let rest = Vector3::new((width - w) / 2.0, height - h, 0.0);
        glow.SetOffset(rest)?;
        glow.SetCenterPoint(Vector3::new(w / 2.0, h, 0.0))?;
        glow.SetOpacity(0.0)?;
        glow.StartAnimation(
            h!("Scale"),
            &vector(
                c,
                &[
                    (0.0, Vector3::new(0.7, 0.35, 1.0)),
                    (1.3, Vector3::new(1.0, 1.0, 1.0)),
                    (2.6, Vector3::new(1.04, 1.06, 1.0)),
                    (3.6, Vector3::new(1.0, 0.8, 1.0)),
                ],
                delay,
                &out,
            )?,
        )?;
        glow.StartAnimation(
            h!("Opacity"),
            &scalar(
                c,
                &[(0.0, 0.0), (0.7, peak), (2.5, peak * 0.8), (3.6, 0.0)],
                delay,
                &linear,
            )?,
        )?;
        root.Children()?.InsertAtTop(&glow)?;
    }
    Ok(root)
}

/* ------------------------------------------------------------- Aurora */

/// One soft ribbon: fades toward the bottom and the sides, with a slow swell.
fn aurora_image(w: u32, h: u32, color: [f32; 3], phase: f32) -> IconImage {
    to_image(w, h, |x, y| {
        let u = (x as f32 + 0.5) / w as f32;
        let v = (y as f32 + 0.5) / h as f32;
        let side = smooth(0.0, 0.28, u) * smooth(1.0, 0.72, u);
        let swell = 0.55 + 0.45 * (u * 6.2 + phase).sin();
        let fall = (1.0 - v).powf(1.7);
        let a = (0.62 * side * swell * fall).clamp(0.0, 1.0);
        [color[0] * a, color[1] * a, color[2] * a, a]
    })
}

fn aurora_scene(
    gfx: &Gfx,
    width: f32,
    height: f32,
    percent: u8,
    scale: f32,
) -> Result<ContainerVisual> {
    let c = &gfx.compositor;
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let ease = c.CreateCubicBezierEasingFunction(Vector2::new(0.4, 0.0), Vector2::new(0.2, 1.0))?;
    let ease: CompositionEasingFunction = ease.cast()?;
    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let base = level_color(percent);
    let cool = mix3(base, [0.30, 0.80, 1.0], 0.55);
    let pale = mix3(base, [1.0, 1.0, 1.0], 0.45);
    let _ = scale;

    // (colour, width share, height share, drift from, drift to, delay, phase)
    let ribbons = [
        (base, 0.95f32, 1.00f32, -0.20f32, 0.05f32, 0.0f32, 0.0f32),
        (cool, 0.80, 0.78, 0.30, 0.05, 0.3, 2.1),
        (pale, 0.55, 0.52, -0.05, 0.40, 0.6, 4.0),
    ];
    for (color, wide, tall, from, to, delay, phase) in ribbons {
        let (w, h) = (width * wide, height * tall);
        let ribbon = stretched(gfx, &aurora_image(160, 40, color, phase), w, h)?;
        let start = Vector3::new(width * from, -h * 0.12, 0.0);
        let end = Vector3::new(width * to, 0.0, 0.0);
        ribbon.SetOffset(start)?;
        ribbon.SetOpacity(0.0)?;
        ribbon.StartAnimation(
            h!("Offset"),
            &vector(c, &[(0.0, start), (4.4, end)], delay, &ease)?,
        )?;
        ribbon.StartAnimation(
            h!("Opacity"),
            &scalar(
                c,
                &[(0.0, 0.0), (1.1, 1.0), (3.0, 0.9), (4.2, 0.0)],
                delay,
                &linear,
            )?,
        )?;
        root.Children()?.InsertAtTop(&ribbon)?;
    }
    Ok(root)
}

fn create_window(rect: RECT) -> Result<HWND> {
    static CLASS: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    let class = w!("WhirlpoolCharge");
    // SAFETY: The class is registered once; the window is destroyed by
    // `Charge::drop`, or by the caller if building the scene fails.
    unsafe {
        let instance = GetModuleHandleW(None)?;
        CLASS.get_or_init(|| {
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(charge_proc),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            });
        });
        // Click-through: the animation never takes a click from what is under it.
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

unsafe extern "system" fn charge_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: Default handling for our own window.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_medallion_is_round_opaque_inside_and_clear_outside() {
        let image = medal_image(96, 80);
        assert_eq!(image.bgra.len(), 96 * 96 * 4);
        let alpha = |x: usize, y: usize| image.bgra[(y * 96 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0);
        assert!(alpha(48, 70) > 200, "the digits sit inside the disc");
    }

    #[test]
    fn the_capsule_and_the_edges_are_soft_and_premultiplied() {
        let capsule = capsule_image(200, 52, 64);
        assert_eq!(capsule.bgra.len(), 200 * 52 * 4);
        assert_eq!(capsule.bgra[3], 0, "the corner is clear");
        let edge = edge_image(Side::Top, 64, 16, level_color(15), 0.3);
        // The rim is brighter than the inside edge of the glow.
        assert!(edge.bgra[3] > edge.bgra[(15 * 64) * 4 + 3]);
        for pixel in edge.bgra.chunks_exact(4) {
            assert!(pixel[0] <= pixel[3] && pixel[1] <= pixel[3] && pixel[2] <= pixel[3]);
        }
    }

    #[test]
    fn the_colour_follows_the_level() {
        assert!(level_color(10)[0] > level_color(10)[1]);
        assert!(level_color(40)[0] > 0.9 && level_color(40)[1] > 0.7);
        assert!(level_color(90)[1] > level_color(90)[0]);
    }

    #[test]
    fn the_bolt_covers_the_middle_and_not_the_corners() {
        assert!(inside_bolt(0.0, 0.0));
        assert!(!inside_bolt(0.9, 0.9));
    }
}
