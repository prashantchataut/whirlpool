//! Fallback for the stock volume flyout.
//!
//! The volume keys are taken by `mediakeys` before Windows sees them. A volume
//! change from somewhere else (a headset button, an app's own slider with the
//! flyout hook of the OEM driver) can still make Windows open its flyout. The
//! dock calls [`hide_stock_flyout`] for a moment after such a change.
//!
//! Only a small, visible, untitled flyout window of `explorer.exe` is touched.
//! Quick settings, Start and the calendar are taller than the size limit.

use windows::{
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM, RECT},
        Graphics::Dwm::{DWMWA_CLOAK, DwmSetWindowAttribute},
        System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW,
                GetWindowThreadProcessId, IsWindowVisible, SW_HIDE, ShowWindow,
            },
        },
    },
    core::{BOOL, PWSTR},
};

/// Window classes that host the Windows 10 and Windows 11 volume flyout.
const HOST_CLASSES: [&str; 2] = ["XamlExplorerHostIslandWindow", "NativeHWNDHost"];
/// The flyout is about 360 x 90 logical pixels; anything larger is another surface.
const MAX_WIDTH: i32 = 480;
const MAX_HEIGHT: i32 = 180;

/// Hides the stock flyout if it is on screen. Returns whether one was hidden.
#[must_use]
pub fn hide_stock_flyout() -> bool {
    let mut hidden = 0u32;
    // SAFETY: `visit` runs synchronously inside this call and only touches
    // `hidden` through the pointer passed here.
    unsafe {
        let _ = EnumWindows(Some(visit), LPARAM((&raw mut hidden) as isize));
    }
    hidden > 0
}

unsafe extern "system" fn visit(window: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: Read-only window queries on a handle Windows just gave us.
    unsafe {
        if is_stock_flyout(window) {
            // Cloaking another process's window is often refused; hiding it is not.
            let cloak = BOOL(1);
            let _ = DwmSetWindowAttribute(
                window,
                DWMWA_CLOAK,
                (&raw const cloak).cast(),
                size_of::<BOOL>() as u32,
            );
            let _ = ShowWindow(window, SW_HIDE);
            *(lparam.0 as *mut u32) += 1;
        }
    }
    BOOL(1)
}

/// # Safety
/// `window` must be a handle from `EnumWindows`.
unsafe fn is_stock_flyout(window: HWND) -> bool {
    // SAFETY: Only queries.
    unsafe {
        if !IsWindowVisible(window).as_bool() || GetWindowTextLengthW(window) != 0 {
            return false;
        }
        let mut class = [0u16; 64];
        let length = GetClassNameW(window, &mut class).max(0) as usize;
        let class = String::from_utf16_lossy(&class[..length.min(class.len())]);
        if !HOST_CLASSES.contains(&class.as_str()) {
            return false;
        }
        let mut rect = RECT::default();
        if GetWindowRect(window, &raw mut rect).is_err() {
            return false;
        }
        let dpi = GetDpiForWindow(window).max(96) as f32 / 96.0;
        let (width, height) = (
            (rect.right - rect.left) as f32 / dpi,
            (rect.bottom - rect.top) as f32 / dpi,
        );
        if width <= 0.0 || height <= 0.0 || width > MAX_WIDTH as f32 || height > MAX_HEIGHT as f32 {
            return false;
        }
        owned_by_explorer(window)
    }
}

/// # Safety
/// `window` must be a valid handle.
unsafe fn owned_by_explorer(window: HWND) -> bool {
    // SAFETY: Opens a query-only handle and closes it.
    unsafe {
        let mut process = 0u32;
        GetWindowThreadProcessId(window, Some(&raw mut process));
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process) else {
            return false;
        };
        let mut buffer = [0u16; 260];
        let mut length = buffer.len() as u32;
        let named = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &raw mut length,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if !named {
            return false;
        }
        let path = String::from_utf16_lossy(&buffer[..length as usize]).to_lowercase();
        path.ends_with("\\explorer.exe")
    }
}
