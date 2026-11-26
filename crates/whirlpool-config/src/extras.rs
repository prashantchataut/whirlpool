//! Data the newer panels keep in `config.json`: the widget profile (a pinned slot plus favorites), saved
//! workspaces and saved audio presets. Everything read from a file or from an imported profile is
//! validated here, so the panels never see an unknown widget or an oversized list.

use serde::{Deserialize, Serialize};

/// Every widget the dock can show. A profile naming anything else is trimmed to these.
pub const WIDGETS: [&str; 10] = [
    "clock",
    "focus",
    "shelf",
    "clipboard",
    "meetings",
    "downloads",
    "battery",
    "audio",
    "workspaces",
    "calculator",
];

pub const MAX_FAVORITES: usize = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WidgetProfile {
    /// The one live widget shown in full. Empty when none.
    pub pinned: String,
    /// Up to four compact shortcuts, in order.
    pub favorites: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProfileError {
    NotJson,
    WrongKind,
    TooLarge,
}

impl WidgetProfile {
    /// Drops unknown and repeated widgets and anything past four favorites; a favorite that is also the
    /// pinned widget is removed from the favorites.
    pub fn normalized(mut self) -> Self {
        if !WIDGETS.contains(&self.pinned.as_str()) {
            self.pinned.clear();
        }
        let mut seen: Vec<String> = Vec::new();
        for f in self.favorites.drain(..) {
            if WIDGETS.contains(&f.as_str())
                && f != self.pinned
                && !seen.contains(&f)
                && seen.len() < MAX_FAVORITES
            {
                seen.push(f);
            }
        }
        self.favorites = seen;
        self
    }

    /// Pins `id`. The previous pinned widget takes the place of `id` when `id` was a favorite, otherwise
    /// it is dropped from the profile's slot but stays available in the catalogue.
    pub fn pin(&mut self, id: &str) {
        if !WIDGETS.contains(&id) || self.pinned == id {
            return;
        }
        let previous = std::mem::replace(&mut self.pinned, id.to_owned());
        if let Some(at) = self.favorites.iter().position(|f| f == id) {
            if previous.is_empty() {
                self.favorites.remove(at);
            } else {
                self.favorites[at] = previous;
            }
        }
    }

    /// Stars or unstars `id`. Returns false when four favorites are already set.
    pub fn toggle_favorite(&mut self, id: &str) -> bool {
        if let Some(at) = self.favorites.iter().position(|f| f == id) {
            self.favorites.remove(at);
            return true;
        }
        if !WIDGETS.contains(&id) || id == self.pinned || self.favorites.len() >= MAX_FAVORITES {
            return false;
        }
        self.favorites.push(id.to_owned());
        true
    }

    /// Moves a favorite one place earlier (`-1`) or later (`1`).
    pub fn move_favorite(&mut self, id: &str, by: i32) {
        let Some(at) = self.favorites.iter().position(|f| f == id) else {
            return;
        };
        let to = (at as i32 + by).clamp(0, self.favorites.len() as i32 - 1) as usize;
        let item = self.favorites.remove(at);
        self.favorites.insert(to, item);
    }

    pub fn export(&self) -> String {
        let body = serde_json::json!({ "whirlpool": "widget-profile", "version": 1, "pinned": self.pinned, "favorites": self.favorites });
        serde_json::to_string_pretty(&body).unwrap_or_default()
    }

    pub fn import(text: &str) -> Result<WidgetProfile, ProfileError> {
        if text.len() > 8 * 1024 {
            return Err(ProfileError::TooLarge);
        }
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|_| ProfileError::NotJson)?;
        if value.get("whirlpool").and_then(|v| v.as_str()) != Some("widget-profile") {
            return Err(ProfileError::WrongKind);
        }
        let text_of =
            |v: Option<&serde_json::Value>| v.and_then(|x| x.as_str()).unwrap_or("").to_owned();
        let favorites = value
            .get("favorites")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        Ok(WidgetProfile {
            pinned: text_of(value.get("pinned")),
            favorites,
        }
        .normalized())
    }
}

/// One window a workspace should find again: matched by program, then by a piece of its title.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WorkspaceWindow {
    pub exe: String,
    pub title_contains: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Workspace {
    pub name: String,
    /// A folder opened in Explorer.
    pub folder: String,
    /// Only `https://` addresses are kept.
    pub urls: Vec<String>,
    pub windows: Vec<WorkspaceWindow>,
    /// `side`, `stacked` or `grid`.
    pub layout: String,
}

impl Workspace {
    pub fn normalized(mut self) -> Self {
        self.name = self.name.trim().chars().take(40).collect();
        self.urls
            .retain(|u| u.starts_with("https://") && !u.contains(char::is_whitespace));
        self.urls.truncate(12);
        self.windows.truncate(12);
        if !["side", "stacked", "grid"].contains(&self.layout.as_str()) {
            self.layout = "grid".into();
        }
        self
    }
}

/// A saved sound setup: the default output and the level of each app that had a stream.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AudioPreset {
    pub name: String,
    /// Endpoint id of the default output, as Windows names it.
    pub output: String,
    /// 0 to 100.
    pub master: u8,
    pub apps: Vec<AppLevel>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppLevel {
    pub exe: String,
    pub volume: u8,
    pub muted: bool,
    /// Output endpoint id this app was routed to; empty for the default.
    pub route: String,
}

/// What reapplying a preset must do right now, given the apps that currently have a stream.
#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    pub set_output: bool,
    pub master: u8,
    /// (exe, volume, muted, route) for apps that are present.
    pub apply: Vec<(String, u8, bool, String)>,
    /// Apps in the preset with no stream now: reapply after they open one.
    pub waiting: Vec<String>,
}

impl AudioPreset {
    pub fn plan(&self, current_output: &str, running: &[String]) -> Plan {
        let mut apply = Vec::new();
        let mut waiting = Vec::new();
        for a in &self.apps {
            if running.iter().any(|r| r.eq_ignore_ascii_case(&a.exe)) {
                apply.push((a.exe.clone(), a.volume.min(100), a.muted, a.route.clone()));
            } else {
                waiting.push(a.exe.clone());
            }
        }
        Plan {
            set_output: !self.output.is_empty() && self.output != current_output,
            master: self.master.min(100),
            apply,
            waiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_normalises_unknown_repeated_and_extra_widgets() {
        let p = WidgetProfile {
            pinned: "nope".into(),
            favorites: [
                "clock", "clock", "focus", "shelf", "bogus", "meetings", "audio", "battery",
            ]
            .map(String::from)
            .to_vec(),
        }
        .normalized();
        assert_eq!(p.pinned, "");
        assert_eq!(p.favorites, ["clock", "focus", "shelf", "meetings"]);
    }

    #[test]
    fn pinning_a_favorite_swaps_with_the_previous_pin() {
        let mut p = WidgetProfile {
            pinned: "clock".into(),
            favorites: vec!["focus".into(), "shelf".into()],
        };
        p.pin("shelf");
        assert_eq!(p.pinned, "shelf");
        assert_eq!(p.favorites, ["focus", "clock"]);
        p.pin("battery");
        assert_eq!(p.pinned, "battery");
        assert_eq!(p.favorites, ["focus", "clock"]);
    }

    #[test]
    fn favorites_stop_at_four_and_can_move() {
        let mut p = WidgetProfile::default();
        for w in ["clock", "focus", "shelf", "meetings"] {
            assert!(p.toggle_favorite(w));
        }
        assert!(!p.toggle_favorite("audio"));
        p.move_favorite("meetings", -2);
        assert_eq!(p.favorites, ["clock", "meetings", "focus", "shelf"]);
        p.move_favorite("clock", -1);
        assert_eq!(p.favorites[0], "clock");
        assert!(p.toggle_favorite("clock"));
        assert_eq!(p.favorites.len(), 3);
    }

    #[test]
    fn export_then_import_round_trips_and_refuses_other_files() {
        let p = WidgetProfile {
            pinned: "focus".into(),
            favorites: vec!["clock".into(), "audio".into()],
        };
        assert_eq!(WidgetProfile::import(&p.export()), Ok(p));
        assert_eq!(
            WidgetProfile::import("{\"pinned\":\"clock\"}"),
            Err(ProfileError::WrongKind)
        );
        assert_eq!(WidgetProfile::import("nope"), Err(ProfileError::NotJson));
        assert_eq!(
            WidgetProfile::import(&" ".repeat(9000)),
            Err(ProfileError::TooLarge)
        );
        let hostile = "{\"whirlpool\":\"widget-profile\",\"pinned\":\"<script>\",\"favorites\":[\"clock\",5,\"../x\"]}";
        let got = WidgetProfile::import(hostile).unwrap();
        assert_eq!(
            (got.pinned.as_str(), got.favorites),
            ("", vec!["clock".to_owned()])
        );
    }

    #[test]
    fn workspace_keeps_only_https_and_a_known_layout() {
        let w = Workspace {
            name: "  Docs  ".into(),
            urls: vec![
                "https://a.example/x".into(),
                "javascript:alert(1)".into(),
                "http://b.example".into(),
                "https://c d".into(),
            ],
            layout: "diagonal".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!(w.name, "Docs");
        assert_eq!(w.urls, ["https://a.example/x"]);
        assert_eq!(w.layout, "grid");
    }

    #[test]
    fn audio_preset_plan_waits_for_absent_apps() {
        let p = AudioPreset {
            name: "Focus".into(),
            output: "{out-2}".into(),
            master: 40,
            apps: vec![
                AppLevel {
                    exe: "spotify.exe".into(),
                    volume: 30,
                    muted: false,
                    route: "{out-2}".into(),
                },
                AppLevel {
                    exe: "chrome.exe".into(),
                    volume: 0,
                    muted: true,
                    route: String::new(),
                },
            ],
        };
        let plan = p.plan("{out-1}", &["Spotify.exe".to_owned()]);
        assert!(plan.set_output);
        assert_eq!(
            plan.apply,
            vec![("spotify.exe".to_owned(), 30, false, "{out-2}".to_owned())]
        );
        assert_eq!(plan.waiting, ["chrome.exe"]);
        assert!(!p.plan("{out-2}", &[]).set_output);
    }

    #[test]
    fn old_files_without_the_new_keys_still_load() {
        let w: Workspace = serde_json::from_str("{\"name\":\"x\"}").unwrap();
        assert_eq!(w.layout, "");
        let p: WidgetProfile = serde_json::from_str("{}").unwrap();
        assert!(p.pinned.is_empty());
    }
}
