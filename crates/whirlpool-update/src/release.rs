//! GitHub Releases JSON, asset choice and `SHA256SUMS.txt`.

use serde::Deserialize;

use crate::version::Version;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
}

impl Arch {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            Self::X64
        }
    }

    /// File name of the installer for this architecture.
    #[must_use]
    pub const fn installer_name(self) -> &'static str {
        match self {
            Self::X64 => "Whirlpool-Setup.exe",
            Self::Arm64 => "Whirlpool-Setup-arm64.exe",
        }
    }
}

pub const SUMS_NAME: &str = "SHA256SUMS.txt";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Stable,
    Beta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub version: Version,
    pub prerelease: bool,
    pub notes: String,
    pub page: String,
    pub published: String,
    pub assets: Vec<Asset>,
}

impl Release {
    #[must_use]
    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|asset| asset.name == name)
    }
}

#[derive(Deserialize)]
struct RawRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<RawAsset>,
}

#[derive(Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

impl RawRelease {
    fn into_release(self, repository: &str) -> Option<Release> {
        let version = Version::parse(&self.tag_name)?;
        // Only ever follow links that belong to this repository.
        if !self.html_url.starts_with(repository) {
            return None;
        }
        let assets = self
            .assets
            .into_iter()
            .filter(|asset| asset.browser_download_url.starts_with(repository))
            .map(|asset| Asset {
                name: asset.name,
                url: asset.browser_download_url,
                size: asset.size,
            })
            .collect();
        Some(Release {
            prerelease: self.prerelease || version.is_prerelease(),
            tag: self.tag_name,
            version,
            notes: self.body.unwrap_or_default(),
            page: self.html_url,
            published: self.published_at.unwrap_or_default(),
            assets,
        })
    }
}

/// Parses the `/releases` array and returns the highest version allowed by `channel`.
pub fn pick_latest(json: &str, repository: &str, channel: Channel) -> Option<Release> {
    let raw: Vec<RawRelease> = serde_json::from_str(json).ok()?;
    raw.into_iter()
        .filter(|release| !release.draft)
        .filter_map(|release| release.into_release(repository))
        .filter(|release| channel == Channel::Beta || !release.prerelease)
        .max_by(|a, b| a.version.cmp(&b.version))
}

/// Parses one release object (`/releases/tags/<tag>`).
pub fn parse_one(json: &str, repository: &str) -> Option<Release> {
    let raw: RawRelease = serde_json::from_str(json).ok()?;
    raw.into_release(repository)
}

/// Finds the SHA-256 of `file` in a `sha256sum`-style list (`<hex>  <name>` or `<hex> *<name>`).
#[must_use]
pub fn hash_for(sums: &str, file: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (hash, name) = line.trim().split_once(char::is_whitespace)?;
        let name = name.trim().trim_start_matches('*');
        let valid = hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit());
        (valid && name == file).then(|| hash.to_ascii_lowercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO: &str = "https://github.com/o/r";

    fn sample() -> String {
        format!(
            r#"[
          {{"tag_name":"v4.2.0-rc2","html_url":"{REPO}/releases/tag/v4.2.0-rc2","prerelease":true,"draft":false,"body":"rc2 notes","assets":[{{"name":"Whirlpool-Setup.exe","browser_download_url":"{REPO}/releases/download/v4.2.0-rc2/Whirlpool-Setup.exe","size":10}}]}},
          {{"tag_name":"v4.1.0","html_url":"{REPO}/releases/tag/v4.1.0","prerelease":false,"draft":false,"body":"old"}},
          {{"tag_name":"v9.9.9","html_url":"{REPO}/releases/tag/v9.9.9","prerelease":false,"draft":true}},
          {{"tag_name":"v7.0.0","html_url":"https://evil.example/x","prerelease":false,"draft":false}},
          {{"tag_name":"nightly","html_url":"{REPO}/releases/tag/nightly","prerelease":true,"draft":false}}
        ]"#
        )
    }

    #[test]
    fn stable_skips_prereleases_drafts_and_foreign_links() {
        let latest = pick_latest(&sample(), REPO, Channel::Stable).expect("a release");
        assert_eq!(latest.tag, "v4.1.0");
    }

    #[test]
    fn beta_takes_the_highest_prerelease() {
        let latest = pick_latest(&sample(), REPO, Channel::Beta).expect("a release");
        assert_eq!(latest.tag, "v4.2.0-rc2");
        assert_eq!(latest.notes, "rc2 notes");
        assert!(latest.asset("Whirlpool-Setup.exe").is_some());
    }

    #[test]
    fn finds_hashes() {
        let hash = "a".repeat(64);
        let sums = format!(
            "{hash}  Whirlpool-Setup.exe\n{}  *other.zip\n",
            "B".repeat(64)
        );
        assert_eq!(hash_for(&sums, "Whirlpool-Setup.exe"), Some(hash));
        assert_eq!(hash_for(&sums, "other.zip"), Some("b".repeat(64)));
        assert_eq!(hash_for(&sums, "missing.exe"), None);
        assert_eq!(
            hash_for("short  Whirlpool-Setup.exe", "Whirlpool-Setup.exe"),
            None
        );
    }
}
