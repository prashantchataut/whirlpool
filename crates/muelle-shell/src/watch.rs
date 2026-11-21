//! Telling a window when a registry key changes, without polling and without
//! a thread of its own.

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, INVALID_HANDLE_VALUE, LPARAM, WPARAM},
        System::{
            Registry::{
                HKEY, HKEY_CURRENT_USER, KEY_NOTIFY, REG_NOTIFY_CHANGE_LAST_SET,
                REG_NOTIFY_CHANGE_NAME, REG_NOTIFY_THREAD_AGNOSTIC, RegCloseKey,
                RegNotifyChangeKeyValue, RegOpenKeyExW,
            },
            Threading::{
                CreateEventW, INFINITE, RegisterWaitForSingleObject, UnregisterWaitEx,
                WT_EXECUTEONLYONCE,
            },
        },
        UI::WindowsAndMessaging::PostMessageW,
    },
    core::PCWSTR,
};

/// Posts `message` to a window when a key under `HKEY_CURRENT_USER` changes:
/// a value is written, or (with `subtree`) a key below it is added, removed
/// or written.
///
/// The registry signals an event and the system thread pool posts the
/// message. Each registration fires once; the receiver calls
/// [`RegistryWatch::arm`] again when it handles the message, so a burst of
/// changes costs one message until then.
pub struct RegistryWatch {
    key: HKEY,
    subtree: bool,
    event: HANDLE,
    wait: HANDLE,
    /// Where the thread pool posts: window and message. Boxed so its address,
    /// handed to the thread pool, never moves.
    target: Box<(isize, u32)>,
}

impl RegistryWatch {
    #[must_use]
    pub fn new(path: PCWSTR, subtree: bool, hwnd: isize, message: u32) -> Option<Self> {
        let mut key = HKEY::default();
        // SAFETY: Opens a key for notifications only; closed in `Drop`.
        unsafe {
            RegOpenKeyExW(HKEY_CURRENT_USER, path, None, KEY_NOTIFY, &mut key)
                .ok()
                .ok()?;
        }
        // SAFETY: An auto-reset event owned by this watch, closed in `Drop`.
        let Ok(event) = (unsafe { CreateEventW(None, false, false, None) }) else {
            // SAFETY: The key opened above.
            unsafe {
                let _ = RegCloseKey(key);
            }
            return None;
        };
        let mut watch = Self {
            key,
            subtree,
            event,
            wait: HANDLE::default(),
            target: Box::new((hwnd, message)),
        };
        watch.arm().then_some(watch)
    }

    /// Listens for the next change.
    pub fn arm(&mut self) -> bool {
        // SAFETY: Every handle belongs to this watch; `target` outlives the
        // wait, which `Drop` cancels before freeing it.
        unsafe {
            if !self.wait.is_invalid() {
                // Without an event to wait on: the one-shot wait has run.
                let _ = UnregisterWaitEx(self.wait, None);
                self.wait = HANDLE::default();
            }
            // Thread-agnostic, so the registration survives whichever thread made it.
            let filter =
                REG_NOTIFY_CHANGE_LAST_SET | REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_THREAD_AGNOSTIC;
            if RegNotifyChangeKeyValue(self.key, self.subtree, filter, Some(self.event), true)
                .is_err()
            {
                return false;
            }
            RegisterWaitForSingleObject(
                &mut self.wait,
                self.event,
                Some(changed),
                Some(std::ptr::from_ref(&*self.target).cast()),
                INFINITE,
                WT_EXECUTEONLYONCE,
            )
            .is_ok()
        }
    }
}

unsafe extern "system" fn changed(context: *mut std::ffi::c_void, _timed_out: bool) {
    // SAFETY: `context` points at the watch's boxed target, alive until the
    // wait is cancelled.
    let (hwnd, message) = unsafe { *context.cast::<(isize, u32)>() };
    // SAFETY: Posting is asynchronous; a closed window just drops it.
    unsafe {
        let _ = PostMessageW(Some(HWND(hwnd as *mut _)), message, WPARAM(0), LPARAM(0));
    }
}

impl Drop for RegistryWatch {
    fn drop(&mut self) {
        // SAFETY: Waits for a running callback to finish before the target
        // is freed, then releases this watch's handles once.
        unsafe {
            if !self.wait.is_invalid() {
                let _ = UnregisterWaitEx(self.wait, Some(INVALID_HANDLE_VALUE));
            }
            let _ = CloseHandle(self.event);
            let _ = RegCloseKey(self.key);
        }
    }
}
