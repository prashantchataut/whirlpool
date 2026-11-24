//! Drag motion for other programs' windows.
//!
//! Windows offers no supported way to transform (tilt, scale, warp) another program's window, so
//! none of that is attempted. What Whirlpool can honestly do, and does while a window is being moved:
//!
//! - draw a soft shadow on a layered window directly behind it, so it reads as lifted;
//! - draw a preview of where it would snap when the pointer reaches a screen edge or corner;
//! - on release, carry it to the snapped position (or back into reach, if it was dropped mostly off
//!   screen) on a critically damped spring, while the lifted shadow settles away.
//!
//! The work happens on a thread that exists only between `EVENT_SYSTEM_MOVESIZESTART` and `END`, paced
//! by `DwmFlush`; with no window being moved, nothing runs (rule 5 of AGENTS.md).
//! Resizing a window is recognised by its size changing and left alone.

use std::{
    ffi::c_void,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
    thread,
    time::{Duration, Instant},
};

use whirlpool_core::{
    snap::{self, Area, Zone},
    spring::Spring,
};
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
        Graphics::{
            Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmFlush, DwmGetWindowAttribute},
            Gdi::{
                AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
                CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject,
                GetDC, GetMonitorInfoW, HGDIOBJ, MONITOR_DEFAULTTONEAREST, MONITORINFO,
                MonitorFromPoint, ReleaseDC, SelectObject,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
                EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART, GetCursorPos, GetWindowRect,
                IsWindow, IsZoomed, MSG, OBJID_WINDOW, PM_REMOVE, PeekMessageW, RegisterClassW,
                SW_HIDE, SW_MAXIMIZE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
                SetWindowPos, ShowWindow, TranslateMessage, ULW_ALPHA, UpdateLayeredWindow,
                WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WNDCLASSW, WS_EX_LAYERED,
                WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
            },
        },
    },
    core::{PCWSTR, w},
};

const CLASS: PCWSTR = w!("Whirlpool.DragOverlay");
/// Room around the window for the shadow to fade out in, and how far the lift moves it down.
const SHADOW_MARGIN: i32 = 64;
const SHADOW_SIGMA: f32 = 20.0;
const SHADOW_LIFT: i32 = 18;
const SHADOW_PEAK: f32 = 0.60;
/// How close to a side counts as touching it, and how far along a side a corner reaches (pixels).
const EDGE: i32 = 8;
const CORNER: i32 = 140;
/// A window dropped with less than this much showing, or its title bar off the work area, is pulled back.
const GRAB: i32 = 80;
const TITLE: i32 = 48;
const PREVIEW_INSET: i32 = 10;

static ENABLED: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);
static HOOK: AtomicIsize = AtomicIsize::new(0);

/// Turns the motion on or off from the settings; takes effect for the next drag.
pub fn configure(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// Listens for the start and end of window moves. Call on the thread that runs the message loop.
pub fn install() {
    // SAFETY: Delivered through this thread's message loop; no code runs in other processes.
    unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_MOVESIZESTART,
            EVENT_SYSTEM_MOVESIZEEND,
            None,
            Some(on_move_size),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        HOOK.store(hook.0 as isize, Ordering::Relaxed);
    }
}

pub fn uninstall() {
    let hook = HOOK.swap(0, Ordering::Relaxed);
    if hook != 0 {
        // SAFETY: The hook came from `SetWinEventHook` above and is released once.
        unsafe {
            let _ = UnhookWinEvent(HWINEVENTHOOK(hook as *mut c_void));
        }
    }
}

unsafe extern "system" fn on_move_size(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    if object != OBJID_WINDOW.0 {
        return;
    }
    if event == EVENT_SYSTEM_MOVESIZESTART {
        if ENABLED.load(Ordering::Relaxed) && !ACTIVE.swap(true, Ordering::AcqRel) {
            STOP.store(false, Ordering::Release);
            let window = hwnd.0 as isize;
            let spawned = thread::Builder::new()
                .name("whirlpool-drag".into())
                .spawn(move || {
                    run(HWND(window as *mut c_void));
                    ACTIVE.store(false, Ordering::Release);
                });
            if spawned.is_err() {
                ACTIVE.store(false, Ordering::Release);
            }
        }
    } else if event == EVENT_SYSTEM_MOVESIZEEND {
        STOP.store(true, Ordering::Release);
    }
}

unsafe extern "system" fn overlay_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: Default handling.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

fn to_area(rect: RECT) -> Area {
    Area {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

/// The window rectangle (with Windows' invisible resize borders) and the visible one.
fn frames(hwnd: HWND) -> Option<(Area, Area)> {
    let mut window = RECT::default();
    let mut visible = RECT::default();
    // SAFETY: Out-pointers to local structures of the sizes passed.
    unsafe {
        GetWindowRect(hwnd, &mut window).ok()?;
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            std::ptr::from_mut(&mut visible).cast(),
            size_of::<RECT>() as u32,
        )
        .is_err()
        {
            visible = window;
        }
    }
    Some((to_area(window), to_area(visible)))
}

fn work_area_at(x: i32, y: i32) -> Option<Area> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` has its size set, as the call requires.
    unsafe {
        let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        GetMonitorInfoW(monitor, &mut info)
            .as_bool()
            .then(|| to_area(info.rcWork))
    }
}

/// Draws `fill` into a 32-bit premultiplied bitmap and puts it on the layered window.
fn present(
    hwnd: HWND,
    size: (i32, i32),
    origin: (i32, i32),
    alpha: u8,
    fill: &dyn Fn(&mut [u32], i32, i32),
) -> bool {
    if size.0 <= 0 || size.1 <= 0 {
        return false;
    }
    // SAFETY: Every GDI object created here is released before returning; the pixel slice covers
    // exactly the section the system allocated.
    unsafe {
        let screen = GetDC(None);
        let dc = CreateCompatibleDC(Some(screen));
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size.0,
                biHeight: -size.1,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut c_void = std::ptr::null_mut();
        let ok = match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(bitmap) => {
                let pixels =
                    std::slice::from_raw_parts_mut(bits.cast::<u32>(), (size.0 * size.1) as usize);
                fill(pixels, size.0, size.1);
                let old = SelectObject(dc, HGDIOBJ(bitmap.0));
                let blend = BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: alpha,
                    AlphaFormat: AC_SRC_ALPHA as u8,
                };
                let shown = UpdateLayeredWindow(
                    hwnd,
                    Some(screen),
                    Some(&POINT {
                        x: origin.0,
                        y: origin.1,
                    }),
                    Some(&SIZE {
                        cx: size.0,
                        cy: size.1,
                    }),
                    Some(dc),
                    Some(&POINT { x: 0, y: 0 }),
                    COLORREF(0),
                    Some(&blend),
                    ULW_ALPHA,
                )
                .is_ok();
                SelectObject(dc, old);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                shown
            }
            Err(_) => false,
        };
        let _ = DeleteDC(dc);
        ReleaseDC(None, screen);
        ok
    }
}

/// Changes only the opacity of a layered window already showing a bitmap.
fn set_alpha(hwnd: HWND, alpha: u8) {
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: alpha,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    // SAFETY: Only the blend function is given, which leaves the bitmap and position as they are.
    unsafe {
        let _ = UpdateLayeredWindow(
            hwnd,
            None,
            None,
            None,
            None,
            None,
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
    }
}

fn create_overlay(topmost: bool) -> Option<HWND> {
    // SAFETY: Registers a class (a repeat registration fails harmlessly) and creates a window owned by
    // this thread, which also pumps its messages.
    unsafe {
        let instance = GetModuleHandleW(None).ok()?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(overlay_proc),
            hInstance: instance.into(),
            lpszClassName: CLASS,
            ..Default::default()
        };
        RegisterClassW(&class);
        let mut ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT;
        if topmost {
            ex |= WS_EX_TOPMOST;
        }
        CreateWindowExW(
            ex,
            CLASS,
            w!("Whirlpool drag overlay"),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .ok()
    }
}

/// The shadow of a `width` x `height` window: a blurred rectangle, lowered by the lift, drawn as the
/// product of two one-dimensional profiles (see `snap::shadow_profile`).
fn fill_shadow(pixels: &mut [u32], full_width: i32, full_height: i32) {
    let body_width = (full_width - 2 * SHADOW_MARGIN) as f32;
    let body_height = (full_height - 2 * SHADOW_MARGIN) as f32;
    let margin = SHADOW_MARGIN as f32;
    let across: Vec<f32> = (0..full_width)
        .map(|x| snap::shadow_profile(x as f32 + 0.5, margin, margin + body_width, SHADOW_SIGMA))
        .collect();
    let down: Vec<f32> = (0..full_height)
        .map(|y| {
            snap::shadow_profile(
                y as f32 + 0.5,
                margin + SHADOW_LIFT as f32,
                margin + SHADOW_LIFT as f32 + body_height,
                SHADOW_SIGMA,
            )
        })
        .collect();
    for (y, row) in pixels.chunks_exact_mut(full_width as usize).enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            let alpha = (SHADOW_PEAK * across[x] * down[y] * 255.0) as u32;
            // Black, so premultiplied colour is zero and only alpha remains.
            *pixel = alpha << 24;
        }
    }
}

/// A translucent rounded panel with a brighter rim: where the window would land.
fn fill_preview(pixels: &mut [u32], width: i32, height: i32) {
    let (half_w, half_h, radius) = (width as f32 / 2.0, height as f32 / 2.0, 16.0_f32);
    let (red, green, blue) = (120.0_f32, 182.0_f32, 255.0_f32);
    for (y, row) in pixels.chunks_exact_mut(width as usize).enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            let px = x as f32 + 0.5 - half_w;
            let py = y as f32 + 0.5 - half_h;
            let qx = px.abs() - (half_w - radius);
            let qy = py.abs() - (half_h - radius);
            // Signed distance to the rounded rectangle: negative inside.
            let d = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
            let inside = (0.5 - d).clamp(0.0, 1.0);
            let rim = inside * (d + 3.0).clamp(0.0, 1.0);
            let alpha = (inside * 0.16 + rim * 0.55).min(1.0);
            let channel = |c: f32| (c * alpha) as u32;
            *pixel = ((alpha * 255.0) as u32) << 24
                | channel(red) << 16
                | channel(green) << 8
                | channel(blue);
        }
    }
}

fn cursor() -> Option<(i32, i32)> {
    let mut point = POINT::default();
    // SAFETY: Out-pointer to a local structure.
    unsafe { GetCursorPos(&mut point).ok()? };
    Some((point.x, point.y))
}

fn pump_messages() {
    let mut msg = MSG::default();
    // SAFETY: Messages of the overlay windows this thread owns.
    unsafe {
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn frame_wait() {
    // SAFETY: Blocks until the next composition pass, which is the pacing used everywhere else.
    unsafe {
        let _ = DwmFlush();
    }
}

fn place_shadow(shadow: HWND, behind: HWND, visible: Area) {
    // SAFETY: Moves our own window directly behind the dragged one without activating it.
    unsafe {
        let _ = SetWindowPos(
            shadow,
            Some(behind),
            visible.left - SHADOW_MARGIN,
            visible.top - SHADOW_MARGIN,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

fn run(target: HWND) {
    // SAFETY: Plain queries about a window handle that may be gone; each reports failure.
    let usable = unsafe { IsWindow(Some(target)).as_bool() && !IsZoomed(target).as_bool() };
    let Some((_, start)) = frames(target).filter(|_| usable) else {
        wait_for_end();
        return;
    };
    let (Some(shadow), Some(preview)) = (create_overlay(false), create_overlay(true)) else {
        wait_for_end();
        return;
    };
    let size = (start.width(), start.height());
    let shadow_size = (size.0 + 2 * SHADOW_MARGIN, size.1 + 2 * SHADOW_MARGIN);
    let shadow_shown = present(
        shadow,
        shadow_size,
        (start.left - SHADOW_MARGIN, start.top - SHADOW_MARGIN),
        255,
        &fill_shadow,
    );
    if shadow_shown {
        // SAFETY: Shows our own window without taking the focus.
        unsafe {
            let _ = ShowWindow(shadow, SW_SHOWNOACTIVATE);
        }
    }

    let mut zone: Option<Zone> = None;
    let mut shown_zone: Option<Zone> = None;
    let mut fade = Spring::at(0.0, 26.0);
    let mut resized = false;
    let mut last = Instant::now();
    let mut visible = start;
    let mut work = work_area_at(start.left, start.top);
    while !STOP.load(Ordering::Acquire) {
        frame_wait();
        pump_messages();
        // SAFETY: A handle query.
        if !unsafe { IsWindow(Some(target)).as_bool() } {
            break;
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32().clamp(0.001, 0.05);
        last = now;
        let Some((_, now_visible)) = frames(target) else {
            continue;
        };
        if (now_visible.width() - size.0).abs() > 2 || (now_visible.height() - size.1).abs() > 2 {
            // The size is changing: this is a resize, not a move. Leave the window alone.
            resized = true;
            // SAFETY: Hides our own windows.
            unsafe {
                let _ = ShowWindow(shadow, SW_HIDE);
                let _ = ShowWindow(preview, SW_HIDE);
            }
            break;
        }
        visible = now_visible;
        if shadow_shown {
            place_shadow(shadow, target, visible);
        }
        if let Some((x, y)) = cursor() {
            work = work_area_at(x, y).or(work);
            zone = work.and_then(|work| snap::zone_at(x, y, work, EDGE, CORNER));
        }
        match (zone, work) {
            (Some(zone_now), Some(work)) => {
                if shown_zone != Some(zone_now) {
                    let rect = snap::zone_rect(zone_now, work);
                    let inset = Area {
                        left: rect.left + PREVIEW_INSET,
                        top: rect.top + PREVIEW_INSET,
                        right: rect.right - PREVIEW_INSET,
                        bottom: rect.bottom - PREVIEW_INSET,
                    };
                    if present(
                        preview,
                        (inset.width(), inset.height()),
                        (inset.left, inset.top),
                        0,
                        &fill_preview,
                    ) {
                        // SAFETY: Shows our own window without taking the focus.
                        unsafe {
                            let _ = ShowWindow(preview, SW_SHOWNOACTIVATE);
                        }
                    }
                    fade = Spring::at(if shown_zone.is_some() { 0.45 } else { 0.0 }, 26.0);
                    shown_zone = Some(zone_now);
                }
                fade.target = 1.0;
            }
            _ => {
                if shown_zone.take().is_some() {
                    // SAFETY: Hides our own window.
                    unsafe {
                        let _ = ShowWindow(preview, SW_HIDE);
                    }
                }
            }
        }
        if shown_zone.is_some() {
            fade.step(dt);
            set_alpha(preview, (fade.value.clamp(0.0, 1.0) * 255.0) as u8);
        }
    }

    // SAFETY: Hides our own window.
    unsafe {
        let _ = ShowWindow(preview, SW_HIDE);
    }
    if !resized {
        settle(target, shadow, shadow_shown, zone, work, visible);
    }
    // SAFETY: Destroys the windows this thread created.
    unsafe {
        let _ = DestroyWindow(shadow);
        let _ = DestroyWindow(preview);
    }
}

/// Waits for the end of a move we are not decorating, so a second drag does not start meanwhile.
fn wait_for_end() {
    while !STOP.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }
}

/// After release: carry the window to where it belongs on a spring while the lifted shadow settles.
fn settle(
    target: HWND,
    shadow: HWND,
    shadow_shown: bool,
    zone: Option<Zone>,
    work: Option<Area>,
    dropped: Area,
) {
    // Windows' own snapping acts on release; give it a moment, then see where the window is.
    thread::sleep(Duration::from_millis(60));
    let Some((window, visible)) = frames(target) else {
        return;
    };
    let wanted = match (zone, work) {
        (Some(Zone::Maximize), _) => {
            // SAFETY: A show command on the dragged window, as the title bar's own maximise does.
            unsafe {
                if !IsZoomed(target).as_bool() {
                    let _ = ShowWindow(target, SW_MAXIMIZE);
                }
            }
            None
        }
        (Some(zone), Some(work)) => Some(snap::zone_rect(zone, work)),
        (None, Some(work)) => Some(snap::rescue(visible, work, GRAB, TITLE)),
        _ => None,
    }
    .filter(|wanted| {
        // Already there (Windows snapped it itself, or it was in reach): do not move it again.
        (wanted.left - visible.left).abs() > 6
            || (wanted.top - visible.top).abs() > 6
            || (wanted.right - visible.right).abs() > 6
            || (wanted.bottom - visible.bottom).abs() > 6
    });
    let _ = dropped;

    // The invisible borders Windows adds around a window, so the visible edge lands on the target.
    let border = Area {
        left: visible.left - window.left,
        top: visible.top - window.top,
        right: window.right - visible.right,
        bottom: window.bottom - visible.bottom,
    };
    let omega = 20.0;
    let mut edges = [
        Spring::at(window.left as f32, omega),
        Spring::at(window.top as f32, omega),
        Spring::at(window.right as f32, omega),
        Spring::at(window.bottom as f32, omega),
    ];
    if let Some(wanted) = wanted {
        edges[0].target = (wanted.left - border.left) as f32;
        edges[1].target = (wanted.top - border.top) as f32;
        edges[2].target = (wanted.right + border.right) as f32;
        edges[3].target = (wanted.bottom + border.bottom) as f32;
    }
    let mut lift = Spring::at(1.0, 18.0);
    lift.target = 0.0;
    let begun = Instant::now();
    let mut last = begun;
    loop {
        frame_wait();
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32().clamp(0.001, 0.05);
        last = now;
        lift.step(dt);
        let mut current = visible;
        if wanted.is_some() {
            for edge in &mut edges {
                edge.step(dt);
            }
            let [left, top, right, bottom] = edges.map(|edge| edge.value.round() as i32);
            // SAFETY: Moves and sizes the dragged window, as a program's own code could.
            unsafe {
                let _ = SetWindowPos(
                    target,
                    None,
                    left,
                    top,
                    (right - left).max(1),
                    (bottom - top).max(1),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            current = Area {
                left: left + border.left,
                top: top + border.top,
                right: right - border.right,
                bottom: bottom - border.bottom,
            };
        }
        if shadow_shown {
            if wanted.is_some() {
                // The shadow bitmap is sized for the dropped window; follow its corner while it fades.
                place_shadow(shadow, target, current);
            }
            set_alpha(shadow, (lift.value.clamp(0.0, 1.0) * 255.0) as u8);
        }
        let moving = wanted.is_some() && !edges.iter().all(Spring::is_settled);
        if (!moving && lift.is_settled()) || begun.elapsed() > Duration::from_millis(900) {
            break;
        }
    }
    if let Some(wanted) = wanted {
        // Land exactly, whatever the last spring step rounded to.
        // SAFETY: As above.
        unsafe {
            let _ = SetWindowPos(
                target,
                None,
                wanted.left - border.left,
                wanted.top - border.top,
                wanted.width() + border.left + border.right,
                wanted.height() + border.top + border.bottom,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
}
