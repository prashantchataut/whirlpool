//! Clipboard history for the search palette: the last few copied texts, kept in this process's
//! memory only. It is event driven (Windows tells us when the clipboard changes), so nothing polls.
//! Nothing is written to disk. Items that password managers mark as private are never stored, and
//! turning the setting off drops everything at once.

use std::sync::Mutex;

use windows::{
    Win32::{
        Foundation::{HGLOBAL, HWND},
        System::{
            DataExchange::{
                AddClipboardFormatListener, CloseClipboard, GetClipboardData,
                GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
                RegisterClipboardFormatW, RemoveClipboardFormatListener,
            },
            Memory::{GlobalLock, GlobalSize, GlobalUnlock},
        },
    },
    core::w,
};

const CF_UNICODETEXT: u32 = 13;
const KEEP: usize = 20;
/// Longer clips are not worth holding on to (and could be a pasted file's contents).
const MAX_CHARS: usize = 4000;

static CLIPS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static LAST_SEQUENCE: Mutex<u32> = Mutex::new(0);

pub fn enable(window: HWND, on: bool) {
    // SAFETY: Registers or removes our own window as a clipboard listener; both are harmless to repeat.
    unsafe {
        if on {
            let _ = AddClipboardFormatListener(window);
        } else {
            let _ = RemoveClipboardFormatListener(window);
        }
    }
    if !on && let Ok(mut clips) = CLIPS.lock() {
        clips.clear();
        clips.shrink_to_fit();
    }
}

/// The newest clips first.
pub fn snapshot() -> Vec<String> {
    CLIPS.lock().map(|clips| clips.clone()).unwrap_or_default()
}

/// A password manager says "do not keep this" through one of these formats.
fn marked_private() -> bool {
    // SAFETY: Format registration and availability checks take no pointers we own.
    unsafe {
        [
            w!("ExcludeClipboardContentFromMonitorProcessing"),
            w!("CanIncludeInClipboardHistory"),
            w!("CanUploadToCloudClipboard"),
        ]
        .iter()
        .enumerate()
        .any(|(index, name)| {
            let format = RegisterClipboardFormatW(*name);
            // The first excludes when present; the other two exclude when present and set to 0, which
            // we cannot cheaply read, so only the first counts as a certain "private".
            index == 0 && format != 0 && IsClipboardFormatAvailable(format).is_ok()
        })
    }
}

fn read_text(window: HWND) -> Option<String> {
    // SAFETY: The clipboard is opened and closed in this scope; the global memory is locked only while
    // it is copied out, and its size bounds the read.
    unsafe {
        OpenClipboard(Some(window)).ok()?;
        let result = (|| {
            let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
            let global = HGLOBAL(handle.0);
            let size = GlobalSize(global) / 2;
            let pointer = GlobalLock(global).cast::<u16>();
            if pointer.is_null() {
                return None;
            }
            let units = std::slice::from_raw_parts(pointer, size);
            let end = units
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(units.len());
            let text = String::from_utf16_lossy(&units[..end]);
            let _ = GlobalUnlock(global);
            Some(text)
        })();
        let _ = CloseClipboard();
        result
    }
}

/// `WM_CLIPBOARDUPDATE`.
pub fn changed(window: HWND) {
    // SAFETY: Plain query of the clipboard change counter.
    let sequence = unsafe { GetClipboardSequenceNumber() };
    if let Ok(mut last) = LAST_SEQUENCE.lock() {
        if *last == sequence {
            return;
        }
        *last = sequence;
    }
    if marked_private() {
        return;
    }
    let Some(text) = read_text(window) else {
        return;
    };
    if text.trim().is_empty() || text.chars().count() > MAX_CHARS {
        return;
    }
    if let Ok(mut clips) = CLIPS.lock() {
        clips.retain(|existing| *existing != text);
        clips.insert(0, text);
        clips.truncate(KEEP);
    }
}
