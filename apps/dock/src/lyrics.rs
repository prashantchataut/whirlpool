//! Synced lyrics, only when the user turned them on in Settings. One request per song, made on a
//! short-lived thread, to the public LRCLIB service: title, artist, album and length, nothing else.
//! The lines of the current song live in memory and are dropped with the song or the setting.

use std::{sync::Mutex, thread, time::Duration};

use whirlpool_core::lrc::{self, Line};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::PostMessageW,
};

use crate::media::Track;

enum Phase {
    Idle,
    Loading,
    Found(Vec<Line>),
    Missing,
}

struct State {
    key: String,
    phase: Phase,
}

static STATE: Mutex<State> = Mutex::new(State {
    key: String::new(),
    phase: Phase::Idle,
});

fn key_of(track: &Track) -> String {
    format!(
        "{}\u{1}{}\u{1}{}",
        track.title.trim().to_lowercase(),
        track.artist.trim().to_lowercase(),
        track.album.trim().to_lowercase()
    )
}

/// Starts the lookup for this song if it has not been asked for yet. When the answer arrives the
/// window gets `message` once, so it can redraw.
pub fn request(track: &Track, window: isize, message: u32) {
    if track.title.trim().is_empty() {
        return;
    }
    let key = key_of(track);
    {
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        if state.key == key {
            return;
        }
        state.key.clone_from(&key);
        state.phase = Phase::Loading;
    }
    let url = lrc::request_url(
        &track.title,
        &track.artist,
        &track.album,
        track.duration_secs,
    );
    let _ = thread::Builder::new()
        .name("whirlpool-lyrics".into())
        .spawn(move || {
            let found = fetch(&url);
            let mut changed = false;
            if let Ok(mut state) = STATE.lock()
                && state.key == key
            {
                state.phase = match found {
                    Some(lines) if !lines.is_empty() => Phase::Found(lines),
                    _ => Phase::Missing,
                };
                changed = true;
            }
            if changed {
                // SAFETY: Posts a plain message to the dock's own bar window; a stale handle just fails.
                unsafe {
                    let _ =
                        PostMessageW(Some(HWND(window as *mut _)), message, WPARAM(0), LPARAM(0));
                }
            }
        });
}

fn fetch(url: &str) -> Option<Vec<Line>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent(concat!(
            "Whirlpool/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/prashantchataut/whirlpool)"
        ))
        .build()
        .ok()?;
    let response = client.get(url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.text().ok()?;
    Some(lrc::parse(&lrc::extract_synced(&body)?))
}

/// The line to show right now, if lyrics for this song are loaded and one is due.
pub fn line(track: &Track) -> Option<String> {
    let state = STATE.lock().ok()?;
    if state.key != key_of(track) {
        return None;
    }
    let Phase::Found(lines) = &state.phase else {
        return None;
    };
    let at = lrc::line_at(lines, track.position_ms())?;
    let text = lines[at].text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// How long until the next line is due, for a timer; `None` when nothing is waiting.
pub fn wait_ms(track: &Track) -> Option<u32> {
    if !track.playing {
        return None;
    }
    let state = STATE.lock().ok()?;
    if state.key != key_of(track) {
        return None;
    }
    let Phase::Found(lines) = &state.phase else {
        return None;
    };
    let ms = lrc::next_change_ms(lines, track.position_ms())?;
    Some(ms.clamp(150, 10_000) as u32)
}

/// Forgets everything (the setting was turned off, or the song changed to nothing).
pub fn clear() {
    if let Ok(mut state) = STATE.lock()
        && !matches!(state.phase, Phase::Idle)
    {
        state.key = String::new();
        state.phase = Phase::Idle;
    }
}
