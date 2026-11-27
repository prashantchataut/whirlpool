//! Clock drawn on the desktop itself, behind every window.
//!
//! A click-through window that sits just above the desktop window in the z-order, so apps cover it
//! and icons stay on top. Time on one line, the date under it, a soft shadow for readability on
//! bright pictures. It is redrawn once a minute by a timer the dock arms only while the clock is on.

use anyhow::{Context, Result};
use whirlpool_core::clock::{ClockOptions, DateStyle, DateStyleOpt};
use windows::{
    UI::Composition::{CompositionBrush, ContainerVisual, Desktop::DesktopWindowTarget},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WinRT::Composition::ICompositorDesktopInterop},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetShellWindow, HWND_BOTTOM,
            RegisterClassW, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetWindowPos, WNDCLASSW, WS_EX_LAYERED,
            WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
            WS_POPUP,
        },
    },
    core::{HSTRING, Interface, w},
};
use windows_numerics::{Vector2, Vector3};

use crate::gfx::{Gfx, Ink};

/// Logical size of the time and of the date line, and their weights (light time).
const TIME_SIZE: f32 = 96.0;
const DATE_SIZE: f32 = 22.0;
const TIME_WEIGHT: i32 = 300;
const DATE_WEIGHT: i32 = 400;

/// What the widget shows now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub time: String,
    pub date: String,
    pub family: Option<String>,
}

/// Time without seconds and the long date, in the user's regional format.
#[must_use]
pub fn content(options: &ClockOptions) -> Content {
    let mut options = options.clone();
    options.date = DateStyleOpt(DateStyle::Off);
    options.zones.clear();
    options.seconds = false;
    let family = options.font.family().map(str::to_owned);
    Content {
        time: whirlpool_shell::system::clock_text_for(&options),
        date: DateStyle::Long
            .pattern()
            .map(whirlpool_shell::system::date_text)
            .unwrap_or_default(),
        family,
    }
}

pub struct DesktopClock {
    hwnd: HWND,
    target: DesktopWindowTarget,
    drawn: Content,
}

impl DesktopClock {
    /// Shows the clock; `None` (and a line in the log) if it could not be built.
    pub fn show(gfx: &Gfx, content: &Content) -> Option<Self> {
        match Self::build(gfx, content) {
            Ok(clock) => Some(clock),
            Err(error) => {
                whirlpool_shell::log::line(&format!("desktop clock: {error:#}"));
                None
            }
        }
    }

    /// Redraws when the text changed and puts the window back above the desktop (a click on the
    /// desktop raises the desktop window over it).
    pub fn update(&mut self, gfx: &Gfx, content: &Content) {
        match scene(gfx, content) {
            Ok((root, width, height)) => {
                if self.drawn != *content && self.target.SetRoot(&root).is_ok() {
                    self.drawn = content.clone();
                }
                place(self.hwnd, width, height);
            }
            Err(error) => whirlpool_shell::log::line(&format!("desktop clock: {error:#}")),
        }
    }

    fn build(gfx: &Gfx, content: &Content) -> Result<Self> {
        let (root, width, height) = scene(gfx, content)?;
        let hwnd = create_window()?;
        let built = || -> Result<DesktopWindowTarget> {
            // SAFETY: `hwnd` is the live window created above.
            let target = unsafe {
                gfx.compositor
                    .cast::<ICompositorDesktopInterop>()?
                    .CreateDesktopWindowTarget(hwnd, false)?
            };
            target.SetRoot(&root)?;
            Ok(target)
        };
        let target = match built() {
            Ok(target) => target,
            Err(error) => {
                // SAFETY: Our own window, created above.
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return Err(error);
            }
        };
        place(hwnd, width, height);
        Ok(Self {
            hwnd,
            target,
            drawn: content.clone(),
        })
    }
}

impl Drop for DesktopClock {
    fn drop(&mut self) {
        // SAFETY: Our own window, destroyed once.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Visual tree plus the window size it needs, in physical pixels.
fn scene(gfx: &Gfx, content: &Content) -> Result<(ContainerVisual, f32, f32)> {
    let monitor = whirlpool_shell::system::primary_monitor().context("no monitor")?;
    let scale = monitor.scale.max(1.0);
    let family = content.family.as_deref();
    let time =
        |ink| gfx.text_brush_styled(&content.time, scale, TIME_SIZE, family, TIME_WEIGHT, ink);
    let date =
        |ink| gfx.text_brush_styled(&content.date, scale, DATE_SIZE, family, DATE_WEIGHT, ink);
    let (time_light, time_dark) = (time(Ink::Light)?, time(Ink::Dark)?);
    let (date_light, date_dark) = (date(Ink::Light)?, date(Ink::Dark)?);

    let pad = (20.0 * scale).round();
    let gap = (2.0 * scale).round();
    let width = (time_light.width.max(date_light.width) + 2.0 * pad).ceil();
    let height = (time_light.height + gap + date_light.height + 2.0 * pad).ceil();
    let drop = (3.0 * scale).round();

    let c = &gfx.compositor;
    let root = c.CreateContainerVisual()?;
    root.SetSize(Vector2::new(width, height))?;
    let put = |label: &crate::gfx::LabelBrush, x: f32, y: f32, opacity: f32| -> Result<()> {
        let visual = c.CreateSpriteVisual()?;
        visual.SetSize(Vector2::new(label.width, label.height))?;
        visual.SetOffset(Vector3::new(x.round(), y.round(), 0.0))?;
        visual.SetBrush(&label.brush.cast::<CompositionBrush>()?)?;
        visual.SetOpacity(opacity)?;
        root.Children()?.InsertAtTop(&visual)?;
        Ok(())
    };
    let date_y = pad + time_light.height + gap;
    // Shadows first (below), then the light text.
    put(&time_dark, pad, pad + drop, 0.35)?;
    put(&date_dark, pad, date_y + drop, 0.35)?;
    put(&time_light, pad, pad, 0.96)?;
    put(&date_light, pad, date_y, 0.86)?;
    Ok((root, width, height))
}

/// Upper left of the primary display, just above the desktop window.
fn place(hwnd: HWND, width: f32, height: f32) {
    let Some(monitor) = whirlpool_shell::system::primary_monitor() else {
        return;
    };
    let work_w = (monitor.work_right - monitor.work_left) as f32;
    let work_h = (monitor.work_bottom - monitor.work_top) as f32;
    let x = monitor.work_left + (work_w * 0.05) as i32;
    let y = monitor.work_top + (work_h * 0.07) as i32;
    // SAFETY: Our own window. Inserting after the shell window puts it directly above the desktop
    // (icons stay above it, apps cover it); without a shell window it goes to the bottom.
    unsafe {
        let shell = GetShellWindow();
        let after = if shell.0.is_null() {
            HWND_BOTTOM
        } else {
            shell
        };
        let _ = SetWindowPos(
            hwnd,
            Some(after),
            x,
            y,
            width as i32,
            height as i32,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
}

fn create_window() -> Result<HWND> {
    static CLASS: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    let class = w!("WhirlpoolDesktopClock");
    // SAFETY: The class is registered once; the window is destroyed by `DesktopClock::drop`, or by
    // the caller if building the scene fails.
    unsafe {
        let instance = GetModuleHandleW(None)?;
        CLASS.get_or_init(|| {
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(proc),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            });
        });
        // Click-through and out of Alt+Tab: the clock never takes a click from the desktop.
        Ok(CreateWindowExW(
            WS_EX_LAYERED
                | WS_EX_TRANSPARENT
                | WS_EX_NOREDIRECTIONBITMAP
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE,
            class,
            &HSTRING::new(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(instance.into()),
            None,
        )?)
    }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: Default handling for our own window.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}
