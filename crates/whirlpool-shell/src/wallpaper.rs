//! Putting a rendered wallpaper on the desktop.
//!
//! The picture itself is drawn by `whirlpool_core::wallpaper` at the exact
//! size of the screen; this module only saves it and asks Windows to use it.

use std::{
    env, fs, io,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use whirlpool_core::wallpaper;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE,
    SPIF_UPDATEINIFILE, SystemParametersInfoW,
};
use windows::{
    Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RegSetKeyValueW},
    core::{PCWSTR, w},
};

fn folder() -> PathBuf {
    PathBuf::from(env::var_os("LOCALAPPDATA").unwrap_or_default())
        .join("Whirlpool")
        .join("wallpapers")
}

/// Draws the preset `id` for the primary screen and makes it the wallpaper.
///
/// # Errors
/// Fails if the preset is unknown, the file cannot be written or Windows
/// refuses the change.
pub fn apply(id: &str) -> io::Result<()> {
    let preset = wallpaper::find(id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unknown wallpaper"))?;
    // SAFETY: Read-only system metrics.
    let (width, height) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let raster = wallpaper::render(preset, width.max(1280) as u32, height.max(720) as u32);
    let dir = folder();
    fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{}.bmp", preset.id));
    fs::write(&file, wallpaper::encode_bmp(&raster))?;
    set(&file)?;
    // Keep one picture on disk: the one Windows now points at.
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path() != file {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

fn set(file: &Path) -> io::Result<()> {
    let mut wide: Vec<u16> = file.as_os_str().encode_wide().collect();
    wide.push(0);
    // SAFETY: `wide` is a NUL-terminated path that outlives the call, and
    // SPI_SETDESKWALLPAPER only reads it.
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_mut_ptr().cast()),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    }
    .map_err(|error| io::Error::other(error.to_string()))
}

/// Sets a text value under `Control Panel\Desktop`.
fn desktop_value(name: PCWSTR, value: &str) {
    let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is NUL-terminated and outlives the call; the byte count
    // covers the terminator, as REG_SZ requires.
    unsafe {
        let _ = RegSetKeyValueW(
            HKEY_CURRENT_USER,
            w!("Control Panel\\Desktop"),
            name,
            REG_SZ.0,
            Some(wide.as_ptr().cast()),
            (wide.len() * 2) as u32,
        );
    }
}

/// Makes a photograph (JPEG bytes) the wallpaper, scaled to fill the screen.
///
/// # Errors
/// Fails if the file cannot be written or Windows refuses the change.
pub fn apply_photo(id: &str, jpeg: &[u8]) -> io::Result<()> {
    let dir = folder();
    fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{id}.jpg"));
    fs::write(&file, jpeg)?;
    // 10 = Fill, not tiled. Without this a "Center" setting left over from
    // another tool would show the 1440p photo as a small crop.
    desktop_value(w!("WallpaperStyle"), "10");
    desktop_value(w!("TileWallpaper"), "0");
    set(&file)?;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path() != file {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}
