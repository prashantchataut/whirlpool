//! Dock icon styles. Each style is a per-pixel function of a straight (not premultiplied) RGBA pixel,
//! so it works on any icon bitmap and never touches the file the icon came from.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// The icon as the app drew it.
    Default,
    /// Darker, slightly muted: for dark wallpapers and dark mode.
    Dark,
    /// A see-through glass glyph: light, desaturated, translucent.
    Clear,
    /// One colour: the icon's light and shade mapped onto a tint.
    Tinted,
    /// The icon on a rounded plate (a squircle), light or dark to suit the icon. Geometric: see
    /// [`reshape`].
    Squircle,
    /// Every icon cropped to its visible bounds and scaled to one common size. See [`reshape`].
    Normalized,
}

fn luma(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Styles one pixel. Fully transparent pixels stay fully transparent.
pub fn style_pixel(style: Style, tint: [u8; 3], px: [u8; 4]) -> [u8; 4] {
    if px[3] == 0 || matches!(style, Style::Default | Style::Squircle | Style::Normalized) {
        return px;
    }
    let (r, g, b) = (
        f32::from(px[0]) / 255.0,
        f32::from(px[1]) / 255.0,
        f32::from(px[2]) / 255.0,
    );
    let a = f32::from(px[3]) / 255.0;
    let l = luma(r, g, b);
    let (nr, ng, nb, na) = match style {
        Style::Default | Style::Squircle | Style::Normalized => (r, g, b, a),
        Style::Dark => {
            // Mute 30 %, then pull the level down with a gentle curve that keeps detail in the shade.
            let (mr, mg, mb) = (mix(r, l, 0.3), mix(g, l, 0.3), mix(b, l, 0.3));
            let k = |v: f32| (v.powf(1.25) * 0.62 + 0.02).min(1.0);
            (k(mr), k(mg), k(mb), a)
        }
        Style::Clear => {
            // Light glass: mostly white, shaped by the icon's own light; translucent where it is dark.
            let glass = mix(0.62, 1.0, l);
            let alpha = a * (0.28 + 0.62 * l);
            (glass, glass, glass * 1.0, alpha)
        }
        Style::Tinted => {
            let (tr, tg, tb) = (
                f32::from(tint[0]) / 255.0,
                f32::from(tint[1]) / 255.0,
                f32::from(tint[2]) / 255.0,
            );
            // Shade runs from a deep tint to the tint itself; highlights lift towards white.
            let shade = (0.22 + 0.78 * l.powf(0.85)).min(1.0);
            let hi = ((l - 0.7) / 0.3).clamp(0.0, 1.0) * 0.35;
            (
                mix(tr * shade, 1.0, hi),
                mix(tg * shade, 1.0, hi),
                mix(tb * shade, 1.0, hi),
                a,
            )
        }
    };
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    [q(nr), q(ng), q(nb), q(na)]
}

/// Styles a whole RGBA8 buffer in place (`len` must be a multiple of 4; a trailing remainder is left).
pub fn apply_rgba(style: Style, tint: [u8; 3], pixels: &mut [u8]) {
    if style == Style::Default {
        return;
    }
    for px in pixels.chunks_exact_mut(4) {
        let out = style_pixel(style, tint, [px[0], px[1], px[2], px[3]]);
        px.copy_from_slice(&out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TINT: [u8; 3] = [40, 140, 220];

    #[test]
    fn default_should_change_nothing() {
        assert_eq!(
            style_pixel(Style::Default, TINT, [10, 200, 30, 255]),
            [10, 200, 30, 255]
        );
    }

    #[test]
    fn transparent_pixels_should_stay_transparent_in_every_style() {
        for s in [Style::Dark, Style::Clear, Style::Tinted] {
            assert_eq!(style_pixel(s, TINT, [255, 255, 255, 0])[3], 0);
        }
    }

    #[test]
    fn dark_should_be_darker_but_keep_alpha() {
        let out = style_pixel(Style::Dark, TINT, [200, 180, 160, 255]);
        assert!(out[0] < 200 && out[1] < 180 && out[2] < 160);
        assert_eq!(out[3], 255);
    }

    #[test]
    fn clear_should_be_translucent_and_grey() {
        let out = style_pixel(Style::Clear, TINT, [30, 60, 200, 255]);
        assert!(out[3] < 200, "alpha {}", out[3]);
        assert_eq!(out[0], out[1]);
    }

    #[test]
    fn clear_should_keep_light_parts_more_opaque_than_dark_parts() {
        let dark = style_pixel(Style::Clear, TINT, [10, 10, 10, 255])[3];
        let light = style_pixel(Style::Clear, TINT, [250, 250, 250, 255])[3];
        assert!(light > dark);
    }

    #[test]
    fn tinted_should_follow_the_tint_hue_and_keep_order_of_light() {
        let mid = style_pixel(Style::Tinted, TINT, [128, 128, 128, 255]);
        assert!(mid[2] > mid[1] && mid[1] > mid[0], "{mid:?}");
        let shadow = style_pixel(Style::Tinted, TINT, [20, 20, 20, 255]);
        let light = style_pixel(Style::Tinted, TINT, [230, 230, 230, 255]);
        assert!(light[2] > mid[2] && mid[2] > shadow[2]);
        assert_eq!(mid[3], 255);
    }

    #[test]
    fn apply_rgba_should_style_every_pixel_and_ignore_a_trailing_remainder() {
        let mut buf = vec![200, 200, 200, 255, 100, 100, 100, 255, 7, 7];
        apply_rgba(Style::Dark, TINT, &mut buf);
        assert!(buf[0] < 200 && buf[4] < 100);
        assert_eq!(&buf[8..], &[7, 7]);
    }
}

// ---- geometric styles -------------------------------------------------------------------------

/// Bounds (x0, y0, x1, y1, inclusive) of the pixels that are visible.
fn visible_bounds(bgra: &[u8], size: usize) -> Option<(usize, usize, usize, usize)> {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..size {
        for x in 0..size {
            if bgra[(y * size + x) * 4 + 3] > 12 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x0 != usize::MAX).then_some((x0, y0, x1, y1))
}

/// Bilinear sample of premultiplied BGRA at a continuous position (pixel centres at +0.5).
fn sample(src: &[u8], size: usize, x: f32, y: f32) -> [f32; 4] {
    let (fx, fy) = (x - 0.5, y - 0.5);
    let (ix, iy) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - ix, fy - iy);
    let mut out = [0.0f32; 4];
    for (dy, wy) in [(0.0, 1.0 - ty), (1.0, ty)] {
        for (dx, wx) in [(0.0, 1.0 - tx), (1.0, tx)] {
            let (px, py) = (ix + dx, iy + dy);
            if px < 0.0 || py < 0.0 || px >= size as f32 || py >= size as f32 {
                continue;
            }
            let at = (py as usize * size + px as usize) * 4;
            for (k, o) in out.iter_mut().enumerate() {
                *o += f32::from(src[at + k]) * wx * wy;
            }
        }
    }
    out
}

/// Draws `src` scaled so its visible bounds fit a `target` x `target` box centred in the canvas,
/// over whatever `dst` already holds (premultiplied "over").
fn draw_fitted(
    dst: &mut [f32],
    src: &[u8],
    size: usize,
    bounds: (usize, usize, usize, usize),
    target: f32,
) {
    let (x0, y0, x1, y1) = bounds;
    let side = ((x1 - x0 + 1).max(y1 - y0 + 1)) as f32;
    let scale = target / side;
    let (cx, cy) = ((x0 + x1 + 1) as f32 / 2.0, (y0 + y1 + 1) as f32 / 2.0);
    let mid = size as f32 / 2.0;
    for dy in 0..size {
        for dx in 0..size {
            let sx = cx + (dx as f32 + 0.5 - mid) / scale;
            let sy = cy + (dy as f32 + 0.5 - mid) / scale;
            let p = sample(src, size, sx, sy);
            let at = (dy * size + dx) * 4;
            let inv = 1.0 - p[3] / 255.0;
            for k in 0..4 {
                dst[at + k] = p[k] + dst[at + k] * inv;
            }
        }
    }
}

/// The geometric styles, on a premultiplied BGRA bitmap of `size` x `size`. `Normalized` fills 92 %
/// of the square with the icon's visible part, so a small glyph and a full-bleed one end up the same
/// size. `Squircle` shrinks the icon onto a rounded plate: dark under a light icon, light under a
/// dark one, so it never disappears. Every other style returns the input unchanged.
#[must_use]
pub fn reshape(style: Style, bgra: &[u8], size: usize) -> Vec<u8> {
    if !matches!(style, Style::Squircle | Style::Normalized)
        || bgra.len() < size * size * 4
        || size < 8
    {
        return bgra.to_vec();
    }
    let Some(bounds) = visible_bounds(bgra, size) else {
        return bgra.to_vec();
    };
    let mut canvas = vec![0.0f32; size * size * 4];
    if style == Style::Squircle {
        // Plate colour from the icon's mean brightness.
        let (mut sum, mut n) = (0.0f32, 0.0f32);
        for px in bgra.chunks_exact(4).filter(|p| p[3] > 12) {
            let a = f32::from(px[3]) / 255.0;
            sum += luma(
                f32::from(px[2]) / 255.0 / a,
                f32::from(px[1]) / 255.0 / a,
                f32::from(px[0]) / 255.0 / a,
            );
            n += 1.0;
        }
        let light_icon = n > 0.0 && sum / n > 0.62;
        let (top, bottom) = if light_icon {
            (62.0, 34.0)
        } else {
            (246.0, 218.0)
        };
        let half = size as f32 * 0.46;
        let mid = size as f32 / 2.0;
        for y in 0..size {
            for x in 0..size {
                let (u, v) = ((x as f32 + 0.5 - mid) / half, (y as f32 + 0.5 - mid) / half);
                let d = (u.abs().powi(5) + v.abs().powi(5)).powf(0.2);
                let cover = ((1.0 - d) * half * 0.9 + 0.5).clamp(0.0, 1.0);
                if cover <= 0.0 {
                    continue;
                }
                let g = mix(top, bottom, y as f32 / size as f32) * cover;
                let at = (y * size + x) * 4;
                canvas[at] = g;
                canvas[at + 1] = g;
                canvas[at + 2] = g;
                canvas[at + 3] = 255.0 * cover;
            }
        }
        draw_fitted(&mut canvas, bgra, size, bounds, size as f32 * 0.6);
    } else {
        draw_fitted(&mut canvas, bgra, size, bounds, size as f32 * 0.92);
    }
    canvas
        .iter()
        .map(|v| v.round().clamp(0.0, 255.0) as u8)
        .collect()
}

#[cfg(test)]
mod reshape_tests {
    use super::*;

    fn square(size: usize, from: usize, to: usize, bgra: [u8; 4]) -> Vec<u8> {
        let mut v = vec![0u8; size * size * 4];
        for y in from..to {
            for x in from..to {
                v[(y * size + x) * 4..(y * size + x) * 4 + 4].copy_from_slice(&bgra);
            }
        }
        v
    }

    #[test]
    fn other_styles_are_untouched() {
        let icon = square(32, 8, 24, [10, 20, 200, 255]);
        assert_eq!(reshape(Style::Default, &icon, 32), icon);
        assert_eq!(reshape(Style::Dark, &icon, 32), icon);
        assert_eq!(
            reshape(Style::Normalized, &vec![0; 32 * 32 * 4], 32),
            vec![0; 32 * 32 * 4]
        );
    }

    #[test]
    fn normalized_scales_a_small_icon_up_and_keeps_it_centred() {
        let icon = square(64, 24, 40, [0, 0, 255, 255]);
        let out = reshape(Style::Normalized, &icon, 64);
        let (x0, y0, x1, y1) = visible_bounds(&out, 64).expect("visible");
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        assert!((57..=63).contains(&w) && (57..=63).contains(&h), "{w}x{h}");
        assert!((x0 as i32 - (63 - x1) as i32).abs() <= 1, "centred");
        assert_eq!(out[(32 * 64 + 32) * 4 + 2], 255, "colour kept");
    }

    #[test]
    fn squircle_has_round_corners_and_a_plate() {
        let icon = square(64, 16, 48, [255, 255, 255, 255]);
        let out = reshape(Style::Squircle, &icon, 64);
        assert_eq!(out[3], 0, "corner is empty");
        let edge = (32 * 64 + 4) * 4;
        assert!(
            out[edge + 3] > 200,
            "plate reaches near the edge: {}",
            out[edge + 3]
        );
        // A white icon sits on a dark plate, so the plate beside it is dark.
        let beside = (32 * 64 + 8) * 4;
        assert!(
            out[beside] < 100 && out[beside + 3] > 200,
            "dark plate: {:?}",
            &out[beside..beside + 4]
        );
        // A dark icon gets a light plate.
        let dark = square(64, 16, 48, [20, 20, 20, 255]);
        let out = reshape(Style::Squircle, &dark, 64);
        assert!(out[beside] > 180, "light plate: {}", out[beside]);
    }
}
