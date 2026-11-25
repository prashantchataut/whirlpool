//! Minimal macOS-style menu bar across the top of the screen.
//!
//! Hiding the Windows taskbar also hides its clock and status area; this bar
//! brings them back: the name of the app in front, the battery, a control
//! center button (opens the Windows quick settings: network, volume,
//! brightness) and the date and time (opens the notification center).
//!
//! Cost while idle: one timer that fires once a minute, on the minute, to
//! redraw the clock. Everything else is event-driven.
//!
//! With auto-hide on, the bar reserves no space and stays off screen. Resting
//! the pointer briefly at the top edge reveals it.
//!
//! Status items, right to left as on a Mac: clock, control center, search,
//! network, volume, battery, then Task View and Widgets. Search, Task View and
//! Widgets appear only if the user has them on the Windows taskbar. Network
//! and volume changes arrive through separate notifications; nothing polls.

use anyhow::Result;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, AtomicIsize, Ordering},
    },
    time::{Duration, Instant},
};
use whirlpool_config::{IslandStyle, Language};
use whirlpool_shell::{
    accessories::{Kind as AccessoryKind, Snapshot as AccessorySnapshot},
    appbar::{AppBar, Edge},
    launcher,
    status::{self, AudioWatch, Network, NetworkWatch, Privacy, TaskbarButtons, Volume},
    system::{self, Battery},
    watch::RegistryWatch,
};
use windows::{
    Foundation::{TimeSpan, TypedEventHandler},
    UI::{
        Color,
        Composition::{
            CompositionBatchTypes, CompositionBrush, CompositionRoundedRectangleGeometry,
            CompositionScopedBatch, CompositionSurfaceBrush, Compositor, ContainerVisual,
            Desktop::DesktopWindowTarget, ShapeVisual, SpriteVisual, VisualCollection,
        },
    },
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SYSTEMTIME, WPARAM},
        Graphics::Dwm::{DWMWA_USE_HOSTBACKDROPBRUSH, DwmSetWindowAttribute},
        System::{
            LibraryLoader::GetModuleHandleW, SystemInformation::GetLocalTime,
            WinRT::Composition::ICompositorDesktopInterop,
        },
        UI::Input::KeyboardAndMouse::{
            GetCapture, ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
        },
        UI::WindowsAndMessaging::{
            CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, HHOOK,
            HWND_TOPMOST, IDC_ARROW, KillTimer, LoadCursorW, MSLLHOOKSTRUCT,
            PBT_POWERSETTINGCHANGE, PostMessageW, RegisterClassW, SWP_HIDEWINDOW, SWP_NOACTIVATE,
            SWP_SHOWWINDOW, SetCursor, SetTimer, SetWindowPos, SetWindowsHookExW,
            UnhookWindowsHookEx, WH_MOUSE_LL, WM_APP, WM_CAPTURECHANGED, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCHITTEST,
            WM_RBUTTONDOWN, WM_SETCURSOR, WM_SETTINGCHANGE, WM_TIMECHANGE, WM_TIMER,
            WM_XBUTTONDOWN, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
            WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    },
    core::{BOOL, Interface, h, w},
};
use windows_numerics::{Vector2, Vector3};

use crate::{
    accessories, app, edge_watch,
    gfx::{Gfx, Ink, LabelBrush},
    mascot,
    media::{self, Control as MediaControl, Track},
    scene::Theme,
    strings,
    usage::{
        self, ActiveProviders, Monitor as UsageMonitor, Provider, ProviderUsage, Snapshot, State,
    },
};

mod layout;
mod player_draw;
mod surfaces;
#[cfg(test)]
mod tests;
mod usage_draw;
pub(crate) mod visuals;

use layout::*;
use player_draw::*;
use surfaces::*;
use usage_draw::*;
pub(crate) use visuals::*;

// Logical pixels.
/// Height and top gap chosen in Settings. Geometry code is spread over this
/// file and the dock, so they are read from here instead of passed around.
static BAR_HEIGHT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(28);
static BAR_GAP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Pixels the bottom edge is already taken by the dock. A bottom bar sits
/// above them instead of underneath the dock.
static DOCK_LIFT: AtomicI32 = AtomicI32::new(0);

/// Tells the bar how much of the bottom edge the dock covers. Returns true
/// when the value changed and the bar must be laid out again.
pub fn set_dock_lift(pixels: i32) -> bool {
    DOCK_LIFT.swap(pixels.max(0), Ordering::Relaxed) != pixels.max(0)
}

fn dock_lift() -> i32 {
    DOCK_LIFT.load(Ordering::Relaxed)
}

/// Starts the quick settings panel (a short-lived `whirlpool.exe`) next to
/// the bar. If it cannot start, Windows' own panel opens instead.
fn open_quick_panel(right: i32, anchor_y: i32, bottom: bool) {
    let started = std::env::current_exe().and_then(|exe| {
        std::process::Command::new(exe)
            .arg("--quick-settings")
            .arg(right.to_string())
            .arg(anchor_y.to_string())
            .arg(if bottom { "bottom" } else { "top" })
            .spawn()
    });
    if started.is_err() {
        launcher::open_quick_settings();
    }
}

fn bar_height() -> f32 {
    BAR_HEIGHT.load(Ordering::Relaxed) as f32
}

/// Stores the chosen size and gap before anything is laid out.
fn configure(widgets: &MenuBarWidgets) {
    BAR_AT_BOTTOM.store(
        widgets.position == whirlpool_config::BarPosition::Bottom,
        Ordering::Relaxed,
    );
    BAR_HEIGHT.store(widgets.size.height() as u32, Ordering::Relaxed);
    BAR_GAP.store(
        if widgets.floating { widgets.gap } else { 0 },
        Ordering::Relaxed,
    );
}

/// Side inset and top gap of a floating bar, in pixels (zero when docked).
fn float_px(widgets: &MenuBarWidgets, scale: f32) -> (i32, i32) {
    if !widgets.floating {
        return (0, 0);
    }
    (
        (widgets.inset as f32 * scale).round() as i32,
        (widgets.gap as f32 * scale).round() as i32,
    )
}

/// The screen rectangle the bar lays itself out in.
fn inset_screen(screen: RECT, widgets: &MenuBarWidgets, scale: f32) -> RECT {
    configure(widgets);
    let (inset, gap) = float_px(widgets, scale);
    let (mut left, mut right) = (screen.left + inset, screen.right - inset);
    if widgets.floating && widgets.width < 100 {
        let room = right - left;
        let narrow = (room as f32 * widgets.width.clamp(40, 100) as f32 / 100.0).round() as i32;
        match widgets.align {
            whirlpool_config::BarAlign::Left => right = left + narrow,
            whirlpool_config::BarAlign::Right => left = right - narrow,
            whirlpool_config::BarAlign::Center => {
                left += (room - narrow) / 2;
                right = left + narrow;
            }
        }
    }
    if widgets.position == whirlpool_config::BarPosition::Bottom {
        return RECT {
            left,
            right,
            top: screen.top,
            bottom: screen.bottom - gap - dock_lift(),
        };
    }
    RECT {
        left,
        right,
        top: screen.top + gap,
        bottom: screen.bottom,
    }
}
const SIDE_MARGIN: f32 = 14.0;
const ITEM_GAP: f32 = 16.0;
/// Extra clickable room around an item.
const HIT_SLOP: f32 = 7.0;

const MEDIA_CHANGED: u32 = WM_APP + 28;
const VOLUME_CAPTURE_LOST: u32 = WM_APP + 31;
const AUDIO_CHANGED: u32 = WM_APP + 32;
const AUDIO_TICK: u32 = WM_APP + 33;
const VOLUME_TIMER: usize = 4;
const ACCESSORIES_CHANGED: u32 = WM_APP + 30;

const CLOCK_TIMER: usize = 1;
const POINTER_TIMER: usize = 2;
const POINTER_CHECK_MS: u32 = 100;
const HIDE_ANIMATION_TIMER: usize = 3;
const REVEAL_MS: u32 = 180;
const HIDE_MS: u32 = 130;
// Recovery only; normal completion comes from the compositor, not this timer.
const HIDE_FALLBACK_MS: u32 = HIDE_MS + 250;
const HIDE_ANIMATION_DONE: u32 = WM_APP + 29;
const BUBBLE_VERTICAL_INSET: f32 = 2.5;
const BUBBLE_MUSIC_WIDTH: f32 = 164.0;
const BUBBLE_PROVIDER_WIDTH: f32 = 62.0;
const BUBBLE_USAGE_PADDING: f32 = 16.0;
const PANEL_GAP: f32 = 5.0;
const PANEL_SHADOW_MARGIN: f32 = 20.0;
const WM_POWERBROADCAST: u32 = 0x0218;
const WM_MOUSELEAVE: u32 = 0x02A3;
/// AppBar notifications need a message id; the bar ignores them.
const APPBAR_CALLBACK: u32 = WM_APP + 20;
/// Posted by the network watch; audio uses its own coalesced notification.
const STATUS_CHANGED: u32 = WM_APP + 21;
/// Posted by the bar to itself to update from the message loop.
const TICK: u32 = WM_APP + 22;
const USAGE_CHANGED: u32 = WM_APP + 23;
const DISMISS_ISLAND: u32 = WM_APP + 24;
/// Privacy activity or the user's taskbar preferences changed.
const REGISTRY_CHANGED: u32 = WM_APP + 25;
/// `wParam` of WM_SETTINGCHANGE when the work area changes, which every
/// AppBar reservation (the dock's, this bar's) causes.
const SPI_SETWORKAREA: usize = 0x002F;
/// A `TICK` is already posted.
static TICK_PENDING: AtomicBool = AtomicBool::new(false);
static AUDIO_TICK_PENDING: AtomicBool = AtomicBool::new(false);
static TRIGGERS_LEFT: AtomicI32 = AtomicI32::new(0);
static TRIGGERS_RIGHT: AtomicI32 = AtomicI32::new(0);
static PANEL_LEFT: AtomicI32 = AtomicI32::new(0);
static PANEL_RIGHT: AtomicI32 = AtomicI32::new(0);
static BAR_TOP: AtomicI32 = AtomicI32::new(0);
static BAR_BOTTOM: AtomicI32 = AtomicI32::new(0);
static PANEL_BOTTOM: AtomicI32 = AtomicI32::new(0);
/// Top of the popup strip when the bar sits on the bottom edge.
static PANEL_TOP: AtomicI32 = AtomicI32::new(0);
static BAR_AT_BOTTOM: AtomicBool = AtomicBool::new(false);
/// The hook exists only while the island is expanded.
static ISLAND_HOOK_WINDOW: AtomicIsize = AtomicIsize::new(0);
/// Segoe Fluent Icons code points.
pub(crate) mod glyph {
    pub const ETHERNET: char = '\u{E839}';
    pub const OFFLINE: char = '\u{F384}';
    pub const VOLUME: [char; 4] = ['\u{E992}', '\u{E993}', '\u{E994}', '\u{E995}'];
    pub const MUTE: char = '\u{E74F}';
    pub const SEARCH: char = '\u{E721}';
    pub const TASK_VIEW: char = '\u{E7C4}';
    pub const WIDGETS: char = '\u{F246}';
    /// The taskbar's "show hidden icons" arrow.
    pub const HIDDEN_ICONS: char = '\u{E70E}';
    pub const MICROPHONE: char = '\u{E720}';
    pub const CAMERA: char = '\u{E722}';
    pub const MOUSE: char = '\u{E962}';
    pub const HEADSET: char = '\u{E95B}';
    pub const BLUETOOTH: char = '\u{E702}';
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    AccessoryIndicator,
    QuickSettings,
    Notifications,
    Sound,
    Search,
    TaskView,
    Widgets,
    HiddenIcons,
    InputLanguage,
    Microphone,
    Camera,
    Bluetooth,
}

/// How the clock is written and what a click on it does.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClockSettings {
    pub options: whirlpool_core::clock::ClockOptions,
    /// A click opens the clock panel (otherwise the Windows notification centre).
    pub panel: bool,
}

/// Opens (or closes) the clock panel, a short-lived window of its own.
fn open_hub(tab: &str) {
    use std::os::windows::process::CommandExt;
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe)
            .arg("--hub")
            .arg(tab)
            .creation_flags(0x0800_0000)
            .spawn();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuBarWidgets {
    pub search: Option<bool>,
    pub task_view: Option<bool>,
    pub widgets: Option<bool>,
    pub bluetooth: bool,
    pub battery_percent: Option<bool>,
    pub island_style: IslandStyle,
    /// Size of the island and of its opened panel.
    pub island_preset: whirlpool_config::IslandPreset,
    /// Strength of the bar's tint over the glass, in percent.
    pub opacity: u32,
    pub date: bool,
    pub hour24: Option<bool>,
    pub size: whirlpool_config::BarSize,
    /// A rounded bar inset from the screen edges instead of an edge-to-edge one.
    pub floating: bool,
    pub inset: u32,
    pub gap: u32,
    pub radius: u32,
    /// Where a narrower floating bar sits.
    pub align: whirlpool_config::BarAlign,
    /// The screen edge the bar sits on.
    pub position: whirlpool_config::BarPosition,
    /// Width of a floating bar, in percent of the room it has.
    pub width: u32,
    pub tint: whirlpool_config::BarTint,
    /// A hairline along the edge of the bar.
    pub border: bool,
}

impl Default for MenuBarWidgets {
    fn default() -> Self {
        Self {
            search: None,
            task_view: None,
            widgets: None,
            bluetooth: true,
            battery_percent: None,
            island_style: IslandStyle::Notch,
            island_preset: whirlpool_config::IslandPreset::Classic,
            opacity: 50,
            date: true,
            hour24: None,
            size: whirlpool_config::BarSize::Regular,
            floating: false,
            inset: 24,
            gap: 8,
            radius: 14,
            align: whirlpool_config::BarAlign::Center,
            position: whirlpool_config::BarPosition::Top,
            width: 100,
            tint: whirlpool_config::BarTint::Ocean,
            border: true,
        }
    }
}

/// Network, volume and taskbar buttons, read only when Windows reports a change.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Status {
    network: Network,
    volume: Option<Volume>,
    buttons: TaskbarButtons,
    privacy: Privacy,
    battery_percentage: bool,
}

/// What the bar was last drawn for.
#[derive(Clone, PartialEq)]
struct Look {
    accessories: [Option<AccessoryReading>; 2],
    language: Language,
    screen: RECT,
    scale: f32,
    light: bool,
    /// See `system::transparency_enabled`.
    glass: bool,
    app_name: String,
    clock: String,
    /// Family and weight the clock is drawn in.
    clock_font: (Option<&'static str>, i32),
    /// The input language indicator, when more than one is installed.
    input: Option<String>,
    battery: Option<Battery>,
    status: Status,
    widgets: MenuBarWidgets,
    usage: Snapshot,
    usage_enabled: bool,
    active_providers: ActiveProviders,
    panel: Option<Island>,
    usage_refreshing: bool,
    track: Option<Track>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AccessoryReading {
    kind: AccessoryKind,
    percent: Option<u8>,
}

// Compare only visible readings: unrelated names/devices do not redraw the bar.
fn accessory_readings(snapshot: &AccessorySnapshot) -> [Option<AccessoryReading>; 2] {
    snapshot.compact().map(|device| {
        device.map(|device| AccessoryReading {
            kind: device.kind,
            percent: device.percent,
        })
    })
}

impl Look {
    fn panel_content_height(&self) -> f32 {
        match self.panel {
            Some(Island::Music) => 184.0,
            Some(Island::Usage) => {
                74.0 + Provider::ALL
                    .into_iter()
                    .filter(|provider| self.active_providers.has(*provider))
                    .map(|provider| {
                        usage_row_height(
                            self.usage.get(provider),
                            island_metrics(&self.widgets).row_height,
                        )
                    })
                    .sum::<f32>()
            }
            None => 0.0,
        }
    }

    fn panel_height(&self) -> f32 {
        let natural = self.panel_content_height();
        if self.panel == Some(Island::Usage) {
            let available = (self.screen.bottom - self.screen.top) as f32 / self.scale
                - bar_height()
                - PANEL_GAP
                - PANEL_SHADOW_MARGIN;
            natural.min(available.max(0.0))
        } else {
            natural
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Island {
    Music,
    Usage,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Span {
    left: f32,
    right: f32,
}

impl Span {
    fn width(self) -> f32 {
        self.right - self.left
    }
}

#[derive(Clone, Copy)]
enum IslandAction {
    Toggle(Island),
    Media(MediaControl),
    Volume(Span),
    Mute,
    Refresh,
    ScrollUsage(i8),
}

#[derive(Clone, Copy)]
struct IslandHit {
    bounds: (f32, f32, f32, f32),
    action: IslandAction,
}

/// A retained, clipped body. Wheel/arrow input only changes its transform and
/// arrow opacity; no quota queries or redraws.
struct UsageViewport {
    content: ContainerVisual,
    maximum: f32,
    scale: f32,
    bounds: (f32, f32, f32, f32),
    up: SpriteVisual,
    down: SpriteVisual,
}

impl IslandHit {
    fn at(&self, x: f32, y: f32) -> bool {
        let (left, top, right, bottom) = self.bounds;
        x >= left && x < right && y >= top && y < bottom
    }
}

struct ExitAnimation {
    batch: CompositionScopedBatch,
    completed: i64,
}

/// Only these retained visuals move during a volume gesture. Album artwork,
/// track labels and the rest of the menu bar keep their existing visual tree.
struct VolumeControls {
    fill: CompositionRoundedRectangleGeometry,
    knob: ShapeVisual,
    value: SpriteVisual,
    geometry: IslandGeometry,
    left: f32,
    width: f32,
    position_percent: Option<u8>,
    label_percent: u8,
}

impl VolumeControls {
    fn move_to(&mut self, percent: u8) -> Result<()> {
        if self.position_percent == Some(percent) {
            return Ok(());
        }
        let px = |v| self.geometry.px(v);
        let filled = self.width * f32::from(percent) / 100.0;
        self.fill.SetSize(Vector2::new(filled, px(4.0)))?;
        self.fill
            .SetCornerRadius(Vector2::new(px(2.0).min(filled / 2.0), px(2.0)))?;
        self.knob.SetOffset(Vector3::new(
            self.left + filled - px(5.0),
            px(160.0) - px(5.0),
            0.0,
        ))?;
        self.position_percent = Some(percent);
        Ok(())
    }

    fn paint(&mut self, gfx: &Gfx, percent: u8) -> Result<()> {
        self.move_to(percent)?;
        if self.label_percent != percent {
            let label = gfx.text_brush_sized(
                &format!("{percent}%"),
                self.geometry.scale,
                10.5,
                false,
                self.geometry.ink(),
            )?;
            self.value
                .SetBrush(&label.brush.cast::<CompositionBrush>()?)?;
            self.value
                .SetSize(Vector2::new(label.width, label.height))?;
            self.label_percent = percent;
        }
        Ok(())
    }
}

impl Drop for ExitAnimation {
    fn drop(&mut self) {
        let _ = self.batch.RemoveCompleted(self.completed);
    }
}

pub struct MenuBar {
    accessories: Option<accessories::Monitor>,
    accessories_enabled: bool,
    accessory_retry_at: Instant,
    hwnd: HWND,
    _target: DesktopWindowTarget,
    /// Stable parent: content refreshes must not restart reveal animations.
    surface: ContainerVisual,
    appbar: Option<AppBar>,
    app_name: String,
    look: Option<Look>,
    /// Clickable spans, in window pixels.
    zones: Vec<(f32, f32, Action)>,
    visible: bool,
    auto_hide: bool,
    /// With auto-hide on: the pointer is at the top edge, so the bar shows.
    revealed: bool,
    clock: ClockSettings,
    hide_until: Option<Instant>,
    animation_epoch: usize,
    exit_animation: Option<ExitAnimation>,
    /// Where the full bar goes on screen.
    bounds: RECT,
    status: Status,
    widgets: MenuBarWidgets,
    usage: Option<UsageMonitor>,
    /// Where each provider sits in the closed island, as screen x.
    usage_anchors: Vec<(Provider, i32)>,
    /// A moment for the crab, waiting for the dock (see `take_cue`).
    cue: Option<(mascot::Act, i32)>,
    media: Option<media::Watch>,
    /// "Show what is playing": off, nothing listens to media sessions.
    media_enabled: bool,
    cover: Option<(
        Option<Arc<whirlpool_shell::icons::IconImage>>,
        CompositionSurfaceBrush,
    )>,
    volume_drag: Option<Span>,
    pending_volume: Option<u8>,
    volume_target: Option<u8>,
    volume_controls: Option<VolumeControls>,
    usage_scroll: f32,
    usage_viewport: Option<UsageViewport>,
    suppress_volume_release: bool,
    panel: Option<Island>,
    visual_height: i32,
    /// Bottom edge only: where the bar and the popup sit in the window, so
    /// pointer positions can be mapped back to the layout (see `layout_y`).
    bar_dy: f32,
    popup_dy: f32,
    triggers_span: Span,
    panel_span: Span,
    island_hits: Vec<IslandHit>,
    outside_click_hook: Option<HHOOK>,
    /// Held for their callbacks; dropping them unsubscribes.
    _network: Option<NetworkWatch>,
    /// Hears when apps start or stop using the microphone or camera.
    privacy_watch: Option<RegistryWatch>,
    /// Windows' taskbar preferences, including the battery percentage.
    settings_watch: Option<RegistryWatch>,
    /// Energy saver on or off arrives as `WM_POWERBROADCAST`; glass follows.
    _saver_watch: whirlpool_shell::power::SaverWatch,
    audio: Option<AudioWatch>,
}

unsafe extern "system" fn bar_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            launcher::note_menu_bar_press();
            app::menu_bar_volume_press((l.0 & 0xFFFF) as i16 as f32, (l.0 >> 16) as i16 as f32);
        }
        WM_LBUTTONUP => {
            let x = (l.0 & 0xFFFF) as i16;
            let y = (l.0 >> 16) as i16;
            if !app::menu_bar_volume_release(Some(f32::from(x))) {
                app::menu_bar_clicked(f32::from(x), f32::from(y));
            }
        }
        WM_MOUSEMOVE => {
            let x = (l.0 & 0xFFFF) as i16;
            let y = (l.0 >> 16) as i16;
            app::menu_bar_pointer_moved(f32::from(x), f32::from(y));
        }
        WM_MOUSELEAVE => app::menu_bar_check_pointer(),
        WM_MOUSEWHEEL => {
            let delta = (w.0 >> 16) as u16 as i16;
            let point = POINT {
                x: (l.0 & 0xFFFF) as i16 as i32,
                y: (l.0 >> 16) as i16 as i32,
            };
            if app::menu_bar_scrolled(point, delta) {
                return LRESULT(0);
            }
            return unsafe { DefWindowProcW(hwnd, msg, w, l) };
        }
        WM_TIMER if w.0 == POINTER_TIMER => app::menu_bar_check_pointer(),
        WM_TIMER if w.0 == VOLUME_TIMER => app::menu_bar_volume_flush(),
        WM_CAPTURECHANGED => {
            // Avoid reentering the app borrow inside Set/ReleaseCapture.
            unsafe {
                let _ = PostMessageW(Some(hwnd), VOLUME_CAPTURE_LOST, WPARAM(0), LPARAM(0));
            }
        }
        VOLUME_CAPTURE_LOST => {
            // A delayed notification must not cancel a newer drag.
            if unsafe { GetCapture() } != hwnd {
                app::menu_bar_volume_release(None);
            }
        }
        WM_TIMER if w.0 == HIDE_ANIMATION_TIMER => app::menu_bar_finish_hide(None),
        HIDE_ANIMATION_DONE => app::menu_bar_finish_hide(Some(w.0)),
        DISMISS_ISLAND => app::menu_bar_dismiss_island(),
        REGISTRY_CHANGED => app::menu_bar_registry_changed(),
        ACCESSORIES_CHANGED => app::menu_bar_accessories_changed(),
        MEDIA_CHANGED => app::menu_bar_media_changed(),
        AUDIO_CHANGED => {
            if !AUDIO_TICK_PENDING.swap(true, Ordering::Relaxed) {
                unsafe {
                    let _ = PostMessageW(Some(hwnd), AUDIO_TICK, WPARAM(0), LPARAM(0));
                }
            }
        }
        AUDIO_TICK => {
            AUDIO_TICK_PENDING.store(false, Ordering::Relaxed);
            app::menu_bar_audio_changed();
        }
        WM_SETCURSOR => {
            // SAFETY: Keep the standard arrow throughout the bar and popups.
            if let Ok(arrow) = unsafe { LoadCursorW(None, IDC_ARROW) } {
                unsafe { SetCursor(Some(arrow)) };
                return LRESULT(1);
            }
            return unsafe { DefWindowProcW(hwnd, msg, w, l) };
        }
        WM_NCHITTEST => {
            let x = (l.0 & 0xFFFF) as i16 as i32;
            let y = (l.0 >> 16) as i16 as i32;
            let outside_panel_x =
                x < PANEL_LEFT.load(Ordering::Relaxed) || x >= PANEL_RIGHT.load(Ordering::Relaxed);
            let transparent = if BAR_AT_BOTTOM.load(Ordering::Relaxed) {
                y < BAR_TOP.load(Ordering::Relaxed)
                    && (y < PANEL_TOP.load(Ordering::Relaxed) || outside_panel_x)
            } else {
                y >= BAR_BOTTOM.load(Ordering::Relaxed)
                    && (y >= PANEL_BOTTOM.load(Ordering::Relaxed) || outside_panel_x)
            };
            if transparent {
                return LRESULT(-1);
            }
            // SAFETY: Default hit testing for the bar and the island.
            return unsafe { DefWindowProcW(hwnd, msg, w, l) };
        }
        // The minute changed, the charger was plugged or unplugged, the
        // network or volume changed, or a Windows setting (taskbar buttons).
        // Nothing the bar shows depends on the work area.
        WM_SETTINGCHANGE if w.0 == SPI_SETWORKAREA => {}
        // Some of these are sent while the dock is busy reserving space, and
        // updating then was dropped. From the message loop the dock is never
        // busy, and a burst of them makes a single update.
        // WM_TIMECHANGE: the clock or time zone was changed.
        WM_TIMER | WM_POWERBROADCAST | WM_TIMECHANGE | STATUS_CHANGED | USAGE_CHANGED
        | WM_SETTINGCHANGE => {
            if msg == WM_POWERBROADCAST && matches!(w.0, 7 | 18) {
                app::menu_bar_accessories_reset();
            }
            if msg == WM_POWERBROADCAST && w.0 == PBT_POWERSETTINGCHANGE as usize {
                // SAFETY: Windows owns the setting payload for this message.
                unsafe {
                    whirlpool_shell::power::power_setting_changed(l);
                }
            }
            if !TICK_PENDING.swap(true, Ordering::Relaxed) {
                // SAFETY: Posting to our own live window.
                unsafe {
                    let _ = PostMessageW(Some(hwnd), TICK, WPARAM(0), LPARAM(0));
                }
            }
        }
        TICK => {
            TICK_PENDING.store(false, Ordering::Relaxed);
            app::menu_bar_tick();
        }
        // SAFETY: Standard forwarding of unhandled messages.
        _ => return unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
    LRESULT(0)
}

unsafe extern "system" fn outside_click_proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0
        && matches!(
            w.0 as u32,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
        )
    {
        // SAFETY: Windows provides an MSLLHOOKSTRUCT for mouse hook actions.
        let point = unsafe { (*(l.0 as *const MSLLHOOKSTRUCT)).pt };
        let in_triggers = point.y >= BAR_TOP.load(Ordering::Relaxed)
            && point.y < BAR_BOTTOM.load(Ordering::Relaxed)
            && point.x >= TRIGGERS_LEFT.load(Ordering::Relaxed)
            && point.x < TRIGGERS_RIGHT.load(Ordering::Relaxed);
        let in_panel_rows = if BAR_AT_BOTTOM.load(Ordering::Relaxed) {
            point.y >= PANEL_TOP.load(Ordering::Relaxed)
                && point.y < BAR_TOP.load(Ordering::Relaxed)
        } else {
            point.y >= BAR_BOTTOM.load(Ordering::Relaxed)
                && point.y < PANEL_BOTTOM.load(Ordering::Relaxed)
        };
        let in_panel = in_panel_rows
            && point.x >= PANEL_LEFT.load(Ordering::Relaxed)
            && point.x < PANEL_RIGHT.load(Ordering::Relaxed);
        if !in_triggers && !in_panel {
            let window = ISLAND_HOOK_WINDOW.load(Ordering::Relaxed);
            if window != 0 {
                // SAFETY: The window remains live until the hook is removed in Drop.
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(window as *mut _)),
                        DISMISS_ISLAND,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }
            }
        }
    }
    // SAFETY: Never consume the click; the target application must receive it.
    unsafe { CallNextHookEx(None, code, w, l) }
}

impl MenuBar {
    pub fn create(
        gfx: &Gfx,
        usage_enabled: bool,
        media_enabled: bool,
        auto_hide: bool,
        widgets: MenuBarWidgets,
    ) -> Result<Self> {
        let class = w!("WhirlpoolMenuBar");
        // SAFETY: Standard class registration and window creation on the UI
        // thread; the window is destroyed in `Drop`.
        let hwnd = unsafe {
            let instance = GetModuleHandleW(None)?;
            // Registering an existing class fails harmlessly.
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(bar_proc),
                hInstance: instance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                lpszClassName: class,
                ..Default::default()
            });
            let hwnd = CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class,
                w!("Whirlpool"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance.into()),
                None,
            )?;
            // Without this, the system backdrop brush appears black.
            let enable = BOOL::from(true);
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_HOSTBACKDROPBRUSH,
                (&raw const enable).cast(),
                size_of::<BOOL>() as u32,
            );
            hwnd
        };
        let target = unsafe {
            // SAFETY: `hwnd` is the live window created above.
            gfx.compositor
                .cast::<ICompositorDesktopInterop>()?
                .CreateDesktopWindowTarget(hwnd, true)?
        };
        // Our compositor owns the transition; do not combine it with DWM's
        // native show/hide effect, which can present a cached opaque frame.
        system::set_window_transitions(hwnd.0 as isize, false);
        let surface = gfx.compositor.CreateContainerVisual()?;
        target.SetRoot(&surface)?;
        let mut bar = Self {
            accessories: None,
            accessories_enabled: false,
            accessory_retry_at: Instant::now() + Duration::from_secs(60),
            hwnd,
            _target: target,
            surface,
            appbar: AppBar::register(hwnd, APPBAR_CALLBACK),
            app_name: String::new(),
            look: None,
            zones: Vec::new(),
            visible: true,
            auto_hide,
            revealed: false,
            clock: ClockSettings::default(),
            hide_until: None,
            animation_epoch: 0,
            exit_animation: None,
            bounds: RECT::default(),
            status: Status {
                network: Network::Offline,
                volume: None,
                buttons: TaskbarButtons::default(),
                privacy: Privacy::default(),
                battery_percentage: false,
            },
            widgets,
            usage: usage_enabled.then(|| UsageMonitor::new(hwnd.0 as isize, USAGE_CHANGED)),
            usage_anchors: Vec::new(),
            cue: None,
            media: media_enabled
                .then(|| media::Watch::start(hwnd, MEDIA_CHANGED))
                .flatten(),
            media_enabled,
            cover: None,
            volume_drag: None,
            pending_volume: None,
            volume_target: None,
            volume_controls: None,
            usage_scroll: 0.0,
            usage_viewport: None,
            suppress_volume_release: false,
            panel: None,
            visual_height: 0,
            bar_dy: 0.0,
            popup_dy: 0.0,
            triggers_span: Span::default(),
            panel_span: Span::default(),
            island_hits: Vec::new(),
            outside_click_hook: None,
            _network: NetworkWatch::new(hwnd.0 as isize, STATUS_CHANGED),
            privacy_watch: status::watch_privacy(hwnd.0 as isize, REGISTRY_CHANGED),
            settings_watch: status::watch_taskbar_settings(hwnd.0 as isize, REGISTRY_CHANGED),
            _saver_watch: whirlpool_shell::power::SaverWatch::new(hwnd.0 as isize),
            audio: AudioWatch::new(hwnd.0 as isize, AUDIO_CHANGED),
        };
        AUDIO_TICK_PENDING.store(false, Ordering::Relaxed);
        bar.update_status();
        bar.refresh(gfx)?;
        bar.schedule_next_minute();
        Ok(bar)
    }

    pub fn set_app_name(&mut self, gfx: &Gfx, name: &str) -> Result<()> {
        if self.app_name == name {
            return Ok(());
        }
        name.clone_into(&mut self.app_name);
        self.refresh(gfx)
    }

    /// Hidden while an app is full screen.
    /// How tall the bar is at `scale`: the strip it keeps at the top of the
    /// primary display unless it hides itself. A side dock starts below it.
    fn at_bottom(&self) -> bool {
        self.widgets.position == whirlpool_config::BarPosition::Bottom
    }

    /// Window-local pointer y to the layout the bar is drawn in (bar at 0,
    /// popup below it). Identity on the top edge.
    fn layout_y(&self, y: f32) -> f32 {
        if !self.at_bottom() {
            y
        } else if y >= self.bar_dy {
            y - self.bar_dy
        } else {
            y - self.popup_dy
        }
    }

    /// Screen y of the window's top edge.
    fn window_top(&self) -> i32 {
        if self.at_bottom() {
            self.bounds.bottom - self.visual_height
        } else {
            self.bounds.top
        }
    }

    /// Screen y that flyouts opened from the bar hang from.
    fn anchor_y(&self) -> i32 {
        if self.at_bottom() {
            self.bounds.top
        } else {
            self.bounds.bottom
        }
    }

    /// Is this screen row inside the popup's strip (next to the bar)?
    fn in_panel_rows(&self, y: i32) -> bool {
        if self.at_bottom() {
            y >= PANEL_TOP.load(Ordering::Relaxed) && y < self.bounds.top
        } else {
            y >= self.bounds.bottom && y < PANEL_BOTTOM.load(Ordering::Relaxed)
        }
    }

    pub fn strip_height(scale: f32) -> i32 {
        ((bar_height() + BAR_GAP.load(Ordering::Relaxed) as f32) * scale).round() as i32
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        if !visible {
            self.release_volume(None)?;
        }
        if self.visible == visible {
            return Ok(());
        }
        self.visible = visible;
        self.schedule_next_minute();
        if !visible && self.auto_hide {
            self.revealed = false;
        }
        self.place()
    }

    pub fn set_auto_hide(&mut self, auto_hide: bool) -> Result<()> {
        if self.auto_hide == auto_hide {
            return Ok(());
        }
        self.auto_hide = auto_hide;
        self.revealed = false;
        self.hide_until = None;
        self.place()
    }

    pub fn set_usage_enabled(&mut self, gfx: &Gfx, enabled: bool) -> Result<()> {
        if self.usage.is_some() == enabled {
            return Ok(());
        }
        self.usage = enabled.then(|| UsageMonitor::new(self.hwnd.0 as isize, USAGE_CHANGED));
        self.refresh(gfx)
    }

    /// Shows or hides what is playing. Off, the media watch stops too, so
    /// nothing follows playback changes.
    pub fn set_media_enabled(&mut self, gfx: &Gfx, enabled: bool) -> Result<()> {
        if self.media_enabled == enabled {
            return Ok(());
        }
        self.media_enabled = enabled;
        self.media = enabled
            .then(|| media::Watch::start(self.hwnd, MEDIA_CHANGED))
            .flatten();
        self.refresh(gfx)
    }

    pub fn set_accessories_enabled(&mut self, gfx: &Gfx, enabled: bool) -> Result<()> {
        if self.accessories_enabled == enabled {
            return Ok(());
        }
        self.accessories_enabled = enabled;
        if enabled {
            self.restart_accessories(gfx)
        } else {
            self.accessories = None;
            self.refresh(gfx)
        }
    }

    pub fn accessories_changed(&mut self, gfx: &Gfx) -> Result<()> {
        if self.accessories_enabled && self.visible && (!self.auto_hide || self.revealed) {
            self.refresh(gfx)?;
        }
        Ok(())
    }

    pub fn restart_accessories(&mut self, gfx: &Gfx) -> Result<()> {
        // Resume/audio notifications and the minute tick must never start a
        // disabled reader, including after a previously enabled one failed.
        if !self.accessories_enabled {
            return Ok(());
        }
        self.accessory_retry_at = Instant::now() + Duration::from_secs(60);
        self.accessories = None;
        self.accessories = accessories::Monitor::new(
            self.hwnd.0 as isize,
            ACCESSORIES_CHANGED,
            self.visible && (!self.auto_hide || self.revealed),
        )
        .inspect_err(|error| eprintln!("whirlpool: accesorios: {error:#}"))
        .ok();
        self.refresh(gfx)
    }

    pub fn set_widgets(&mut self, gfx: &Gfx, widgets: MenuBarWidgets) -> Result<()> {
        if self.widgets == widgets {
            return Ok(());
        }
        self.widgets = widgets;
        self.refresh(gfx)
    }

    /// The pointer reached the top edge or left the bar.
    pub fn hover(&mut self, gfx: &Gfx, inside: bool) -> Result<()> {
        if self.volume_drag.is_some() && !inside {
            return Ok(());
        }
        if !inside && self.panel.take().is_some() {
            self.refresh(gfx)?;
        }
        if !self.auto_hide || self.revealed == inside {
            return Ok(());
        }
        let was_hiding = self.hide_until.is_some();
        let animated = self.visible && system::animations_enabled();
        self.animation_epoch = self.animation_epoch.wrapping_add(1);
        self.exit_animation = None;
        self.revealed = inside;
        // Seconds tick only while the bar is on screen.
        self.schedule_next_minute();
        self.hide_until = (!inside && animated)
            .then(|| Instant::now() + Duration::from_millis(u64::from(HIDE_FALLBACK_MS)));
        if !animated {
            self.reset_surface()?;
        } else if inside && !was_hiding {
            self.surface
                .SetOffset(Vector3::new(0.0, -self.reveal_lift(), 0.0))?;
            self.surface.SetOpacity(0.0)?;
        }
        self.place()?;
        if animated && let Err(error) = self.animate_visibility(gfx, inside) {
            whirlpool_shell::log::line(&format!("animación de barra superior: {error:#}"));
            self.hide_until = None;
            self.reset_surface()?;
            self.place()?;
        }
        if inside {
            let mut track = TRACKMOUSEEVENT {
                cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: self.hwnd,
                dwHoverTime: 0,
            };
            // SAFETY: `track` is a valid structure with its size initialized.
            unsafe { TrackMouseEvent(&mut track)? };
        }
        self.refresh(gfx)
    }

    fn reveal_lift(&self) -> f32 {
        let lift = (self.bounds.bottom - self.bounds.top) as f32 * 0.3;
        // The bar slides away toward the edge it sits on.
        if self.at_bottom() { -lift } else { lift }
    }

    fn animate_visibility(&mut self, gfx: &Gfx, shown: bool) -> Result<()> {
        let c = &gfx.compositor;
        let ease = if shown {
            c.CreateCubicBezierEasingFunction(Vector2::new(0.2, 0.8), Vector2::new(0.2, 1.0))?
        } else {
            c.CreateCubicBezierEasingFunction(Vector2::new(0.4, 0.0), Vector2::new(1.0, 1.0))?
        };
        let duration = TimeSpan {
            Duration: i64::from(if shown { REVEAL_MS } else { HIDE_MS }) * 10_000,
        };
        let slide = c.CreateScalarKeyFrameAnimation()?;
        slide.SetDuration(duration)?;
        slide.InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))?;
        slide.InsertKeyFrameWithEasingFunction(
            1.0,
            if shown { 0.0 } else { -self.reveal_lift() },
            &ease,
        )?;
        let fade = c.CreateScalarKeyFrameAnimation()?;
        fade.SetDuration(duration)?;
        fade.InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))?;
        fade.InsertKeyFrameWithEasingFunction(1.0, if shown { 1.0 } else { 0.0 }, &ease)?;
        let batch = (!shown)
            .then(|| c.CreateScopedBatch(CompositionBatchTypes::Animation))
            .transpose()?;
        let started = (|| -> Result<()> {
            self.surface.StartAnimation(h!("Offset.Y"), &slide)?;
            self.surface.StartAnimation(h!("Opacity"), &fade)?;
            Ok(())
        })();
        if let Some(batch) = batch {
            let window = self.hwnd.0 as isize;
            let epoch = self.animation_epoch;
            let completed = batch.Completed(&TypedEventHandler::new(move |_, _| {
                // SAFETY: Only scalar state is posted back to the UI thread.
                // A cancelled transition's already queued reply is ignored.
                unsafe {
                    let _ = PostMessageW(
                        Some(HWND(window as *mut _)),
                        HIDE_ANIMATION_DONE,
                        WPARAM(epoch),
                        LPARAM(0),
                    );
                }
                Ok(())
            }));
            // Register before ending the scope. End it even on failure so
            // unrelated later animations cannot become part of this exit.
            let ended = batch.End();
            started?;
            ended?;
            let completed = completed?;
            self.exit_animation = Some(ExitAnimation { batch, completed });
        } else {
            started?;
        }
        Ok(())
    }

    fn park_surface(&self) -> Result<()> {
        self.surface.StopAnimation(h!("Offset.Y"))?;
        self.surface.StopAnimation(h!("Opacity"))?;
        self.surface
            .SetOffset(Vector3::new(0.0, -self.reveal_lift(), 0.0))?;
        self.surface.SetOpacity(0.0)?;
        Ok(())
    }

    fn reset_surface(&self) -> Result<()> {
        self.surface.StopAnimation(h!("Offset.Y"))?;
        self.surface.StopAnimation(h!("Opacity"))?;
        self.surface.SetOffset(Vector3::new(0.0, 0.0, 0.0))?;
        self.surface.SetOpacity(1.0)?;
        Ok(())
    }

    pub fn finish_hide(&mut self, gfx: &Gfx, completed_epoch: Option<usize>) -> Result<()> {
        let mut point = POINT::default();
        // SAFETY: Read-only cursor query before completing an exit transition.
        unsafe { GetCursorPos(&mut point)? };
        self.finish_hide_at(gfx, Instant::now(), point, completed_epoch)
    }

    fn finish_hide_at(
        &mut self,
        gfx: &Gfx,
        now: Instant,
        point: POINT,
        completed_epoch: Option<usize>,
    ) -> Result<()> {
        // A queued timer from a cancelled transition cannot hide a reopened bar.
        if completed_epoch.is_some_and(|epoch| epoch != self.animation_epoch) {
            return Ok(());
        }
        let Some(deadline) = self.hide_until else {
            return Ok(());
        };
        self.pointer_left_at(gfx, point)?;
        if self.hide_until.is_none() {
            return Ok(());
        }
        if completed_epoch.is_some() || now >= deadline {
            self.hide_until = None;
            self.exit_animation = None;
        }
        self.place()
    }

    /// A leave notification can already be queued when the hidden window is
    /// shown. Check the pointer rather than hiding a newly revealed bar on it.
    pub fn check_pointer(&mut self, gfx: &Gfx) -> Result<()> {
        if self.panel.is_some() {
            // SAFETY: Read-only Escape state; checked only while a popup is open.
            if unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x1B) } < 0 {
                return self.dismiss_island(gfx);
            }
        }
        let mut point = POINT::default();
        // SAFETY: Read-only cursor query in the same DPI context as `bounds`.
        unsafe { GetCursorPos(&mut point)? };
        self.pointer_left_at(gfx, point)
    }

    fn pointer_left_at(&mut self, gfx: &Gfx, point: POINT) -> Result<()> {
        let x = (point.x - self.bounds.left) as f32;
        if let Some(span) = self.volume_drag {
            self.queue_volume(span, x);
            // A nonactivating background window may not receive mouse-up
            // outside its visible bounds. Reuse the existing popup pointer
            // check to finish the gesture without ever stealing app focus.
            let pressed = unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SWAPBUTTON};
                let primary = if GetSystemMetrics(SM_SWAPBUTTON) == 0 {
                    0x01
                } else {
                    0x02
                };
                windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(primary) < 0
            };
            if pressed {
                return self.paint_volume_preview(gfx);
            }
            self.release_volume(Some(x))?;
        }
        let within_bar = point.x >= self.bounds.left
            && point.x < self.bounds.right
            && point.y >= self.bounds.top
            && point.y < self.bounds.bottom;
        // Include the gap between the island and its popup so crossing it
        // does not close the popup before the pointer reaches its controls.
        let within_panel = self.panel.is_some()
            && x >= self.panel_span.left
            && x < self.panel_span.right
            && self.in_panel_rows(point.y);
        if (within_bar || within_panel) && self.hide_until.is_some() {
            self.hover(gfx, true)?;
        } else if !within_bar && !within_panel {
            self.hover(gfx, false)?;
        }
        Ok(())
    }

    /// Leaving the island sideways still stays inside the full-width bar window.
    pub fn pointer_moved(&mut self, gfx: &Gfx, x: f32, y: f32) -> Result<()> {
        let y = self.layout_y(y);
        if let Some(span) = self.volume_drag {
            self.queue_volume(span, x);
            return self.paint_volume_preview(gfx);
        }
        let span = if y < (self.bounds.bottom - self.bounds.top) as f32 {
            self.triggers_span
        } else {
            self.panel_span
        };
        if self.panel.is_some() && (x < span.left || x >= span.right) {
            self.dismiss_island(gfx)?;
        }
        self.hover(gfx, true)
    }

    pub fn dismiss_island(&mut self, gfx: &Gfx) -> Result<()> {
        self.release_volume(None)?;
        if self.panel.take().is_some() {
            self.refresh(gfx)?;
        }
        Ok(())
    }

    /// Positions the window and reserves (or frees) its strip of the screen.
    fn place(&mut self) -> Result<()> {
        let bounds = self.bounds;
        let bar_height = bounds.bottom - bounds.top;
        if bar_height <= 0 {
            return Ok(());
        }
        let hidden = self.auto_hide && !self.revealed;
        if !self.visible || !self.auto_hide {
            self.hide_until = None;
            self.exit_animation = None;
        }
        let on_screen = self.visible && (!hidden || self.hide_until.is_some());
        if let Some(monitor) = &self.accessories {
            monitor.set_visible(on_screen && !hidden);
        }
        if !on_screen {
            // Keep the hidden tree transparent. Restoring opacity here races
            // HWND hiding and flashes its last opaque frame on the next show.
            self.park_surface()?;
        } else if !self.auto_hide {
            self.reset_surface()?;
        }
        if let Some(appbar) = &mut self.appbar {
            // An auto-hiding bar leaves the whole screen to the windows.
            let (inset, gap) = self
                .look
                .as_ref()
                .map_or((0, 0), |look| float_px(&look.widgets, look.scale));
            let strip = if self.auto_hide { 0 } else { bar_height + gap };
            let monitor = RECT {
                left: bounds.left - inset,
                right: bounds.right + inset,
                top: bounds.top - gap,
                bottom: bounds.bottom,
            };
            if self.widgets.position == whirlpool_config::BarPosition::Bottom {
                appbar.reserve(
                    Edge::Bottom,
                    RECT {
                        top: bounds.top,
                        bottom: bounds.bottom + gap + dock_lift(),
                        ..monitor
                    },
                    strip,
                );
            } else {
                appbar.reserve(Edge::Top, monitor, strip);
            }
        }
        // Hidden, the bar keeps no window on screen, not even a thin strip:
        // a strip along the top edge took the clicks meant for the close
        // button of maximized windows and for browser tabs. The edge watcher
        // watches the pointer instead.
        let show = if on_screen {
            SWP_SHOWWINDOW
        } else {
            SWP_HIDEWINDOW
        };
        let window_top = self.window_top();
        // SAFETY: Our own live window.
        unsafe {
            SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                bounds.left,
                window_top,
                bounds.right - bounds.left,
                self.visual_height,
                SWP_NOACTIVATE | show,
            )?;
            if on_screen && (self.auto_hide || self.panel.is_some()) {
                // TrackMouseEvent may report a leave immediately after showing
                // a window. Check real geometry while revealed, including if
                // no further WM_MOUSEMOVE reaches this window.
                SetTimer(Some(self.hwnd), POINTER_TIMER, POINTER_CHECK_MS, None);
            } else {
                let _ = KillTimer(Some(self.hwnd), POINTER_TIMER);
            }
            let _ = KillTimer(Some(self.hwnd), HIDE_ANIMATION_TIMER);
            if let Some(deadline) = self.hide_until {
                let wait = deadline.saturating_duration_since(Instant::now());
                SetTimer(
                    Some(self.hwnd),
                    HIDE_ANIMATION_TIMER,
                    (wait.as_millis() as u32).max(1),
                    None,
                );
            }
        }
        edge_watch::set_menu_bar_bottom(self.at_bottom());
        let reach = self
            .look
            .as_ref()
            .map_or(0, |look| float_px(&look.widgets, look.scale).1);
        edge_watch::set_menu_bar_reach(reach);
        edge_watch::set_menu_bar((self.visible && hidden).then_some(bounds));
        self.sync_outside_click_hook()
    }

    /// The edge watcher has verified the pointer's dwell on this bar.
    pub fn edge_rested(&mut self, gfx: &Gfx, point: POINT) -> Result<()> {
        if self.visible && self.auto_hide && !self.revealed {
            if whirlpool_shell::tracker::caption_button_at(point, self.bounds) {
                edge_watch::retry_on_move(edge_watch::Target::MenuBar);
            } else {
                self.hover(gfx, true)?;
            }
        }
        Ok(())
    }

    fn sync_outside_click_hook(&mut self) -> Result<()> {
        let needed =
            self.panel_span.width() > 0.0 && self.visible && (!self.auto_hide || self.revealed);
        if needed && self.outside_click_hook.is_none() {
            // SAFETY: This process owns the hook procedure and installs the hook
            // only while the panel is visible.
            let module = unsafe { GetModuleHandleW(None)? };
            let hook = unsafe {
                SetWindowsHookExW(
                    WH_MOUSE_LL,
                    Some(outside_click_proc),
                    Some(module.into()),
                    0,
                )?
            };
            ISLAND_HOOK_WINDOW.store(self.hwnd.0 as isize, Ordering::Relaxed);
            self.outside_click_hook = Some(hook);
        } else if !needed && let Some(hook) = self.outside_click_hook {
            // SAFETY: This hook was installed by this MenuBar.
            unsafe { UnhookWindowsHookEx(hook)? };
            self.outside_click_hook = None;
            ISLAND_HOOK_WINDOW.store(0, Ordering::Relaxed);
        }
        Ok(())
    }

    /// The minute changed, or something the bar shows did: redraw what differs.
    /// Privacy activity or the user's taskbar preferences changed.
    pub fn registry_changed(&mut self, gfx: &Gfx) -> Result<()> {
        if let Some(watch) = &mut self.privacy_watch {
            watch.arm();
        }
        if let Some(watch) = &mut self.settings_watch {
            watch.arm();
        }
        self.tick(gfx)
    }

    pub fn tick(&mut self, gfx: &Gfx) -> Result<()> {
        self.update_status();
        let audio_changed = self
            .audio
            .as_ref()
            .is_some_and(AudioWatch::take_default_changed);
        let retry_due = Instant::now() >= self.accessory_retry_at
            && self.accessories.as_ref().is_none_or(|monitor| {
                monitor.snapshot().state == whirlpool_shell::accessories::State::Unavailable
            });
        if self.accessories_enabled && (audio_changed || retry_due) {
            self.restart_accessories(gfx)?;
        }
        self.schedule_next_minute();
        self.refresh(gfx)
    }

    /// The reading the volume HUD shows: none while the bar's own volume
    /// popup is open (the popup is the feedback then).
    pub fn hud_volume(&self) -> Option<Volume> {
        if self.volume_controls.is_some() {
            return None;
        }
        self.status.volume
    }

    /// Ordinary audio notifications do not requery network, registry, providers
    /// or clock. A default-device change still restarts enabled accessories.
    pub fn audio_changed(&mut self, gfx: &Gfx) -> Result<()> {
        let volume = self.audio.as_mut().and_then(AudioWatch::volume);
        if self
            .audio
            .as_ref()
            .is_some_and(AudioWatch::take_default_changed)
            && self.accessories_enabled
        {
            self.status.volume = volume;
            self.restart_accessories(gfx)?;
        }
        self.apply_volume_reading(gfx, volume)
    }

    fn apply_volume_reading(&mut self, gfx: &Gfx, volume: Option<Volume>) -> Result<()> {
        self.status.volume = volume;
        if let Some(look) = &self.look
            && same_volume_indicator(look.status.volume, volume)
        {
            if let Some(volume) = volume
                && let Some(controls) = &mut self.volume_controls
            {
                controls.paint(gfx, self.volume_target.unwrap_or(volume.percent))?;
            }
            if let Some(look) = &mut self.look {
                look.status.volume = volume;
            }
            return Ok(());
        }
        // Availability, mute and speaker-glyph changes affect layout, but do
        // not justify a new provider/process or system-status scan either.
        let Some(mut look) = self.look.clone() else {
            return self.refresh(gfx);
        };
        look.status.volume = volume;
        self.apply_look(gfx, look)
    }

    fn update_status(&mut self) {
        self.status = Status {
            network: status::network(),
            volume: self.audio.as_mut().and_then(AudioWatch::volume),
            buttons: status::taskbar_buttons(),
            privacy: status::privacy(),
            battery_percentage: status::battery_percentage_enabled(),
        };
    }

    /// One wake-up per minute, aligned to the minute, instead of polling. With seconds turned on
    /// it is one per second, but only while the bar is on screen: hidden, covered by a full-screen
    /// program or auto-hidden away, it drops back to one per minute.
    fn schedule_next_minute(&self) {
        let shown = self.visible && (!self.auto_hide || self.revealed);
        let seconds = whirlpool_core::clock::should_tick_seconds(
            self.clock.options.seconds,
            shown,
            false,
            false,
        );
        // SAFETY: Read-only query, then a timer on our own window (setting an
        // existing timer id replaces it).
        unsafe {
            let now: SYSTEMTIME = GetLocalTime();
            let wait = whirlpool_core::clock::next_tick_ms(
                u32::from(now.wSecond),
                u32::from(now.wMilliseconds),
                seconds,
            );
            SetTimer(Some(self.hwnd), CLOCK_TIMER, wait, None);
        }
    }

    /// Applies the clock's look. Redraws when it changed.
    pub fn set_clock(&mut self, gfx: &Gfx, clock: ClockSettings) -> Result<()> {
        if self.clock == clock {
            return Ok(());
        }
        self.clock = clock;
        self.schedule_next_minute();
        self.refresh(gfx)
    }

    pub fn click(&mut self, gfx: &Gfx, x: f32, y: f32) -> Result<()> {
        let y = self.layout_y(y);
        if let Some(hit) = self.island_hits.iter().find(|hit| hit.at(x, y)) {
            match hit.action {
                IslandAction::Toggle(island) => {
                    self.panel = toggle_panel(self.panel, island);
                    if self.panel.is_some() {
                        let mut track = TRACKMOUSEEVENT {
                            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: self.hwnd,
                            dwHoverTime: 0,
                        };
                        // SAFETY: `track` describes our live window.
                        unsafe { TrackMouseEvent(&mut track)? };
                    }
                }
                IslandAction::Refresh => {
                    if let Some(usage) = &mut self.usage {
                        usage.refresh(true);
                    }
                }
                IslandAction::Media(control) => {
                    if let Some(media) = &self.media
                        && let Some(track) = self.look.as_ref().and_then(|look| look.track.as_ref())
                    {
                        media.send(track, control);
                    }
                    return Ok(());
                }
                IslandAction::Volume(_) => return Ok(()), // Handled on press/drag.
                IslandAction::Mute => {
                    if let Some(audio) = &mut self.audio {
                        audio.toggle_mute()?;
                    }
                    return self.audio_changed(gfx);
                }
                IslandAction::ScrollUsage(direction) => {
                    self.scroll_usage(f32::from(direction) * 72.0)?;
                    return Ok(());
                }
            }
            return self.refresh(gfx);
        }
        if y >= (self.bounds.bottom - self.bounds.top) as f32 {
            return Ok(());
        }
        self.dismiss_island(gfx)?;
        let hit = self
            .zones
            .iter()
            .find(|(left, right, _)| x >= *left && x < *right);
        // Where a flyout opened from this item should hang: under its centre.
        let anchor = hit.map_or((0, 0), |(left, right, _)| {
            (
                self.bounds.left + ((left + right) / 2.0) as i32,
                self.anchor_y(),
            )
        });
        match hit.map(|(_, _, action)| *action) {
            Some(Action::AccessoryIndicator) => {}
            Some(Action::QuickSettings) => {
                open_quick_panel(self.bounds.right, self.anchor_y(), self.at_bottom());
            }
            // The clock: its panel, or the Windows notification centre when the panel is off.
            Some(Action::Notifications) => {
                if self.clock.panel {
                    open_hub("clock");
                } else {
                    launcher::open_notifications();
                }
            }
            Some(Action::Sound) => launcher::open_sound(),
            Some(Action::Search) => launcher::open_search(),
            Some(Action::TaskView) => launcher::open_task_view(),
            Some(Action::Widgets) => launcher::open_widgets(),
            Some(Action::Bluetooth) => launcher::open_bluetooth(),
            Some(Action::HiddenIcons) => launcher::toggle_hidden_icons(anchor),
            Some(Action::InputLanguage) => {} // Opened outside the app borrow.
            // Windows' own pages list which apps use each device.
            Some(Action::Microphone) => {
                let _ = launcher::open("ms-settings:privacy-microphone");
            }
            Some(Action::Camera) => {
                let _ = launcher::open("ms-settings:privacy-webcam");
            }
            None => {}
        }
        Ok(())
    }

    pub fn input_switcher_anchor(&self, x: f32, y: f32) -> Option<POINT> {
        let y = self.layout_y(y);
        if y < 0.0 || y >= (self.bounds.bottom - self.bounds.top) as f32 {
            return None;
        }
        self.zones
            .iter()
            .find(|(left, right, action)| {
                x >= *left && x < *right && matches!(action, Action::InputLanguage)
            })
            .map(|(left, right, _)| POINT {
                x: self.bounds.left + ((left + right) * 0.5) as i32,
                y: if self.at_bottom() {
                    self.bounds.top - 6
                } else {
                    self.bounds.bottom + 6
                },
            })
    }

    /// Redraws the bar if anything it shows has changed.
    pub fn refresh(&mut self, gfx: &Gfx) -> Result<()> {
        let Some(monitor) = system::primary_monitor() else {
            return Ok(());
        };
        let active_providers = if self.usage.is_some() {
            usage::active_providers()
        } else {
            ActiveProviders::default()
        };
        let track = self.media.as_ref().and_then(media::Watch::snapshot);
        self.panel = available_panel(self.panel, track.is_some(), active_providers.any());
        if !active_providers.any() {
            if let Some(usage) = &mut self.usage {
                usage.mark_inactive();
            }
        } else if self.visible
            && (!self.auto_hide || self.revealed)
            && let Some(usage) = &mut self.usage
        {
            usage.refresh(false);
        }
        let look = Look {
            accessories: self.accessories.as_ref().map_or([None, None], |monitor| {
                accessory_readings(&monitor.snapshot())
            }),
            language: strings::language(),
            screen: inset_screen(monitor.screen, &self.widgets, monitor.scale),
            scale: monitor.scale,
            light: system::uses_light_theme(),
            glass: system::transparency_enabled(),
            app_name: self.app_name.clone(),
            clock: system::clock_text_for(&self.clock.options),
            clock_font: (
                self.clock.options.font.family(),
                self.clock.options.weight.value(),
            ),
            input: system::input_language(),
            battery: system::battery(),
            status: self.status,
            widgets: self.widgets,
            usage: self
                .usage
                .as_ref()
                .map_or_else(Snapshot::default, UsageMonitor::snapshot),
            usage_enabled: self.usage.is_some() && active_providers.any(),
            active_providers,
            panel: self.panel,
            usage_refreshing: self.usage.as_ref().is_some_and(UsageMonitor::is_refreshing),
            track,
        };
        self.apply_look(gfx, look)
    }

    /// Track changes only need to update the contents of the island.
    pub fn media_changed(&mut self, gfx: &Gfx) -> Result<()> {
        let Some(mut look) = self.look.clone() else {
            return self.refresh(gfx);
        };
        look.track = self.media.as_ref().and_then(media::Watch::snapshot);
        self.panel = available_panel(self.panel, look.track.is_some(), look.usage_enabled);
        look.panel = self.panel;
        self.apply_look(gfx, look)
    }

    fn apply_look(&mut self, gfx: &Gfx, mut look: Look) -> Result<()> {
        look.active_providers = look.usage.visible_providers(look.active_providers);
        look.usage_enabled &= look.active_providers.any();
        self.panel = available_panel(look.panel, look.track.is_some(), look.usage_enabled);
        look.panel = self.panel;
        if look.panel != Some(Island::Usage) {
            self.usage_scroll = 0.0;
        }
        let moved = self
            .look
            .as_ref()
            .is_some_and(|old| old.screen != look.screen || old.scale != look.scale);
        if look.panel != Some(Island::Music) || moved {
            self.release_volume(None)?;
        }
        if self.look.as_ref() == Some(&look) {
            return Ok(());
        }
        let moment = self
            .look
            .as_ref()
            .and_then(|old| usage_moment(&old.usage, &look.usage, &look.active_providers));
        self.build(gfx, &look)?;
        // A crowded bar can have no room even for a compact trigger. Do not
        // keep a ghost popup that reappears by itself when space returns.
        if look.panel.is_some() && self.panel_span.width() <= 0.0 {
            self.panel = None;
            look.panel = None;
            self.build(gfx, &look)?;
        }
        self.look = Some(look);
        if let Some((provider, act)) = moment
            && let Some(&(_, x)) = self.usage_anchors.iter().find(|(p, _)| *p == provider)
        {
            self.cue = Some((act, x));
        }
        Ok(())
    }

    /// A moment worth the crab, found by the last update: the act and the
    /// screen x of the provider in the island. Taken once.
    pub fn take_cue(&mut self) -> Option<(mascot::Act, POINT)> {
        let (act, x) = self.cue.take()?;
        let shown = self.visible && (!self.auto_hide || self.revealed);
        shown.then_some((
            act,
            POINT {
                x,
                y: self.anchor_y(),
            },
        ))
    }

    fn build(&mut self, gfx: &Gfx, look: &Look) -> Result<()> {
        let c = &gfx.compositor;
        let scale = look.scale;
        let px = |logical: f32| (logical * scale).round();
        let screen = look.screen;
        let (width, height) = ((screen.right - screen.left) as f32, px(bar_height()));
        let panel_bottom = height + px(PANEL_GAP) + px(look.panel_height());
        let visual_height = if look.panel.is_some() {
            panel_bottom + px(PANEL_SHADOW_MARGIN)
        } else {
            height
        };
        let ink = if look.light { Ink::Dark } else { Ink::Light };

        let root = c.CreateContainerVisual()?;
        root.SetSize(Vector2 {
            X: width,
            Y: visual_height,
        })?;
        let bottom = look.widgets.position == whirlpool_config::BarPosition::Bottom;
        let root_layers = root.Children()?;
        // On the bottom edge the bar sits at the foot of the window and the
        // popup above it; each is a rigid block, so nothing inside is mirrored.
        let (layers, popup_layers, bar_dy, popup_dy) = if bottom {
            let bar_layer = c.CreateContainerVisual()?;
            bar_layer.SetSize(Vector2 {
                X: width,
                Y: height,
            })?;
            bar_layer.SetOffset(Vector3::new(0.0, visual_height - height, 0.0))?;
            let popup_layer = c.CreateContainerVisual()?;
            popup_layer.SetSize(Vector2 {
                X: width,
                Y: visual_height,
            })?;
            let dy = if look.panel.is_some() {
                px(PANEL_SHADOW_MARGIN) - (height + px(PANEL_GAP))
            } else {
                0.0
            };
            popup_layer.SetOffset(Vector3::new(0.0, dy, 0.0))?;
            root_layers.InsertAtTop(&bar_layer)?;
            root_layers.InsertAtTop(&popup_layer)?;
            (
                bar_layer.Children()?,
                popup_layer.Children()?,
                visual_height - height,
                dy,
            )
        } else {
            (root_layers.clone(), root_layers, 0.0, 0.0)
        };
        // Without transparency the bar is solid, as macOS draws it with
        // "Reduce transparency".
        let alpha = (look.widgets.opacity.clamp(10, 100) * 255 / 100) as u8;
        let [tint_r, tint_g, tint_b] = if look.light {
            look.widgets.tint.light()
        } else {
            look.widgets.tint.dark()
        };
        let tint = rgba(tint_r, tint_g, tint_b, if look.glass { alpha } else { 255 });
        let backdrop = if look.glass {
            Some(c.CreateHostBackdropBrush()?.cast()?)
        } else {
            None
        };
        let fills: [windows::UI::Composition::CompositionBrush; 1] =
            [c.CreateColorBrushWithColor(tint)?.cast()?];
        // The bar's material, clipped to rounded corners when it floats.
        let material = c.CreateContainerVisual()?;
        material.SetSize(Vector2 {
            X: width,
            Y: height,
        })?;
        if look.widgets.floating && look.widgets.radius > 0 {
            let radius = px(look.widgets.radius as f32).min(height / 2.0);
            let outline = c.CreateRoundedRectangleGeometry()?;
            outline.SetSize(Vector2 {
                X: width,
                Y: height,
            })?;
            outline.SetCornerRadius(Vector2 {
                X: radius,
                Y: radius,
            })?;
            material.SetClip(&c.CreateGeometricClipWithGeometry(&outline)?)?;
        }
        for brush in backdrop.iter().chain(&fills) {
            let fill = c.CreateSpriteVisual()?;
            fill.SetSize(Vector2 {
                X: width,
                Y: height,
            })?;
            fill.SetBrush(brush)?;
            material.Children()?.InsertAtTop(&fill)?;
        }
        if look.widgets.border {
            let thin = px(1.0).max(1.0);
            let line = c.CreateSpriteVisual()?;
            line.SetSize(Vector2 { X: width, Y: thin })?;
            line.SetOffset(Vector3::new(
                0.0,
                if bottom { 0.0 } else { height - thin },
                0.0,
            ))?;
            let edge = if look.light {
                rgba(0, 0, 0, 34)
            } else {
                rgba(255, 255, 255, 40)
            };
            line.SetBrush(&c.CreateColorBrushWithColor(edge)?)?;
            material.Children()?.InsertAtTop(&line)?;
        }
        layers.InsertAtTop(&material)?;

        let place = |layers: &windows::UI::Composition::VisualCollection,
                     item: &LabelBrush,
                     left: f32|
         -> Result<SpriteVisual> {
            let visual = c.CreateSpriteVisual()?;
            visual.SetSize(Vector2 {
                X: item.width,
                Y: item.height,
            })?;
            visual.SetOffset(Vector3 {
                X: left.round(),
                Y: ((height - item.height) / 2.0).round(),
                Z: 0.0,
            })?;
            visual.SetBrush(&item.brush)?;
            layers.InsertAtTop(&visual)?;
            Ok(visual)
        };

        self.zones.clear();
        let mut left_end = px(SIDE_MARGIN);
        if !look.app_name.is_empty() {
            let name = gfx.text_brush_fitted(
                &look.app_name,
                scale,
                13.0,
                true,
                ink,
                px(220.0).min(width * 0.22),
            )?;
            place(&layers, &name, px(SIDE_MARGIN))?;
            left_end += name.width;
        }

        // Right side, from the edge inward.
        let slop = px(HIT_SLOP);
        let mut right = width - px(SIDE_MARGIN);
        let status = look.status;
        let symbol = |layers: &[(char, f32)]| gfx.glyph_brush(layers, scale, ink);
        let mut items: Vec<(LabelBrush, Action)> = vec![
            (
                gfx.text_brush_styled(
                    &look.clock,
                    scale,
                    13.0,
                    look.clock_font.0,
                    look.clock_font.1,
                    ink,
                )?,
                Action::Notifications,
            ),
            (gfx.control_center_brush(scale, ink)?, Action::QuickSettings),
        ];
        // Privacy indicators, as the Windows tray shows them: something is
        // using the microphone or camera right now.
        let privacy = status.privacy;
        for (on, glyph, action) in [
            (privacy.microphone, glyph::MICROPHONE, Action::Microphone),
            (privacy.camera, glyph::CAMERA, Action::Camera),
        ] {
            if on {
                items.push((symbol(&[(glyph, 1.0)])?, action));
            }
        }
        let show_search = look.widgets.search.unwrap_or(status.buttons.search);
        let show_task_view = look.widgets.task_view.unwrap_or(status.buttons.task_view);
        let show_widgets = look.widgets.widgets.unwrap_or(status.buttons.widgets);
        let show_battery_percentage = look
            .widgets
            .battery_percent
            .unwrap_or(status.battery_percentage);
        let show_bluetooth = look.widgets.bluetooth;

        if show_search {
            items.push((symbol(&[(glyph::SEARCH, 1.0)])?, Action::Search));
        }
        if show_bluetooth {
            items.push((symbol(&[(glyph::BLUETOOTH, 1.0)])?, Action::Bluetooth));
        }
        match status.network {
            Network::Wifi { bars, has_internet } => {
                items.push((
                    gfx.wifi_brush(bars, has_internet, scale, ink)?,
                    Action::QuickSettings,
                ));
            }
            Network::Ethernet => {
                items.push((symbol(&[(glyph::ETHERNET, 1.0)])?, Action::QuickSettings));
            }
            Network::Offline => {
                items.push((symbol(&[(glyph::OFFLINE, 1.0)])?, Action::QuickSettings));
            }
        }
        if let Some(volume) = status.volume {
            let speaker = if volume.muted {
                glyph::MUTE
            } else {
                glyph::VOLUME[usize::from(volume.level.min(3))]
            };
            items.push((symbol(&[(speaker, 1.0)])?, Action::Sound));
        }
        let mut battery_label = None;
        if let Some(battery) = look.battery {
            if show_battery_percentage {
                battery_label = Some((
                    items.len(),
                    gfx.text_brush(&format!("{}%", battery.percent), scale, false, ink)?,
                ));
            }
            items.push((
                gfx.battery_brush(battery.percent, battery.charging, battery.saver, scale, ink)?,
                Action::QuickSettings,
            ));
        }
        let mut accessory_labels = Vec::new();
        for device in look.accessories.into_iter().flatten().rev() {
            let value = device
                .percent
                .map_or_else(|| "—".to_owned(), |percent| format!("{percent}%"));
            accessory_labels.push((items.len(), gfx.text_brush(&value, scale, false, ink)?));
            let icon = match device.kind {
                AccessoryKind::Mouse => glyph::MOUSE,
                AccessoryKind::Headset => glyph::HEADSET,
            };
            items.push((symbol(&[(icon, 1.0)])?, Action::AccessoryIndicator));
        }
        if let Some(language) = &look.input {
            items.push((
                gfx.text_brush(language, scale, false, ink)?,
                Action::InputLanguage,
            ));
        }
        // The notification area (antivirus, OneDrive, apps in the background)
        // lives only in the hidden taskbar; this arrow opens its flyout.
        items.push((symbol(&[(glyph::HIDDEN_ICONS, 1.0)])?, Action::HiddenIcons));
        if show_task_view {
            items.push((symbol(&[(glyph::TASK_VIEW, 1.0)])?, Action::TaskView));
        }
        if show_widgets {
            items.push((symbol(&[(glyph::WIDGETS, 1.0)])?, Action::Widgets));
        }
        for (index, (item, action)) in items.iter().enumerate() {
            let zone_right = right;
            if let Some((_, label)) = battery_label
                .iter()
                .chain(accessory_labels.iter())
                .find(|(item_index, _)| *item_index == index)
            {
                right -= label.width;
                place(&layers, label, right)?;
                right -= px(6.0);
            }
            let left = right - item.width;
            place(&layers, item, left)?;
            // The battery icon and its percentage form one control.
            // Accessory readings are passive status indicators, not controls.
            if *action != Action::AccessoryIndicator {
                self.zones.push((left - slop, zone_right + slop, *action));
            }
            right = left - px(ITEM_GAP);
        }

        self.island_hits.clear();
        self.volume_controls = None;
        self.usage_viewport = None;
        self.triggers_span = Span::default();
        self.panel_span = Span::default();
        self.draw_island(
            gfx,
            look,
            &layers,
            &popup_layers,
            width,
            height,
            left_end,
            right,
        )?;
        if let Some(drag) = self.volume_drag {
            let new_span = self.island_hits.iter().find_map(|hit| match hit.action {
                IslandAction::Volume(span) => Some(span),
                _ => None,
            });
            if new_span != Some(drag) {
                self.release_volume(None)?;
            }
        }
        self.surface.SetSize(Vector2 {
            X: width,
            Y: visual_height,
        })?;
        let contents = self.surface.Children()?;
        contents.RemoveAll()?;
        contents.InsertAtTop(&root)?;
        self.visual_height = if self.panel_span.width() > 0.0 {
            visual_height
        } else {
            height
        } as i32;
        self.bounds = if bottom {
            RECT {
                top: screen.bottom - height as i32,
                ..screen
            }
        } else {
            RECT {
                bottom: screen.top + height as i32,
                ..screen
            }
        };
        self.bar_dy = if self.panel_span.width() > 0.0 {
            bar_dy
        } else {
            0.0
        };
        self.popup_dy = popup_dy;
        TRIGGERS_LEFT.store(
            screen.left + self.triggers_span.left as i32,
            Ordering::Relaxed,
        );
        TRIGGERS_RIGHT.store(
            screen.left + self.triggers_span.right as i32,
            Ordering::Relaxed,
        );
        PANEL_LEFT.store(screen.left + self.panel_span.left as i32, Ordering::Relaxed);
        PANEL_RIGHT.store(
            screen.left + self.panel_span.right as i32,
            Ordering::Relaxed,
        );
        let clickable_height = if self.panel_span.width() > 0.0 {
            panel_bottom
        } else {
            height
        };
        BAR_AT_BOTTOM.store(bottom, Ordering::Relaxed);
        if bottom {
            BAR_TOP.store(screen.bottom - height as i32, Ordering::Relaxed);
            BAR_BOTTOM.store(screen.bottom, Ordering::Relaxed);
            PANEL_TOP.store(screen.bottom - clickable_height as i32, Ordering::Relaxed);
            PANEL_BOTTOM.store(screen.bottom, Ordering::Relaxed);
        } else {
            BAR_TOP.store(screen.top, Ordering::Relaxed);
            BAR_BOTTOM.store(screen.top + height as i32, Ordering::Relaxed);
            PANEL_TOP.store(screen.top, Ordering::Relaxed);
            PANEL_BOTTOM.store(screen.top + clickable_height as i32, Ordering::Relaxed);
        }
        self.place()
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "The island is laid out between the measured bar items"
    )]
    fn draw_island(
        &mut self,
        gfx: &Gfx,
        look: &Look,
        layers: &VisualCollection,
        popup_layers: &VisualCollection,
        width: f32,
        bar_height: f32,
        left_end: f32,
        right_start: f32,
    ) -> Result<()> {
        let px = |logical: f32| (logical * look.scale).round();
        // A notch is welded to the top edge; on the bottom edge the island is a bubble.
        let style = if look.widgets.position == whirlpool_config::BarPosition::Bottom {
            IslandStyle::Bubble
        } else {
            look.widgets.island_style
        };
        let cover = if let Some(track) = &look.track {
            let cached =
                self.cover
                    .as_ref()
                    .is_some_and(|(source, _)| match (source, &track.artwork) {
                        (Some(old), Some(new)) => Arc::ptr_eq(old, new),
                        (None, None) => true,
                        _ => false,
                    });
            if !cached {
                self.cover = media::artwork(track)
                    .map(|image| {
                        let surface = gfx.image_surface(image)?;
                        Ok::<_, anyhow::Error>((
                            track.artwork.clone(),
                            gfx.compositor.CreateSurfaceBrushWithSurface(&surface)?,
                        ))
                    })
                    .transpose()?;
            }
            self.cover.as_ref().map(|(_, brush)| brush.clone())
        } else {
            self.cover = None;
            None
        };
        let prepared_music = if style == IslandStyle::Bubble
            && let Some(track) = &look.track
        {
            Some(prepare_bubble_music(gfx, look, track)?)
        } else {
            None
        };
        let triggers = match style {
            IslandStyle::Notch => trigger_layout_for(
                island_metrics(&look.widgets).single_width,
                Span {
                    left: left_end + px(8.0),
                    right: right_start - px(8.0),
                },
                width / 2.0,
                look.scale,
                look.track.is_some(),
                look.active_providers.count(),
            ),
            IslandStyle::Bubble => bubble_trigger_layout(
                Span {
                    left: left_end + px(8.0),
                    right: right_start,
                },
                look.scale,
                prepared_music.as_ref().map(|music| music.width),
                look.active_providers.count(),
            ),
        };
        let [(_, music), (_, usage)] = triggers;
        let (Some(first), Some(last)) = (music.or(usage), usage.or(music)) else {
            return Ok(());
        };
        self.triggers_span = Span {
            left: first.left,
            right: last.right,
        };
        let geometry = IslandGeometry::new(self.triggers_span, 0.0, look);
        let (compact, compact_height, divider_inset) = match style {
            IslandStyle::Notch => (
                island_surface(gfx, look, layers, geometry, bar_height)?,
                bar_height,
                px(7.0),
            ),
            IslandStyle::Bubble => {
                let (surface, pill_height) =
                    bubble_surface(gfx, look, layers, geometry, bar_height)?;
                (surface, pill_height, px(5.0))
            }
        };
        let divider = music
            .zip(usage)
            .map(|(music, usage)| ((music.right + usage.left) / 2.0).round());
        let c = &gfx.compositor;
        if let Some(x) = divider {
            add_visual(
                c,
                &compact,
                &c.CreateColorBrushWithColor(geometry.border())?.cast()?,
                x - self.triggers_span.left,
                divider_inset,
                px(1.0).max(1.0),
                (compact_height - 2.0 * divider_inset).max(1.0),
            )?;
        }
        let providers = island_providers(look);
        for (kind, span) in triggers {
            let Some(span) = span else {
                continue;
            };
            let geometry = IslandGeometry::new(span, 0.0, look);
            let segment = c.CreateContainerVisual()?;
            segment.SetSize(Vector2::new(geometry.width, compact_height))?;
            segment.SetOffset(Vector3::new(span.left - self.triggers_span.left, 0.0, 0.0))?;
            let content = segment.Children()?;
            compact.InsertAtTop(&segment)?;
            match kind {
                Island::Music => {
                    if let Some(track) = &look.track {
                        match style {
                            IslandStyle::Notch => draw_compact_player(
                                gfx,
                                &content,
                                geometry,
                                geometry.width,
                                compact_height,
                                track,
                                cover.as_ref(),
                            )?,
                            IslandStyle::Bubble => {
                                if let Some(prepared) = &prepared_music {
                                    draw_bubble_player(
                                        gfx,
                                        &content,
                                        geometry,
                                        compact_height,
                                        track,
                                        prepared,
                                        cover.as_ref(),
                                    )?;
                                }
                            }
                        }
                    }
                }
                Island::Usage => {
                    let anchors = match style {
                        IslandStyle::Notch => {
                            draw_compact_usage(gfx, &content, geometry, compact_height, &providers)?
                        }
                        IslandStyle::Bubble => {
                            draw_bubble_usage(gfx, &content, geometry, compact_height, &providers)?
                        }
                    };
                    self.usage_anchors = anchors
                        .into_iter()
                        .map(|(provider, x)| (provider, look.screen.left + (span.left + x) as i32))
                        .collect();
                }
            }
            let (left, right) = match kind {
                Island::Music => (span.left, divider.unwrap_or(span.right)),
                Island::Usage => (divider.unwrap_or(span.left), span.right),
            };
            self.island_hits.push(IslandHit {
                bounds: (left, 0.0, right, bar_height),
                action: IslandAction::Toggle(kind),
            });
            if look.panel != Some(kind) {
                continue;
            }
            let metrics = island_metrics(&look.widgets);
            let desired = px(match kind {
                // The music panel follows the preset but never gets too narrow for its controls.
                Island::Music => (360.0 * metrics.expanded_width / 440.0).round().max(320.0),
                Island::Usage => metrics.expanded_width,
            });
            self.panel_span = match style {
                IslandStyle::Notch => panel_span(span, width, desired),
                IslandStyle::Bubble => bubble_panel_span(self.triggers_span, width, desired),
            };
            let geometry = IslandGeometry::new(self.panel_span, bar_height + px(PANEL_GAP), look);
            let height = px(look.panel_height());
            let theme = Theme::for_popup(look.light, look.glass);
            let content = popup_surface(gfx, theme, popup_layers, geometry, height)?;
            match kind {
                Island::Music => {
                    if let Some(track) = &look.track {
                        self.draw_player(gfx, &content, geometry, track, cover.as_ref())?;
                    }
                }
                Island::Usage => {
                    self.draw_usage_panel(gfx, &content, geometry, height, look, &providers)?
                }
            }
        }
        Ok(())
    }

    fn draw_usage_panel(
        &mut self,
        gfx: &Gfx,
        content: &VisualCollection,
        geometry: IslandGeometry,
        island_height: f32,
        look: &Look,
        providers: &[IslandProvider<'_>],
    ) -> Result<()> {
        let c = &gfx.compositor;
        let scale = geometry.scale;
        let px = |v| geometry.px(v);
        let island_width = geometry.width;
        let ink = geometry.ink();
        let title = strings::text();
        add_panel_text(
            gfx,
            content,
            title.usage_title,
            px(28.0),
            px(15.0),
            scale,
            13.0,
            true,
            ink,
            0.94,
        )?;
        let button_text = if look.usage_refreshing {
            title.usage_refreshing
        } else {
            title.usage_refresh
        };
        let button_label = gfx.text_brush_sized(button_text, scale, 11.5, true, ink)?;
        let button_width = button_label.width + px(20.0);
        let button_height = px(27.0);
        let button_left = island_width - px(23.0) - button_width;
        let button_top = px(8.0);
        add_rounded_box(
            c,
            content,
            button_left,
            button_top,
            button_width,
            button_height,
            px(13.5),
            geometry.control_fill(),
            Some(geometry.border()),
        )?;
        add_visual(
            c,
            content,
            &button_label.brush.cast()?,
            button_left + px(10.0),
            button_top + (button_height - button_label.height) / 2.0,
            button_label.width,
            button_label.height,
        )?;
        if !look.usage_refreshing {
            self.island_hits.push(geometry.hit(
                button_left,
                button_top,
                button_width,
                button_height,
                IslandAction::Refresh,
            ));
        }
        let body_top = px(42.0);
        let body_height = (island_height - body_top - px(26.0)).max(0.0);
        let content_height = px((look.panel_content_height() - 68.0).max(0.0));
        let maximum = ((content_height - body_height) / scale).max(0.0);
        self.usage_scroll = self.usage_scroll.clamp(0.0, maximum);
        let viewport = c.CreateContainerVisual()?;
        viewport.SetOffset(Vector3::new(0.0, body_top, 0.0))?;
        viewport.SetSize(Vector2::new(island_width, body_height))?;
        viewport.SetClip(&c.CreateInsetClip()?)?;
        let body = c.CreateContainerVisual()?;
        body.SetSize(Vector2::new(island_width, content_height))?;
        body.SetOffset(Vector3::new(0.0, -px(self.usage_scroll), 0.0))?;
        viewport.Children()?.InsertAtTop(&body)?;
        content.InsertAtTop(&viewport)?;
        let rows = body.Children()?;
        let mut row_top = 6.0;
        for (provider, usage, accent) in providers {
            let y = px(row_top);
            row_top += usage_row_height(usage, island_metrics(&look.widgets).row_height);
            let inset = px(23.0);
            add_visual(
                c,
                &rows,
                &c.CreateColorBrushWithColor(geometry.border())?.cast()?,
                inset,
                y - px(6.0),
                island_width - 2.0 * inset,
                1.0,
            )?;
            draw_usage_row(
                gfx,
                &rows,
                *provider,
                usage,
                *accent,
                scale,
                look.light,
                island_width,
                y,
            )?;
        }
        let bounds = (
            geometry.left,
            geometry.top + body_top,
            geometry.left + island_width,
            geometry.top + body_top + body_height,
        );
        if maximum > 0.0 {
            let [up, down] = [
                ('\u{E70E}', body_top + px(2.0), -1),
                ('\u{E70D}', body_top + body_height - px(22.0), 1),
            ]
            .map(|(symbol, top, direction)| -> Result<SpriteVisual> {
                let label = gfx.glyph_brush(&[(symbol, 1.0)], scale * 0.7, ink)?;
                let visual = add_visual(
                    c,
                    content,
                    &label.brush.cast()?,
                    island_width - px(22.0) + (px(18.0) - label.width) / 2.0,
                    top + (px(20.0) - label.height) / 2.0,
                    label.width,
                    label.height,
                )?;
                self.island_hits.push(geometry.hit(
                    island_width - px(22.0),
                    top,
                    px(18.0),
                    px(20.0),
                    IslandAction::ScrollUsage(direction),
                ));
                Ok(visual)
            });
            self.usage_viewport = Some(UsageViewport {
                content: body,
                maximum,
                scale,
                bounds,
                up: up?,
                down: down?,
            });
            self.position_usage_body()?;
        }
        let newest_age = providers
            .iter()
            .filter_map(|provider| provider.1.age_minutes())
            .min();
        if let Some(minutes) = newest_age {
            let status = format!("{} {minutes} min", title.usage_updated);
            let label = gfx.text_brush_sized(&status, scale, 11.0, false, ink)?;
            add_visual(
                c,
                content,
                &label.brush.cast()?,
                (island_width - label.width) / 2.0,
                island_height - px(21.0),
                label.width,
                label.height,
            )?
            .SetOpacity(0.53)?;
        }
        Ok(())
    }

    fn draw_player(
        &mut self,
        gfx: &Gfx,
        layers: &VisualCollection,
        geometry: IslandGeometry,
        track: &Track,
        cover: Option<&CompositionSurfaceBrush>,
    ) -> Result<()> {
        let px = |v| geometry.px(v);
        let c = &gfx.compositor;
        let ink = geometry.ink();
        add_panel_text(
            gfx,
            layers,
            strings::text().media_title,
            px(26.0),
            px(15.0),
            geometry.scale,
            12.5,
            true,
            ink,
            0.88,
        )?;
        let chevron = gfx.glyph_brush(&[('', 0.72)], geometry.scale, ink)?;
        let x = geometry.width - px(22.0) - chevron.width;
        add_visual(
            c,
            layers,
            &chevron.brush.cast()?,
            x,
            (px(46.0) - chevron.height) / 2.0,
            chevron.width,
            chevron.height,
        )?;
        self.island_hits.push(geometry.hit(
            x - px(6.0),
            px(7.0),
            chevron.width + px(12.0),
            px(32.0),
            IslandAction::Toggle(Island::Music),
        ));
        let tile = px(34.0);
        if let Some(cover) = cover {
            add_visual(c, layers, &cover.cast()?, px(25.0), px(46.0), tile, tile)?;
        }
        let available = geometry.width - px(91.0);
        let title = gfx.text_brush_fitted(
            track_title(track),
            geometry.scale,
            13.0,
            true,
            ink,
            available,
        )?;
        let title_y = if track.artist.trim().is_empty() {
            px(63.0) - title.height / 2.0
        } else {
            px(47.0)
        };
        add_visual(
            c,
            layers,
            &title.brush.cast()?,
            px(69.0),
            title_y,
            title.width,
            title.height,
        )?;
        if !track.artist.trim().is_empty() {
            let artist =
                gfx.text_brush_fitted(&track.artist, geometry.scale, 11.0, false, ink, available)?;
            add_visual(
                c,
                layers,
                &artist.brush.cast()?,
                px(69.0),
                px(65.0),
                artist.width,
                artist.height,
            )?
            .SetOpacity(0.69)?;
        }
        let center = geometry.width / 2.0;
        for (control, glyph, enabled, offset) in [
            (MediaControl::Previous, '\u{E892}', track.previous, -44.0),
            (
                MediaControl::PlayPause,
                if track.playing {
                    '\u{E769}'
                } else {
                    '\u{E768}'
                },
                track.toggle,
                0.0,
            ),
            (MediaControl::Next, '\u{E893}', track.next, 44.0),
        ] {
            let size = px(34.0);
            let x = center + px(offset) - size / 2.0;
            let y = px(89.0);
            let main = matches!(control, MediaControl::PlayPause);
            if main {
                add_rounded_box(
                    c,
                    layers,
                    x,
                    y,
                    size,
                    size,
                    size / 2.0,
                    geometry.control_fill(),
                    Some(geometry.border()),
                )?;
            }
            let symbol = gfx.glyph_brush(&[(glyph, 1.0)], geometry.scale, ink)?;
            add_visual(
                c,
                layers,
                &symbol.brush.cast()?,
                x + (size - symbol.width) / 2.0,
                y + (size - symbol.height) / 2.0,
                symbol.width,
                symbol.height,
            )?
            .SetOpacity(if enabled { 1.0 } else { 0.30 })?;
            if enabled {
                self.island_hits
                    .push(geometry.hit(x, y, size, size, IslandAction::Media(control)));
            }
        }
        add_panel_text(
            gfx,
            layers,
            strings::text().media_volume,
            px(25.0),
            px(135.0),
            geometry.scale,
            10.5,
            false,
            ink,
            0.65,
        )?;
        if let Some(volume) = self.status.volume {
            let symbol = if volume.muted {
                glyph::MUTE
            } else {
                glyph::VOLUME[volume.level.min(3) as usize]
            };
            let label = gfx.glyph_brush(&[(symbol, 1.0)], geometry.scale, ink)?;
            add_visual(
                c,
                layers,
                &label.brush.cast()?,
                px(25.0),
                px(160.0) - label.height / 2.0,
                label.width,
                label.height,
            )?;
            self.island_hits.push(geometry.hit(
                px(18.0),
                px(148.0),
                px(32.0),
                px(27.0),
                IslandAction::Mute,
            ));
            let left = px(59.0);
            let width = (geometry.width - left - px(66.0)).max(px(1.0));
            let percent = self.volume_target.unwrap_or(volume.percent);
            add_rounded_box(
                c,
                layers,
                left,
                px(158.0),
                width,
                px(4.0),
                px(2.0),
                geometry.control_fill(),
                None,
            )?;
            let (_, fill) = retained_rounded_box(
                c,
                layers,
                left,
                px(158.0),
                width,
                px(4.0),
                px(2.0),
                if geometry.light {
                    rgba(40, 48, 58, 220)
                } else {
                    rgba(240, 244, 250, 235)
                },
                None,
            )?;
            let dot = px(10.0);
            let (knob, _) = retained_rounded_box(
                c,
                layers,
                left - dot / 2.0,
                px(160.0) - dot / 2.0,
                dot,
                dot,
                dot / 2.0,
                rgba(245, 247, 250, 255),
                Some(geometry.border()),
            )?;
            let label =
                gfx.text_brush_sized(&format!("{percent}%"), geometry.scale, 10.5, false, ink)?;
            let value = add_visual(
                c,
                layers,
                &label.brush.cast()?,
                geometry.width - px(51.0),
                px(153.0),
                label.width,
                label.height,
            )?;
            value.SetOpacity(0.8)?;
            let mut controls = VolumeControls {
                fill,
                knob,
                value,
                geometry,
                left,
                width,
                position_percent: None,
                label_percent: percent,
            };
            controls.move_to(percent)?;
            self.volume_controls = Some(controls);
            self.island_hits.push(geometry.hit(
                left - px(6.0),
                px(147.0),
                width + px(12.0),
                px(29.0),
                IslandAction::Volume(Span {
                    left: geometry.left + left,
                    right: geometry.left + left + width,
                }),
            ));
        } else {
            add_panel_text(
                gfx,
                layers,
                strings::text().media_no_output,
                px(25.0),
                px(155.0),
                geometry.scale,
                11.0,
                false,
                ink,
                0.5,
            )?;
        }
        Ok(())
    }

    pub fn press_volume(&mut self, gfx: &Gfx, x: f32, y: f32) -> Result<()> {
        let y = self.layout_y(y);
        self.suppress_volume_release = false;
        let Some(hit) = self
            .island_hits
            .iter()
            .find(|hit| hit.at(x, y) && matches!(hit.action, IslandAction::Volume(_)))
        else {
            return Ok(());
        };
        let IslandAction::Volume(span) = hit.action else {
            return Ok(());
        };
        self.volume_drag = Some(span);
        // SAFETY: Capture only for a live slider drag, never to activate the bar.
        unsafe {
            SetCapture(self.hwnd);
        }
        self.queue_volume(span, x);
        self.paint_volume_preview(gfx)
    }

    pub fn scroll_usage_at(&mut self, point: POINT, delta: i16) -> Result<bool> {
        if !self.visible || (self.auto_hide && !self.revealed) || delta == 0 {
            return Ok(false);
        }
        let Some(viewport) = &self.usage_viewport else {
            return Ok(false);
        };
        let (x, y) = (
            (point.x - self.bounds.left) as f32,
            self.layout_y((point.y - self.window_top()) as f32),
        );
        let (left, top, right, bottom) = viewport.bounds;
        if x < left || x >= right || y < top || y >= bottom {
            return Ok(false);
        }
        self.scroll_usage(-f32::from(delta) / 120.0 * 48.0)
    }

    fn scroll_usage(&mut self, delta: f32) -> Result<bool> {
        let Some(viewport) = &self.usage_viewport else {
            return Ok(false);
        };
        let next = (self.usage_scroll + delta).clamp(0.0, viewport.maximum);
        if next != self.usage_scroll {
            self.usage_scroll = next;
            self.position_usage_body()?;
        }
        Ok(true)
    }

    fn position_usage_body(&mut self) -> Result<()> {
        let Some(viewport) = &self.usage_viewport else {
            return Ok(());
        };
        let offset = (self.usage_scroll * viewport.scale).round();
        viewport
            .content
            .SetOffset(Vector3::new(0.0, -offset, 0.0))?;
        viewport
            .up
            .SetOpacity(if self.usage_scroll > 0.0 { 0.8 } else { 0.22 })?;
        viewport
            .down
            .SetOpacity(if self.usage_scroll < viewport.maximum {
                0.8
            } else {
                0.22
            })?;
        Ok(())
    }

    fn paint_volume_preview(&mut self, gfx: &Gfx) -> Result<()> {
        if let Some(percent) = self.volume_target
            && let Some(controls) = &mut self.volume_controls
        {
            controls.paint(gfx, percent)?;
        }
        Ok(())
    }

    fn queue_volume(&mut self, span: Span, x: f32) {
        let percent = volume_percent(span, x);
        self.volume_target = Some(percent);
        if let Some(controls) = &mut self.volume_controls {
            // Optimistic GPU-only feedback is independent of endpoint latency.
            // Audio writes still proceed if device loss interrupts this paint;
            // the existing graphics heartbeat owns recovery.
            let _ = controls.move_to(percent);
        }
        if self.pending_volume == Some(percent)
            || (self.pending_volume.is_none()
                && self
                    .status
                    .volume
                    .is_some_and(|v| v.percent == percent && !v.muted))
        {
            return;
        }
        let arm = self.pending_volume.is_none();
        self.pending_volume = Some(percent);
        if arm {
            // At most 30 endpoint writes/second during dragging; no idle timer.
            unsafe {
                SetTimer(Some(self.hwnd), VOLUME_TIMER, 33, None);
            }
        }
    }

    pub fn flush_volume(&mut self) -> Result<()> {
        // SAFETY: Our own short-lived drag timer.
        unsafe {
            let _ = KillTimer(Some(self.hwnd), VOLUME_TIMER);
        }
        if let Some(percent) = self.pending_volume.take()
            && let Some(audio) = &mut self.audio
        {
            audio.set_percent(percent)?;
        }
        Ok(())
    }

    pub fn release_volume(&mut self, x: Option<f32>) -> Result<bool> {
        let Some(span) = self.volume_drag.take() else {
            return Ok(x.is_some() && std::mem::take(&mut self.suppress_volume_release));
        };
        self.suppress_volume_release = true;
        if let Some(x) = x {
            self.queue_volume(span, x);
        }
        // Clear state first: ReleaseCapture posts a capture-lost notification.
        unsafe {
            if GetCapture() == self.hwnd {
                let _ = ReleaseCapture();
            }
        }
        if let Err(error) = self.flush_volume() {
            whirlpool_shell::log::line(&format!("ajustar volumen: {error:#}"));
        }
        self.volume_target = None;
        Ok(true)
    }
}

fn volume_percent(span: Span, x: f32) -> u8 {
    if span.width() <= 0.0 || !x.is_finite() {
        return 0;
    }
    (((x - span.left) / span.width()).clamp(0.0, 1.0) * 100.0).round() as u8
}

fn same_volume_indicator(old: Option<Volume>, new: Option<Volume>) -> bool {
    old.map(|v| (v.level, v.muted)) == new.map(|v| (v.level, v.muted))
}

type IslandProvider<'a> = (Provider, &'a ProviderUsage, Color);

fn island_providers(look: &Look) -> Vec<IslandProvider<'_>> {
    Provider::ALL
        .into_iter()
        .filter(|provider| look.active_providers.has(*provider))
        .map(|provider| {
            let accent = match provider {
                Provider::Claude => rgba(217, 119, 87, 255),
                Provider::Codex => rgba(135, 190, 245, 255),
                Provider::OpenCode => rgba(132, 196, 166, 255),
                Provider::Gemini => rgba(111, 146, 255, 255),
            };
            (provider, look.usage.get(provider), accent)
        })
        .collect()
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        self.accessories = None;
        self.exit_animation = None;
        self.media = None;
        ISLAND_HOOK_WINDOW.store(0, Ordering::Relaxed);
        edge_watch::set_menu_bar(None);
        if let Some(hook) = self.outside_click_hook.take() {
            // SAFETY: Do not leave a callback pointing at a destroyed window.
            unsafe {
                let _ = UnhookWindowsHookEx(hook);
            }
        }
        // Give the reserved strip back before the window goes away.
        self.appbar = None;
        // SAFETY: Our own window, destroyed once.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
