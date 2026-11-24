//! Whirlpool's own glass: a software refraction shader.
//!
//! Given a picture of what is behind a panel, it draws the panel as a thick rounded lens: the middle
//! shows the backdrop plainly (after an optional frost), and a bevel along the rim bends the view so
//! the backdrop beyond the edge is pulled in, with colour fringing and a soft highlight. It runs once
//! when a panel opens (not per frame) on the CPU, so it needs no GPU feature and no extra process.

/// An RGBA8 picture, row-major, straight alpha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            rgba: vec![0; width * height * 4],
        }
    }

    fn px(&self, x: i64, y: i64) -> [f32; 3] {
        let xi = x.clamp(0, self.width as i64 - 1) as usize;
        let yi = y.clamp(0, self.height as i64 - 1) as usize;
        let i = (yi * self.width + xi) * 4;
        [
            f32::from(self.rgba[i]),
            f32::from(self.rgba[i + 1]),
            f32::from(self.rgba[i + 2]),
        ]
    }

    /// Bilinear sample at a fractional position.
    fn sample(&self, x: f32, y: f32) -> [f32; 3] {
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (xi, yi) = (x0 as i64, y0 as i64);
        let (a, b, c, d) = (
            self.px(xi, yi),
            self.px(xi + 1, yi),
            self.px(xi, yi + 1),
            self.px(xi + 1, yi + 1),
        );
        let mut out = [0.0; 3];
        for k in 0..3 {
            let top = a[k] + (b[k] - a[k]) * fx;
            let bottom = c[k] + (d[k] - c[k]) * fx;
            out[k] = top + (bottom - top) * fy;
        }
        out
    }
}

/// Look of the glass. All lengths are in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    /// Corner radius of the panel.
    pub radius: f32,
    /// Width of the bending rim.
    pub bevel: f32,
    /// Most the view is displaced, at the very edge.
    pub strength: f32,
    /// Extra displacement of red and less of blue, as a fraction (0.0 = no colour fringe).
    pub dispersion: f32,
    /// Box-blur radius applied to the backdrop first (0 = none).
    pub frost: usize,
    /// Brightness added along the lit rim, 0.0..1.0.
    pub highlight: f32,
}

impl Glass {
    /// A look scaled by one control, 0 (flat) to 100 (strong).
    pub fn from_percent(percent: u8, radius: f32) -> Self {
        let k = f32::from(percent.min(100)) / 100.0;
        Self {
            radius,
            bevel: (radius * 1.1).max(10.0),
            strength: 26.0 * k,
            dispersion: 0.22 * k,
            frost: (6.0 + 6.0 * (1.0 - k)) as usize,
            highlight: 0.28 * k,
        }
    }
}

/// Signed distance from `(px, py)` to the edge of a rounded rectangle `(w, h, r)` centred on the
/// origin: negative inside. Also gives the outward direction.
fn rounded_rect_sdf(px: f32, py: f32, w: f32, h: f32, r: f32) -> (f32, f32, f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    let (qx, qy) = (px.abs() - (w / 2.0 - r), py.abs() - (h / 2.0 - r));
    let (ox, oy) = (qx.max(0.0), qy.max(0.0));
    let outside = (ox * ox + oy * oy).sqrt();
    let dist = outside + qx.max(qy).min(0.0) - r;
    // Outward direction.
    let (mut dx, mut dy) = if outside > 1e-4 {
        (ox / outside, oy / outside)
    } else if qx > qy {
        (1.0, 0.0)
    } else {
        (0.0, 1.0)
    };
    dx *= px.signum();
    dy *= py.signum();
    (dist, dx, dy)
}

/// A frosted copy of `src`: averaged down by `factor` (so it is smaller), then box-blurred three
/// times, which is close to a Gaussian. Draw it stretched over the area it was taken from; the
/// stretch is part of the blur. Runs once when a panel opens.
pub fn frost(src: &Image, factor: usize, radius: usize) -> Image {
    if src.width == 0 || src.height == 0 {
        return src.clone();
    }
    let factor = factor.max(1);
    let (w, h) = ((src.width / factor).max(1), (src.height / factor).max(1));
    let mut small = Image::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0u32; 3];
            let mut count = 0u32;
            for dy in 0..factor {
                for dx in 0..factor {
                    let sx = (x * factor + dx).min(src.width - 1);
                    let sy = (y * factor + dy).min(src.height - 1);
                    let i = (sy * src.width + sx) * 4;
                    for (channel, total) in sum.iter_mut().enumerate() {
                        *total += u32::from(src.rgba[i + channel]);
                    }
                    count += 1;
                }
            }
            let o = (y * w + x) * 4;
            for (channel, total) in sum.iter().enumerate() {
                small.rgba[o + channel] = (total / count) as u8;
            }
            small.rgba[o + 3] = 255;
        }
    }
    let mut out = small;
    for _ in 0..3 {
        out = box_blur(&out, radius);
    }
    out
}

/// Separable box blur (edge pixels repeat). Returns a copy.
pub fn box_blur(src: &Image, radius: usize) -> Image {
    if radius == 0 || src.width == 0 || src.height == 0 {
        return src.clone();
    }
    let (w, h) = (src.width, src.height);
    let n = (2 * radius + 1) as f32;
    let mut tmp = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for c in 0..3 {
            let mut acc = 0.0;
            for dx in -(radius as i64)..=radius as i64 {
                acc += src.px(dx, y as i64)[c];
            }
            for x in 0..w {
                tmp[(y * w + x) * 3 + c] = acc / n;
                acc += src.px((x + radius + 1) as i64, y as i64)[c];
                acc -= src.px(x as i64 - radius as i64, y as i64)[c];
            }
        }
    }
    let at = |x: i64, y: i64, c: usize| {
        tmp[(y.clamp(0, h as i64 - 1) as usize * w + x.clamp(0, w as i64 - 1) as usize) * 3 + c]
    };
    let mut out = Image::new(w, h);
    for x in 0..w {
        for c in 0..3 {
            let mut acc = 0.0;
            for dy in -(radius as i64)..=radius as i64 {
                acc += at(x as i64, dy, c);
            }
            for y in 0..h {
                out.rgba[(y * w + x) * 4 + c] = (acc / n + 0.5) as u8;
                acc += at(x as i64, (y + radius + 1) as i64, c);
                acc -= at(x as i64, y as i64 - radius as i64, c);
            }
        }
    }
    for i in 0..w * h {
        out.rgba[i * 4 + 3] = 255;
    }
    out
}

/// How far the view bends at depth `t` into the bevel (0 at the inner edge, 1 at the rim): a smooth
/// convex profile, flat where the bevel starts and steepest at the rim.
pub fn bend(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t
}

/// Draws the panel `(x, y, w, h)` of `backdrop` as glass. The result is `w` by `h`; pixels outside the
/// rounded corners are fully transparent, with a one-pixel soft edge.
pub fn refract(backdrop: &Image, rect: (usize, usize, usize, usize), glass: &Glass) -> Image {
    let (rx, ry, rw, rh) = rect;
    let mut out = Image::new(rw, rh);
    if rw == 0 || rh == 0 || backdrop.width == 0 || backdrop.height == 0 {
        return out;
    }
    let source = box_blur(backdrop, glass.frost);
    let (cx, cy) = (rw as f32 / 2.0, rh as f32 / 2.0);
    for y in 0..rh {
        for x in 0..rw {
            let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let (dist, ox, oy) = rounded_rect_sdf(px, py, rw as f32, rh as f32, glass.radius);
            let depth = -dist; // pixels inside the edge
            let coverage = (depth + 0.5).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }
            let t = if depth >= glass.bevel {
                0.0
            } else {
                1.0 - depth / glass.bevel
            };
            let pull = bend(t) * glass.strength;
            let (sx, sy) = (rx as f32 + x as f32 + 0.5, ry as f32 + y as f32 + 0.5);
            let chan = |k: f32, c: usize| {
                let d = pull * k;
                source.sample(sx + ox * d - 0.5, sy + oy * d - 0.5)[c]
            };
            let (r, g, b) = (
                chan(1.0 + glass.dispersion, 0),
                chan(1.0, 1),
                chan(1.0 - glass.dispersion, 2),
            );
            // A soft highlight on the rim facing the light (up and to the left).
            let facing = (-ox * 0.6 - oy * 0.8).max(0.0);
            let lit = glass.highlight * 255.0 * facing * t.powi(3);
            let i = (y * rw + x) * 4;
            out.rgba[i] = (r + lit).clamp(0.0, 255.0) as u8;
            out.rgba[i + 1] = (g + lit).clamp(0.0, 255.0) as u8;
            out.rgba[i + 2] = (b + lit).clamp(0.0, 255.0) as u8;
            out.rgba[i + 3] = (coverage * 255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frost_keeps_a_flat_colour_and_shrinks_the_picture() {
        let mut flat = Image::new(64, 32);
        for px in flat.rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[40, 120, 200, 255]);
        }
        let out = frost(&flat, 4, 3);
        assert_eq!((out.width, out.height), (16, 8));
        assert!(out.rgba.chunks_exact(4).all(|p| p[..3] == [40, 120, 200]));
    }

    #[test]
    fn frost_spreads_a_bright_dot() {
        let mut dot = Image::new(32, 32);
        for px in dot.rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 255]);
        }
        let centre = (16 * 32 + 16) * 4;
        dot.rgba[centre..centre + 3].copy_from_slice(&[255, 255, 255]);
        let out = frost(&dot, 2, 2);
        let max = out.rgba.chunks_exact(4).map(|p| p[0]).max().unwrap_or(0);
        assert!(max < 255, "the dot is softened, got {max}");
        assert!(max > 0, "and still there");
    }

    #[test]
    fn frost_of_nothing_is_nothing() {
        assert_eq!(frost(&Image::new(0, 0), 4, 2).width, 0);
    }

    /// A horizontal ramp: red grows with x, green grows with y.
    fn ramp(w: usize, h: usize) -> Image {
        let mut img = Image::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                img.rgba[i] = (x * 255 / (w - 1)) as u8;
                img.rgba[i + 1] = (y * 255 / (h - 1)) as u8;
                img.rgba[i + 2] = 90;
                img.rgba[i + 3] = 255;
            }
        }
        img
    }

    fn glass(strength: f32, dispersion: f32) -> Glass {
        Glass {
            radius: 24.0,
            bevel: 24.0,
            strength,
            dispersion,
            frost: 0,
            highlight: 0.0,
        }
    }

    #[test]
    fn centre_should_show_the_backdrop_unchanged() {
        let back = ramp(200, 120);
        let out = refract(&back, (40, 20, 120, 80), &glass(20.0, 0.2));
        let i = ((80 / 2) * 120 + 60) * 4;
        let j = ((20 + 40) * 200 + 40 + 60) * 4;
        assert_eq!(&out.rgba[i..i + 3], &back.rgba[j..j + 3]);
        assert_eq!(out.rgba[i + 3], 255);
    }

    #[test]
    fn zero_strength_should_be_a_plain_cut_out() {
        let back = ramp(200, 120);
        let out = refract(&back, (40, 20, 120, 80), &glass(0.0, 0.0));
        let (x, y) = (3usize, 40usize);
        let i = (y * 120 + x) * 4;
        let j = ((20 + y) * 200 + 40 + x) * 4;
        assert_eq!(&out.rgba[i..i + 3], &back.rgba[j..j + 3]);
    }

    #[test]
    fn rim_should_pull_in_content_from_beyond_the_edge() {
        let back = ramp(200, 120);
        let flat = refract(&back, (40, 20, 120, 80), &glass(0.0, 0.0));
        let bent = refract(&back, (40, 20, 120, 80), &glass(20.0, 0.0));
        // Left rim, middle height: bending samples further left, so red is lower than the plain cut.
        let i = (40 * 120 + 2) * 4;
        assert!(
            bent.rgba[i] < flat.rgba[i],
            "{} !< {}",
            bent.rgba[i],
            flat.rgba[i]
        );
    }

    #[test]
    fn dispersion_should_separate_colour_channels_at_the_rim_only() {
        let back = ramp(200, 120);
        let plain = refract(&back, (40, 20, 120, 80), &glass(20.0, 0.0));
        let split = refract(&back, (40, 20, 120, 80), &glass(20.0, 0.4));
        let rim = (40 * 120 + 3) * 4;
        assert_ne!(plain.rgba[rim], split.rgba[rim]);
        let mid = (40 * 120 + 60) * 4;
        assert_eq!(plain.rgba[mid], split.rgba[mid]);
    }

    #[test]
    fn corners_should_be_transparent_and_the_inside_opaque() {
        let back = ramp(200, 120);
        let out = refract(&back, (40, 20, 120, 80), &glass(10.0, 0.1));
        assert_eq!(out.rgba[3], 0);
        let i = (40 * 120 + 60) * 4;
        assert_eq!(out.rgba[i + 3], 255);
    }

    #[test]
    fn sdf_should_be_negative_inside_and_zero_on_the_edge() {
        assert!(rounded_rect_sdf(0.0, 0.0, 100.0, 60.0, 10.0).0 < -25.0);
        assert!(rounded_rect_sdf(50.0, 0.0, 100.0, 60.0, 10.0).0.abs() < 1e-3);
        assert!(rounded_rect_sdf(60.0, 0.0, 100.0, 60.0, 10.0).0 > 9.0);
    }

    #[test]
    fn bend_should_be_flat_at_the_start_and_grow_to_one_at_the_rim() {
        assert_eq!(bend(0.0), 0.0);
        assert!((bend(1.0) - 1.0).abs() < 1e-5);
        assert!(bend(0.5) > bend(0.25));
    }

    #[test]
    fn box_blur_should_average_and_keep_flat_images() {
        let mut flat = Image::new(10, 10);
        flat.rgba.iter_mut().for_each(|v| *v = 100);
        let blurred_flat = box_blur(&flat, 3);
        assert!(
            blurred_flat
                .rgba
                .chunks(4)
                .all(|p| p[..3] == [100, 100, 100])
        );
        let mut spike = Image::new(9, 1);
        spike.rgba.iter_mut().for_each(|v| *v = 0);
        spike.rgba[4 * 4] = 255;
        let blurred = box_blur(&spike, 1);
        assert!(blurred.rgba[4 * 4] < 255 && blurred.rgba[3 * 4] > 0);
    }

    #[test]
    fn from_percent_should_scale_the_look() {
        assert_eq!(Glass::from_percent(0, 20.0).strength, 0.0);
        assert!(Glass::from_percent(100, 20.0).strength > Glass::from_percent(50, 20.0).strength);
    }
}
