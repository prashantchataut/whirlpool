//! Per-monitor wallpapers through the shell's `IDesktopWallpaper`: one picture for every display,
//! or a different one for each, with fill/fit/stretch/tile/center. Windows keeps the pictures; this
//! program holds no image in memory after the call returns.

use std::{io, path::Path};

use windows::{
    Win32::{
        Foundation::RECT,
        System::{
            Com::{
                CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoTaskMemFree, CoUninitialize,
            },
            Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
        },
        UI::Shell::{
            DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT, DWPOS_STRETCH,
            DWPOS_TILE, DesktopWallpaper, IDesktopWallpaper,
        },
    },
    core::{HSTRING, PCWSTR, w},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Fill,
    Fit,
    Stretch,
    Tile,
    Center,
}

impl Fit {
    const fn position(self) -> DESKTOP_WALLPAPER_POSITION {
        match self {
            Self::Fill => DWPOS_FILL,
            Self::Fit => DWPOS_FIT,
            Self::Stretch => DWPOS_STRETCH,
            Self::Tile => DWPOS_TILE,
            Self::Center => DWPOS_CENTER,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monitor {
    /// The device path Windows uses to address this display.
    pub id: String,
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

fn with_shell<T>(
    work: impl FnOnce(&IDesktopWallpaper) -> windows::core::Result<T>,
) -> io::Result<T> {
    // SAFETY: COM is initialised and released on this thread around the calls; the interface
    // does not outlive the closure.
    unsafe {
        let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL)
            .and_then(|shell| work(&shell));
        if com.is_ok() {
            CoUninitialize();
        }
        result.map_err(io::Error::other)
    }
}

/// The attached displays, in the order Windows lists them.
#[must_use]
pub fn monitors() -> Vec<Monitor> {
    with_shell(|shell| {
        // SAFETY: Plain calls on a live interface; each returned string is freed once.
        unsafe {
            let mut found = Vec::new();
            for index in 0..shell.GetMonitorDevicePathCount()? {
                let path = shell.GetMonitorDevicePathAt(index)?;
                let id = path.to_string().unwrap_or_default();
                let rect: windows::core::Result<RECT> = shell.GetMonitorRECT(PCWSTR(path.0));
                CoTaskMemFree(Some(path.0.cast()));
                if let (false, Ok(rect)) = (id.is_empty(), rect) {
                    found.push(Monitor {
                        id,
                        left: rect.left,
                        top: rect.top,
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    });
                }
            }
            Ok(found)
        }
    })
    .unwrap_or_default()
}

/// Sets `path` as the wallpaper of one display (`monitor`), or of all of them (`None`).
///
/// # Errors
/// Fails if Windows refuses the picture or the display.
pub fn set(monitor: Option<&str>, path: &Path, fit: Fit) -> io::Result<()> {
    let file = HSTRING::from(path.as_os_str());
    let id = monitor.map(HSTRING::from);
    with_shell(|shell| {
        let target = id.as_ref().map_or(PCWSTR::null(), |id| PCWSTR(id.as_ptr()));
        // SAFETY: `target` and `file` outlive the calls.
        unsafe {
            shell.SetPosition(fit.position())?;
            shell.SetWallpaper(target, PCWSTR(file.as_ptr()))
        }
    })
}

/// Whether apps are set to the dark theme (Settings > Personalization > Colors).
#[must_use]
pub fn apps_use_dark() -> bool {
    let mut value = 1u32;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: `value` and `size` describe a valid 4-byte buffer.
    let read = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut value).cast()),
            Some(&mut size),
        )
    };
    read.is_ok() && value == 0
}
