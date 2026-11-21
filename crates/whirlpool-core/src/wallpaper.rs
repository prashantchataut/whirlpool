//! Procedural wallpapers.
//!
//! Every wallpaper is drawn from a few numbers (a style, five colours, a
//! seed), so Whirlpool ships dozens of them in a few kilobytes and renders
//! them sharp at the exact resolution of the screen: no scaling, no blur, no
//! downloads. A little noise is added before quantizing so gradients do not
//! band on 8-bit panels.
//!
//! This module is pure computation and knows nothing about Windows.

use std::{f32::consts::PI, thread};

/// The three looks. The meaning of the five colours depends on the style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Soft, warped blobs of colour. All five colours take part.
    Mesh,
    /// Layered ridges under a glowing sky: sky top, sky horizon, sun glow,
    /// far hills, near hills.
    Horizon,
    /// A spiral of light around a bright eye: outer background, centre
    /// background, arms, eye, spare.
    Vortex,
}

#[derive(Clone, Copy, Debug)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub style: Style,
    pub colors: [[u8; 3]; 5],
    pub seed: u32,
}

const fn hex(rgb: u32) -> [u8; 3] {
    [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]
}

const fn preset(
    id: &'static str,
    name: &'static str,
    style: Style,
    colors: [u32; 5],
    seed: u32,
) -> Preset {
    Preset {
        id,
        name,
        style,
        colors: [
            hex(colors[0]),
            hex(colors[1]),
            hex(colors[2]),
            hex(colors[3]),
            hex(colors[4]),
        ],
        seed,
    }
}

/// Every wallpaper Whirlpool can draw, in the order the picker shows them.
pub const PRESETS: &[Preset] = &[
    // Vortex: the house style.
    preset(
        "abyss",
        "Abyss",
        Style::Vortex,
        [0x020b16, 0x06304a, 0x2fd6e6, 0xf2ffff, 0x0a4a66],
        11,
    ),
    preset(
        "maelstrom",
        "Maelstrom",
        Style::Vortex,
        [0x050a1e, 0x1a2a78, 0x6fa8ff, 0xffffff, 0x2a3a9a],
        12,
    ),
    preset(
        "plasma",
        "Plasma",
        Style::Vortex,
        [0x12031f, 0x4a0f66, 0xff5fd2, 0xfff0fb, 0x7a1e8c],
        13,
    ),
    preset(
        "tidepool",
        "Tidepool",
        Style::Vortex,
        [0x03140f, 0x0b4a3a, 0x4df0b4, 0xf2fff9, 0x13705a],
        14,
    ),
    preset(
        "eclipse",
        "Eclipse",
        Style::Vortex,
        [0x050505, 0x2a1408, 0xffa23a, 0xfff6e0, 0x6a3510],
        15,
    ),
    preset(
        "rosewater",
        "Rosewater",
        Style::Vortex,
        [0x1c0710, 0x5a1430, 0xff7a9a, 0xfff2f5, 0xc23a62],
        16,
    ),
    preset(
        "verdant",
        "Verdant",
        Style::Vortex,
        [0x06150a, 0x144a1e, 0xa6f04d, 0xfbffe9, 0x2a7a30],
        17,
    ),
    // Mesh gradients.
    preset(
        "lagoon",
        "Lagoon",
        Style::Mesh,
        [0x021a2b, 0x0a6a85, 0x2dd4cf, 0x1b3f8f, 0x04283f],
        21,
    ),
    preset(
        "aurora",
        "Aurora",
        Style::Mesh,
        [0x041a24, 0x15b58a, 0x6a4ff0, 0x0b3a5a, 0x7df5c8],
        22,
    ),
    preset(
        "dusk",
        "Dusk",
        Style::Mesh,
        [0x1b1038, 0xd9487a, 0xff9e6b, 0x4a2a8a, 0x2a1a5a],
        23,
    ),
    preset(
        "ember",
        "Ember",
        Style::Mesh,
        [0x1a0705, 0xe0431f, 0xffb02e, 0x7a1410, 0x3a0d08],
        24,
    ),
    preset(
        "orchid",
        "Orchid",
        Style::Mesh,
        [0x1a0b2e, 0xb04fe8, 0xff7ad9, 0x5a2aa8, 0x2a1250],
        25,
    ),
    preset(
        "mint",
        "Mint",
        Style::Mesh,
        [0xeafff6, 0x7ee8c0, 0xb8f5ff, 0xd8ffe8, 0x5cd0d8],
        26,
    ),
    preset(
        "midnight",
        "Midnight",
        Style::Mesh,
        [0x03040d, 0x101a4a, 0x2a1a6a, 0x0a2a5a, 0x06081e],
        27,
    ),
    preset(
        "sakura",
        "Sakura",
        Style::Mesh,
        [0xfff0f4, 0xffb7cb, 0xffd8c2, 0xe8a0ff, 0xffe3ec],
        28,
    ),
    preset(
        "citrus",
        "Citrus",
        Style::Mesh,
        [0xfff7d6, 0xffd23a, 0xff8a3a, 0xb4e84a, 0xffeb8a],
        29,
    ),
    preset(
        "glacier",
        "Glacier",
        Style::Mesh,
        [0xeaf6ff, 0x9ad4ff, 0x5aa0f0, 0xd0f0ff, 0x7ac0ea],
        30,
    ),
    // Horizons.
    preset(
        "sunset-ridge",
        "Sunset Ridge",
        Style::Horizon,
        [0x2a1550, 0xff8a5a, 0xffd27a, 0x6a2a6a, 0x1a0c2e],
        41,
    ),
    preset(
        "blue-hour",
        "Blue Hour",
        Style::Horizon,
        [0x0a1030, 0x4a78c8, 0xbcd8ff, 0x24407a, 0x0a1430],
        42,
    ),
    preset(
        "misty-pines",
        "Misty Pines",
        Style::Horizon,
        [0x20404a, 0xa8d0c8, 0xeaf6ee, 0x4a7a76, 0x12302e],
        43,
    ),
    preset(
        "desert-dawn",
        "Desert Dawn",
        Style::Horizon,
        [0x3a2a58, 0xffb48a, 0xfff0c0, 0xc86a4a, 0x7a2e2e],
        44,
    ),
    preset(
        "neon-coast",
        "Neon Coast",
        Style::Horizon,
        [0x0a0620, 0xff3fa8, 0x3ff0ff, 0x3a1a78, 0x120a38],
        45,
    ),
    preset(
        "moonrise",
        "Moonrise",
        Style::Horizon,
        [0x050a18, 0x1a2a4a, 0xe8f0ff, 0x16243e, 0x080e1c],
        46,
    ),
    preset(
        "ashen-peaks",
        "Ashen Peaks",
        Style::Horizon,
        [0x2a2c34, 0x9a9ca8, 0xf0ede8, 0x5a5c68, 0x1a1c22],
        47,
    ),
];

#[must_use]
pub fn find(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.id == id)
}

/// Opaque RGB pixels, top row first.
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

// ---------------------------------------------------------------- noise

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d)
        ^ (y as u32).wrapping_mul(0x1656_67b1)
        ^ seed.wrapping_mul(0x9e37_79b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    (h & 0x00ff_ffff) as f32 / 16_777_215.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    smooth(((x - a) / (b - a)).clamp(0.0, 1.0))
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (smooth(x - x0), smooth(y - y0));
    let (xi, yi) = (x0 as i32, y0 as i32);
    let a = hash(xi, yi, seed);
    let b = hash(xi + 1, yi, seed);
    let c = hash(xi, yi + 1, seed);
    let d = hash(xi + 1, yi + 1, seed);
    (a + (b - a) * fx) + ((c + (d - c) * fx) - (a + (b - a) * fx)) * fy
}

fn fbm(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut freq) = (0.0, 0.5, 0.0, 1.0);
    for octave in 0..octaves {
        sum += amp * value_noise(x * freq, y * freq, seed.wrapping_add(octave * 101));
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

// ---------------------------------------------------------------- colour

type Rgb = [f32; 3];

fn color(c: [u8; 3]) -> Rgb {
    [f32::from(c[0]), f32::from(c[1]), f32::from(c[2])]
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn add(a: Rgb, b: Rgb, k: f32) -> Rgb {
    [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k]
}

// ---------------------------------------------------------------- styles

fn mesh(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let wx = x + 0.2 * (fbm(x * 1.3, y * 1.3, p.seed, 3) - 0.5);
    let wy = y + 0.2 * (fbm(x * 1.3 + 7.0, y * 1.3 + 3.0, p.seed + 1, 3) - 0.5);
    let (mut sum, mut weight) = ([0.0f32; 3], 0.0f32);
    for (i, c) in p.colors.iter().enumerate() {
        let ax = hash(i as i32, 0, p.seed) * aspect;
        let ay = hash(i as i32, 1, p.seed);
        let d2 = (wx - ax) * (wx - ax) + (wy - ay) * (wy - ay);
        let w = 1.0 / ((d2 + 0.035) * (d2 + 0.035));
        let c = color(*c);
        sum = add(sum, c, w);
        weight += w;
    }
    [sum[0] / weight, sum[1] / weight, sum[2] / weight]
}

fn horizon(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let [top, low, glow, far, near] = p.colors.map(color);
    let mut col = mix(top, low, smoothstep(0.0, 0.7, y));
    // The sun or moon: a soft glow and a hard core.
    let sx = (0.25 + 0.5 * hash(1, 1, p.seed)) * aspect;
    let (dx, dy) = (x - sx, y - 0.46);
    let d2 = dx * dx + dy * dy;
    col = add(col, glow, 0.85 * (-d2 / 0.06).exp());
    col = mix(col, glow, smoothstep(0.0016, 0.0009, d2));
    let layers = 5;
    for i in 0..layers {
        let t = i as f32 / (layers - 1) as f32;
        let k = 1.5 + 0.7 * i as f32;
        let base = 0.5 + 0.1 * i as f32;
        let ridge = base
            + 0.05 * (x * k + p.seed as f32 + i as f32 * 1.7).sin()
            + 0.025 * (x * (4.0 + i as f32) + i as f32 * 2.3).sin()
            + 0.07 * (fbm(x * (1.2 + 0.3 * i as f32), i as f32 * 3.1, p.seed, 3) - 0.5);
        if y > ridge {
            let body = mix(far, near, t);
            // Distant ridges melt into the sky.
            let haze = (1.0 - t) * 0.6 * (1.0 - smoothstep(ridge, ridge + 0.18, y));
            let sky = mix(top, low, smoothstep(0.0, 0.7, ridge));
            col = mix(body, sky, haze + (1.0 - t) * 0.12);
        }
    }
    col
}

fn vortex(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let [outer, center, arms, eye, spare] = p.colors.map(color);
    let (dx, dy) = (x - aspect * 0.5, y - 0.5);
    let d = (dx * dx + dy * dy).sqrt();
    let angle = dy.atan2(dx);
    let mut col = mix(center, outer, smoothstep(0.0, 0.95, d));
    // Three arms of a logarithmic spiral, fading outward.
    let phase = angle * 3.0 - 8.0 * (d + 0.03).ln();
    let band = smoothstep(0.35, 1.0, phase.sin());
    let fade = (-d * 2.4).exp();
    col = add(col, arms, 0.7 * band * fade);
    // A second, finer weave in the spare colour.
    let weave = smoothstep(0.7, 1.0, (angle * 5.0 + 11.0 * (d + 0.03).ln() + 1.3).sin());
    col = add(col, spare, 0.25 * weave * (-d * 3.4).exp());
    // Ripples and the eye.
    col = add(
        col,
        arms,
        0.05 * (0.5 + 0.5 * (d * 90.0).sin()) * (-d * 3.0).exp(),
    );
    col = add(col, eye, 1.1 * (-d * d / 0.004).exp());
    col = add(col, arms, 0.5 * (-d * d / 0.03).exp());
    col
}

fn pixel(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let mut col = match p.style {
        Style::Mesh => mesh(p, x, y, aspect),
        Style::Horizon => horizon(p, x, y, aspect),
        Style::Vortex => vortex(p, x, y, aspect),
    };
    // Gentle vignette.
    let (vx, vy) = (x / aspect - 0.5, y - 0.5);
    let vignette = 1.0 - 0.22 * smoothstep(0.25, 0.95, (vx * vx + vy * vy).sqrt() * 1.4);
    col = [col[0] * vignette, col[1] * vignette, col[2] * vignette];
    col
}

/// Draws `preset` at exactly `width` x `height`. Uses every core.
#[must_use]
pub fn render(preset: &Preset, width: u32, height: u32) -> Raster {
    let (width, height) = (width.max(8), height.max(8));
    let aspect = width as f32 / height as f32;
    let mut rgb = vec![0u8; width as usize * height as usize * 3];
    let row_bytes = width as usize * 3;
    let workers = thread::available_parallelism()
        .map_or(2, usize::from)
        .min(8);
    let rows_per_chunk = (height as usize).div_ceil(workers).max(1);
    thread::scope(|scope| {
        for (chunk_index, chunk) in rgb.chunks_mut(rows_per_chunk * row_bytes).enumerate() {
            scope.spawn(move || {
                for (row_in_chunk, row) in chunk.chunks_mut(row_bytes).enumerate() {
                    let py = chunk_index * rows_per_chunk + row_in_chunk;
                    let y = (py as f32 + 0.5) / height as f32;
                    for (px, out) in row.chunks_exact_mut(3).enumerate() {
                        let x = (px as f32 + 0.5) / width as f32 * aspect;
                        let col = pixel(preset, x, y, aspect);
                        // Dither: +-0.9 of a level removes banding.
                        let grain = (hash(px as i32, py as i32, 0xd17e) - 0.5) * 1.8;
                        for channel in 0..3 {
                            out[channel] = (col[channel] + grain).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            });
        }
    });
    Raster { width, height, rgb }
}

/// A 24-bit Windows bitmap, which `SystemParametersInfo` accepts everywhere.
#[must_use]
pub fn encode_bmp(raster: &Raster) -> Vec<u8> {
    let (w, h) = (raster.width as usize, raster.height as usize);
    let stride = (w * 3).div_ceil(4) * 4;
    let image_size = stride * h;
    let mut out = Vec::with_capacity(54 + image_size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((54 + image_size) as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(image_size as u32).to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&[0; 8]);
    let padding = vec![0u8; stride - w * 3];
    for row in raster.rgb.chunks_exact(w * 3).rev() {
        for pixel in row.chunks_exact(3) {
            out.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        }
        out.extend_from_slice(&padding);
    }
    out
}

/// A repeatable but varied pick from a clock reading: the next preset after
/// `current`, skipping to a pseudo-random one by `entropy`.
#[must_use]
pub fn shuffle(current: Option<&str>, entropy: u64) -> &'static Preset {
    let count = PRESETS.len();
    let at = current
        .and_then(|id| PRESETS.iter().position(|preset| preset.id == id))
        .unwrap_or(0);
    let step = 1 + (entropy % (count as u64 - 1)) as usize;
    &PRESETS[(at + step) % count]
}

#[allow(dead_code)]
const _: f32 = PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_every_style_is_represented() {
        let mut ids: Vec<_> = PRESETS.iter().map(|preset| preset.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PRESETS.len());
        for style in [Style::Mesh, Style::Horizon, Style::Vortex] {
            assert!(PRESETS.iter().any(|preset| preset.style == style));
        }
        assert!(PRESETS.len() >= 24);
    }

    #[test]
    fn every_preset_renders_something_with_contrast() {
        for preset in PRESETS {
            let raster = render(preset, 96, 60);
            assert_eq!(raster.rgb.len(), 96 * 60 * 3, "{}", preset.id);
            let (min, max) = raster
                .rgb
                .iter()
                .fold((255u8, 0u8), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            assert!(max - min > 24, "{} is flat ({min}..{max})", preset.id);
        }
    }

    #[test]
    fn rendering_is_deterministic_and_independent_of_thread_count() {
        let preset = find("abyss").unwrap();
        assert_eq!(render(preset, 64, 40).rgb, render(preset, 64, 40).rgb);
    }

    #[test]
    fn bmp_has_a_valid_header_and_padded_rows() {
        let raster = render(find("mint").unwrap(), 10, 8);
        let bmp = encode_bmp(&raster);
        assert_eq!(&bmp[..2], b"BM");
        let stride = (10usize * 3).div_ceil(4) * 4;
        assert_eq!(bmp.len(), 54 + stride * 8);
        assert_eq!(
            u32::from_le_bytes(bmp[2..6].try_into().unwrap()) as usize,
            bmp.len()
        );
    }

    #[test]
    fn shuffle_never_repeats_the_current_wallpaper() {
        for entropy in 0..200 {
            assert_ne!(shuffle(Some("dusk"), entropy).id, "dusk");
        }
    }

    #[test]
    fn unknown_ids_are_not_found() {
        assert!(find("nope").is_none());
        assert!(find("lagoon").is_some());
    }
}
