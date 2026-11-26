//! The wallpaper studio's network and desktop side: download one 4K picture from 4kwallpapers.com
//! into `%LOCALAPPDATA%\Whirlpool\Wallpapers` and make it the desktop wallpaper
//! (`SystemParametersInfoW(SPI_SETDESKWALLPAPER)`). Nothing is fetched until the user asks: a
//! preview is requested when the studio opens, a picture when Apply is pressed. No account, no
//! identifier, no telemetry; the only server contacted is 4kwallpapers.com.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use reqwest::{blocking, header};
use whirlpool_core::studio::{self, Entry};

const AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36";

/// Why a download did not produce a picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// No connection.
    Offline,
    /// The site answered with something that is not a picture (a bot check, a missing file).
    Refused(u16),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline => f.write_str("offline"),
            Self::Refused(code) => write!(f, "refused ({code})"),
        }
    }
}

/// `%LOCALAPPDATA%\Whirlpool\Wallpapers`.
#[must_use]
pub fn cache_dir() -> PathBuf {
    crate::wallpaper::folder()
}

fn client() -> Result<blocking::Client, Failure> {
    blocking::Client::builder()
        .user_agent(AGENT)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|_| Failure::Offline)
}

fn get(client: &blocking::Client, url: &str, accept: &str) -> Result<Vec<u8>, Failure> {
    let response = client
        .get(url)
        .header(header::ACCEPT, accept)
        .header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9")
        .header(header::REFERER, format!("{}/", studio::SITE))
        .send()
        .map_err(|_| Failure::Offline)?;
    let status = response.status();
    if !status.is_success() {
        return Err(Failure::Refused(status.as_u16()));
    }
    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|_| Failure::Offline)
}

fn get_image(client: &blocking::Client, url: &str) -> Result<Vec<u8>, Failure> {
    let bytes = get(
        client,
        url,
        "image/avif,image/webp,image/jpeg,image/*;q=0.8",
    )?;
    if studio::looks_like_image(&bytes) {
        Ok(bytes)
    } else {
        Err(Failure::Refused(200))
    }
}

/// The 800 px preview of an entry, cached on disk after the first time.
///
/// # Errors
/// See [`Failure`].
pub fn thumbnail(entry: &Entry) -> Result<Vec<u8>, Failure> {
    let file = cache_dir()
        .join("previews")
        .join(format!("{}.jpg", entry.id));
    if let Ok(bytes) = fs::read(&file)
        && studio::looks_like_image(&bytes)
    {
        return Ok(bytes);
    }
    let bytes = get_image(&client()?, &entry.thumb_url())?;
    if let Some(dir) = file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(&file, &bytes);
    Ok(bytes)
}

/// The path of an entry's 4K file if it was downloaded before.
#[must_use]
pub fn cached(entry: &Entry) -> Option<PathBuf> {
    let file = cache_dir().join(entry.file_name());
    file.is_file().then_some(file)
}

/// Downloads the 4K picture (if it is not on disk yet), sets it as the wallpaper and returns its path.
/// The page is read first for the exact 4K link; the usual address is the fallback.
///
/// # Errors
/// See [`Failure`]; a wallpaper that Windows refuses is reported as `Refused(0)`.
pub fn apply(entry: &Entry) -> Result<PathBuf, Failure> {
    let file = match cached(entry) {
        Some(file) => file,
        None => {
            let client = client()?;
            let link = get(&client, &entry.page_url(), "text/html")
                .ok()
                .and_then(|page| studio::find_download_link(&String::from_utf8_lossy(&page)))
                .unwrap_or_else(|| entry.download_url());
            let bytes = get_image(&client, &link)?;
            let file = cache_dir().join(entry.file_name());
            fs::create_dir_all(cache_dir()).map_err(|_| Failure::Refused(0))?;
            fs::write(&file, &bytes).map_err(|_| Failure::Refused(0))?;
            file
        }
    };
    crate::wallpaper::apply_file(&file).map_err(|_| Failure::Refused(0))?;
    prune(&file);
    Ok(file)
}

/// Keeps the four newest downloads; the picture in use is never removed.
fn prune(keep: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir()) else {
        return;
    };
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jpg"))
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .collect();
    files.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, path) in files.into_iter().skip(4) {
        if path != keep {
            let _ = fs::remove_file(path);
        }
    }
}
