//! How other Whirlpool processes talk to the running dock.

use windows::{
    Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::{FindWindowW, PostMessageW, RegisterWindowMessageW},
    },
    core::{PCWSTR, w},
};

pub const DOCK_WINDOW_CLASS: PCWSTR = w!("WhirlpoolWindow");
/// Name of the registered window message meaning "the configuration file changed".
pub const CONFIG_CHANGED: PCWSTR = w!("Whirlpool.ConfigChanged");

/// `true` if a dock is running in this session.
#[must_use]
pub fn is_running() -> bool {
    // SAFETY: Read-only window lookup.
    unsafe { FindWindowW(DOCK_WINDOW_CLASS, None).is_ok() }
}

/// Tells the dock, if it is running, to reload its configuration. A window
/// message instead of a file watcher: the dock spends nothing while idle.
pub fn notify_config_changed() {
    // SAFETY: Looks up a window and posts a message without associated data.
    unsafe {
        if let Ok(dock) = FindWindowW(DOCK_WINDOW_CLASS, None) {
            let message = RegisterWindowMessageW(CONFIG_CHANGED);
            let _ = PostMessageW(Some(dock), message, WPARAM(0), LPARAM(0));
        }
    }
}
