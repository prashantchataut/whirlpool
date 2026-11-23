use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    API, Arch, Channel, Http, NetError, REPOSITORY, Release, SUMS_NAME, Version, hash_for,
    parse_one, pick_latest,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    UpToDate,
    Available(Release),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    Offline,
    RateLimited { minutes: u64 },
    Http(u16),
    BadResponse,
    NoRelease,
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offline => {
                f.write_str("No connection to GitHub. Check your network and try again.")
            }
            Self::RateLimited { minutes } => write!(
                f,
                "GitHub is limiting requests from this network. Try again in about {} min.",
                (*minutes).max(1)
            ),
            Self::Http(code) => write!(f, "GitHub answered with HTTP {code}."),
            Self::BadResponse => f.write_str("GitHub sent an answer Whirlpool could not read."),
            Self::NoRelease => f.write_str("No matching release was found."),
        }
    }
}

fn map_net(error: &NetError) -> CheckError {
    match error {
        NetError::Offline | NetError::Io(_) | NetError::Cancelled => CheckError::Offline,
        NetError::Status {
            rate_limited: true,
            retry_after,
            ..
        } => CheckError::RateLimited {
            minutes: retry_after.unwrap_or(3600).div_ceil(60),
        },
        NetError::Status { code, .. } => CheckError::Http(*code),
    }
}

/// Looks for the newest release the channel allows and compares it with `current`.
pub fn check(http: &dyn Http, current: &Version, channel: Channel) -> Result<Outcome, CheckError> {
    let body = http
        .get_text(&format!("{API}/releases?per_page=30"))
        .map_err(|error| map_net(&error))?;
    let latest = pick_latest(&body, REPOSITORY, channel).ok_or(CheckError::NoRelease)?;
    Ok(if latest.version > *current {
        Outcome::Available(latest)
    } else {
        Outcome::UpToDate
    })
}

/// One specific release, for rollback.
pub fn check_tag(http: &dyn Http, tag: &str) -> Result<Release, CheckError> {
    if !tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        return Err(CheckError::NoRelease);
    }
    let body = http
        .get_text(&format!("{API}/releases/tags/{tag}"))
        .map_err(|error| map_net(&error))?;
    parse_one(&body, REPOSITORY).ok_or(CheckError::BadResponse)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    Downloading { done: u64, total: Option<u64> },
    Verifying,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateError {
    /// The release has no installer for this CPU or no checksum list.
    MissingAsset(String),
    Net(CheckError),
    /// The download does not match the published checksum. The file is deleted.
    HashMismatch {
        expected: String,
        actual: String,
    },
    Disk(String),
    Cancelled,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAsset(name) => write!(f, "This release has no {name}."),
            Self::Net(error) => error.fmt(f),
            Self::HashMismatch { .. } => f.write_str(
                "The download does not match the published SHA-256 and was deleted. Nothing was installed.",
            ),
            Self::Disk(why) => write!(f, "Could not write the download: {why}"),
            Self::Cancelled => f.write_str("Cancelled."),
        }
    }
}

#[must_use]
pub fn sha256_hex(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut hex, byte| {
            use std::fmt::Write;
            let _ = write!(hex, "{byte:02x}");
            hex
        }))
}

/// Downloads the installer for `arch` into `dir/<tag>/` and checks it against the release's
/// `SHA256SUMS.txt`. Returns the verified path. A file that fails the check is deleted.
pub fn download_and_verify(
    http: &dyn Http,
    release: &Release,
    arch: Arch,
    dir: &Path,
    progress: &mut dyn FnMut(Progress) -> bool,
) -> Result<PathBuf, UpdateError> {
    let name = arch.installer_name();
    let installer = release
        .asset(name)
        .ok_or_else(|| UpdateError::MissingAsset(name.to_owned()))?;
    let sums = release
        .asset(SUMS_NAME)
        .ok_or_else(|| UpdateError::MissingAsset(SUMS_NAME.to_owned()))?;
    let sums_text = http
        .get_text(&sums.url)
        .map_err(|error| UpdateError::Net(map_net(&error)))?;
    let expected = hash_for(&sums_text, name)
        .ok_or_else(|| UpdateError::MissingAsset(format!("{name} entry in {SUMS_NAME}")))?;

    let folder = dir.join(&release.tag);
    fs::create_dir_all(&folder).map_err(|error| UpdateError::Disk(error.to_string()))?;
    let target = folder.join(name);
    let partial = folder.join(format!("{name}.part"));
    let _ = fs::remove_file(&partial);
    let result = http.download(&installer.url, &partial, &mut |done, total| {
        progress(Progress::Downloading {
            done,
            total: total.or((installer.size > 0).then_some(installer.size)),
        })
    });
    match result {
        Ok(()) => {}
        Err(NetError::Cancelled) => {
            let _ = fs::remove_file(&partial);
            return Err(UpdateError::Cancelled);
        }
        Err(NetError::Io(why)) => {
            let _ = fs::remove_file(&partial);
            return Err(UpdateError::Disk(why));
        }
        Err(error) => {
            let _ = fs::remove_file(&partial);
            return Err(UpdateError::Net(map_net(&error)));
        }
    }
    if !progress(Progress::Verifying) {
        let _ = fs::remove_file(&partial);
        return Err(UpdateError::Cancelled);
    }
    let actual = sha256_hex(&partial).map_err(|error| UpdateError::Disk(error.to_string()))?;
    if actual != expected {
        let _ = fs::remove_file(&partial);
        return Err(UpdateError::HashMismatch { expected, actual });
    }
    let _ = fs::remove_file(&target);
    fs::rename(&partial, &target).map_err(|error| UpdateError::Disk(error.to_string()))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    struct Fake {
        releases: String,
        sums: String,
        payload: Vec<u8>,
        fail: RefCell<Option<NetError>>,
    }

    impl Http for Fake {
        fn get_text(&self, url: &str) -> Result<String, NetError> {
            if let Some(error) = self.fail.borrow().clone() {
                return Err(error);
            }
            if url.ends_with(SUMS_NAME) {
                Ok(self.sums.clone())
            } else if url.contains("/releases/tags/") {
                Ok(
                    serde_json::from_str::<Vec<serde_json::Value>>(&self.releases)
                        .map_err(|e| NetError::Io(e.to_string()))?[0]
                        .to_string(),
                )
            } else {
                Ok(self.releases.clone())
            }
        }

        fn download(
            &self,
            _url: &str,
            dest: &Path,
            progress: &mut dyn FnMut(u64, Option<u64>) -> bool,
        ) -> Result<(), NetError> {
            let total = self.payload.len() as u64;
            for chunk in 1..=4u64 {
                if !progress(total * chunk / 4, Some(total)) {
                    return Err(NetError::Cancelled);
                }
            }
            fs::write(dest, &self.payload).map_err(|e| NetError::Io(e.to_string()))
        }
    }

    fn digest(bytes: &[u8]) -> String {
        let dir = std::env::temp_dir().join(format!("wp-digest-{}", bytes.len()));
        fs::create_dir_all(&dir).expect("dir");
        let file = dir.join("x");
        fs::write(&file, bytes).expect("write");
        sha256_hex(&file).expect("hash")
    }

    fn fake(payload: &[u8], good_hash: bool) -> Fake {
        let base = format!("{REPOSITORY}/releases/download/v4.2.0-rc2");
        let releases = format!(
            r#"[{{"tag_name":"v4.2.0-rc2","html_url":"{REPOSITORY}/releases/tag/v4.2.0-rc2","prerelease":true,"body":"notes","assets":[
              {{"name":"Whirlpool-Setup.exe","browser_download_url":"{base}/Whirlpool-Setup.exe","size":{len}}},
              {{"name":"SHA256SUMS.txt","browser_download_url":"{base}/SHA256SUMS.txt","size":1}}]}}]"#,
            len = payload.len()
        );
        let hash = if good_hash {
            digest(payload)
        } else {
            "0".repeat(64)
        };
        Fake {
            releases,
            sums: format!("{hash}  Whirlpool-Setup.exe\n"),
            payload: payload.to_vec(),
            fail: RefCell::new(None),
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wp-update-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn finds_newer_prerelease_only_on_beta() {
        let http = fake(b"installer", true);
        let current = Version::parse("4.2.0-rc1").expect("v");
        assert_eq!(
            check(&http, &current, Channel::Stable),
            Err(CheckError::NoRelease)
        );
        match check(&http, &current, Channel::Beta) {
            Ok(Outcome::Available(release)) => assert_eq!(release.tag, "v4.2.0-rc2"),
            other => panic!("expected an update, got {other:?}"),
        }
        let same = Version::parse("4.2.0-rc2").expect("v");
        assert_eq!(check(&http, &same, Channel::Beta), Ok(Outcome::UpToDate));
    }

    #[test]
    fn downloads_and_verifies() {
        let http = fake(b"pretend installer bytes", true);
        let current = Version::parse("4.2.0-rc1").expect("v");
        let Ok(Outcome::Available(release)) = check(&http, &current, Channel::Beta) else {
            panic!("no update");
        };
        let mut seen = Vec::new();
        let dir = scratch("ok");
        let path = download_and_verify(&http, &release, Arch::X64, &dir, &mut |p| {
            seen.push(p);
            true
        })
        .expect("verified");
        assert_eq!(fs::read(&path).expect("read"), b"pretend installer bytes");
        assert!(matches!(seen.first(), Some(Progress::Downloading { .. })));
        assert_eq!(seen.last(), Some(&Progress::Verifying));
    }

    #[test]
    fn refuses_and_deletes_a_tampered_download() {
        let http = fake(b"tampered", false);
        let current = Version::parse("4.2.0-rc1").expect("v");
        let Ok(Outcome::Available(release)) = check(&http, &current, Channel::Beta) else {
            panic!("no update");
        };
        let dir = scratch("bad");
        let error = download_and_verify(&http, &release, Arch::X64, &dir, &mut |_| true)
            .expect_err("must fail");
        assert!(matches!(error, UpdateError::HashMismatch { .. }));
        let folder = dir.join("v4.2.0-rc2");
        assert!(!folder.join("Whirlpool-Setup.exe").exists());
        assert!(!folder.join("Whirlpool-Setup.exe.part").exists());
    }

    #[test]
    fn arm64_without_an_arm64_installer_says_so() {
        let http = fake(b"x", true);
        let current = Version::parse("1.0.0").expect("v");
        let Ok(Outcome::Available(release)) = check(&http, &current, Channel::Beta) else {
            panic!("no update");
        };
        let error =
            download_and_verify(&http, &release, Arch::Arm64, &scratch("arm"), &mut |_| true)
                .expect_err("no arm64 asset");
        assert_eq!(
            error,
            UpdateError::MissingAsset("Whirlpool-Setup-arm64.exe".into())
        );
    }

    #[test]
    fn cancel_leaves_nothing_behind() {
        let http = fake(b"abcdefgh", true);
        let current = Version::parse("1.0.0").expect("v");
        let Ok(Outcome::Available(release)) = check(&http, &current, Channel::Beta) else {
            panic!("no update");
        };
        let dir = scratch("cancel");
        let error = download_and_verify(&http, &release, Arch::X64, &dir, &mut |_| false)
            .expect_err("cancelled");
        assert_eq!(error, UpdateError::Cancelled);
        assert!(!dir.join("v4.2.0-rc2/Whirlpool-Setup.exe.part").exists());
    }

    #[test]
    fn offline_and_rate_limit_have_clear_messages() {
        let http = fake(b"x", true);
        let current = Version::parse("1.0.0").expect("v");
        *http.fail.borrow_mut() = Some(NetError::Offline);
        let offline = check(&http, &current, Channel::Stable).expect_err("offline");
        assert_eq!(offline, CheckError::Offline);
        assert!(offline.to_string().contains("network"));
        *http.fail.borrow_mut() = Some(NetError::Status {
            code: 403,
            retry_after: Some(1800),
            rate_limited: true,
        });
        let limited = check(&http, &current, Channel::Stable).expect_err("limited");
        assert_eq!(limited, CheckError::RateLimited { minutes: 30 });
        assert!(limited.to_string().contains("30 min"));
        *http.fail.borrow_mut() = Some(NetError::Status {
            code: 500,
            retry_after: None,
            rate_limited: false,
        });
        assert_eq!(
            check(&http, &current, Channel::Stable),
            Err(CheckError::Http(500))
        );
    }

    #[test]
    fn rollback_fetches_a_tag() {
        let http = fake(b"x", true);
        assert_eq!(
            check_tag(&http, "v4.2.0-rc2").expect("tag").tag,
            "v4.2.0-rc2"
        );
        assert_eq!(
            check_tag(&http, "../etc").expect_err("unsafe"),
            CheckError::NoRelease
        );
    }
}
