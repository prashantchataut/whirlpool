//! The updater as Settings and the command line use it: background threads around
//! `whirlpool-update`. This is the only network access in Whirlpool; it goes to
//! api.github.com and github.com, only when the user allows it or asks for it.

use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

use whirlpool_update::{
    Arch, CheckError, Client, Http, Outcome, State as Saved, Version, check, check_tag,
    download_and_verify, install,
};
pub use whirlpool_update::{Channel, Progress, REPOSITORY, Release};

#[derive(Clone, Debug)]
pub enum State {
    Off,
    Checking,
    Current,
    Available(Release),
    Working(Progress),
    Restarting,
    Failed(String),
}

pub enum Msg {
    Checked(Result<Outcome, CheckError>),
    Progress(Progress),
    Launched,
    Failed(String),
}

/// `%LOCALAPPDATA%\Whirlpool\updates`.
#[must_use]
pub fn dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map_or_else(std::env::temp_dir, PathBuf::from)
        .join("Whirlpool")
        .join("updates")
}

#[must_use]
pub fn running() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION"))
        .unwrap_or_else(|| Version::parse("0.0.0").expect("literal"))
}

#[must_use]
pub const fn channel(prereleases: bool) -> Channel {
    if prereleases {
        Channel::Beta
    } else {
        Channel::Stable
    }
}

/// The version installed before the last update, if it differs from the running one.
#[must_use]
pub fn rollback_target() -> Option<String> {
    let previous = Saved::load(&dir()).previous?;
    (Version::parse(&previous)? != running()).then_some(previous)
}

fn record_check(outcome: &Result<Outcome, CheckError>) {
    let mut saved = Saved::load(&dir());
    saved.last_checked = Saved::now();
    if let Ok(found) = outcome {
        saved.available = match found {
            Outcome::Available(release) => Some(release.version.to_string()),
            Outcome::UpToDate => None,
        };
    }
    saved.save(&dir());
}

fn look(prereleases: bool) -> Result<Outcome, CheckError> {
    let client = Client::new().map_err(|_| CheckError::Offline)?;
    let outcome = check(&client, &running(), channel(prereleases));
    record_check(&outcome);
    outcome
}

/// Checks on a thread; one message arrives on the receiver.
pub fn spawn_check(prereleases: bool) -> mpsc::Receiver<Msg> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let _ = send.send(Msg::Checked(look(prereleases)));
    });
    receive
}

/// The dock's periodic look: records what it found for Settings and, when `remind` is on and a
/// reminder is due (see `State::reminder_due`), shows one Windows notification. It never
/// downloads or installs anything.
pub fn background_check(prereleases: bool, remind: bool) {
    thread::spawn(move || match look(prereleases) {
        Ok(Outcome::Available(release)) => {
            whirlpool_shell::log::line(&format!("update available: {}", release.tag));
            if remind {
                remind_if_due(&release);
            }
        }
        Ok(Outcome::UpToDate) => whirlpool_shell::log::line("update check: up to date"),
        Err(error) => whirlpool_shell::log::line(&format!("update check failed: {error}")),
    });
}

fn remind_if_due(release: &Release) {
    let mut saved = Saved::load(&dir());
    let now = Saved::now();
    if !saved.reminder_due(now) {
        return;
    }
    match toast(&release.version.to_string(), &release.tag) {
        Ok(()) => {
            saved.note_reminded(now);
            saved.save(&dir());
            whirlpool_shell::log::line(&format!("update reminder shown: {}", release.tag));
        }
        Err(why) => whirlpool_shell::log::line(&format!("update reminder failed: {why}")),
    }
}

/// Shows the "update available" notification through Windows PowerShell (present on every
/// supported Windows), hidden, for the second or so it needs. Nothing stays running.
fn toast(version: &str, tag: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script =
        whirlpool_update::remind::toast_script(version, &running().to_string(), tag, REPOSITORY)
            .ok_or("the version text is not plain")?;
    let status = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &whirlpool_update::remind::encoded_command(&script),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("powershell exited with {status}"))
    }
}

/// A plain Windows notification (and a line in the notification centre), through hidden PowerShell.
/// Blocks for about a second: call it from a short-lived thread.
pub fn notify(title: &str, body: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let script = whirlpool_update::remind::message_toast_script(title, body)
        .ok_or("the text is not plain")?;
    crate::push_notification("Whirlpool", title, body);
    let status = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &whirlpool_update::remind::encoded_command(&script),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("powershell exited with {status}"))
    }
}

/// Seconds until the next background look is due, or `None` when none is wanted.
#[must_use]
pub fn background_due_in(days: u32) -> Option<u64> {
    Saved::load(&dir()).due_in(days, Saved::now())
}

fn fetch_and_start(
    http: &dyn Http,
    release: &Release,
    cancel: &AtomicBool,
    quiet: bool,
    report: &mut dyn FnMut(Progress),
) -> Result<(), String> {
    let path = download_and_verify(http, release, Arch::current(), &dir(), &mut |progress| {
        report(progress);
        !cancel.load(Ordering::Relaxed)
    })
    .map_err(|error| error.to_string())?;
    let mut saved = Saved::load(&dir());
    saved.previous = Some(running().to_string());
    saved.available = None;
    saved.save(&dir());
    whirlpool_shell::log::line(&format!("update: starting {}", path.display()));
    install(&path, quiet).map_err(|error| error.to_string())
}

fn spawn_fetch(
    release: Option<Release>,
    tag: Option<String>,
    cancel: Arc<AtomicBool>,
) -> mpsc::Receiver<Msg> {
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let result = (|| {
            let client = Client::new().map_err(|_| CheckError::Offline.to_string())?;
            let release = match (release, tag) {
                (Some(release), _) => release,
                (None, Some(tag)) => check_tag(&client, &tag).map_err(|error| error.to_string())?,
                (None, None) => return Err(CheckError::NoRelease.to_string()),
            };
            fetch_and_start(&client, &release, &cancel, false, &mut |progress| {
                let _ = send.send(Msg::Progress(progress));
            })
        })();
        let _ = send.send(match result {
            Ok(()) => Msg::Launched,
            Err(why) => Msg::Failed(why),
        });
    });
    receive
}

/// Downloads, verifies and starts Setup for `release`. `Msg::Launched` means Setup is running and
/// this process should exit.
pub fn spawn_install(release: Release, cancel: Arc<AtomicBool>) -> mpsc::Receiver<Msg> {
    spawn_fetch(Some(release), None, cancel)
}

/// Same for an earlier release, by tag.
pub fn spawn_rollback(version: &str, cancel: Arc<AtomicBool>) -> mpsc::Receiver<Msg> {
    spawn_fetch(None, Some(format!("v{version}")), cancel)
}

/// `whirlpool --update [--check-only] [--beta] [--rollback]`. Prints to stdout and the log, and
/// returns the process exit code: 0 done or up to date, 10 an update exists (`--check-only`), 1 failure.
#[must_use]
pub fn cli(flags: &[&str]) -> i32 {
    let say = |text: &str| {
        println!("{text}");
        whirlpool_shell::log::line(&format!("update cli: {text}"));
    };
    let has = |flag: &str| flags.contains(&flag);
    if has("--remind-test") {
        // `whirlpool --update --remind-test`: shows the reminder for a made-up version, to prove
        // the notification script runs on this machine. Records nothing.
        let code = match toast("99.0.0-test", "v99.0.0-test") {
            Ok(()) => {
                say("reminder toast shown");
                0
            }
            Err(why) => {
                say(&format!("reminder failed: {why}"));
                1
            }
        };
        whirlpool_shell::log::flush();
        return code;
    }
    let code = (|| {
        let client = match Client::new() {
            Ok(client) => client,
            Err(_) => {
                say("could not start the network client");
                return 1;
            }
        };
        say(&format!("running {}", running()));
        let cancel = AtomicBool::new(false);
        let release = if has("--rollback") {
            let Some(previous) = rollback_target() else {
                say("nothing to roll back to");
                return 1;
            };
            match check_tag(&client, &format!("v{previous}")) {
                Ok(release) => release,
                Err(error) => {
                    say(&format!("failed: {error}"));
                    return 1;
                }
            }
        } else {
            let outcome = check(&client, &running(), channel(has("--beta")));
            record_check(&outcome);
            match outcome {
                Ok(Outcome::UpToDate) => {
                    say("up to date");
                    return 0;
                }
                Ok(Outcome::Available(release)) => {
                    say(&format!("available {}", release.tag));
                    release
                }
                Err(error) => {
                    say(&format!("failed: {error}"));
                    return 1;
                }
            }
        };
        if has("--check-only") {
            return 10;
        }
        let mut last = 0;
        let result = fetch_and_start(&client, &release, &cancel, true, &mut |progress| {
            if let Progress::Downloading {
                done,
                total: Some(total),
            } = progress
            {
                let percent = done * 100 / total.max(1);
                if percent >= last + 10 {
                    last = percent;
                    say(&format!("downloading {percent}%"));
                }
            } else if progress == Progress::Verifying {
                say("verifying sha-256");
            }
        });
        match result {
            Ok(()) => {
                say("sha-256 verified; setup started");
                0
            }
            Err(why) => {
                say(&format!("failed: {why}"));
                1
            }
        }
    })();
    whirlpool_shell::log::flush();
    code
}
