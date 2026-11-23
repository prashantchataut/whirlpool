//! Whirlpool: a macOS-style dock for Windows 11.
//!
//! Usage:
//! - `whirlpool`                     starts or recovers the dock
//! - `whirlpool --quit`              closes the running dock
//! - `whirlpool --restore`           restores the Windows taskbar
//! - `whirlpool --autostart on|off`  enables or disables startup with Windows
//! - `whirlpool --guard <pid>`       internal watchdog process
//! - `whirlpool --usage-worker`      internal one-shot quota reader
//! - `whirlpool --shell-worker`      internal one-shot shell action

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accessories;
mod app;
mod assets;
mod charge;
mod drop;
mod edge_watch;
mod flight;
mod folder_menu;
mod gfx;
mod group_view;
mod guard;
mod mascot;
mod media;
mod menu;
mod menubar;
mod multi_window_view;
mod overlay;
mod recovery;
mod scene;
mod shell_work;
mod strings;
mod thumbnails;
mod usage;

use std::env;

use anyhow::{Context, Result, bail};
use whirlpool_shell::{autostart, taskbar};
use windows::{
    Win32::{
        Foundation::{ERROR_ALREADY_EXISTS, GetLastError},
        System::{
            Diagnostics::Debug::{SEM_FAILCRITICALERRORS, SEM_NOOPENFILEERRORBOX, SetErrorMode},
            Threading::CreateMutexW,
            WinRT::{
                CreateDispatcherQueueController, DQTAT_COM_STA, DQTYPE_THREAD_CURRENT,
                DispatcherQueueOptions,
            },
        },
        UI::{
            HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
            WindowsAndMessaging::{
                FindWindowW, MB_ICONERROR, MB_OK, MessageBoxW, SW_RESTORE, SetForegroundWindow,
                ShowWindow,
            },
        },
    },
    core::{HSTRING, Owned, w},
};

fn main() -> Result<()> {
    // Suppress system modal error dialogs (missing DLLs, missing ordinals in 3rd-party shell extensions).
    // SAFETY: Affects process-level error mode without side effects on memory or concurrency.
    unsafe {
        let _ = SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX);
    }
    let args: Vec<String> = env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => run_dock(false),
        [recovery::RECOVERED_FLAG] => run_dock(true),
        ["--quit"] => {
            app::request_quit();
            Ok(())
        }
        ["--restore"] => {
            app::request_quit();
            taskbar::give_back();
            Ok(())
        }
        ["--autostart", "on"] => {
            let exe = env::current_exe().context("no se pudo localizar whirlpool.exe")?;
            Ok(autostart::set(Some(&exe.to_string_lossy()))?)
        }
        ["--autostart", "off"] => Ok(autostart::set(None)?),
        [guard::FLAG, pid] => guard::run(pid.parse().context("pid no válido")?),
        ["--usage-worker"] => usage::worker_main(),
        [accessories::FLAG] => accessories::worker_main(None),
        [accessories::FLAG, visibility] => accessories::worker_main(Some(visibility)),
        [accessories::USB_FLAG] => accessories::usb_worker_main(),
        [shell_work::FLAG] => shell_work::worker_main(),
        ["--wallpaper-next", rest @ ..] => {
            // One rotation step now, using the saved settings; `N` skips ahead N pictures.
            let config = whirlpool_config::load(&whirlpool_config::default_path())
                .ok()
                .flatten()
                .unwrap_or_default();
            let skip = rest.first().and_then(|n| n.parse().ok()).unwrap_or(0);
            let mut state = whirlpool_settings::rotate::State::new(config.wallpaper_every_minutes)
                .skipping(skip);
            let dark = whirlpool_settings::rotate::dark_now();
            let result = whirlpool_settings::rotate::advance(&config, dark, &mut state);
            whirlpool_shell::log::line(&format!("wallpaper-next: {result:?}"));
            whirlpool_shell::log::flush();
            std::process::exit(i32::from(result.is_err()))
        }
        ["--overlay"] => overlay::run(),
        ["--overlay-toggle"] => {
            overlay::toggle(&env::current_exe().context("exe")?);
            Ok(())
        }
        ["--version"] => {
            whirlpool_shell::log::line(concat!("whirlpool ", env!("CARGO_PKG_VERSION")));
            whirlpool_shell::log::flush();
            Ok(())
        }
        ["--update", flags @ ..] => {
            let code = whirlpool_settings::update::cli(flags);
            std::process::exit(code)
        }
        ["--settings"] => open_settings(false),
        ["--settings-compat"] => open_settings(true),
        ["--quick-settings", right, y, edge] => open_quick_settings(right, y, edge, false),
        ["--quick-settings-compat", right, y, edge] => open_quick_settings(right, y, edge, true),
        _ => bail!(
            "usage: whirlpool [--settings | --overlay-toggle | --update [--check-only] [--beta] [--rollback] | --version | --quit | --restore | --autostart on|off]"
        ),
    }
}

/// The quick settings panel the menu bar opens. One at a time; a second
/// request while it is open does nothing, since the panel closes itself when
/// it loses focus.
fn open_quick_settings(right: &str, y: &str, edge: &str, compat: bool) -> Result<()> {
    // SAFETY: The static name is valid; the handle is released when the process exits.
    let _single = unsafe {
        Owned::new(CreateMutexW(
            None,
            false,
            w!("Local\\Whirlpool.QuickSettings"),
        )?)
    };
    // SAFETY: Reads the thread's last error immediately after the call.
    if !compat && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Ok(());
    }
    let (right, y) = (
        right.parse().context("invalid x")?,
        y.parse().context("invalid y")?,
    );
    whirlpool_settings::run_quick(right, y, edge == "bottom", compat).map_err(|error| {
        whirlpool_shell::log::line(&format!("quick settings (compat={compat}): {error}"));
        whirlpool_shell::log::flush();
        if !compat && let Ok(exe) = env::current_exe() {
            // No OpenGL (virtual machine, remote desktop): same panel on DirectX.
            let _ = std::process::Command::new(exe)
                .args([
                    "--quick-settings-compat",
                    &right.to_string(),
                    &y.to_string(),
                    edge,
                ])
                .spawn();
        }
        anyhow::anyhow!("quick settings: {error}")
    })
}

/// Starts this program again as the settings window.
fn spawn_settings() -> std::io::Result<std::process::Child> {
    std::process::Command::new(env::current_exe()?)
        .arg("--settings")
        .spawn()
}

/// The same window, drawn with DirectX. Used when OpenGL is not available.
fn spawn_settings_compat() -> std::io::Result<std::process::Child> {
    std::process::Command::new(env::current_exe()?)
        .arg("--settings-compat")
        .spawn()
}

/// The settings window. Only one at a time: a second request raises the first.
fn open_settings(compat: bool) -> Result<()> {
    // SAFETY: The static name is valid; the handle is released when the process exits.
    let single = unsafe { Owned::new(CreateMutexW(None, false, w!("Local\\Whirlpool.Settings"))?) };
    // SAFETY: Reads the thread's last error immediately after the call.
    if !compat && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // SAFETY: Plain window lookups on the desktop; failures are ignored.
        unsafe {
            if let Ok(window) = FindWindowW(None, w!("Whirlpool")) {
                let _ = ShowWindow(window, SW_RESTORE);
                let _ = SetForegroundWindow(window);
            }
        }
        return Ok(());
    }
    if !compat {
        // A crash inside the graphics driver aborts the process. Before that
        // happens, start the DirectX window so the user still gets Settings.
        std::panic::set_hook(Box::new(|info| {
            whirlpool_shell::log::line(&format!("settings crashed, retrying with DirectX: {info}"));
            whirlpool_shell::log::flush();
            let _ = spawn_settings_compat();
        }));
    }
    let shown = if compat {
        whirlpool_settings::run_compat()
    } else {
        whirlpool_settings::run()
    };
    shown.map_err(|error| {
        // The program has no console, so a failure to open the window would
        // be silent. Say what happened and where the details are.
        whirlpool_shell::log::line(&format!("settings (compat={compat}): {error}"));
        whirlpool_shell::log::flush();
        if !compat {
            let _ = std::panic::take_hook();
            drop(single);
            if spawn_settings_compat().is_ok() {
                return anyhow::anyhow!("settings: OpenGL failed, DirectX window started");
            }
        }
        let text = format!(
            "Whirlpool Settings could not open its window.\n\n{error}\n\n\
             Neither OpenGL nor DirectX could create a window. This can happen in a \
             remote desktop session or a virtual machine without graphics acceleration. \
             Details: %LOCALAPPDATA%\\Whirlpool\\whirlpool.log"
        );
        // SAFETY: A plain message box owned by no window; the strings outlive the call.
        unsafe {
            MessageBoxW(
                None,
                &HSTRING::from(text),
                w!("Whirlpool"),
                MB_OK | MB_ICONERROR,
            );
        }
        anyhow::anyhow!("settings: {error}")
    })
}

fn run_dock(recovered: bool) -> Result<()> {
    // SAFETY: The static name is valid; the handle is released when the process exits.
    let _single = unsafe { Owned::new(CreateMutexW(None, false, w!("Local\\Whirlpool.Dock"))?) };
    // SAFETY: Reads the thread's last error immediately after the call.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        guard::recover_existing()?;
        // Starting Whirlpool again while it runs opens its settings.
        if !recovered {
            let _ = spawn_settings();
        }
        return Ok(());
    }

    recovery::configure(recovered);
    let startup = std::time::Instant::now();
    whirlpool_shell::log::line(concat!(
        "proceso iniciado, versión ",
        env!("CARGO_PKG_VERSION")
    ));
    // Keep the recovery channel alive through startup errors and final cleanup.
    let _recovery_event = guard::spawn()?;
    let result = initialize_dock();
    if let Err(error) = &result {
        recovery::graphics_error(error);
        whirlpool_shell::log::line(&format!(
            "el dock falló tras {} ms desde el inicio del proceso: {error:#}",
            startup.elapsed().as_millis()
        ));
    }
    taskbar::give_back_on_exit();
    whirlpool_shell::log::flush();
    result
}

fn initialize_dock() -> Result<()> {
    // SAFETY: Sets DPI awareness (if not already locked by the application manifest)
    // and initializes the dispatcher before creating windows.
    let _queue = unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        // The compositor requires a dispatcher queue on its thread; this also
        // initializes COM, which icon extraction requires.
        CreateDispatcherQueueController(DispatcherQueueOptions {
            dwSize: size_of::<DispatcherQueueOptions>() as u32,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_STA,
        })?
    };

    // Moves the `Run` autostart of 0.1.9-0.1.10 to the logon task, off the UI thread.
    let _ = std::thread::Builder::new()
        .name("whirlpool-autostart".into())
        .spawn(autostart::migrate);
    app::run()
}
