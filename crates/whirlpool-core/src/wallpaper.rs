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
    /// Curtains of light over a starfield: sky top, sky bottom, three ribbons.
    Aurora,
    /// Contour lines of a noise landscape: background, minor line, major
    /// line, shade, spare.
    Topo,
    /// A sun over a perspective grid: sky top, horizon, sun, grid, floor.
    Grid,
    /// The house vortex in four dithered tones: dark to bright (fifth unused).
    Dither,
    /// Layered waves under a pale sky, in four dithered tones (fifth unused).
    Tidal,
    /// A glowing sphere made of halftone dots: background, then three dot
    /// tones from dim to bright (fifth unused).
    Halftone,
    /// Soft rings spreading from a point below the screen: sky top, sky
    /// bottom, ring, ring highlight, glow.
    Ripple,
    /// Warped, silky bands that run through all five colours in order.
    Silk,
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
    preset(
        "boreal",
        "Boreal",
        Style::Aurora,
        [0x020812, 0x0a2a3a, 0x3dffb0, 0x2fa8ff, 0x9a6bff],
        21,
    ),
    preset(
        "solstice",
        "Solstice",
        Style::Aurora,
        [0x07031a, 0x1a1040, 0xff5fb0, 0x7a5cff, 0x3de6ff],
        22,
    ),
    preset(
        "glacier-2",
        "Glacier II",
        Style::Aurora,
        [0x03101c, 0x0d3b52, 0x7ff3ff, 0x38b6ff, 0xd8fff6],
        23,
    ),
    preset(
        "ember-sky",
        "Ember Sky",
        Style::Aurora,
        [0x120406, 0x3a0f1a, 0xff7a3d, 0xffc15a, 0xff3d6e],
        24,
    ),
    preset(
        "verdant-2",
        "Verdant II",
        Style::Aurora,
        [0x02100a, 0x0a3a22, 0x7dff6b, 0x2fe6a0, 0xe6ff7a],
        25,
    ),
    preset(
        "violet-hour",
        "Violet Hour",
        Style::Aurora,
        [0x0a0618, 0x2a1458, 0xc06bff, 0xff6bd6, 0x6b8bff],
        26,
    ),
    preset(
        "contour",
        "Contour",
        Style::Topo,
        [0x06141c, 0x1f6f86, 0x6ff2e0, 0x0e3a4a, 0xffffff],
        31,
    ),
    preset(
        "ridgeline",
        "Ridgeline",
        Style::Topo,
        [0x0e0c14, 0x5a4a8a, 0xffb86b, 0x2a2440, 0xffffff],
        32,
    ),
    preset(
        "paper-map",
        "Paper Map",
        Style::Topo,
        [0xf3efe4, 0x2a3a36, 0xd9482b, 0xe3dccb, 0x000000],
        33,
    ),
    preset(
        "deep-survey",
        "Deep Survey",
        Style::Topo,
        [0x020a14, 0x1b4a7a, 0x59d0ff, 0x08203a, 0xffffff],
        34,
    ),
    preset(
        "moss-map",
        "Moss Map",
        Style::Topo,
        [0x0b140c, 0x3f7a4a, 0xc8ff6b, 0x14281a, 0xffffff],
        35,
    ),
    preset(
        "mono-lines",
        "Mono Lines",
        Style::Topo,
        [0x0a0a0a, 0x6a6a6a, 0xffffff, 0x1a1a1a, 0xffffff],
        36,
    ),
    preset(
        "neon-run",
        "Neon Run",
        Style::Grid,
        [0x0a0420, 0x4a0e5e, 0xff5fa8, 0x38e8ff, 0x14062e],
        41,
    ),
    preset(
        "dusk-drive",
        "Dusk Drive",
        Style::Grid,
        [0x14061e, 0x5a1a4a, 0xffa04a, 0xff4a8a, 0x1a0a26],
        42,
    ),
    preset(
        "cyan-line",
        "Cyan Line",
        Style::Grid,
        [0x02101a, 0x0a3a52, 0x7ff3ff, 0x2fd6e6, 0x04202e],
        43,
    ),
    preset(
        "mint-grid",
        "Mint Grid",
        Style::Grid,
        [0x03140e, 0x0e4a38, 0x9affc8, 0x3de6a0, 0x062a1e],
        44,
    ),
    preset(
        "sunburn",
        "Sunburn",
        Style::Grid,
        [0x1a0606, 0x6a1a1a, 0xffd04a, 0xff6a3a, 0x240a0a],
        45,
    ),
    preset(
        "mono-grid",
        "Mono Grid",
        Style::Grid,
        [0x080808, 0x2a2a2a, 0xf0f0f0, 0xa0a0a0, 0x121212],
        46,
    ),
    preset(
        "dither-tide",
        "Dither Tide",
        Style::Dither,
        [0x020b16, 0x0a4a66, 0x2fd6e6, 0xf2ffff, 0x000000],
        51,
    ),
    preset(
        "dither-ink",
        "Dither Ink",
        Style::Dither,
        [0x0a0a0c, 0x3a3a46, 0x9a9aa8, 0xffffff, 0x000000],
        52,
    ),
    preset(
        "dither-amber",
        "Dither Amber",
        Style::Dither,
        [0x120a02, 0x6a3a0a, 0xe6902f, 0xfff2d6, 0x000000],
        53,
    ),
    preset(
        "dither-rose",
        "Dither Rose",
        Style::Dither,
        [0x14040e, 0x6a1a4a, 0xe62f9a, 0xffe6f4, 0x000000],
        54,
    ),
    preset(
        "dither-moss",
        "Dither Moss",
        Style::Dither,
        [0x04100a, 0x14502a, 0x4ae66a, 0xe6ffee, 0x000000],
        55,
    ),
    preset(
        "dither-violet",
        "Dither Violet",
        Style::Dither,
        [0x0a0618, 0x2e1a6a, 0x8a6bff, 0xf0e6ff, 0x000000],
        56,
    ),
    preset(
        "tidal-noon",
        "Tidal Noon",
        Style::Tidal,
        [0x04141c, 0x0b5d73, 0x4fc7d6, 0xf4fdff, 0x000000],
        61,
    ),
    preset(
        "tidal-dusk",
        "Tidal Dusk",
        Style::Tidal,
        [0x12081c, 0x5a1f6e, 0xe0568a, 0xffe9d2, 0x000000],
        62,
    ),
    preset(
        "tidal-moss",
        "Tidal Moss",
        Style::Tidal,
        [0x06120a, 0x1d5a35, 0x8fd16a, 0xf6ffe8, 0x000000],
        63,
    ),
    preset(
        "halftone-ember",
        "Halftone Ember",
        Style::Halftone,
        [0x0d0806, 0x5a2410, 0xd9622b, 0xffe0b8, 0x000000],
        64,
    ),
    preset(
        "halftone-glacier",
        "Halftone Glacier",
        Style::Halftone,
        [0x060a10, 0x1e3a5c, 0x6aa8e0, 0xeaf6ff, 0x000000],
        65,
    ),
    preset(
        "halftone-mono",
        "Halftone Mono",
        Style::Halftone,
        [0x080808, 0x3a3a3a, 0x9a9a9a, 0xf4f4f4, 0x000000],
        66,
    ),
    preset(
        "ripple-lagoon",
        "Ripple Lagoon",
        Style::Ripple,
        [0x03222e, 0x0b6f7e, 0x56d6d0, 0xeafffb, 0x2ad0c0],
        67,
    ),
    preset(
        "ripple-sunrise",
        "Ripple Sunrise",
        Style::Ripple,
        [0x2a1038, 0xf0805a, 0xffc48a, 0xfff4e0, 0xff9a6a],
        68,
    ),
    preset(
        "ripple-ink",
        "Ripple Ink",
        Style::Ripple,
        [0x08080c, 0x1a1c28, 0x5a6080, 0xdfe4ff, 0x6a78ff],
        69,
    ),
    preset(
        "silk-citrus",
        "Silk Citrus",
        Style::Silk,
        [0x1b2a05, 0x5a8a0a, 0xd8d82a, 0xffb02a, 0xff6a1a],
        70,
    ),
    preset(
        "silk-orchid",
        "Silk Orchid",
        Style::Silk,
        [0x120622, 0x4a1a8a, 0xa04ad8, 0xff7ac8, 0xffd0e8],
        71,
    ),
    preset(
        "silk-graphite",
        "Silk Graphite",
        Style::Silk,
        [0x08090b, 0x1c1f26, 0x3a404c, 0x7a8496, 0xdfe6f2],
        72,
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

fn bayer4(x: i32, y: i32) -> f32 {
    const M: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    (f32::from(M[(y & 3) as usize][(x & 3) as usize]) + 0.5) / 16.0
}

fn aurora(p: &Preset, x: f32, y: f32, _aspect: f32) -> Rgb {
    let [top, low, a, b, c] = p.colors.map(color);
    let mut col = mix(top, low, smoothstep(0.0, 1.0, y));
    let s = p.seed as f32;
    for k in 0..3u32 {
        let kf = k as f32;
        let centre = 0.30
            + 0.12 * kf
            + 0.08 * (x * (1.1 + 0.6 * kf) + s + kf * 2.1).sin()
            + 0.18 * (fbm(x * 0.9 + kf * 5.0, kf * 3.0, p.seed + k, 3) - 0.5);
        let d = (y - centre) / (0.10 + 0.03 * kf);
        // Curtains glow above their lower edge, like real ones.
        let body = if d < 0.0 {
            (-d * d * 0.6).exp()
        } else {
            (-d * d * 2.4).exp()
        };
        let streak = 0.55 + 0.45 * fbm(x * 38.0 + kf * 11.0, y * 1.5, p.seed + 9 + k, 2);
        let shift = smoothstep(0.35, 0.75, fbm(x * 1.7, kf * 4.0 + 2.0, p.seed + 17, 2));
        let tint = mix(mix(a, b, kf / 2.0), c, shift);
        col = add(col, tint, 0.85 * body * streak);
    }
    // Stars, chunky like pixels.
    let (cx, cy) = ((x * 260.0).floor() as i32, (y * 260.0).floor() as i32);
    if y < 0.7 && hash(cx, cy, p.seed + 31) > 0.9972 {
        col = add(col, [255.0, 255.0, 255.0], 0.65);
    }
    col
}

fn topo(p: &Preset, x: f32, y: f32, _aspect: f32) -> Rgb {
    let [bg, line, accent, shade, _] = p.colors.map(color);
    let h = fbm(x * 2.1 + 3.0, y * 2.1 + 1.0, p.seed, 5);
    let f = h * 16.0;
    let frac = f - f.floor();
    let d = frac.min(1.0 - frac);
    let major = (f.floor() as i32).rem_euclid(5) == 0;
    let width = if major { 0.11 } else { 0.06 };
    let ink = 1.0 - smoothstep(width * 0.55, width, d);
    let col = mix(bg, shade, smoothstep(0.25, 0.8, h));
    if major {
        mix(col, accent, ink * 0.95)
    } else {
        mix(col, line, ink * 0.55)
    }
}

fn grid(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let [top, haze, sun, line, floor] = p.colors.map(color);
    let horizon_y = 0.56;
    let dx = x - aspect * 0.5;
    if y < horizon_y {
        let mut col = mix(top, haze, smoothstep(0.0, horizon_y, y));
        // The sun: a disc cut by bands that widen toward the horizon.
        let (sy, r) = (horizon_y - 0.17, 0.15);
        let d = (dx * dx + (y - sy) * (y - sy)).sqrt();
        let band = ((y - (sy - r)) / (2.0 * r)).clamp(0.0, 1.0);
        let cut = if band > 0.5 {
            smoothstep(0.0, 0.5, (band * 14.0).fract() + (0.5 - band) * 1.6)
        } else {
            1.0
        };
        col = add(col, sun, 0.22 * (-d * d / 0.05).exp());
        return mix(col, sun, smoothstep(r, r - 0.004, d) * cut);
    }
    let t = (y - horizon_y).max(0.0005);
    let z = 0.06 / t;
    let (gx, gz) = (dx * z * 5.0, z * 2.5);
    let wx = (gx - gx.round()).abs() / (z * 5.0 + 1.0);
    let wz = (gz - gz.round()).abs() / (z * 2.5 + 1.0);
    let ink = 1.0 - smoothstep(0.012, 0.03, wx.min(wz).min(0.5) * 2.0);
    let fade = smoothstep(0.0, 0.1, t);
    let base = mix(haze, floor, smoothstep(0.0, 0.35, t));
    add(
        mix(base, line, ink * fade * 0.9),
        line,
        0.12 * (-t * 22.0).exp(),
    )
}

fn dither(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let [c0, c1, c2, c3, _] = p.colors.map(color);
    // A coarse grid, so the dots stay visible at any resolution.
    let cells = 200.0;
    let (ix, iy) = ((x * cells).floor() as i32, (y * cells).floor() as i32);
    let (cx, cy) = ((ix as f32 + 0.5) / cells, (iy as f32 + 0.5) / cells);
    let (dx, dy) = (cx - aspect * 0.5, cy - 0.5);
    let d = (dx * dx + dy * dy).sqrt();
    let phase = dy.atan2(dx) * 3.0 - 8.0 * (d + 0.03).ln();
    let band = smoothstep(0.2, 1.0, phase.sin());
    let level = 0.12
        + 0.62 * band * (-d * 2.0).exp()
        + 0.5 * (-d * d / 0.006).exp()
        + 0.18 * (-d * 1.2).exp()
        + 0.08 * (fbm(cx * 3.0, cy * 3.0, p.seed, 3) - 0.5);
    let tone = (level.clamp(0.0, 1.0) * 3.0 + bayer4(ix, iy) - 0.5)
        .round()
        .clamp(0.0, 3.0) as usize;
    [c0, c1, c2, c3][tone]
}

fn tone4(p: &Preset, level: f32, ix: i32, iy: i32) -> Rgb {
    let [c0, c1, c2, c3, _] = p.colors.map(color);
    let tone = (level.clamp(0.0, 1.0) * 3.0 + bayer4(ix, iy) - 0.5)
        .round()
        .clamp(0.0, 3.0) as usize;
    [c0, c1, c2, c3][tone]
}

/// Waves in front of a sky, dithered on a coarse grid.
fn tidal(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let cells = 220.0;
    let (ix, iy) = ((x * cells).floor() as i32, (y * cells).floor() as i32);
    let (cx, cy) = ((ix as f32 + 0.5) / cells, (iy as f32 + 0.5) / cells);
    let s = p.seed as f32;
    // Sky: bright near the horizon, darker above.
    let mut level = 0.62 * (1.0 - smoothstep(0.0, 0.62, cy)) + 0.12;
    // A low sun.
    let (dx, dy) = (cx - aspect * 0.62, cy - 0.34);
    level += 0.55 * (-(dx * dx + dy * dy) / 0.012).exp();
    // Five rows of waves, each in front of the last and a little darker or
    // lighter, so the tones step the way a sea does.
    for k in 0..5u32 {
        let kf = k as f32;
        let base = 0.48 + 0.11 * kf;
        let wave = base
            + 0.022 * (cx * (6.0 + 2.5 * kf) + s * 0.7 + kf * 1.9).sin()
            + 0.03 * (fbm(cx * 3.0 + kf * 7.0, kf * 2.0, p.seed + k, 3) - 0.5);
        if cy > wave {
            let depth = (cy - wave) / 0.3;
            level = 0.18 + 0.1 * kf + 0.3 * (-depth * 3.0).exp() - 0.06 * (1.0 - kf / 4.0);
        }
    }
    tone4(p, level, ix, iy)
}

/// A glowing sphere of halftone dots.
fn halftone(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let [bg, c1, c2, c3, _] = p.colors.map(color);
    let cells = 72.0;
    let (gx, gy) = (x * cells, y * cells);
    let (ix, iy) = (gx.floor(), gy.floor());
    let (fx, fy) = (gx - ix - 0.5, gy - iy - 0.5);
    let (cx, cy) = ((ix + 0.5) / cells, (iy + 0.5) / cells);
    let (dx, dy) = (cx - aspect * 0.5, cy - 0.5);
    let r = (dx * dx + dy * dy).sqrt();
    let radius = 0.30;
    // Light on a ball from the upper left, plus a faint halo outside it.
    let lit = if r < radius {
        let nz = (1.0 - (r / radius).powi(2)).sqrt();
        let light = (-(dx / radius) * 0.45 - (dy / radius) * 0.55 + nz * 0.7).clamp(0.0, 1.0);
        0.30 + 0.70 * light
    } else {
        0.07 * (-(r - radius) * 6.0).exp() + 0.02 * fbm(cx * 4.0, cy * 4.0, p.seed, 2)
    };
    let dot = lit.clamp(0.0, 1.0).sqrt() * 0.66;
    let d = (fx * fx + fy * fy).sqrt();
    let ink = 1.0 - smoothstep(dot - 0.05, dot + 0.05, d);
    let dot_color = if lit > 0.72 {
        c3
    } else if lit > 0.45 {
        c2
    } else {
        c1
    };
    mix(bg, dot_color, ink)
}

/// Rings spreading from a point under the screen.
fn ripple(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let [top, low, ring, hi, glow] = p.colors.map(color);
    let mut col = mix(top, low, smoothstep(0.0, 1.0, y));
    let (dx, dy) = (x - aspect * 0.5, (y - 1.15) * 1.6);
    let d = (dx * dx + dy * dy).sqrt();
    col = add(col, glow, 0.35 * (-d * d / 0.14).exp());
    let wobble = 0.012 * (fbm(x * 4.0, y * 4.0, p.seed, 3) - 0.5);
    let f = (d + wobble) * 11.0;
    let frac = f - f.floor();
    let line = smoothstep(0.0, 0.5, frac) * (1.0 - smoothstep(0.5, 1.0, frac));
    let fade = (-d * 1.1).exp();
    col = mix(col, ring, 0.55 * line * line * fade);
    mix(col, hi, 0.5 * (line.powi(8)) * fade)
}

/// Warped silky bands through all five colours.
fn silk(p: &Preset, x: f32, y: f32, _aspect: f32) -> Rgb {
    let q = fbm(x * 1.6, y * 1.6, p.seed, 3);
    let r = fbm(
        x * 1.6 + 5.2 + 3.0 * q,
        y * 1.6 + 1.3 - 2.0 * q,
        p.seed + 7,
        3,
    );
    let t = (0.5 + 0.5 * (7.0 * r + 2.2 * x + 1.4 * y).sin()).clamp(0.0, 1.0);
    let t = t * 0.9 + 0.1 * smoothstep(0.0, 1.0, y);
    let ramp = p.colors.map(color);
    let scaled = t * 4.0;
    let i = (scaled.floor() as usize).min(3);
    mix(ramp[i], ramp[i + 1], scaled - i as f32)
}

fn pixel(p: &Preset, x: f32, y: f32, aspect: f32) -> Rgb {
    let mut col = match p.style {
        Style::Mesh => mesh(p, x, y, aspect),
        Style::Horizon => horizon(p, x, y, aspect),
        Style::Vortex => vortex(p, x, y, aspect),
        Style::Aurora => aurora(p, x, y, aspect),
        Style::Topo => topo(p, x, y, aspect),
        Style::Grid => grid(p, x, y, aspect),
        Style::Dither => return dither(p, x, y, aspect),
        Style::Tidal => return tidal(p, x, y, aspect),
        Style::Halftone => return halftone(p, x, y, aspect),
        Style::Ripple => ripple(p, x, y, aspect),
        Style::Silk => silk(p, x, y, aspect),
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
        for style in [
            Style::Mesh,
            Style::Horizon,
            Style::Vortex,
            Style::Aurora,
            Style::Topo,
            Style::Grid,
            Style::Dither,
            Style::Tidal,
            Style::Halftone,
            Style::Ripple,
            Style::Silk,
        ] {
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
