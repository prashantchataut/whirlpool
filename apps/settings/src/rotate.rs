//! Wallpaper rotation: which pictures, which display, applied through the shell. The dock calls
//! [`advance`] from one coarse timer that exists only while rotation is on.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use whirlpool_config::{Config, WallpaperFit};
use whirlpool_core::rotation;
use whirlpool_shell::desktop_wallpaper::{self, Fit};

use crate::photos;

const PICTURE_TYPES: [&str; 5] = ["jpg", "jpeg", "png", "bmp", "webp"];
/// More than this in one folder is almost certainly the wrong folder.
const MAX_PICTURES: usize = 5000;

/// Where rotation stands between ticks.
pub struct State {
    tick: u64,
    seed: u64,
}

impl State {
    /// Starts at a step derived from the clock, so a restart continues the sequence instead of
    /// showing the first picture again.
    #[must_use]
    pub fn new(every_minutes: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Self {
            tick: now / (u64::from(every_minutes.max(1)) * 60),
            seed: now / 86_400 + 1,
        }
    }
}

impl State {
    /// Moves `steps` further along, for scripted runs (`whirlpool --wallpaper-next 3`).
    #[must_use]
    pub const fn skipping(mut self, steps: u64) -> Self {
        self.tick = self.tick.wrapping_add(steps);
        self
    }
}

#[must_use]
pub const fn fit(config: WallpaperFit) -> Fit {
    match config {
        WallpaperFit::Fill => Fit::Fill,
        WallpaperFit::Fit => Fit::Fit,
        WallpaperFit::Stretch => Fit::Stretch,
        WallpaperFit::Tile => Fit::Tile,
        WallpaperFit::Center => Fit::Center,
    }
}

/// Pictures in `folder`, sorted by name.
#[must_use]
pub fn pictures_in(folder: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(folder)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                        PICTURE_TYPES
                            .iter()
                            .any(|kind| e.eq_ignore_ascii_case(kind))
                    })
                })
                .take(MAX_PICTURES)
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found
}

/// The folder in use now: the dark one while apps are dark and one is set, otherwise the main one.
#[must_use]
pub fn active_folder(config: &Config, dark: bool) -> Option<&str> {
    if dark && let Some(folder) = config.wallpaper_dark_folder.as_deref() {
        return Some(folder);
    }
    config.wallpaper_folder.as_deref()
}

/// Changes the wallpaper by one step. Returns how many displays were set.
///
/// # Errors
/// Fails when the source has no pictures or Windows refuses every one.
pub fn advance(config: &Config, dark: bool, state: &mut State) -> io::Result<usize> {
    let folder_pictures = active_folder(config, dark).map(|f| pictures_in(Path::new(f)));
    let count = folder_pictures
        .as_ref()
        .map_or(photos::PHOTOS.len(), Vec::len);
    if count == 0 {
        return Err(io::Error::other("the wallpaper folder has no pictures"));
    }
    let monitors = if config.wallpaper_per_monitor {
        desktop_wallpaper::monitors()
    } else {
        Vec::new()
    };
    let slots = monitors.len().max(1);
    let mut applied = 0;
    let mut last_error = None;
    for slot in 0..slots {
        let Some(index) = rotation::pick(
            count,
            config.wallpaper_shuffle,
            state.tick,
            slot,
            state.seed,
        ) else {
            continue;
        };
        let path = match &folder_pictures {
            Some(list) => list[index].clone(),
            None => {
                let photo = &photos::PHOTOS[index];
                let dir = whirlpool_shell::wallpaper::folder();
                fs::create_dir_all(&dir)?;
                let file = dir.join(format!("rotation-{slot}.jpg"));
                fs::write(&file, photo.jpeg)?;
                file
            }
        };
        let monitor = monitors.get(slot).map(|m| m.id.as_str());
        match desktop_wallpaper::set(monitor, &path, fit(config.wallpaper_fit)) {
            Ok(()) => applied += 1,
            Err(error) => last_error = Some(error),
        }
    }
    state.tick = state.tick.wrapping_add(1);
    if applied == 0 {
        return Err(
            last_error.unwrap_or_else(|| io::Error::other("no display accepted the picture"))
        );
    }
    Ok(applied)
}

/// Whether Windows apps are in dark mode right now.
#[must_use]
pub fn dark_now() -> bool {
    desktop_wallpaper::apps_use_dark()
}
