//! Bluetooth on or off, as Windows reports it.
//!
//! Nothing here polls. Windows raises `Radio.StateChanged` when the user (or a
//! key) turns Bluetooth on or off; the handler only posts a message to the dock
//! window. The radios are looked up once, on a short-lived thread, because
//! asking for them can take a moment and must not hold the message loop.

use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use windows::{
    Devices::Radios::{Radio, RadioKind, RadioState},
    Foundation::TypedEventHandler,
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::PostMessageW,
    },
    core::IInspectable,
};

type Registered = Arc<Mutex<Vec<(Radio, i64)>>>;

pub struct BluetoothWatch {
    radios: Registered,
}

impl BluetoothWatch {
    /// Posts `message` to `window` whenever a Bluetooth radio changes state.
    #[must_use]
    pub fn new(window: isize, message: u32) -> Self {
        let radios: Registered = Arc::default();
        let store = Arc::clone(&radios);
        let _ = std::thread::Builder::new()
            .name("bluetooth".into())
            .spawn(move || register(window, message, &store));
        Self { radios }
    }

    /// `Some(true)` when a Bluetooth radio is on, `None` when there is none
    /// (or it has not been found yet).
    #[must_use]
    pub fn is_on(&self) -> Option<bool> {
        let radios = self.radios.lock().ok()?;
        if radios.is_empty() {
            return None;
        }
        Some(
            radios
                .iter()
                .any(|(radio, _)| radio.State().ok() == Some(RadioState::On)),
        )
    }
}

impl Drop for BluetoothWatch {
    fn drop(&mut self) {
        if let Ok(radios) = self.radios.lock() {
            for (radio, token) in radios.iter() {
                let _ = radio.RemoveStateChanged(*token);
            }
        }
    }
}

fn register(window: isize, message: u32, store: &Registered) {
    let Ok(operation) = Radio::GetRadiosAsync() else {
        return;
    };
    let (send, receive) = mpsc::channel();
    let waiting = operation.SetCompleted(&windows_future::AsyncOperationCompletedHandler::new(
        move |_, _| {
            let _ = send.send(());
            Ok(())
        },
    ));
    if waiting.is_err() || receive.recv_timeout(Duration::from_secs(5)).is_err() {
        let _ = operation.Cancel();
        return;
    }
    let Ok(list) = operation.GetResults() else {
        return;
    };
    let mut found = Vec::new();
    for radio in list {
        if radio.Kind().ok() != Some(RadioKind::Bluetooth) {
            continue;
        }
        let handler = TypedEventHandler::<Radio, IInspectable>::new(move |_, _| {
            // SAFETY: Only posts a message; a window that no longer exists ignores it.
            unsafe {
                let _ = PostMessageW(Some(HWND(window as *mut _)), message, WPARAM(0), LPARAM(0));
            }
            Ok(())
        });
        if let Ok(token) = radio.StateChanged(&handler) {
            found.push((radio, token));
        }
    }
    if let Ok(mut guard) = store.lock() {
        *guard = found;
    }
}
