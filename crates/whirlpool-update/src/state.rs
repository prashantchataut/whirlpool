//! What the updater remembers between runs, in `%LOCALAPPDATA%\Whirlpool\updates\state.json`.

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct State {
    /// Unix seconds of the last successful or failed look.
    pub last_checked: u64,
    /// Newest version found, if newer than the one running at that time.
    pub available: Option<String>,
    /// The version that was installed before the last update; the rollback target.
    pub previous: Option<String>,
    /// The version the last reminder was about.
    pub reminded: Option<String>,
    /// Unix seconds of the last reminder.
    pub reminded_at: u64,
    /// How many reminders that version has had.
    pub reminder_count: u32,
}

/// A version is announced when found, then again after this long, up to [`MAX_REMINDERS`] times.
pub const REMIND_AFTER_SECS: u64 = 3 * 86_400;
pub const MAX_REMINDERS: u32 = 3;

impl State {
    const FILE: &'static str = "state.json";

    #[must_use]
    pub fn load(dir: &Path) -> Self {
        fs::read_to_string(dir.join(Self::FILE))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, dir: &Path) {
        if fs::create_dir_all(dir).is_ok()
            && let Ok(text) = serde_json::to_string_pretty(self)
        {
            let _ = fs::write(dir.join(Self::FILE), text);
        }
    }

    #[must_use]
    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }

    /// Whether a reminder about `available` should be shown now: at once for a version not yet
    /// announced, then every three days, at most three times per version. Never when nothing is
    /// available. A clock set back does not postpone it forever.
    #[must_use]
    pub fn reminder_due(&self, now: u64) -> bool {
        let Some(available) = &self.available else {
            return false;
        };
        if self.reminded.as_ref() != Some(available) {
            return true;
        }
        let since = if self.reminded_at > now {
            REMIND_AFTER_SECS
        } else {
            now - self.reminded_at
        };
        self.reminder_count < MAX_REMINDERS && since >= REMIND_AFTER_SECS
    }

    /// Records that a reminder about the available version was shown.
    pub fn note_reminded(&mut self, now: u64) {
        if self.reminded == self.available {
            self.reminder_count += 1;
        } else {
            self.reminded.clone_from(&self.available);
            self.reminder_count = 1;
        }
        self.reminded_at = now;
    }

    /// Seconds until the next background look is due; zero when it is due now.
    /// `every_days == 0` means never.
    #[must_use]
    pub fn due_in(&self, every_days: u32, now: u64) -> Option<u64> {
        if every_days == 0 {
            return None;
        }
        let every = u64::from(every_days) * 86_400;
        // A clock set back past the last look must not postpone it forever.
        if self.last_checked == 0 || self.last_checked > now {
            return Some(0);
        }
        Some(every.saturating_sub(now - self.last_checked))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_dates() {
        let state = State {
            last_checked: 1_000_000,
            ..State::default()
        };
        assert_eq!(state.due_in(0, 2_000_000), None);
        assert_eq!(state.due_in(7, 1_000_000 + 86_400), Some(6 * 86_400));
        assert_eq!(state.due_in(1, 1_000_000 + 90_000), Some(0));
        assert_eq!(state.due_in(1, 5), Some(0));
        assert_eq!(State::default().due_in(7, 100), Some(0));
    }

    #[test]
    fn persists() {
        let dir = std::env::temp_dir().join("wp-state-test");
        let _ = fs::remove_dir_all(&dir);
        let state = State {
            last_checked: 5,
            available: Some("4.3.0".into()),
            previous: Some("4.2.0".into()),
            ..State::default()
        };
        state.save(&dir);
        assert_eq!(State::load(&dir), state);
        assert_eq!(State::load(&dir.join("nope")), State::default());
    }

    #[test]
    fn a_new_version_is_announced_at_once_then_three_times_at_most() {
        let day = 86_400;
        let mut state = State {
            available: Some("4.3.0".into()),
            ..State::default()
        };
        assert!(state.reminder_due(100));
        state.note_reminded(100);
        assert!(!state.reminder_due(100 + day), "not again the next day");
        assert!(state.reminder_due(100 + 3 * day));
        state.note_reminded(100 + 3 * day);
        state.note_reminded(100 + 6 * day);
        assert_eq!(state.reminder_count, 3);
        assert!(!state.reminder_due(100 + 99 * day), "three is the limit");
        state.available = Some("4.4.0".into());
        assert!(
            state.reminder_due(100 + 99 * day),
            "a newer version starts over"
        );
        state.note_reminded(100 + 99 * day);
        assert_eq!(state.reminder_count, 1);
    }

    #[test]
    fn nothing_available_means_no_reminder() {
        assert!(!State::default().reminder_due(1_000_000));
        let state = State {
            available: None,
            reminded: Some("4.3.0".into()),
            ..State::default()
        };
        assert!(!state.reminder_due(1_000_000));
    }

    #[test]
    fn a_clock_set_back_does_not_silence_reminders() {
        let state = State {
            available: Some("4.3.0".into()),
            reminded: Some("4.3.0".into()),
            reminded_at: 9_000_000,
            reminder_count: 1,
            ..State::default()
        };
        assert!(state.reminder_due(1_000));
    }
}
