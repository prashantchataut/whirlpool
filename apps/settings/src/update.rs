//! Update check against GitHub Releases. This is the only network access in
//! the settings app, and it runs only when the user allows it.

use std::{sync::mpsc, thread};

use serde::Deserialize;
use windows_future as _;

pub const REPOSITORY: &str = "https://github.com/prashantchataut/whirlpool";
const LATEST: &str = "https://api.github.com/repos/prashantchataut/whirlpool/releases/latest";

#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
    pub page: String,
}

#[derive(Clone, Debug)]
pub enum State {
    Off,
    Checking,
    Current,
    Available(Release),
    Failed,
}

#[derive(Deserialize)]
struct Latest {
    tag_name: String,
    html_url: String,
}

/// Starts a check on its own thread; the result arrives on the receiver.
pub fn check() -> mpsc::Receiver<Option<Release>> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let _ = send.send(fetch());
    });
    receive
}

fn fetch() -> Option<Release> {
    // Windows' own HTTP client: system proxy and certificates, no extra TLS
    // library in the executable.
    use windows::{Foundation::Uri, Web::Http::HttpClient, core::HSTRING};
    let client = HttpClient::new().ok()?;
    let headers = client.DefaultRequestHeaders().ok()?;
    headers
        .UserAgent()
        .ok()?
        .TryParseAdd(&HSTRING::from(concat!(
            "whirlpool-settings/",
            env!("CARGO_PKG_VERSION")
        )))
        .ok()?;
    headers
        .Accept()
        .ok()?
        .TryParseAdd(&HSTRING::from("application/vnd.github+json"))
        .ok()?;
    let uri = Uri::CreateUri(&HSTRING::from(LATEST)).ok()?;
    let body = client.GetStringAsync(&uri).ok()?.join().ok()?;
    let latest: Latest = serde_json::from_str(&body.to_string()).ok()?;
    let page = latest.html_url;
    // Only ever open a release page of this repository.
    page.starts_with(REPOSITORY).then(|| Release {
        version: latest.tag_name.trim_start_matches('v').to_owned(),
        page,
    })
}

/// Whether `candidate` is a higher dotted version than the running one.
pub fn is_newer(candidate: &str) -> bool {
    fn parts(version: &str) -> Vec<u32> {
        version
            .split(['.', '-'])
            .map_while(|part| part.parse().ok())
            .collect()
    }
    parts(candidate) > parts(env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions_compare_numerically() {
        assert!(is_newer("99.0.0"));
        assert!(!is_newer("0.0.1"));
        assert!(!is_newer(env!("CARGO_PKG_VERSION")));
    }
}
