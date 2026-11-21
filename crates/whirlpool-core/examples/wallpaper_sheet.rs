//! Writes a contact sheet of every wallpaper preset as a PPM on stdout:
//! `cargo run -p whirlpool-core --example wallpaper_sheet > sheet.ppm`
use std::io::Write;
use whirlpool_core::wallpaper::{PRESETS, render};

fn main() {
    let (w, h, cols) = (320u32, 200u32, 6u32);
    let rows = (PRESETS.len() as u32).div_ceil(cols);
    let (sw, sh) = (w * cols, h * rows);
    let mut sheet = vec![20u8; (sw * sh * 3) as usize];
    for (i, preset) in PRESETS.iter().enumerate() {
        let tile = render(preset, w, h);
        let (cx, cy) = ((i as u32 % cols) * w, (i as u32 / cols) * h);
        for y in 0..h {
            let src = (y * w * 3) as usize;
            let dst = (((cy + y) * sw + cx) * 3) as usize;
            sheet[dst..dst + (w * 3) as usize]
                .copy_from_slice(&tile.rgb[src..src + (w * 3) as usize]);
        }
    }
    let mut out = std::io::stdout().lock();
    write!(out, "P6\n{sw} {sh}\n255\n").unwrap();
    out.write_all(&sheet).unwrap();
}
