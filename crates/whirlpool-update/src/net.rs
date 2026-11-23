//! The real HTTP client: reqwest with the system certificate store and proxy.

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

use reqwest::{StatusCode, blocking, header};

use crate::{Http, NetError};

pub struct Client {
    inner: blocking::Client,
}

impl Client {
    pub fn new() -> Result<Self, NetError> {
        let inner = blocking::Client::builder()
            .user_agent(concat!("whirlpool-updater/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(900))
            .build()
            .map_err(|error| NetError::Io(error.to_string()))?;
        Ok(Self { inner })
    }

    fn send(&self, url: &str, accept: &str) -> Result<blocking::Response, NetError> {
        let response = self
            .inner
            .get(url)
            .header(header::ACCEPT, accept)
            .send()
            .map_err(|_| NetError::Offline)?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let headers = response.headers();
        let number = |name: &str| headers.get(name)?.to_str().ok()?.parse::<u64>().ok();
        let retry_after = number("retry-after").or_else(|| {
            let reset = number("x-ratelimit-reset")?;
            Some(reset.saturating_sub(crate::State::now()))
        });
        let exhausted = number("x-ratelimit-remaining") == Some(0);
        Err(NetError::Status {
            code: status.as_u16(),
            retry_after,
            rate_limited: status == StatusCode::TOO_MANY_REQUESTS
                || (status == StatusCode::FORBIDDEN && exhausted),
        })
    }
}

impl Http for Client {
    fn get_text(&self, url: &str) -> Result<String, NetError> {
        self.send(url, "application/vnd.github+json, text/plain")?
            .text()
            .map_err(|error| NetError::Io(error.to_string()))
    }

    fn download(
        &self,
        url: &str,
        dest: &Path,
        progress: &mut dyn FnMut(u64, Option<u64>) -> bool,
    ) -> Result<(), NetError> {
        let mut response = self.send(url, "application/octet-stream")?;
        let total = response.content_length();
        let mut file = File::create(dest).map_err(|error| NetError::Io(error.to_string()))?;
        let mut buffer = vec![0u8; 128 * 1024];
        let mut done = 0u64;
        let mut last_report = std::time::Instant::now();
        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|error| NetError::Io(error.to_string()))?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])
                .map_err(|error| NetError::Io(error.to_string()))?;
            done += read as u64;
            if last_report.elapsed() >= Duration::from_millis(100) {
                last_report = std::time::Instant::now();
                if !progress(done, total) {
                    return Err(NetError::Cancelled);
                }
            }
        }
        if !progress(done, total) {
            return Err(NetError::Cancelled);
        }
        file.flush()
            .map_err(|error| NetError::Io(error.to_string()))
    }
}
