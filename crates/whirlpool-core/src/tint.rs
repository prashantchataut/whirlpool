//! Colours taken from the wallpaper, so panels can match what is behind them.
//! Pure functions: the caller supplies decoded pixels.

/// What a wallpaper looks like at a glance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    /// Mean colour of the picture, sRGB.
    pub average: [u8; 3],
    /// The dominant saturated colour, if the picture has one.
    pub vivid: Option<[u8; 3]>,
    /// Mean relative luminance, 0 (black) to 1 (white).
    pub luma: f32,
}

/// Samples a BGRA bitmap (at most about 4,000 pixels are read).
#[must_use]
pub fn profile_from_bgra(width: usize, height: usize, bgra: &[u8]) -> Option<Profile> {
    let pixels = width.checked_mul(height)?;
    if pixels == 0 || bgra.len() < pixels * 4 {
        return None;
    }
    let step = (pixels / 4096).max(1);
    let (mut sum, mut count) = ([0.0f64; 3], 0.0f64);
    let (mut luma_sum, mut hue_x, mut hue_y, mut weight, mut sat_w, mut val_w) =
        (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut index = 0;
    while index < pixels {
        let at = index * 4;
        let (b, g, r) = (
            f64::from(bgra[at]),
            f64::from(bgra[at + 1]),
            f64::from(bgra[at + 2]),
        );
        sum[0] += r;
        sum[1] += g;
        sum[2] += b;
        count += 1.0;
        luma_sum += f64::from(luminance([r as u8, g as u8, b as u8]));
        let (hue, saturation, value) = to_hsv([r as u8, g as u8, b as u8]);
        if saturation > 0.25 && value > 0.2 {
            let w = saturation * saturation * (0.3 + value);
            let angle = hue.to_radians();
            hue_x += angle.cos() * w;
            hue_y += angle.sin() * w;
            weight += w;
            sat_w += saturation * w;
            val_w += value * w;
        }
        index += step;
    }
    let average = [
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
    ];
    // Needs real colour in a meaningful share of the samples.
    let vivid = (weight / count > 0.04).then(|| {
        let hue = hue_y.atan2(hue_x).to_degrees().rem_euclid(360.0);
        from_hsv(
            hue,
            (sat_w / weight).clamp(0.45, 0.9),
            (val_w / weight).clamp(0.5, 0.95),
        )
    });
    Some(Profile {
        average,
        vivid,
        luma: (luma_sum / count) as f32,
    })
}

/// Relative luminance (WCAG).
#[must_use]
pub fn luminance(rgb: [u8; 3]) -> f32 {
    let channel = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])
}

/// WCAG contrast ratio, 1 to 21.
#[must_use]
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Turns the wallpaper's vivid colour into a fill that carries text: returns `(fill, ink)` where
/// `ink` (white or near-black) has at least 4.5:1 contrast on `fill`.
#[must_use]
pub fn accent_with_ink(vivid: [u8; 3]) -> ([u8; 3], [u8; 3]) {
    const WHITE: [u8; 3] = [255, 255, 255];
    const BLACK: [u8; 3] = [16, 20, 24];
    let (hue, saturation, _) = to_hsv(vivid);
    let bright = from_hsv(hue, saturation.min(0.75), 0.95);
    // Light, bright hues (yellow, lime, cyan) keep their brightness and take dark ink.
    if contrast(bright, BLACK) >= 9.0 {
        return (bright, BLACK);
    }
    let mut value = 0.95;
    while value > 0.2 {
        let fill = from_hsv(hue, saturation, value);
        if contrast(fill, WHITE) >= 4.5 {
            return (fill, WHITE);
        }
        value -= 0.02;
    }
    (from_hsv(hue, saturation, 0.2), WHITE)
}

/// Mixes `a` toward `b` by `t` (0 to 1).
#[must_use]
pub fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    let one = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    [one(a[0], b[0]), one(a[1], b[1]), one(a[2], b[2])]
}

fn to_hsv(rgb: [u8; 3]) -> (f64, f64, f64) {
    let (r, g, b) = (
        f64::from(rgb[0]) / 255.0,
        f64::from(rgb[1]) / 255.0,
        f64::from(rgb[2]) / 255.0,
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (hue, if max == 0.0 { 0.0 } else { delta / max }, max)
}

fn from_hsv(hue: f64, saturation: f64, value: f64) -> [u8; 3] {
    let c = value * saturation;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = value - c;
    let (r, g, b) = match (hue / 60.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(rgb: [u8; 3]) -> Vec<u8> {
        (0..64 * 64)
            .flat_map(|_| [rgb[2], rgb[1], rgb[0], 255])
            .collect()
    }

    #[test]
    fn gray_wallpapers_have_no_vivid_colour() {
        let profile = profile_from_bgra(64, 64, &flat([120, 120, 120])).expect("profile");
        assert_eq!(profile.vivid, None);
        assert_eq!(profile.average, [120, 120, 120]);
    }

    #[test]
    fn a_blue_wallpaper_gives_a_blue_accent() {
        let profile = profile_from_bgra(64, 64, &flat([30, 90, 220])).expect("profile");
        let vivid = profile.vivid.expect("vivid");
        assert!(vivid[2] > vivid[0] + 60, "{vivid:?}");
    }

    #[test]
    fn accents_always_carry_readable_text() {
        for hue in (0..360).step_by(15) {
            for saturation in [0.45, 0.7, 0.9] {
                let vivid = from_hsv(f64::from(hue), saturation, 0.8);
                let (fill, ink) = accent_with_ink(vivid);
                assert!(
                    contrast(fill, ink) >= 4.5,
                    "hue {hue} sat {saturation}: {fill:?} on {ink:?}"
                );
            }
        }
    }

    #[test]
    fn rejects_short_buffers() {
        assert!(profile_from_bgra(10, 10, &[0; 8]).is_none());
        assert!(profile_from_bgra(0, 0, &[]).is_none());
    }

    #[test]
    fn mixes() {
        assert_eq!(mix([0, 0, 0], [200, 100, 50], 0.5), [100, 50, 25]);
        assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
    }
}
