//! Persistent Muelle configuration stored in `%APPDATA%\Muelle\config.json`.

use std::{
    env,
    fs::{self, File, TryLockError},
    io,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CURRENT_VERSION: u32 = 1;

/// Error reading or saving the configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("no se pudo acceder a `{path}`")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("`{path}` no es una configuración válida")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("no se pudo serializar la configuración")]
    Serialize(#[from] serde_json::Error),
}

/// User settings. Missing fields use their default values, so adding fields
/// does not break existing configuration files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub version: u32,
    /// Pinned items in dock order: apps, and the ids of `groups`.
    pub pinned: Vec<String>,
    /// Pinned apps gathered in one dock icon each.
    pub groups: Vec<DockGroup>,
    pub recent: Vec<String>,
    /// The apps pinned to the Windows taskbar when the dock last looked, to
    /// bring pins and unpins made there (even while Muelle was closed) to the
    /// dock. `None` until it first looks.
    pub taskbar_pins: Option<Vec<String>>,
    #[serde(alias = "auto_hide")]
    pub auto_hide: bool,
    /// Icon side at rest, in logical pixels.
    pub icon_size: u32,
    pub magnification: bool,
    /// Side of the icon under the cursor, in logical pixels.
    pub magnified_size: u32,
    /// The screen edge the dock sits on. Bottom by default, as on macOS;
    /// never the top, which belongs to the menu bar.
    pub dock_position: DockPosition,
    /// Genie effect when minimizing. On by default; while enabled, the dock
    /// turns off the system's own window transitions (see plan, "Efecto genio").
    pub genie: bool,
    /// How long the genie takes (issue #27). `Normal` is the original speed.
    pub genie_speed: GenieSpeed,
    /// Minimized windows get a live thumbnail in the right section of the
    /// dock. Off by default: they go into their app's icon instead.
    pub show_minimized: bool,
    /// macOS-style bar across the top with the clock and status items.
    pub menu_bar: bool,
    /// Keep the menu bar off screen until the pointer touches the top edge.
    pub menu_bar_auto_hide: bool,
    /// Show supported AI providers' plan usage in the menu bar.
    pub ai_usage: bool,
    /// Show what is playing (title, controls) in the menu bar. With this and
    /// `ai_usage` off there is no island at all (issue #28).
    pub menu_bar_music: bool,
    /// Visual style and placement of the music and AI usage island in the menu bar.
    pub menu_bar_island_style: IslandStyle,
    /// Opt-in peripheral monitoring. Off means no accessory reader or queries.
    pub accessory_battery: bool,
    /// Show search button in the menu bar (`None` follows Windows taskbar setting).
    pub menu_bar_search: Option<bool>,
    /// Show Task View button in the menu bar (`None` follows Windows taskbar setting).
    pub menu_bar_task_view: Option<bool>,
    /// Show Widgets button in the menu bar (`None` follows Windows taskbar setting).
    pub menu_bar_widgets: Option<bool>,
    /// Show Bluetooth button in the menu bar.
    pub menu_bar_bluetooth: bool,
    /// Show battery percentage in the menu bar (`None` follows Windows setting).
    pub menu_bar_battery_percent: Option<bool>,
    /// The dot under apps that have windows open.
    pub show_indicators: bool,
    /// The section of recently used apps that are not pinned.
    pub show_recents: bool,
    /// Icons bounce while the app they open is starting.
    pub bounce_on_open: bool,
    /// The crab comes out for moments worth it (an AI plan resetting or
    /// running out, the charger, energy saver).
    pub mascot: bool,
    /// Settings asks GitHub Releases for a newer version when it opens. The
    /// only network access in Muelle, so it can be turned off.
    pub check_updates: bool,
    /// With `check_updates`: once a day, while the user is away from the
    /// computer, new versions are downloaded and installed without asking.
    pub auto_update: bool,
    pub language: Language,
    /// A folder shown at the right of the dock whose contents open as a
    /// menu, with a submenu for each folder inside, like a Windows 10 taskbar
    /// toolbar. Only one: the user arranges the folder as they like.
    pub shortcuts_folder: Option<String>,
}

/// Pinned apps the user gathered in one dock icon by dropping one on
/// another. Its `id` stands for it in `pinned`, in its place.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DockGroup {
    pub id: String,
    pub apps: Vec<String>,
}

/// Speed of the genie. Three steps, not a slider: a duration in milliseconds
/// would ask the user to guess, and these three cover what anyone asked for.
/// The screen edge the dock sits on: the three macOS offers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockPosition {
    Left,
    Right,
    /// Also what an unknown value from a newer version reads as (serde
    /// needs this fallback last).
    #[default]
    #[serde(other)]
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenieSpeed {
    Fast,
    Faster,
    /// The original 0.5 s, close to the macOS Dock. Also what an unknown
    /// value from a newer version reads as (serde needs it last).
    #[default]
    #[serde(other)]
    Normal,
}

impl GenieSpeed {
    /// How long one genie flight lasts, minimizing or restoring.
    #[must_use]
    pub fn seconds(self) -> f32 {
        match self {
            Self::Normal => 0.5,
            Self::Fast => 0.35,
            Self::Faster => 0.25,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IslandStyle {
    /// Compact rounded bubble placed next to the right-side status indicators.
    Bubble,
    /// Centered curved notch hanging from the top of the menu bar.
    #[default]
    #[serde(other)]
    Notch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Es,
    En,
    Fr,
    /// Follow the Windows display language. Also what a language added by a
    /// newer version reads as, so the file stays valid for older versions
    /// (serde requires this fallback to be the last variant).
    #[default]
    #[serde(other)]
    Auto,
}

pub const ICON_SIZE_RANGE: (u32, u32) = (32, 96);
pub const MAX_MAGNIFIED_SIZE: u32 = 160;

impl Config {
    /// Brings out-of-range values (a hand-edited file) back into range.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        self.icon_size = self.icon_size.clamp(ICON_SIZE_RANGE.0, ICON_SIZE_RANGE.1);
        self.magnified_size = self
            .magnified_size
            .clamp(self.icon_size, MAX_MAGNIFIED_SIZE);
        self.shortcuts_folder = self
            .shortcuts_folder
            .map(|folder| folder.trim().to_owned())
            .filter(|folder| !folder.is_empty());
        self
    }

    /// Scale of the icon under the cursor (1.0 = no magnification).
    #[must_use]
    pub fn max_scale(&self) -> f32 {
        if self.magnification {
            self.magnified_size as f32 / self.icon_size as f32
        } else {
            1.0
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            pinned: Vec::new(),
            groups: Vec::new(),
            recent: Vec::new(),
            taskbar_pins: None,
            auto_hide: false,
            icon_size: 48,
            magnification: true,
            dock_position: DockPosition::Bottom,
            magnified_size: 77,
            genie: true,
            genie_speed: GenieSpeed::Normal,
            show_minimized: false,
            menu_bar: true,
            menu_bar_auto_hide: false,
            ai_usage: true,
            menu_bar_music: true,
            menu_bar_island_style: IslandStyle::Notch,
            accessory_battery: false,
            menu_bar_search: None,
            menu_bar_task_view: None,
            menu_bar_widgets: None,
            menu_bar_bluetooth: false,
            menu_bar_battery_percent: None,
            show_indicators: true,
            show_recents: true,
            bounce_on_open: true,
            mascot: true,
            check_updates: true,
            auto_update: true,
            language: Language::Auto,
            shortcuts_folder: None,
        }
    }
}

/// Moves the settings folder from before the app was renamed to Muelle.
pub fn migrate_legacy_folder() {
    let roaming = PathBuf::from(env::var_os("APPDATA").unwrap_or_default());
    let (old, new) = (roaming.join("WinDock"), roaming.join("Muelle"));
    if old.is_dir() && !new.exists() {
        let _ = fs::rename(old, new);
    }
}

#[must_use]
pub fn default_path() -> PathBuf {
    PathBuf::from(env::var_os("APPDATA").unwrap_or_default())
        .join("Muelle")
        .join("config.json")
}

/// Reads the configuration. Returns `Ok(None)` if the file does not exist yet.
pub fn load(path: &Path) -> Result<Option<Config>, ConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ConfigError::Io {
                path: path.to_owned(),
                source,
            });
        }
    };
    serde_json::from_str(&text)
        .map(|config: Config| Some(config.sanitized()))
        .map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        })
}

/// Replaces the configuration under the same lock used by incremental edits.
pub fn save(path: &Path, config: &Config) -> Result<(), ConfigError> {
    let _lock = lock_for_write(path)?;
    write_config(path, config)
}

/// Reads, changes and publishes one version while other writers wait. Readers
/// can still load the last complete version without acquiring the lock.
pub fn update(
    path: &Path,
    edit: impl FnOnce(Option<Config>) -> Result<Config, ConfigError>,
) -> Result<Config, ConfigError> {
    let _lock = lock_for_write(path)?;
    let config = edit(load(path)?)?.sanitized();
    write_config(path, &config)?;
    Ok(config)
}

/// Updates only preferences explicitly changed in Settings. The roster and
/// schema version belong to the dock and may have changed since Settings opened.
pub fn update_preferences(
    path: &Path,
    patch: serde_json::Map<String, serde_json::Value>,
) -> Result<Config, ConfigError> {
    update(path, |saved| {
        let mut fields = serde_json::to_value(saved.unwrap_or_default())?;
        if let Some(fields) = fields.as_object_mut() {
            for (key, value) in patch {
                if fields.contains_key(&key)
                    && !matches!(
                        key.as_str(),
                        "version" | "pinned" | "groups" | "recent" | "taskbarPins"
                    )
                {
                    fields.insert(key, value);
                }
            }
        }
        serde_json::from_value(fields).map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        })
    })
}

fn lock_for_write(path: &Path) -> Result<File, ConfigError> {
    let io_error = |source| ConfigError::Io {
        path: path.to_owned(),
        source,
    };
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(io_error)?;
    }
    // Lock a separate file so the atomic rename cannot replace the lock.
    // Closing the handle releases it, including when a process exits.
    let lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("json.lock"))
        .map_err(io_error)?;
    let start = Instant::now();
    loop {
        match lock.try_lock() {
            Ok(()) => return Ok(lock),
            Err(TryLockError::WouldBlock) if start.elapsed() < Duration::from_millis(80) => {
                thread::sleep(Duration::from_millis(4));
            }
            Err(TryLockError::WouldBlock) => {
                return Err(io_error(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Ajustes y el dock están guardando la configuración",
                )));
            }
            Err(TryLockError::Error(error)) => return Err(io_error(error)),
        }
    }
}

fn write_config(path: &Path, config: &Config) -> Result<(), ConfigError> {
    let io_error = |source| ConfigError::Io {
        path: path.to_owned(),
        source,
    };
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_string_pretty(config)?).map_err(io_error)?;
    fs::rename(&temporary, path).map_err(io_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder = env::temp_dir().join(format!("muelle-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        folder.join("config.json")
    }

    #[test]
    fn load_should_return_none_when_file_is_missing() {
        assert_eq!(load(&scratch("missing")).ok(), Some(None));
    }

    #[test]
    fn save_then_load_should_round_trip() {
        let path = scratch("round-trip");
        let config = Config {
            pinned: vec!["explorer".into()],
            ..Config::default()
        };
        save(&path, &config).ok();
        assert_eq!(load(&path).ok(), Some(Some(config)));
    }

    #[test]
    fn settings_patch_keeps_dock_edits_made_since_opening() -> Result<(), ConfigError> {
        let path = scratch("settings-patch");
        let initial = Config::default();
        save(&path, &initial)?;
        update(&path, |saved| {
            let mut config = saved.unwrap_or_default();
            config.auto_hide = true;
            config.pinned.push("browser".into());
            Ok(config)
        })?;
        let patch = serde_json::Map::from_iter([
            ("iconSize".into(), 64.into()),
            ("pinned".into(), serde_json::Value::Array(Vec::new())),
        ]);
        let saved = update_preferences(&path, patch)?;
        assert!(saved.auto_hide);
        assert_eq!(saved.pinned, ["browser"]);
        assert_eq!(saved.icon_size, 64);
        Ok(())
    }

    #[test]
    fn accessory_monitoring_is_opt_in_and_its_preference_round_trips() -> Result<(), ConfigError> {
        let path = scratch("accessory-monitoring");
        assert!(!Config::default().accessory_battery);
        for enabled in [true, false] {
            let saved = update_preferences(
                &path,
                serde_json::Map::from_iter([("accessoryBattery".into(), enabled.into())]),
            )?;
            assert_eq!(saved.accessory_battery, enabled);
            assert_eq!(
                load(&path)?.map(|config| config.accessory_battery),
                Some(enabled)
            );
        }
        Ok(())
    }

    #[test]
    fn settings_patch_keeps_the_dock_groups() -> Result<(), ConfigError> {
        let path = scratch("settings-patch-groups");
        let group = DockGroup {
            id: "muelle:group:1".into(),
            apps: vec!["a".into(), "b".into()],
        };
        save(
            &path,
            &Config {
                pinned: vec![group.id.clone()],
                groups: vec![group.clone()],
                ..Config::default()
            },
        )?;
        let patch = serde_json::Map::from_iter([
            ("iconSize".into(), 64.into()),
            ("groups".into(), serde_json::Value::Array(Vec::new())),
        ]);
        let saved = update_preferences(&path, patch)?;
        assert_eq!(saved.groups, [group]);
        assert_eq!(saved.icon_size, 64);
        Ok(())
    }

    #[test]
    fn a_busy_writer_keeps_the_last_complete_configuration() -> Result<(), ConfigError> {
        let path = scratch("write-contention");
        let original = Config::default();
        save(&path, &original)?;
        let _writer = lock_for_write(&path)?;
        let result = update(&path, |_| panic!("contending writer cannot edit"));
        assert!(
            matches!(result, Err(ConfigError::Io { source, .. }) if source.kind() == io::ErrorKind::WouldBlock)
        );
        assert_eq!(load(&path)?, Some(original));
        Ok(())
    }

    #[test]
    fn load_should_fill_missing_fields_with_defaults() {
        let path = scratch("partial");
        save(&path, &Config::default()).ok();
        fs::write(&path, "{}").ok();
        assert_eq!(load(&path).ok(), Some(Some(Config::default())));
    }

    #[test]
    fn load_should_give_older_files_the_new_behavior_defaults() {
        let path = scratch("before-behavior");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"version":1,"iconSize":56}"#).ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert_eq!(config.icon_size, 56);
        assert!(config.show_indicators && config.show_recents && config.bounce_on_open);
        assert!(config.check_updates);
        // Files from before automatic updates get them, like new installs.
        assert!(config.auto_update);
        assert!(config.ai_usage);
        assert!(config.menu_bar_music);
        assert!(!config.accessory_battery);
        assert!(!config.show_minimized);
    }

    #[test]
    fn load_should_read_dock_groups() {
        let path = scratch("groups");
        save(&path, &Config::default()).ok();
        fs::write(
            &path,
            r#"{"pinned":["a","muelle:group:1"],"groups":[{"id":"muelle:group:1","apps":["b","c"]}]}"#,
        )
        .ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert_eq!(
            config.groups,
            [DockGroup {
                id: "muelle:group:1".into(),
                apps: vec!["b".into(), "c".into()],
            }]
        );
    }

    #[test]
    fn load_should_read_show_minimized() {
        let path = scratch("show-minimized");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"showMinimized":true}"#).ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert!(config.show_minimized);
    }

    #[test]
    fn load_should_read_turned_off_update_checks() {
        let path = scratch("no-update-checks");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"checkUpdates":false}"#).ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert!(!config.check_updates);
    }

    #[test]
    fn load_should_read_the_behavior_settings() {
        let path = scratch("behavior");
        save(&path, &Config::default()).ok();
        fs::write(
            &path,
            r#"{"activeAppClick":"nothing","showRecents":false,"bounceOnOpen":false}"#,
        )
        .ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert!(!config.show_recents && !config.bounce_on_open);
    }

    #[test]
    fn load_should_read_french() {
        let path = scratch("french");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"language":"fr"}"#).ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert_eq!(config.language, Language::Fr);
    }

    #[test]
    fn load_should_read_an_unknown_language_as_auto() {
        let path = scratch("unknown-language");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"language":"de","iconSize":56}"#).ok();
        let config = load(&path).ok().flatten();
        assert_eq!(
            config.map(|config| (config.language, config.icon_size)),
            Some((Language::Auto, 56))
        );
    }

    #[test]
    fn load_should_read_the_shortcuts_folder() {
        let path = scratch("shortcuts-folder");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"shortcutsFolder":"D:\\Racc"}"#).ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert_eq!(config.shortcuts_folder.as_deref(), Some(r"D:\Racc"));
    }

    #[test]
    fn sanitized_should_forget_a_blank_shortcuts_folder() {
        let config = Config {
            shortcuts_folder: Some("  ".into()),
            ..Config::default()
        };
        assert_eq!(config.sanitized().shortcuts_folder, None);
    }

    #[test]
    fn load_should_fail_on_invalid_json() {
        let path = scratch("invalid");
        save(&path, &Config::default()).ok();
        fs::write(&path, "esto no es json").ok();
        assert!(matches!(load(&path), Err(ConfigError::Parse { .. })));
    }

    #[test]
    fn sanitized_should_clamp_icon_size() {
        let config = Config {
            icon_size: 500,
            ..Config::default()
        };
        assert_eq!(config.sanitized().icon_size, ICON_SIZE_RANGE.1);
    }

    #[test]
    fn sanitized_should_never_magnify_below_the_icon_size() {
        let config = Config {
            icon_size: 80,
            magnified_size: 40,
            ..Config::default()
        };
        assert_eq!(config.sanitized().magnified_size, 80);
    }

    #[test]
    fn max_scale_should_be_one_when_magnification_is_off() {
        let config = Config {
            magnification: false,
            ..Config::default()
        };
        assert_eq!(config.max_scale(), 1.0);
    }

    #[test]
    fn genie_should_be_on_by_default() {
        assert!(Config::default().genie);
        assert_eq!(Config::default().genie_speed, GenieSpeed::Normal);
    }

    #[test]
    fn the_dock_stays_at_the_bottom_unless_moved() {
        for json in [r#"{}"#, r#"{"dockPosition":"top"}"#] {
            let config: Config = serde_json::from_str(json).unwrap_or_default();
            assert_eq!(config.dock_position, DockPosition::Bottom);
        }
        let left: Config = serde_json::from_str(r#"{"dockPosition":"left"}"#).unwrap_or_default();
        assert_eq!(left.dock_position, DockPosition::Left);
    }

    #[test]
    fn genie_speed_keeps_the_original_speed_unless_chosen() {
        // Files from before the setting, and values from newer versions.
        for json in [r#"{}"#, r#"{"genieSpeed":"warp"}"#] {
            let config: Config = serde_json::from_str(json).unwrap_or_default();
            assert_eq!(config.genie_speed, GenieSpeed::Normal);
        }
        let fast: Config = serde_json::from_str(r#"{"genieSpeed":"faster"}"#).unwrap_or_default();
        assert_eq!(fast.genie_speed, GenieSpeed::Faster);
        assert!(GenieSpeed::Faster.seconds() < GenieSpeed::Fast.seconds());
        assert!(GenieSpeed::Fast.seconds() < GenieSpeed::Normal.seconds());
    }

    #[test]
    fn load_should_read_files_written_before_camel_case_keys() {
        let path = scratch("legacy-keys");
        save(&path, &Config::default()).ok();
        fs::write(&path, r#"{"version":1,"pinned":["a"],"auto_hide":true}"#).ok();
        let loaded = load(&path).ok().flatten();
        assert_eq!(loaded.map(|config| config.auto_hide), Some(true));
    }
}
