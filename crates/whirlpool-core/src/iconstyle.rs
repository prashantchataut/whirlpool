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
}

fn luma(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Styles one pixel. Fully transparent pixels stay fully transparent.
pub fn style_pixel(style: Style, tint: [u8; 3], px: [u8; 4]) -> [u8; 4] {
    if px[3] == 0 || style == Style::Default {
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
        Style::Default => (r, g, b, a),
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
