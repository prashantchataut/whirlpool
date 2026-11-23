//! The system overlay: `whirlpool.exe --overlay`. A small topmost window with live readings.
//!
//! It is its own short-lived process, started and stopped by the dock's global hotkey, so when it
//! is hidden nothing of it exists: no thread, no timer, no memory. While shown it wakes once per
//! interval (default 1 s) and reads cheap counters; there is no polling loop.
//!
//! What is measured, and what is not:
//! - **Frames:** the Desktop Window Manager's presented frames per second and the display's
//!   refresh rate. This is the compositor, not the focused game: per-application frame rate needs
//!   ETW kernel tracing, which needs administrator rights. The overlay says "Desktop".
//! - CPU (system times), memory, GPU 3D-engine load and video memory in use (performance
//!   counters), network throughput of physical adapters, battery level. Temperatures and power
//!   draw are not shown: Windows exposes them only through vendor drivers.

use std::{
    cell::RefCell,
    mem::size_of,
    time::{Instant, SystemTime},
};

use anyhow::{Result, bail};
use serde_json::{Map, Value, json};
use whirlpool_config::{Config, OverlayCorner, OverlayPreset};
use whirlpool_core::overlay::{self as geom, Corner};
use windows::{
    Win32::{
        Foundation::{COLORREF, FILETIME, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
        Graphics::{
            Dwm::{DWM_TIMING_INFO, DwmGetCompositionTimingInfo},
            Dxgi::{CreateDXGIFactory1, IDXGIFactory1},
            Gdi::{
                BeginPaint, CLEARTYPE_QUALITY, CreateFontW, CreateRoundRectRgn, CreateSolidBrush,
                DEFAULT_CHARSET, DeleteObject, EndPaint, FF_DONTCARE, FW_NORMAL, FW_SEMIBOLD,
                FillRect, GetDC, GetMonitorInfoW, GetTextExtentPoint32W, HFONT, HGDIOBJ,
                InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
                OUT_DEFAULT_PRECIS, PAINTSTRUCT, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
                SetWindowRgn, TRANSPARENT, TextOutW,
            },
        },
        NetworkManagement::IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2},
        System::{
            LibraryLoader::GetModuleHandleW,
            Performance::{
                PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
                PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
                PdhGetFormattedCounterArrayW, PdhOpenQueryW,
            },
            SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
            Threading::GetSystemTimes,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            Shell::{QUNS_BUSY, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW,
                GetMessageW, GetWindowRect, HTCAPTION, KillTimer, LWA_ALPHA, MSG, PostMessageW,
                PostQuitMessage, RegisterClassW, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE,
                SWP_NOZORDER, SetLayeredWindowAttributes, SetTimer, SetWindowPos, ShowWindow,
                TranslateMessage, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE, WM_EXITSIZEMOVE,
                WM_NCHITTEST, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
                WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
            },
        },
    },
    core::{PCWSTR, w},
};

const CLASS: PCWSTR = w!("Whirlpool.Overlay");
const TIMER: usize = 1;
const MARGIN: i32 = 12;
const SNAP_REACH: i32 = 90;

/// Shows the overlay if it is hidden, hides it if it is shown.
pub fn toggle(exe: &std::path::Path) {
    // SAFETY: Looks up our own window class by name.
    match unsafe { FindWindowW(CLASS, PCWSTR::null()) } {
        Ok(window) if !window.is_invalid() => {
            // SAFETY: Posts a close request to the overlay's window.
            unsafe {
                let _ = PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        _ => {
            let _ = std::process::Command::new(exe).arg("--overlay").spawn();
        }
    }
}

#[derive(Default, Clone)]
struct Reading {
    cpu: f32,
    ram_used: f64,
    ram_total: f64,
    gpu: Option<f32>,
    vram_used: Option<f64>,
    vram_total: Option<f64>,
    down: f64,
    up: f64,
    fps: Option<f32>,
    hz: Option<f32>,
    battery: Option<(u8, bool)>,
}

struct Pdh {
    query: PDH_HQUERY,
    engine: PDH_HCOUNTER,
    memory: PDH_HCOUNTER,
}

struct Sampler {
    cpu: Option<(u64, u64, u64)>,
    net: Option<(u64, u64, Instant)>,
    frames: Option<(u64, Instant)>,
    pdh: Option<Pdh>,
    vram_total: Option<f64>,
}

fn filetime(value: FILETIME) -> u64 {
    (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
}

impl Sampler {
    fn new() -> Self {
        // SAFETY: Plain PDH setup; the query is closed in `Drop`.
        let pdh = unsafe {
            let mut query = PDH_HQUERY::default();
            (PdhOpenQueryW(PCWSTR::null(), 0, &raw mut query) == 0).then(|| {
                let (mut engine, mut memory) = (PDH_HCOUNTER::default(), PDH_HCOUNTER::default());
                let _ = PdhAddEnglishCounterW(
                    query,
                    w!(r"\GPU Engine(*)\Utilization Percentage"),
                    0,
                    &raw mut engine,
                );
                let _ = PdhAddEnglishCounterW(
                    query,
                    w!(r"\GPU Adapter Memory(*)\Dedicated Usage"),
                    0,
                    &raw mut memory,
                );
                let _ = PdhCollectQueryData(query);
                Pdh {
                    query,
                    engine,
                    memory,
                }
            })
        };
        // SAFETY: Factory and adapter are released when they go out of scope.
        let vram_total = unsafe {
            CreateDXGIFactory1::<IDXGIFactory1>()
                .ok()
                .and_then(|factory| factory.EnumAdapters1(0).ok())
                .and_then(|adapter| adapter.GetDesc1().ok())
                .map(|desc| desc.DedicatedVideoMemory as f64)
                .filter(|bytes| *bytes > 0.0)
        };
        Self {
            cpu: None,
            net: None,
            frames: None,
            pdh,
            vram_total,
        }
    }

    fn read(&mut self) -> Reading {
        let mut reading = Reading::default();
        let now = Instant::now();

        let (mut idle, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        // SAFETY: Out-pointers to local FILETIMEs.
        if unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }.is_ok() {
            let sample = (filetime(idle), filetime(kernel), filetime(user));
            if let Some(last) = self.cpu {
                let total = (sample.1 - last.1) + (sample.2 - last.2);
                let busy = total.saturating_sub(sample.0 - last.0);
                if total > 0 {
                    reading.cpu = (busy as f64 * 100.0 / total as f64) as f32;
                }
            }
            self.cpu = Some(sample);
        }

        let mut memory = MEMORYSTATUSEX {
            dwLength: size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        // SAFETY: `memory` has its length field set.
        if unsafe { GlobalMemoryStatusEx(&mut memory) }.is_ok() {
            reading.ram_total = memory.ullTotalPhys as f64;
            reading.ram_used = (memory.ullTotalPhys - memory.ullAvailPhys) as f64;
        }

        if let Some(pdh) = &self.pdh {
            // SAFETY: Counters belong to the open query.
            unsafe {
                if PdhCollectQueryData(pdh.query) == 0 {
                    let gpu = counter_items(pdh.engine)
                        .into_iter()
                        .filter(|(name, _)| {
                            name.contains("engtype_3D") || name.contains("engtype_Compute")
                        })
                        .fold(0.0, |sum, (_, value)| sum + value);
                    reading.gpu = Some(gpu.min(100.0) as f32);
                    let used: f64 = counter_items(pdh.memory).into_iter().map(|(_, v)| v).sum();
                    if used > 0.0 {
                        reading.vram_used = Some(used);
                        reading.vram_total = self.vram_total;
                    }
                }
            }
        }

        if let Some((down, up)) = network_totals() {
            if let Some((last_down, last_up, at)) = self.net {
                let seconds = now.duration_since(at).as_secs_f64().max(0.001);
                reading.down = down.saturating_sub(last_down) as f64 / seconds;
                reading.up = up.saturating_sub(last_up) as f64 / seconds;
            }
            self.net = Some((down, up, now));
        }

        let mut timing = DWM_TIMING_INFO {
            cbSize: size_of::<DWM_TIMING_INFO>() as u32,
            ..Default::default()
        };
        // SAFETY: `timing` has its size field set.
        if unsafe { DwmGetCompositionTimingInfo(HWND::default(), &mut timing) }.is_ok() {
            let denominator = timing.rateRefresh.uiDenominator.max(1);
            reading.hz = Some(timing.rateRefresh.uiNumerator as f32 / denominator as f32);
            let displayed = timing.cFrameDisplayed;
            if let Some((last, at)) = self.frames {
                let seconds = now.duration_since(at).as_secs_f64().max(0.001);
                reading.fps = Some((displayed.saturating_sub(last) as f64 / seconds) as f32);
            }
            self.frames = Some((displayed, now));
        }

        reading.battery = whirlpool_shell::system::battery().map(|b| (b.percent, b.charging));
        reading
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        if let Some(pdh) = &self.pdh {
            // SAFETY: Closes the query opened in `new`.
            unsafe {
                let _ = PdhCloseQuery(pdh.query);
            }
        }
    }
}

/// Instance name and value for every instance of a wildcard counter.
unsafe fn counter_items(counter: PDH_HCOUNTER) -> Vec<(String, f64)> {
    // SAFETY: Standard two-call PDH array read into an 8-byte aligned buffer.
    unsafe {
        let (mut size, mut count) = (0u32, 0u32);
        let _ = PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, None);
        if size == 0 {
            return Vec::new();
        }
        let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
        let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        if PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items))
            != 0
        {
            return Vec::new();
        }
        (0..count as usize)
            .map(|i| {
                let item = &*items.add(i);
                (
                    item.szName.to_string().unwrap_or_default(),
                    item.FmtValue.Anonymous.doubleValue,
                )
            })
            .collect()
    }
}

/// Bytes received and sent so far by the physical adapters that are up.
fn network_totals() -> Option<(u64, u64)> {
    // SAFETY: The table is read while allocated and freed once.
    unsafe {
        let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table).is_err() || table.is_null() {
            return None;
        }
        let rows =
            std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        let (mut down, mut up) = (0u64, 0u64);
        for row in rows {
            let hardware = row.InterfaceAndOperStatusFlags._bitfield & 1 != 0;
            if hardware && row.OperStatus.0 == 1 {
                down += row.InOctets;
                up += row.OutOctets;
            }
        }
        FreeMibTable(table.cast());
        Some((down, up))
    }
}

fn gigabytes(bytes: f64) -> String {
    format!("{:.1}", bytes / 1_073_741_824.0)
}

fn rate(bytes_per_second: f64) -> String {
    if bytes_per_second >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_second / 1_048_576.0)
    } else {
        format!("{:.0} KB/s", bytes_per_second / 1024.0)
    }
}

/// The rows to draw: label and value. `Oneline` yields a single row without a label.
fn rows(reading: &Reading, preset: OverlayPreset) -> Vec<(String, String)> {
    let frames = match (reading.fps, reading.hz) {
        (Some(fps), Some(hz)) => format!("{fps:.0} fps / {hz:.0} Hz"),
        (None, Some(hz)) => format!("{hz:.0} Hz"),
        _ => "n/a".to_owned(),
    };
    let ram = format!(
        "{} / {} GB",
        gigabytes(reading.ram_used),
        gigabytes(reading.ram_total)
    );
    match preset {
        OverlayPreset::Minimal => vec![
            ("Desktop".into(), frames),
            ("CPU".into(), format!("{:.0}%", reading.cpu)),
            ("RAM".into(), ram),
        ],
        OverlayPreset::Oneline => {
            let fps = reading
                .fps
                .map_or_else(|| "-".to_owned(), |f| format!("{f:.0} fps"));
            vec![(
                String::new(),
                format!(
                    "{fps}   CPU {:.0}%   RAM {} GB",
                    reading.cpu,
                    gigabytes(reading.ram_used)
                ),
            )]
        }
        OverlayPreset::Detailed => {
            let mut list = vec![
                ("Desktop".to_owned(), frames),
                ("CPU".to_owned(), format!("{:.0}%", reading.cpu)),
                ("RAM".to_owned(), ram),
            ];
            if let Some(gpu) = reading.gpu {
                list.push(("GPU".to_owned(), format!("{gpu:.0}%")));
            }
            if let Some(used) = reading.vram_used {
                let value = reading.vram_total.map_or_else(
                    || format!("{} GB", gigabytes(used)),
                    |total| format!("{} / {} GB", gigabytes(used), gigabytes(total)),
                );
                list.push(("VRAM".to_owned(), value));
            }
            list.push(("Down".to_owned(), rate(reading.down)));
            list.push(("Up".to_owned(), rate(reading.up)));
            if let Some((percent, charging)) = reading.battery {
                list.push((
                    "Battery".to_owned(),
                    format!("{percent}%{}", if charging { " charging" } else { "" }),
                ));
            }
            list
        }
    }
}

struct Overlay {
    config: Config,
    config_stamp: Option<SystemTime>,
    sampler: Sampler,
    rows: Vec<(String, String)>,
    font: HFONT,
    bold: HFONT,
    label_width: i32,
    size: (i32, i32),
}

thread_local! {
    static STATE: RefCell<Option<Overlay>> = const { RefCell::new(None) };
}

fn stamp() -> Option<SystemTime> {
    std::fs::metadata(whirlpool_config::default_path())
        .and_then(|m| m.modified())
        .ok()
}

fn load_config() -> Config {
    whirlpool_config::load(&whirlpool_config::default_path())
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn make_font(config: &Config, dpi: u32, weight: i32) -> HFONT {
    let height =
        -((13.0 * f64::from(config.overlay_scale) / 100.0 * f64::from(dpi) / 96.0).round() as i32);
    let face = if config.overlay_mono {
        w!("Cascadia Mono")
    } else {
        w!("Segoe UI")
    };
    // SAFETY: Creates a GDI font; it is deleted when replaced or on exit.
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            Default::default(),
            CLEARTYPE_QUALITY,
            FF_DONTCARE.0.into(),
            face,
        )
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn work_area_near(point: (i32, i32)) -> (i32, i32, i32, i32) {
    // SAFETY: Monitor queries with a valid MONITORINFO.
    unsafe {
        let monitor = MonitorFromPoint(
            POINT {
                x: point.0,
                y: point.1,
            },
            MONITOR_DEFAULTTONEAREST,
        );
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            let r = info.rcWork;
            (r.left, r.top, r.right, r.bottom)
        } else {
            (0, 0, 1920, 1040)
        }
    }
}

fn corner_of(corner: OverlayCorner) -> Corner {
    match corner {
        OverlayCorner::TopLeft => Corner::TopLeft,
        OverlayCorner::TopRight => Corner::TopRight,
        OverlayCorner::BottomLeft => Corner::BottomLeft,
        OverlayCorner::BottomRight => Corner::BottomRight,
    }
}

impl Overlay {
    /// Measures the rows and resizes and places the window.
    fn layout(&mut self, hwnd: HWND) {
        // SAFETY: Measures with a screen DC and our fonts; everything is released.
        unsafe {
            let dc = GetDC(Some(hwnd));
            let old = SelectObject(dc, HGDIOBJ(self.font.0));
            let measure = |text: &str| {
                let mut size = SIZE::default();
                let _ = GetTextExtentPoint32W(dc, &wide(text), &mut size);
                size
            };
            let line_height = measure("Ag").cy + 4;
            let (mut label_width, mut value_width) = (0, 0);
            for (label, _) in &self.rows {
                label_width = label_width.max(measure(label).cx);
            }
            // Values are painted in the bold face, so they are measured in it.
            SelectObject(dc, HGDIOBJ(self.bold.0));
            for (_, value) in &self.rows {
                value_width = value_width.max(measure(value).cx);
            }
            SelectObject(dc, old);
            ReleaseDC(Some(hwnd), dc);
            let pad = (10.0 * f64::from(self.config.overlay_scale) / 100.0) as i32 + 4;
            let gap = if label_width > 0 { pad * 2 } else { 0 };
            self.label_width = label_width;
            self.size = (
                label_width + gap + value_width + 2 * pad,
                line_height * self.rows.len() as i32 + 2 * pad - 4,
            );
        }
        self.place(hwnd);
    }

    fn place(&self, hwnd: HWND) {
        let (x, y) = match (self.config.overlay_x, self.config.overlay_y) {
            (Some(x), Some(y)) => geom::clamp_inside((x, y), work_area_near((x, y)), self.size),
            _ => {
                let work = work_area_near((0, 0));
                geom::corner_origin(
                    corner_of(self.config.overlay_corner),
                    work,
                    self.size,
                    MARGIN,
                )
            }
        };
        // SAFETY: Moves and rounds our own window.
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                None,
                x,
                y,
                self.size.0,
                self.size.1,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            let radius = (10.0 * f64::from(GetDpiForWindow(hwnd)) / 96.0) as i32;
            let region = CreateRoundRectRgn(
                0,
                0,
                self.size.0 + 1,
                self.size.1 + 1,
                radius * 2,
                radius * 2,
            );
            SetWindowRgn(hwnd, Some(region), true);
        }
    }

    fn tick(&mut self, hwnd: HWND) {
        // SAFETY: Reads the shell's notification state.
        let fullscreen = unsafe { SHQueryUserNotificationState() }
            .is_ok_and(|state| state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_BUSY);
        if fullscreen && !self.config.overlay_over_fullscreen {
            // SAFETY: Hides our window; the readings carry on at the same cost.
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
            return;
        }
        if stamp() != self.config_stamp {
            self.config_stamp = stamp();
            self.config = load_config();
            // SAFETY: Rebuilds fonts and the timer for the new settings.
            unsafe {
                let dpi = GetDpiForWindow(hwnd);
                let _ = DeleteObject(HGDIOBJ(self.font.0));
                let _ = DeleteObject(HGDIOBJ(self.bold.0));
                self.font = make_font(&self.config, dpi, FW_NORMAL.0 as i32);
                self.bold = make_font(&self.config, dpi, FW_SEMIBOLD.0 as i32);
                let _ = SetLayeredWindowAttributes(
                    hwnd,
                    COLORREF(0),
                    (u32::from(self.config.overlay_opacity) * 255 / 100) as u8,
                    LWA_ALPHA,
                );
                let _ = SetTimer(Some(hwnd), TIMER, self.config.overlay_interval_ms, None);
            }
        }
        let reading = self.sampler.read();
        self.rows = rows(&reading, self.config.overlay_preset);
        self.layout(hwnd);
        // SAFETY: Shows without taking focus and asks for a repaint.
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }

    fn paint(&self, hwnd: HWND) {
        // SAFETY: Standard BeginPaint/EndPaint with GDI objects created and deleted here.
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            let back = CreateSolidBrush(COLORREF(0x0018_1410));
            FillRect(
                dc,
                &RECT {
                    left: 0,
                    top: 0,
                    right: self.size.0,
                    bottom: self.size.1,
                },
                back,
            );
            let _ = DeleteObject(HGDIOBJ(back.0));
            SetBkMode(dc, TRANSPARENT);
            let pad = (10.0 * f64::from(self.config.overlay_scale) / 100.0) as i32 + 4;
            let old = SelectObject(dc, HGDIOBJ(self.font.0));
            let probe = |text: &str| {
                let mut size = SIZE::default();
                let _ = GetTextExtentPoint32W(dc, &wide(text), &mut size);
                size
            };
            let line = probe("Ag").cy + 4;
            let mut y = pad - 2;
            for (label, value) in &self.rows {
                if !label.is_empty() {
                    SetTextColor(dc, COLORREF(0x00b4_a898));
                    let _ = TextOutW(dc, pad, y, &wide(label));
                }
                SetTextColor(dc, COLORREF(0x00f3_f1ed));
                SelectObject(dc, HGDIOBJ(self.bold.0));
                let x = if label.is_empty() {
                    pad
                } else {
                    self.size.0 - pad - probe(value).cx
                };
                let _ = TextOutW(dc, x, y, &wide(value));
                SelectObject(dc, HGDIOBJ(self.font.0));
                y += line;
            }
            SelectObject(dc, old);
            let _ = EndPaint(hwnd, &ps);
        }
    }

    /// After a drag: snap to a corner when dropped near one, otherwise remember the spot.
    fn dropped(&mut self, hwnd: HWND) {
        let mut rect = RECT::default();
        // SAFETY: Reads our window's rectangle.
        if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
            return;
        }
        let work = work_area_near((rect.left, rect.top));
        let mut patch = Map::new();
        match geom::snap((rect.left, rect.top), work, self.size, MARGIN, SNAP_REACH) {
            Some(corner) => {
                let name = match corner {
                    Corner::TopLeft => "topleft",
                    Corner::TopRight => "topright",
                    Corner::BottomLeft => "bottomleft",
                    Corner::BottomRight => "bottomright",
                };
                patch.insert("overlayCorner".into(), json!(name));
                patch.insert("overlayX".into(), Value::Null);
                patch.insert("overlayY".into(), Value::Null);
                self.config.overlay_x = None;
                self.config.overlay_y = None;
                self.config.overlay_corner = match corner {
                    Corner::TopLeft => OverlayCorner::TopLeft,
                    Corner::TopRight => OverlayCorner::TopRight,
                    Corner::BottomLeft => OverlayCorner::BottomLeft,
                    Corner::BottomRight => OverlayCorner::BottomRight,
                };
            }
            None => {
                patch.insert("overlayX".into(), json!(rect.left));
                patch.insert("overlayY".into(), json!(rect.top));
                self.config.overlay_x = Some(rect.left);
                self.config.overlay_y = Some(rect.top);
            }
        }
        let _ = whirlpool_config::update_preferences(&whirlpool_config::default_path(), patch);
        self.config_stamp = stamp();
        self.place(hwnd);
    }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_TIMER if w.0 == TIMER => {
            STATE.with(|state| {
                if let Some(overlay) = state.borrow_mut().as_mut() {
                    overlay.tick(hwnd);
                }
            });
            LRESULT(0)
        }
        WM_PAINT => {
            STATE.with(|state| {
                if let Some(overlay) = state.borrow().as_ref() {
                    overlay.paint(hwnd);
                }
            });
            LRESULT(0)
        }
        WM_NCHITTEST => STATE.with(|state| {
            let draggable = state
                .borrow()
                .as_ref()
                .is_some_and(|o| !o.config.overlay_click_through);
            if draggable {
                LRESULT(HTCAPTION as isize)
            } else {
                // SAFETY: Default handling for the message we did not claim.
                unsafe { DefWindowProcW(hwnd, msg, w, l) }
            }
        }),
        WM_EXITSIZEMOVE => {
            STATE.with(|state| {
                if let Some(overlay) = state.borrow_mut().as_mut() {
                    overlay.dropped(hwnd);
                }
            });
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            STATE.with(|state| {
                if let Some(overlay) = state.borrow().as_ref() {
                    overlay.place(hwnd);
                }
            });
            LRESULT(0)
        }
        WM_CLOSE => {
            // SAFETY: Destroys our own window.
            unsafe {
                let _ = KillTimer(Some(hwnd), TIMER);
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: Ends this thread's message loop.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: Default handling.
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
}

/// Runs the overlay until it is closed (by the hotkey, which posts WM_CLOSE).
pub fn run() -> Result<()> {
    // SAFETY: Window class, window and message loop on this thread.
    unsafe {
        if FindWindowW(CLASS, PCWSTR::null()).is_ok_and(|window| !window.is_invalid()) {
            return Ok(());
        }
        let instance = GetModuleHandleW(None)?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: instance.into(),
            lpszClassName: CLASS,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            bail!("could not register the overlay window class");
        }
        let config = load_config();
        let mut ex = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        if config.overlay_click_through {
            ex |= WS_EX_TRANSPARENT;
        }
        let hwnd = CreateWindowExW(
            ex,
            CLASS,
            w!("Whirlpool overlay"),
            WS_POPUP,
            0,
            0,
            10,
            10,
            None,
            None,
            Some(instance.into()),
            None,
        )?;
        let dpi = GetDpiForWindow(hwnd);
        let mut overlay = Overlay {
            font: make_font(&config, dpi, FW_NORMAL.0 as i32),
            bold: make_font(&config, dpi, FW_SEMIBOLD.0 as i32),
            config_stamp: stamp(),
            sampler: Sampler::new(),
            rows: Vec::new(),
            label_width: 0,
            size: (10, 10),
            config,
        };
        SetLayeredWindowAttributes(
            hwnd,
            COLORREF(0),
            (u32::from(overlay.config.overlay_opacity) * 255 / 100) as u8,
            LWA_ALPHA,
        )?;
        let interval = overlay.config.overlay_interval_ms;
        // The first reading has no earlier sample to compare with; the timer supplies the next.
        overlay.tick(hwnd);
        STATE.with(|state| *state.borrow_mut() = Some(overlay));
        SetTimer(Some(hwnd), TIMER, interval, None);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
