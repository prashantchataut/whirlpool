//! Whirlpool's update engine. Everything except the HTTP client and the installer launch is
//! plain logic behind the [`Http`] trait, so the whole flow runs in unit tests.
//!
//! Flow: [`check`] asks GitHub Releases, [`download_and_verify`] fetches the installer for this
//! CPU architecture and refuses it unless its SHA-256 matches `SHA256SUMS.txt` from the same
//! release, [`install`] starts Setup silently and Setup relaunches the program.

mod engine;
mod install;
#[cfg(windows)]
mod net;
mod release;
mod state;
mod version;

pub use engine::{
    CheckError, Outcome, Progress, UpdateError, check, check_tag, download_and_verify, sha256_hex,
};
pub use install::{InstallError, install};
#[cfg(windows)]
pub use net::Client;
pub use release::{Arch, Asset, Channel, Release, SUMS_NAME, hash_for, parse_one, pick_latest};
pub use state::State;
pub use version::Version;

pub const REPOSITORY: &str = "https://github.com/prashantchataut/whirlpool";
pub const API: &str = "https://api.github.com/repos/prashantchataut/whirlpool";

/// A failed request, as the engine needs to tell the cases apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetError {
    /// No connection, DNS failure, timeout.
    Offline,
    /// HTTP status that is not a success. `retry_after` is in seconds when the server said so.
    Status {
        code: u16,
        retry_after: Option<u64>,
        rate_limited: bool,
    },
    /// The transfer broke off or the disk refused it.
    Io(String),
    /// The caller's progress callback asked to stop.
    Cancelled,
}

/// The two requests the engine makes. `download` streams to `dest` and reports `(done, total)`;
/// returning `false` from `progress` cancels.
pub trait Http {
    fn get_text(&self, url: &str) -> Result<String, NetError>;
    fn download(
        &self,
        url: &str,
        dest: &std::path::Path,
        progress: &mut dyn FnMut(u64, Option<u64>) -> bool,
    ) -> Result<(), NetError>;
}
