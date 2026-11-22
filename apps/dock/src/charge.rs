//! The charging animation: a glass medallion that drops in under the menu
//! bar with the battery ring, a bolt and the level, while rings of water
//! spread from it. It plays when the charger is plugged in, never at startup.
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

/// Seconds from the first frame to the window's destruction.
const TOTAL: f32 = 3.6;

pub struct Charge {
    hwnd: HWND,
    _target: DesktopWindowTarget,
    started: Instant,
}

impl Charge {
    /// Plays the animation for a battery at `percent`. `ripples` adds the
    /// rings of water (the Surge style); without them it is a calm halo.
    pub fn play(gfx: &Gfx, percent: u8, ripples: bool) -> Option<Self> {
        match Self::build(gfx, percent, ripples) {
            Ok(charge) => Some(charge),
            Err(error) => {
                whirlpool_shell::log::line(&format!("carga: {error:#}"));
                None
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.started.elapsed() >= Duration::from_secs_f32(TOTAL)
    }

    fn build(gfx: &Gfx, percent: u8, ripples: bool) -> Result<Self> {
        let monitor = whirlpool_shell::system::primary_monitor().context("sin monitor")?;
        let scale = monitor.scale.max(1.0);
        let medal_px = (150.0 * scale).round() as u32;
        let ring_px = (medal_px as f32 * 2.4).round() as u32;
        let side = (medal_px as f32 * 2.6).round() as i32;
        let centre_x = (monitor.work_left + monitor.work_right) / 2;
        let centre_y = monitor.work_top + (medal_px as f32 * 0.5 + 30.0 * scale) as i32;
        let left = centre_x - side / 2;
        let top = centre_y - side / 2;
        let rect = RECT {
            left,
            top,
            right: left + side,
            bottom: top + side,
        };

        let hwnd = create_window(rect)?;
        let built = || -> Result<DesktopWindowTarget> {
            // SAFETY: `hwnd` is the live window created above.
            let target = unsafe {
                gfx.compositor
                    .cast::<ICompositorDesktopInterop>()?
                    .CreateDesktopWindowTarget(hwnd, true)?
            };
            let root = choreograph(gfx, side as f32, medal_px, ring_px, percent, ripples, scale)?;
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
    ripples: bool,
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
    let glow_px = (medal_px as f32 * 2.0).round() as u32;
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
            &[(0.0, 0.0), (0.5, 0.9), (2.4, 0.55), (3.5, 0.0)],
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

    // Rings of water: three, one after another.
    if ripples {
        let image = ring_image(ring_px);
        for index in 0..3 {
            let ring = sprite(gfx, &image)?;
            let half = ring_px as f32 / 2.0;
            ring.SetOffset(Vector3::new(mid - half, mid - half, 0.0))?;
            ring.SetCenterPoint(Vector3::new(half, half, 0.0))?;
            ring.SetOpacity(0.0)?;
            ring.SetScale(uniform(0.42))?;
            let delay = 0.25 + index as f32 * 0.42;
            ring.StartAnimation(
                h!("Scale"),
                &vector(c, &[(0.0, uniform(0.42)), (1.9, uniform(1.0))], delay, &out)?,
            )?;
            ring.StartAnimation(
                h!("Opacity"),
                &scalar(c, &[(0.0, 0.0), (0.12, 0.75), (1.9, 0.0)], delay, &linear)?,
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
    fn the_bolt_covers_the_middle_and_not_the_corners() {
        assert!(inside_bolt(0.0, 0.0));
        assert!(!inside_bolt(0.9, 0.9));
    }
}
