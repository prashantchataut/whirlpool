//! Takes the three volume keys before Windows shows its own flyout.
//!
//! A low-level keyboard hook runs on its own thread with its own message loop.
//! For every key except `VK_VOLUME_UP`, `VK_VOLUME_DOWN` and `VK_VOLUME_MUTE`
//! the hook passes the event on at once. For those three it hands the press to
//! a second thread over a channel and swallows the event, so the shell never
//! sees it and never opens the stock volume flyout. The second thread moves the
//! master volume through Core Audio (2 % a press, or mute) and tells the dock
//! what the level is now, so Whirlpool's own pill appears.
//!
//! Privacy: the hook compares each key code with those three values and
//! nothing else. No key is stored, counted, logged or sent anywhere, not even
//! while debugging. There is no logging call in this file.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use whirlpool_core::mediakeys::{VolumeKey, pack, stepped};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Media::Audio::{
            Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator, eConsole,
            eRender,
        },
        System::{
            Com::{
                CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
            },
            Threading::GetCurrentThreadId,
        },
        UI::WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT, MSG, PostMessageW,
            PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN,
            WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
    core::Result,
};

/// What one press asks the audio thread to do.
enum Action {
    Step(i32),
    ToggleMute,
}

/// The audio thread's inbox. `None` while the hook is off.
static INBOX: Mutex<Option<Sender<Action>>> = Mutex::new(None);
/// Mute is a toggle: the key's auto-repeat must not flip it again and again.
static MUTE_HELD: AtomicBool = AtomicBool::new(false);

/// The hook while it runs. Dropping it removes the hook and ends both threads.
pub struct MediaKeys {
    hook_thread: u32,
    threads: Vec<JoinHandle<()>>,
}

impl MediaKeys {
    /// Starts the hook. After each press, `PostMessageW(window, message, 1, value)`
    /// tells the dock the new level (`whirlpool_core::mediakeys::pack`).
    /// `None` when Windows refuses the hook; the stock flyout then keeps working.
    #[must_use]
    pub fn start(window: isize, message: u32) -> Option<Self> {
        let (tx, rx) = mpsc::channel::<Action>();
        let audio = thread::Builder::new()
            .name("volume-keys-audio".into())
            .spawn(move || audio_loop(&rx, window, message))
            .ok()?;
        *INBOX.lock().ok()? = Some(tx);

        let (ready_tx, ready_rx) = mpsc::channel::<Option<u32>>();
        let pump = thread::Builder::new()
            .name("volume-keys-hook".into())
            .spawn(move || hook_loop(&ready_tx))
            .ok()?;
        let hook_thread = match ready_rx.recv() {
            Ok(Some(id)) => id,
            _ => {
                *INBOX.lock().ok()? = None;
                let _ = pump.join();
                let _ = audio.join();
                return None;
            }
        };
        Some(Self {
            hook_thread,
            threads: vec![pump, audio],
        })
    }
}

impl Drop for MediaKeys {
    fn drop(&mut self) {
        // SAFETY: Posts a quit message to the hook thread's own message loop.
        unsafe {
            let _ = PostThreadMessageW(self.hook_thread, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        // Dropping the sender ends the audio thread's loop.
        if let Ok(mut inbox) = INBOX.lock() {
            *inbox = None;
        }
        MUTE_HELD.store(false, Ordering::Relaxed);
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
    }
}

/// Installs the hook and pumps messages until `WM_QUIT`.
fn hook_loop(ready: &Sender<Option<u32>>) {
    // SAFETY: A global low-level hook needs a message loop on the thread that
    // installed it; this thread runs nothing else. The hook is removed before
    // the thread ends.
    unsafe {
        let Ok(hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) else {
            let _ = ready.send(None);
            return;
        };
        let _ = ready.send(Some(GetCurrentThreadId()));
        let mut msg = MSG::default();
        // 0 is WM_QUIT and -1 an error: both end the loop.
        while GetMessageW(&raw mut msg, None, 0, 0).0 > 0 {}
        let _ = UnhookWindowsHookEx(hook);
    }
}

/// The hook procedure. It must return quickly: it only classifies and sends.
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: For `WH_KEYBOARD_LL` with `HC_ACTION`, `lparam` points at a
        // `KBDLLHOOKSTRUCT` that lives for the call.
        let vk = unsafe { (*(lparam.0 as *const KBDLLHOOKSTRUCT)).vkCode };
        if let Some(key) = VolumeKey::from_vk(vk) {
            let message = wparam.0 as u32;
            let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
            let up = message == WM_KEYUP || message == WM_SYSKEYUP;
            if down || up {
                if down {
                    send(key);
                } else if key == VolumeKey::Mute {
                    MUTE_HELD.store(false, Ordering::Relaxed);
                }
                // Swallowed: neither the shell nor any program sees it.
                return LRESULT(1);
            }
        }
    }
    // SAFETY: Passing the event on unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn send(key: VolumeKey) {
    let action = match key {
        VolumeKey::Mute => {
            if MUTE_HELD.swap(true, Ordering::Relaxed) {
                return;
            }
            Action::ToggleMute
        }
        other => Action::Step(other.delta()),
    };
    // Never wait inside the hook: if the lock is busy the press is passed over.
    if let Ok(inbox) = INBOX.try_lock()
        && let Some(sender) = inbox.as_ref()
    {
        let _ = sender.send(action);
    }
}

/// Applies presses in order. Presses that arrived together become one change.
fn audio_loop(inbox: &Receiver<Action>, window: isize, message: u32) {
    // SAFETY: COM is initialised for this thread's life and released at its end.
    let initialised = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() };
    while let Ok(first) = inbox.recv() {
        let (mut delta, mut toggles) = (0, 0u32);
        for action in std::iter::once(first).chain(inbox.try_iter()) {
            match action {
                Action::Step(by) => delta += by,
                Action::ToggleMute => toggles += 1,
            }
        }
        if let Ok((percent, muted)) = apply(delta, toggles) {
            // SAFETY: Only posts a message; a closed window ignores it.
            unsafe {
                let _ = PostMessageW(
                    Some(HWND(window as *mut _)),
                    message,
                    WPARAM(1),
                    LPARAM(pack(percent, muted)),
                );
            }
        }
    }
    if initialised {
        // SAFETY: Pairs with the successful `CoInitializeEx` above.
        unsafe { CoUninitialize() };
    }
}

/// Moves the default output's volume and returns (percent, muted) afterwards.
fn apply(delta: i32, toggles: u32) -> Result<(u8, bool)> {
    // SAFETY: COM calls on objects created here, on a COM-initialised thread.
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        let before = (endpoint.GetMasterVolumeLevelScalar()? * 100.0).round() as u8;
        let mut muted = endpoint.GetMute()?.as_bool();
        if toggles % 2 == 1 {
            muted = !muted;
            endpoint.SetMute(muted, std::ptr::null())?;
        }
        let mut percent = before;
        if delta != 0 {
            percent = stepped(before, delta);
            endpoint.SetMasterVolumeLevelScalar(f32::from(percent) / 100.0, std::ptr::null())?;
            // Volume up from mute unmutes, like the Windows keys do.
            if delta > 0 && muted {
                muted = false;
                endpoint.SetMute(false, std::ptr::null())?;
            }
        }
        Ok((percent, muted))
    }
}
