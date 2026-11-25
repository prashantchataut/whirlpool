//! The shared look of the short-lived panels (Start grid, quick settings, HUDs).
//!
//! One place for the pieces that make them read as one product: the frosted backdrop (a picture of
//! the screen taken before the panel opened, blurred once on the CPU), easing and stagger, soft
//! shadows and gradients. Nothing here animates by itself; callers ask for a progress value and
//! repaint only while it is below 1, so a panel that has settled costs nothing.

#![allow(dead_code)]

use eframe::egui::{
    self, Color32, ColorImage, Context, CornerRadius, Mesh, Painter, Rect, TextureHandle,
    TextureOptions, epaint::Shadow, pos2,
};
use whirlpool_core::glass::{self, Image};

/// Water-theme colours the panels share.
pub const FOAM: Color32 = Color32::from_rgb(0xea, 0xf8, 0xf6);
pub const DEEP: Color32 = Color32::from_rgb(0x07, 0x14, 0x1f);
pub const TEAL: Color32 = Color32::from_rgb(0x2d, 0xd4, 0xc4);
pub const TEAL_DEEP: Color32 = Color32::from_rgb(0x0f, 0x76, 0x7a);

/// Which of the three round-one designs to draw. The variants exist so each direction can be
/// captured and compared on a runner; the chosen one is the default and the capture driver sets
/// the environment variable to show the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    A,
    B,
    C,
}

impl Direction {
    /// `value` is `a`, `b` or `c` in any case, with spaces ignored. Anything else gives `default`.
    pub fn parse(value: Option<&str>, default: Self) -> Self {
        match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            Some("a") => Self::A,
            Some("b") => Self::B,
            Some("c") => Self::C,
            _ => default,
        }
    }

    /// Reads the environment variable `variable`.
    pub fn from_env(variable: &str, default: Self) -> Self {
        Self::parse(std::env::var(variable).ok().as_deref(), default)
    }
}

/// Exponential ease-out: fast at the start, settling softly. Input and output are 0..=1.
pub fn out_expo(t: f32) -> f32 {
    if t >= 1.0 {
        1.0
    } else {
        1.0 - 2f32.powf(-10.0 * t.max(0.0))
    }
}

/// A back ease-out with a small overshoot: the springy settle of a tile arriving.
pub fn out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    let c1 = 1.25;
    1.0 + (c1 + 1.0) * t * t * t + c1 * t * t
}

/// Progress 0..=1 of item `order` in a staggered entrance that started `elapsed` seconds ago.
/// Items start `step` seconds apart (the delay is capped so late items never wait long) and each
/// takes `duration` seconds.
pub fn stagger(elapsed: f32, order: usize, step: f32, duration: f32) -> f32 {
    let delay = (order as f32 * step).min(0.32);
    ((elapsed - delay) / duration).clamp(0.0, 1.0)
}

/// Whether an entrance that began `elapsed` seconds ago still needs frames.
pub fn entrance_running(elapsed: f32) -> bool {
    elapsed < 0.9
}

/// The blurred screen behind a panel. Without a picture (locked session, remote desktop) the
/// panel falls back to a flat, more opaque wash, so text stays readable either way.
pub struct Backdrop {
    texture: Option<TextureHandle>,
}

impl Backdrop {
    /// `shot` is the screen area the panel covers, in physical pixels. `factor` shrinks it before
    /// blurring (bigger is faster and softer); `radius` is the blur radius on the shrunk picture.
    pub fn new(ctx: &Context, shot: Option<Image>, factor: usize, radius: usize) -> Self {
        let texture = shot.map(|image| {
            let frosted = glass::frost(&image, factor, radius);
            let colors =
                ColorImage::from_rgba_unmultiplied([frosted.width, frosted.height], &frosted.rgba);
            ctx.load_texture("whirlpool-backdrop", colors, TextureOptions::LINEAR)
        });
        Self { texture }
    }

    pub fn has_picture(&self) -> bool {
        self.texture.is_some()
    }

    /// Paints the blur over `rect`, then `wash` (with a picture) or `fallback` (without).
    pub fn paint(&self, painter: &Painter, rect: Rect, wash: Color32, fallback: Color32) {
        match &self.texture {
            Some(texture) => {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.rect_filled(rect, CornerRadius::ZERO, wash);
            }
            None => {
                painter.rect_filled(rect, CornerRadius::ZERO, fallback);
            }
        }
    }
}

/// A vertical gradient over `rect` (square corners; for full-bleed use).
pub fn vertical_gradient(painter: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

/// A soft drop shadow under `rect`: offset downward and blurred, never a zero-offset halo.
pub fn drop_shadow(painter: &Painter, rect: Rect, radius: u8, blur: u8, down: i8, alpha: u8) {
    painter.add(
        Shadow {
            offset: [0, down],
            blur,
            spread: 0,
            color: Color32::from_black_alpha(alpha),
        }
        .as_shape(rect, CornerRadius::same(radius)),
    );
}

/// Mixes two colours, `t` = 0 is `a`, 1 is `b`.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        lerp(a.a(), b.a()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_runs_from_zero_to_one() {
        assert!(out_expo(0.0) < 0.01);
        assert!((out_expo(1.0) - 1.0).abs() < 1e-6);
        assert!(out_expo(0.5) > 0.9, "most of the move happens early");
        assert!((out_back(1.0) - 1.0).abs() < 1e-5);
        assert!(
            (0..100).any(|i| out_back(i as f32 / 100.0) > 1.0),
            "the spring overshoots a little"
        );
    }

    #[test]
    fn stagger_delays_later_items_but_caps_the_wait() {
        assert_eq!(stagger(0.0, 0, 0.03, 0.3), 0.0);
        assert!(stagger(0.15, 0, 0.03, 0.3) > stagger(0.15, 4, 0.03, 0.3));
        // Item 500 starts at the cap, not 15 seconds in.
        assert_eq!(stagger(0.32 + 0.3, 500, 0.03, 0.3), 1.0);
    }

    #[test]
    fn mixing_hits_both_ends() {
        let (a, b) = (Color32::from_rgb(0, 0, 0), Color32::from_rgb(200, 100, 50));
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
    }

    #[test]
    fn direction_parses_or_falls_back() {
        assert_eq!(Direction::parse(Some(" B "), Direction::A), Direction::B);
        assert_eq!(Direction::parse(Some("c"), Direction::A), Direction::C);
        assert_eq!(Direction::parse(Some("z"), Direction::C), Direction::C);
        assert_eq!(Direction::parse(None, Direction::A), Direction::A);
    }
}

/// A custom slider: a thin rounded track, an accent fill and a round white knob with a soft shadow and
/// an accent core. `value` is 0..=1, `held` 0..=1 grows the knob while it is hovered or dragged. The
/// knob never leaves `track`: it travels over the track minus one knob radius on each side, so callers
/// map the pointer with the same inset (`SLIDER_INSET`). Shared by quick settings and the volume HUD.
pub const SLIDER_INSET: f32 = 12.0;

pub fn slider(
    painter: &Painter,
    track: Rect,
    value: f32,
    accent: Color32,
    held: f32,
) -> egui::Pos2 {
    let radius = SLIDER_INSET + 1.5 * held;
    let usable = (track.width() - 2.0 * SLIDER_INSET).max(1.0);
    let x = track.left() + SLIDER_INSET + usable * value.clamp(0.0, 1.0);
    let bar = Rect::from_center_size(track.center(), egui::vec2(track.width(), 8.0));
    painter.rect_filled(bar, CornerRadius::same(4), Color32::from_white_alpha(40));
    let mut fill = bar;
    fill.set_right(x);
    painter.rect_filled(fill, CornerRadius::same(4), accent);
    let center = pos2(x, track.center().y);
    painter.circle_filled(
        center + egui::vec2(0.0, 2.5),
        radius + 2.0,
        Color32::from_black_alpha(60),
    );
    painter.circle_filled(center, radius, Color32::WHITE);
    painter.circle_filled(center, radius * 0.38, accent);
    center
}
