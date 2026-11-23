//! Persistent Whirlpool configuration stored in `%APPDATA%\Whirlpool\config.json`.

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
    /// bring pins and unpins made there (even while Whirlpool was closed) to the
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
    /// running out, energy saver). Off by default; stored under its own key
    /// so earlier files that switched it on do not bring it back.
    #[serde(rename = "crab")]
    pub mascot: bool,
    /// Settings asks GitHub Releases for a newer version when it opens. The
    /// only network access in Whirlpool, so it can be turned off.
    pub check_updates: bool,
    /// With `check_updates`: the dock looks in the background every
    /// `update_check_days` days. It only records that a version exists; it
    /// never downloads or installs without a click in Settings.
    pub auto_update: bool,
    /// With `check_updates`: also look a couple of minutes after the dock starts.
    pub update_on_startup: bool,
    /// Days between background looks. 0 turns the periodic look off.
    pub update_check_days: u32,
    /// Offer release candidates as well as stable releases.
    pub update_prereleases: bool,
    /// Change the wallpaper on a timer. Off by default; no timer exists while it is off.
    pub wallpaper_rotation: bool,
    /// Minutes between changes (1 to 1440).
    pub wallpaper_every_minutes: u32,
    pub wallpaper_shuffle: bool,
    /// Folder of pictures to rotate through; `None` uses the photographs bundled with Whirlpool.
    pub wallpaper_folder: Option<String>,
    /// Used instead of `wallpaper_folder` while Windows apps are in dark mode.
    pub wallpaper_dark_folder: Option<String>,
    pub wallpaper_fit: WallpaperFit,
    /// Each display gets a different picture from the same source.
    pub wallpaper_per_monitor: bool,
    pub language: Language,
    /// A folder shown at the right of the dock whose contents open as a
    /// menu, with a submenu for each folder inside, like a Windows 10 taskbar
    /// toolbar. Only one: the user arranges the folder as they like.
    pub shortcuts_folder: Option<String>,
    /// How the dock moves: the feel of magnification and of sliding in and out.
    pub animation_style: AnimationStyle,
    /// How the dock arrives when Whirlpool starts.
    pub startup_animation: StartupAnimation,
    /// The charging scene when the charger is plugged in. Independent of the
    /// other mascot moments, which `mascot` controls.
    pub charging_animation: bool,
    /// How strong the menu bar's tint is over the glass, in percent. 100 is
    /// solid; the default matches the original look.
    pub menu_bar_opacity: u32,
    /// Show the date next to the time in the menu bar.
    pub menu_bar_date: bool,
    /// Force a 24-hour (`Some(true)`) or 12-hour (`Some(false)`) clock;
    /// `None` follows Windows.
    pub menu_bar_24h: Option<bool>,
    /// The wallpaper preset Whirlpool last applied (see `whirlpool-shell`).
    pub wallpaper: Option<String>,
    /// Docked (edge to edge) or floating (a rounded bar inset from the edges).
    pub menu_bar_style: BarStyle,
    pub menu_bar_size: BarSize,
    /// Side margin of a floating bar, in logical pixels.
    pub menu_bar_inset: u32,
    /// Gap between the top of the screen and a floating bar.
    pub menu_bar_gap: u32,
    /// Corner radius of a floating bar.
    pub menu_bar_radius: u32,
    pub charging_style: ChargingStyle,
    /// The screen edge the menu bar sits on.
    pub menu_bar_position: BarPosition,
    /// Where a floating menu bar sits along its edge.
    pub menu_bar_align: BarAlign,
    /// Width of a floating menu bar as a percentage of the screen.
    pub menu_bar_width: u32,
    /// The colour the menu bar is tinted with.
    pub menu_bar_tint: BarTint,
    /// A hairline highlight along the edge of the menu bar.
    pub menu_bar_border: bool,
}

/// The feel of every spring in the dock. Springs never overshoot; the style
/// changes how quickly they settle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationStyle {
    /// The slowest, dreamiest settle. Nothing hurries.
    Dreamy,
    /// Slow and floaty.
    Relaxed,
    /// A little slower than the default, with long, silky settles.
    Fluid,
    /// A little quicker than the default.
    Brisk,
    /// Quick and crisp.
    Snappy,
    /// As fast as motion can be and still read as motion.
    Rapid,
    /// The original feel. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Smooth,
}

impl AnimationStyle {
    /// Multiplier on every spring's angular frequency.
    #[must_use]
    pub fn speed(self) -> f32 {
        match self {
            Self::Dreamy => 0.55,
            Self::Relaxed => 0.7,
            Self::Fluid => 0.85,
            Self::Smooth => 1.0,
            Self::Brisk => 1.3,
            Self::Snappy => 1.6,
            Self::Rapid => 2.1,
        }
    }
}

/// How the dock comes in when it starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StartupAnimation {
    /// The dock simply appears.
    Off,
    /// The dock floats in from far away, slowly.
    Float,
    /// The slowest, softest arrival: the dock drifts in.
    Drift,
    /// A long, soft glide up from the edge.
    Glide,
    /// A quick slide.
    Swift,
    /// Nearly instant, with a hint of motion.
    Dart,
    /// A confident rise: the default. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Rise,
}

impl StartupAnimation {
    /// Angular frequency of the slide, or `None` when there is no animation.
    #[must_use]
    pub fn omega(self) -> Option<f32> {
        match self {
            Self::Off => None,
            Self::Float => Some(2.4),
            Self::Drift => Some(3.6),
            Self::Glide => Some(5.5),
            Self::Rise => Some(9.0),
            Self::Swift => Some(16.0),
            Self::Dart => Some(26.0),
        }
    }
}

/// How the menu bar sits on the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarStyle {
    /// A rounded bar inset from the screen edges.
    Floating,
    /// Edge to edge, flush with the top of the screen. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Docked,
}

/// How tall the menu bar is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarSize {
    Compact,
    Large,
    /// The default. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Regular,
}

impl BarSize {
    /// Height of the bar in logical pixels.
    #[must_use]
    pub fn height(self) -> f32 {
        match self {
            Self::Compact => 24.0,
            Self::Regular => 28.0,
            Self::Large => 34.0,
        }
    }
}

/// Where a floating menu bar sits along the top edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarAlign {
    Left,
    Right,
    /// The default. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Center,
}

/// Which screen edge the menu bar sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarPosition {
    Bottom,
    /// The default. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Top,
}

/// How a wallpaper picture is fitted to the display.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperFit {
    /// Cover the display, cropping the overflow.
    #[default]
    Fill,
    /// Show the whole picture with bars if needed.
    Fit,
    Stretch,
    Tile,
    Center,
}

/// The colour the menu bar is tinted with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarTint {
    /// Neutral dark grey.
    Graphite,
    /// Deep blue.
    Midnight,
    /// Green.
    Forest,
    /// Warm amber.
    Amber,
    /// Soft rose.
    Rose,
    /// Dusk violet.
    Violet,
    /// Cool mint.
    Mint,
    /// Red-orange, like the last light of the day.
    Sunset,
    /// Blue-grey stone.
    Slate,
    /// Warm paper.
    Sand,
    /// The default deep teal. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Ocean,
}

impl BarTint {
    /// The dark-mode tint as `[r, g, b]`.
    #[must_use]
    pub fn dark(self) -> [u8; 3] {
        match self {
            Self::Ocean => [6, 28, 36],
            Self::Graphite => [22, 23, 26],
            Self::Midnight => [10, 14, 38],
            Self::Forest => [8, 30, 20],
            Self::Amber => [42, 26, 8],
            Self::Rose => [40, 12, 24],
            Self::Violet => [26, 14, 44],
            Self::Mint => [8, 38, 32],
            Self::Sunset => [46, 18, 10],
            Self::Slate => [20, 26, 34],
            Self::Sand => [38, 33, 24],
        }
    }

    /// The light-mode tint as `[r, g, b]`.
    #[must_use]
    pub fn light(self) -> [u8; 3] {
        match self {
            Self::Ocean => [244, 252, 250],
            Self::Graphite => [246, 246, 247],
            Self::Midnight => [240, 243, 255],
            Self::Forest => [242, 252, 246],
            Self::Amber => [255, 249, 238],
            Self::Rose => [255, 242, 246],
            Self::Violet => [247, 242, 255],
            Self::Mint => [240, 253, 249],
            Self::Sunset => [255, 244, 238],
            Self::Slate => [241, 244, 248],
            Self::Sand => [252, 249, 241],
        }
    }
}

/// The animation that plays when the charger is plugged in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChargingStyle {
    /// A small glass capsule under the menu bar. Quiet.
    Halo,
    /// The edges of the screen glow like liquid glass: green when the
    /// battery is healthy, yellow in the middle, orange when it is low.
    Edge,
    /// A thin line of light sweeps along the top edge up to the battery level.
    Tide,
    /// A soft glow rises from the bottom of the screen, behind the dock.
    Bloom,
    /// Slow ribbons of colour drift across the top of the screen.
    Aurora,
    /// Rings of water spread from the medallion. The default. Also what an unknown value reads as.
    #[default]
    #[serde(other)]
    Surge,
}

/// Bounds of `Config::menu_bar_opacity`.
pub const MENU_BAR_OPACITY_RANGE: (u32, u32) = (10, 100);

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
        self.menu_bar_opacity = self
            .menu_bar_opacity
            .clamp(MENU_BAR_OPACITY_RANGE.0, MENU_BAR_OPACITY_RANGE.1);
        self.menu_bar_inset = self.menu_bar_inset.min(160);
        self.menu_bar_gap = self.menu_bar_gap.min(32);
        self.menu_bar_radius = self.menu_bar_radius.min(24);
        self.menu_bar_width = self.menu_bar_width.clamp(40, 100);
        self.wallpaper_every_minutes = self.wallpaper_every_minutes.clamp(1, 1440);
        for folder in [&mut self.wallpaper_folder, &mut self.wallpaper_dark_folder] {
            *folder = folder
                .take()
                .map(|f| f.trim().to_owned())
                .filter(|f| !f.is_empty());
        }
        self.wallpaper = self
            .wallpaper
            .map(|id| id.trim().to_owned())
            .filter(|id| !id.is_empty());
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
            mascot: false,
            check_updates: true,
            auto_update: true,
            update_on_startup: false,
            update_check_days: 7,
            update_prereleases: false,
            wallpaper_rotation: false,
            wallpaper_every_minutes: 30,
            wallpaper_shuffle: false,
            wallpaper_folder: None,
            wallpaper_dark_folder: None,
            wallpaper_fit: WallpaperFit::Fill,
            wallpaper_per_monitor: false,
            language: Language::Auto,
            shortcuts_folder: None,
            animation_style: AnimationStyle::Smooth,
            startup_animation: StartupAnimation::Rise,
            charging_animation: true,
            menu_bar_opacity: 50,
            menu_bar_date: true,
            menu_bar_24h: None,
            wallpaper: None,
            menu_bar_style: BarStyle::Docked,
            menu_bar_size: BarSize::Regular,
            menu_bar_inset: 24,
            menu_bar_gap: 8,
            menu_bar_radius: 14,
            charging_style: ChargingStyle::Surge,
            menu_bar_position: BarPosition::Top,
            menu_bar_align: BarAlign::Center,
            menu_bar_width: 100,
            menu_bar_tint: BarTint::Ocean,
            menu_bar_border: true,
        }
    }
}

/// Brings settings over from the earlier names of this app (`WinDock`, then
/// `Muelle`). The first folder that exists wins; the old one is renamed, so a
/// user who switches to Whirlpool keeps pins, groups and every preference.
pub fn migrate_legacy_folder() {
    let roaming = PathBuf::from(env::var_os("APPDATA").unwrap_or_default());
    let new = roaming.join("Whirlpool");
    if new.exists() {
        return;
    }
    for old in ["Muelle", "WinDock"] {
        let old = roaming.join(old);
        if old.is_dir() && fs::rename(&old, &new).is_ok() {
            return;
        }
    }
}

#[must_use]
pub fn default_path() -> PathBuf {
    PathBuf::from(env::var_os("APPDATA").unwrap_or_default())
        .join("Whirlpool")
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
        let folder = env::temp_dir().join(format!("whirlpool-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        folder.join("config.json")
    }

    #[test]
    fn new_look_options_round_trip_and_unknown_values_fall_back() {
        let config: Config = serde_json::from_str(
            r#"{"menuBarTint":"rose","menuBarAlign":"left","menuBarWidth":10,"chargingStyle":"edge","animationStyle":"fluid","startupAnimation":"drift","crab":true}"#,
        )
        .unwrap();
        let config = config.sanitized();
        assert_eq!(config.menu_bar_tint, BarTint::Rose);
        assert_eq!(config.menu_bar_align, BarAlign::Left);
        assert_eq!(config.menu_bar_width, 40);
        assert_eq!(config.charging_style, ChargingStyle::Edge);
        assert_eq!(config.animation_style, AnimationStyle::Fluid);
        assert_eq!(config.startup_animation, StartupAnimation::Drift);
        assert!(config.mascot);
        let odd: Config =
            serde_json::from_str(r#"{"menuBarTint":"plaid","chargingStyle":"future"}"#).unwrap();
        assert_eq!(odd.menu_bar_tint, BarTint::Ocean);
        assert_eq!(odd.charging_style, ChargingStyle::Surge);
        // The old key no longer switches the crab on.
        let old: Config = serde_json::from_str(r#"{"mascot":true}"#).unwrap();
        assert!(!old.mascot);
    }

    #[test]
    fn an_old_config_file_gets_the_new_options_at_their_defaults() {
        let old: Config = serde_json::from_str(r#"{"iconSize":52,"autoHide":true}"#).unwrap();
        assert_eq!(old.icon_size, 52);
        assert_eq!(old.startup_animation, StartupAnimation::Rise);
        assert_eq!(old.animation_style, AnimationStyle::Smooth);
        assert!(old.charging_animation);
        assert_eq!(old.menu_bar_opacity, 50);
    }

    #[test]
    fn values_from_a_newer_version_read_as_the_default() {
        let config: Config =
            serde_json::from_str(r#"{"animationStyle":"wobbly","startupAnimation":"warp"}"#)
                .unwrap();
        assert_eq!(config.animation_style, AnimationStyle::Smooth);
        assert_eq!(config.startup_animation, StartupAnimation::Rise);
    }

    #[test]
    fn out_of_range_opacity_and_blank_wallpaper_are_repaired() {
        let config = Config {
            menu_bar_opacity: 400,
            wallpaper: Some("  ".into()),
            ..Config::default()
        }
        .sanitized();
        assert_eq!(config.menu_bar_opacity, MENU_BAR_OPACITY_RANGE.1);
        assert_eq!(config.wallpaper, None);
        let low = Config {
            menu_bar_opacity: 0,
            ..Config::default()
        }
        .sanitized();
        assert_eq!(low.menu_bar_opacity, MENU_BAR_OPACITY_RANGE.0);
    }

    #[test]
    fn animation_speeds_are_ordered_and_off_has_no_slide() {
        assert!(AnimationStyle::Relaxed.speed() < AnimationStyle::Smooth.speed());
        assert!(AnimationStyle::Smooth.speed() < AnimationStyle::Snappy.speed());
        assert_eq!(StartupAnimation::Off.omega(), None);
        assert!(StartupAnimation::Glide.omega() < StartupAnimation::Swift.omega());
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
            id: "whirlpool:group:1".into(),
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
            r#"{"pinned":["a","whirlpool:group:1"],"groups":[{"id":"whirlpool:group:1","apps":["b","c"]}]}"#,
        )
        .ok();
        let config = load(&path).ok().flatten().unwrap_or_default();
        assert_eq!(
            config.groups,
            [DockGroup {
                id: "whirlpool:group:1".into(),
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

    #[test]
    fn menu_bar_position_reads_bottom_and_defaults_to_top() {
        let bottom: Config = serde_json::from_str(r#"{"menuBarPosition":"bottom"}"#).unwrap();
        assert_eq!(bottom.menu_bar_position, BarPosition::Bottom);
        let odd: Config = serde_json::from_str(r#"{"menuBarPosition":"sideways"}"#).unwrap();
        assert_eq!(odd.menu_bar_position, BarPosition::Top);
        assert_eq!(Config::default().menu_bar_position, BarPosition::Top);
    }
}
