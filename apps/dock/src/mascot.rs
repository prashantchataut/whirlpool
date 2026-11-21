//! The crab: a pixel-art mascot that comes out only for moments worth it
//! (an AI plan resetting or running out, the charger plugged in or out, energy
//! saver starting) and plays a short act, then leaves.
//!
//! Cost: nothing while it is away. An act creates a small click-through
//! window, hands the whole choreography to the compositor as keyframe
//! animations (the body slides smoothly, the legs step at 8 fps, as pixel
//! art does) and is destroyed when it ends. The dock only checks, every
//! 50 ms of those few seconds, whether the pointer is on the crab, so it can
//! react.

use std::time::{Duration, Instant};

use anyhow::Result;
use muelle_core::frame::DockEdge;
use muelle_shell::icons::IconImage;
use windows::{
    Foundation::TimeSpan,
    UI::Composition::{
        CompositionEasingFunction, CompositionStretch, ContainerVisual,
        Desktop::DesktopWindowTarget, SpriteVisual, Visual,
    },
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WinRT::Composition::ICompositorDesktopInterop},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SW_SHOWNOACTIVATE,
            ShowWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
            WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
        },
    },
    core::{HSTRING, Interface, h, w},
};
use windows_numerics::{Matrix4x4, Vector2, Vector3};

use crate::gfx::Gfx;

/* --------------------------------------------------------------- Sprites */

/// Sprite cells. `.` is transparent.
fn color(cell: u8) -> Option<[u8; 3]> {
    Some(match cell {
        b'k' | b'e' => [43, 20, 18],
        b'o' => [232, 116, 64],
        b'h' => [255, 214, 160],
        b'd' => [204, 84, 44],
        b'p' => [244, 140, 140],
        b'm' => [226, 70, 60],
        b'b' => [70, 160, 235],
        b'B' => [170, 215, 250],
        b'r' => [236, 72, 72],
        b'R' => [255, 150, 150],
        b'z' => [235, 238, 245],
        _ => return None,
    })
}

const WIDTH: usize = 22;

/// The body, 22 cells wide, without a face.
const BODY: [&str; 12] = [
    "....kkkkkkkkkkkkkk....",
    "...khhooooooooooook...",
    "...khoooooooooooook...",
    "...kooooooooooooook...",
    "...kooooooooooooook...",
    "kkkkooooooooooooookkkk",
    "kooooooooooooooooooook",
    "kooooooooooooooooooook",
    "kkkkooooooooooooookkkk",
    "...kooooooooooooook...",
    "...kooooooooooooook...",
    "...kddddddddddddddk...",
];

/// The legs standing, and mid-step (the first and third lifted).
const LEGS: [[&str; 3]; 2] = [
    [
        "...kddkkddkkddkkddk...",
        "...kddkkddkkddkkddk...",
        "...kkkkkkkkkkkkkkkk...",
    ],
    [
        "...kddkkddkkddkkddk...",
        "...kkkkkddkkkkkkddk...",
        ".......kkkk....kkkk...",
    ],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Neutral,
    Happy,
    Love,
    Sad,
    Sleep,
}

impl Face {
    #[cfg(test)]
    const ALL: [Self; 5] = [
        Self::Neutral,
        Self::Happy,
        Self::Love,
        Self::Sad,
        Self::Sleep,
    ];

    /// (row, column, cell) drawn over the body.
    fn marks(self) -> &'static [(usize, usize, u8)] {
        match self {
            Self::Neutral => &[
                (5, 7, b'e'),
                (6, 7, b'e'),
                (7, 7, b'e'),
                (5, 14, b'e'),
                (6, 14, b'e'),
                (7, 14, b'e'),
            ],
            Self::Happy => &[
                (6, 6, b'e'),
                (5, 7, b'e'),
                (6, 8, b'e'),
                (6, 13, b'e'),
                (5, 14, b'e'),
                (6, 15, b'e'),
                (7, 10, b'e'),
                (7, 11, b'e'),
                (8, 10, b'm'),
                (8, 11, b'm'),
            ],
            Self::Love => &[
                (5, 6, b'e'),
                (6, 7, b'e'),
                (7, 6, b'e'),
                (5, 15, b'e'),
                (6, 14, b'e'),
                (7, 15, b'e'),
                (7, 4, b'p'),
                (7, 5, b'p'),
                (7, 16, b'p'),
                (7, 17, b'p'),
            ],
            Self::Sad => &[
                (6, 6, b'e'),
                (7, 7, b'e'),
                (6, 8, b'e'),
                (6, 13, b'e'),
                (7, 14, b'e'),
                (6, 15, b'e'),
                (8, 15, b'b'),
                (9, 15, b'b'),
            ],
            Self::Sleep => &[
                (6, 6, b'e'),
                (6, 7, b'e'),
                (6, 8, b'e'),
                (6, 13, b'e'),
                (6, 14, b'e'),
                (6, 15, b'e'),
                (8, 10, b'm'),
                (8, 11, b'm'),
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Emote {
    Heart,
    Drop,
    Sleep,
}

impl Emote {
    fn rows(self) -> &'static [&'static str] {
        match self {
            Self::Heart => &[
                ".kk.kk.", "krRkrrk", "krrrrrk", ".krrrk.", "..krk..", "...k...",
            ],
            Self::Drop => &["..k..", ".kbk.", "kbBbk", "kbbbk", ".kkk."],
            // Light, outlined in `emote_image`: readable on any wallpaper.
            Self::Sleep => &[
                "....zzzz", "......z.", "zzz..z..", "..z.zzzz", ".z......", "zzz.....",
            ],
        }
    }
}

/// The crab with `face`, legs standing or mid-step, `cell` pixels a cell.
fn crab_image(face: Face, step: bool, cell: u32) -> IconImage {
    let mut grid: Vec<Vec<u8>> = BODY
        .iter()
        .chain(LEGS[usize::from(step)].iter())
        .map(|row| {
            let mut row = row.as_bytes().to_vec();
            row.resize(WIDTH, b'.');
            row
        })
        .collect();
    for &(row, column, cell) in face.marks() {
        grid[row][column] = cell;
    }
    paint(&grid, cell)
}

fn emote_image(emote: Emote, cell: u32) -> IconImage {
    let rows: Vec<Vec<u8>> = emote
        .rows()
        .iter()
        .map(|row| row.as_bytes().to_vec())
        .collect();
    if emote == Emote::Sleep {
        return paint(&outlined(&rows), cell);
    }
    paint(&rows, cell)
}

/// `grid` with a one-cell dark outline around everything drawn, the grid
/// growing a cell on each side to hold it.
fn outlined(grid: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let height = grid.len() + 2;
    let width = grid.iter().map(Vec::len).max().unwrap_or(0) + 2;
    let drawn = |y: usize, x: usize| {
        y >= 1
            && x >= 1
            && grid
                .get(y - 1)
                .and_then(|row| row.get(x - 1))
                .is_some_and(|&cell| cell != b'.')
    };
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    if drawn(y, x) {
                        grid[y - 1][x - 1]
                    } else if (y.saturating_sub(1)..=y + 1)
                        .any(|ny| (x.saturating_sub(1)..=x + 1).any(|nx| drawn(ny, nx)))
                    {
                        b'k'
                    } else {
                        b'.'
                    }
                })
                .collect()
        })
        .collect()
}

/// Paints a grid, each cell a `cell` x `cell` square: whole pixels, so the
/// compositor shows it crisp.
fn paint(grid: &[Vec<u8>], cell: u32) -> IconImage {
    let columns = grid.iter().map(Vec::len).max().unwrap_or(0) as u32;
    let (width, height) = (columns * cell, grid.len() as u32 * cell);
    let mut bgra = vec![0u8; (width * height * 4) as usize];
    for (y, row) in grid.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            let Some([r, g, b]) = color(value) else {
                continue;
            };
            for dy in 0..cell {
                let line = (y as u32 * cell + dy) * width;
                for dx in 0..cell {
                    let at = ((line + x as u32 * cell + dx) * 4) as usize;
                    bgra[at..at + 4].copy_from_slice(&[b, g, r, 255]);
                }
            }
        }
    }
    IconImage {
        width,
        height,
        bgra,
    }
}

/* ------------------------------------------------------------------ Acts */

/// Why the crab comes out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    /// An AI plan's limit reset: back to 0 %.
    Fresh,
    /// It passed 90 %.
    Nervous,
    /// It reached 100 %.
    Exhausted,
    /// The charger was plugged in.
    Charging,
    /// The charger was unplugged: back on battery.
    Unplugged,
    /// Energy saver started.
    Sleepy,
}

/// Where the act happens, in screen pixels.
#[derive(Clone, Copy, Debug)]
pub enum Place {
    /// Below a point of the menu bar's island: `x` is the centre, `top` the
    /// bar's bottom edge.
    Under { x: i32, top: i32 },
    /// Walking on the dock's inner rim. `left..right` is the span along the
    /// dock and `top` the rim (screen y on a bottom dock; the screen x of its
    /// inner side on a left or right one, where the crab walks up the wall).
    Along {
        left: i32,
        right: i32,
        top: i32,
        edge: DockEdge,
    },
}

/// The act is choreographed upright, in a "scene" `width` long and `height`
/// tall with the rim at the bottom; on a side dock the window holds it
/// turned a quarter (a pure rotation, so the sprites are never mirrored).
#[derive(Clone, Copy)]
struct Turn {
    edge: DockEdge,
    width: f32,
    height: f32,
}

impl Turn {
    /// Scene point of a point in the window.
    fn to_scene(self, x: f32, y: f32) -> (f32, f32) {
        match self.edge {
            DockEdge::Bottom => (x, y),
            // Head toward the screen's inside: right on a left dock...
            DockEdge::Left => (y, self.height - x),
            // ...left on a right dock, walking from the bottom up.
            DockEdge::Right => (self.width - y, x),
        }
    }

    /// The scene's transform into the window (row vectors, as Composition).
    fn matrix(self) -> Option<Matrix4x4> {
        let (m11, m12, m21, m22, m41, m42) = match self.edge {
            DockEdge::Bottom => return None,
            DockEdge::Left => (0.0, 1.0, -1.0, 0.0, self.height, 0.0),
            DockEdge::Right => (0.0, -1.0, 1.0, 0.0, 0.0, self.width),
        };
        Some(Matrix4x4 {
            M11: m11,
            M12: m12,
            M21: m21,
            M22: m22,
            M33: 1.0,
            M41: m41,
            M42: m42,
            M44: 1.0,
            ..Matrix4x4::default()
        })
    }
}

/// One act, in seconds and cells, independent of the screen.
struct Script {
    seconds: f32,
    /// (time, x) of the crab's left edge, in pixels from the window's left.
    path: Vec<(f32, f32)>,
    /// (start, height in cells) of each hop, 0.45 s long.
    hops: Vec<(f32, f32)>,
    /// Trembling, from..to.
    shake: Option<(f32, f32)>,
    /// From each time on, this face.
    faces: Vec<(f32, Face)>,
    /// Intervals in which the legs step.
    walking: Vec<(f32, f32)>,
    /// The emote and when it shows.
    emote: Option<(Emote, f32, f32)>,
}

const POP: f32 = 0.28;
const STEP_FPS: f32 = 8.0;
const HOP: f32 = 0.45;

impl Script {
    fn new(act: Act, width: f32, crab: f32) -> Self {
        // Under the island the crab stands still, centred.
        let centre = (width - crab) / 2.0;
        match act {
            Act::Fresh => Self {
                seconds: 3.0,
                path: vec![(0.0, centre)],
                hops: vec![(0.35, 5.0), (0.85, 4.0), (1.55, 6.0)],
                shake: None,
                faces: vec![(0.0, Face::Happy), (1.5, Face::Love)],
                walking: Vec::new(),
                emote: Some((Emote::Heart, 1.5, 2.65)),
            },
            Act::Nervous => Self {
                seconds: 2.8,
                path: vec![(0.0, centre)],
                hops: Vec::new(),
                shake: Some((0.35, 2.2)),
                faces: vec![(0.0, Face::Neutral), (0.35, Face::Sad)],
                walking: Vec::new(),
                emote: Some((Emote::Drop, 0.4, 2.4)),
            },
            Act::Exhausted => Self {
                seconds: 3.6,
                path: vec![(0.0, centre)],
                hops: vec![(0.3, 2.0)],
                shake: None,
                faces: vec![(0.0, Face::Sad), (1.0, Face::Sleep)],
                walking: Vec::new(),
                emote: Some((Emote::Sleep, 1.1, 3.25)),
            },
            // Scurries in from the left, stops halfway for a happy hop, and
            // runs off to the right.
            Act::Charging => {
                let middle = width * 0.55 - crab / 2.0;
                Self {
                    seconds: 4.6,
                    path: vec![
                        (0.0, 0.0),
                        (1.9, middle),
                        (2.9, middle),
                        (4.6, width - crab),
                    ],
                    hops: vec![(2.0, 6.0), (3.0, 3.0)],
                    shake: None,
                    faces: vec![(0.0, Face::Happy), (2.0, Face::Love), (2.9, Face::Happy)],
                    walking: vec![(0.0, 1.9), (2.9, 4.6)],
                    emote: Some((Emote::Heart, 2.05, 3.0)),
                }
            }
            Act::Unplugged => Self {
                seconds: 3.2,
                path: vec![(0.0, centre)],
                hops: vec![(0.4, 2.0)],
                shake: None,
                faces: vec![(0.0, Face::Neutral), (0.8, Face::Sad), (2.0, Face::Neutral)],
                walking: Vec::new(),
                emote: Some((Emote::Drop, 0.85, 2.2)),
            },
            // Shuffles in slowly from the right, stops and falls asleep.
            Act::Sleepy => {
                let rest = width * 0.62 - crab / 2.0;
                Self {
                    seconds: 5.4,
                    path: vec![(0.0, width - crab), (2.6, rest), (5.4, rest)],
                    hops: Vec::new(),
                    shake: None,
                    faces: vec![(0.0, Face::Neutral), (2.7, Face::Sleep)],
                    walking: vec![(0.0, 2.6)],
                    emote: Some((Emote::Sleep, 2.9, 5.1)),
                }
            }
        }
    }

    /// The crab's left edge at `t`.
    fn x_at(&self, t: f32) -> f32 {
        let Some(first) = self.path.first() else {
            return 0.0;
        };
        let mut previous = *first;
        for &(time, x) in &self.path {
            if t <= time {
                let span = (time - previous.0).max(f32::EPSILON);
                let progress = ((t - previous.0) / span).clamp(0.0, 1.0);
                return previous.1 + (x - previous.1) * progress;
            }
            previous = (time, x);
        }
        previous.1
    }

    fn face_at(&self, t: f32) -> Face {
        self.faces
            .iter()
            .rev()
            .find(|(from, _)| t >= *from)
            .map_or(Face::Neutral, |(_, face)| *face)
    }

    fn step_at(&self, t: f32) -> bool {
        self.walking
            .iter()
            .any(|(from, to)| t >= *from && t < *to && ((t - from) * STEP_FPS) as u32 % 2 == 1)
    }
}

/* ---------------------------------------------------------------- Mascot */

pub struct Mascot {
    hwnd: HWND,
    _target: DesktopWindowTarget,
    reaction: ContainerVisual,
    poke_heart: SpriteVisual,
    script: Script,
    started: Instant,
    /// The window's top-left corner on screen, and the crab's size.
    origin: POINT,
    /// How the upright scene sits in the window.
    turn: Turn,
    crab: (f32, f32),
    crab_top: f32,
    cell: f32,
    last_poke: Option<Instant>,
}

impl Mascot {
    /// Plays `act` at `place`. `scale` is the monitor's (1.0 at 100 %).
    pub fn play(gfx: &Gfx, act: Act, place: Place, scale: f32) -> Option<Self> {
        match Self::build(gfx, act, place, scale) {
            Ok(mascot) => Some(mascot),
            Err(error) => {
                muelle_shell::log::line(&format!("cangrejo: {error:#}"));
                None
            }
        }
    }

    fn build(gfx: &Gfx, act: Act, place: Place, scale: f32) -> Result<Self> {
        let cell_px = (scale * 1.6).round().max(2.0) as u32;
        let cell = cell_px as f32;
        let crab = (
            (WIDTH as u32 * cell_px) as f32,
            ((BODY.len() + 3) as u32 * cell_px) as f32,
        );
        // Room above the crab for hops and the emote.
        let headroom = 14.0 * cell;
        let height = crab.1 + headroom;
        let (rect, crab_top) = match place {
            Place::Under { x, top } => {
                let width = crab.0 + 8.0 * cell;
                let left = x - (width / 2.0) as i32;
                // Hangs a little below the bar.
                let top = top + cell as i32;
                (
                    RECT {
                        left,
                        top,
                        right: left + width as i32,
                        bottom: top + height as i32,
                    },
                    headroom,
                )
            }
            // Feet one cell into the rim, as if standing on it.
            Place::Along {
                left,
                right,
                top,
                edge: DockEdge::Bottom,
            } => {
                let bottom = top + cell as i32;
                (
                    RECT {
                        left,
                        top: bottom - height as i32,
                        right,
                        bottom,
                    },
                    headroom,
                )
            }
            Place::Along {
                left: start,
                right: end,
                top: rim,
                edge,
            } => {
                let feet = if edge == DockEdge::Left {
                    rim - cell as i32
                } else {
                    rim + cell as i32 - height as i32
                };
                (
                    RECT {
                        left: feet,
                        top: start,
                        right: feet + height as i32,
                        bottom: end,
                    },
                    headroom,
                )
            }
        };
        let turn = match place {
            Place::Along { edge, .. } if edge.is_vertical() => Turn {
                edge,
                width: (rect.bottom - rect.top) as f32,
                height,
            },
            _ => Turn {
                edge: DockEdge::Bottom,
                width: (rect.right - rect.left) as f32,
                height: (rect.bottom - rect.top) as f32,
            },
        };
        let script = Script::new(act, turn.width, crab.0);

        let hwnd = create_window(rect)?;
        let built = || -> Result<(DesktopWindowTarget, ContainerVisual, SpriteVisual)> {
            // SAFETY: `hwnd` is the live window created above.
            let target = unsafe {
                gfx.compositor
                    .cast::<ICompositorDesktopInterop>()?
                    .CreateDesktopWindowTarget(hwnd, true)?
            };
            let (root, reaction, poke_heart) = choreograph(gfx, &script, crab, crab_top, cell)?;
            if let Some(matrix) = turn.matrix() {
                root.SetTransformMatrix(matrix)?;
            }
            target.SetRoot(&root)?;
            Ok((target, reaction, poke_heart))
        };
        let (target, reaction, poke_heart) = match built() {
            Ok(parts) => parts,
            Err(error) => {
                // SAFETY: Our own window, created above.
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                return Err(error);
            }
        };
        // SAFETY: Our own window; shown without taking the focus.
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        Ok(Self {
            hwnd,
            _target: target,
            reaction,
            poke_heart,
            script,
            started: Instant::now(),
            origin: POINT {
                x: rect.left,
                y: rect.top,
            },
            turn,
            crab,
            crab_top,
            cell,
            last_poke: None,
        })
    }

    /// `true` once the act is over and the crab can go.
    pub fn finished(&self) -> bool {
        self.started.elapsed() >= Duration::from_secs_f32(self.script.seconds)
    }

    /// The pointer at `point`: if it is on the crab, a hop and a heart.
    pub fn poke(&mut self, gfx: &Gfx, point: POINT) {
        let t = self.started.elapsed().as_secs_f32();
        // Not while popping in or out.
        if t < POP || t > self.script.seconds - POP {
            return;
        }
        if self
            .last_poke
            .is_some_and(|last| last.elapsed() < Duration::from_millis(900))
        {
            return;
        }
        // Compared in the upright scene, whichever way the window holds it.
        let left = self.script.x_at(t);
        let top = self.crab_top;
        let (x, y) = self.turn.to_scene(
            (point.x - self.origin.x) as f32,
            (point.y - self.origin.y) as f32,
        );
        if x < left
            || x >= left + self.crab.0
            || y < top - 3.0 * self.cell
            || y >= top + self.crab.1
        {
            return;
        }
        self.last_poke = Some(Instant::now());
        let _ = react(
            gfx,
            &self.reaction,
            &self.poke_heart,
            self.cell,
            self.crab_top,
        );
    }
}

impl Drop for Mascot {
    fn drop(&mut self) {
        // SAFETY: Our own window, destroyed once.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// The visual tree and its animations:
///
/// root → path (slides along the act's path)
///          ├ hop (hops, trembles, pops in and out)
///          │   └ reaction (pointer squash and stretch)
///          │       └ one sprite per face and leg pose, switched on and off
///          ├ emote (rises and fades)
///          └ poke heart (only on a poke)
fn choreograph(
    gfx: &Gfx,
    script: &Script,
    crab: (f32, f32),
    crab_top: f32,
    cell: f32,
) -> Result<(ContainerVisual, ContainerVisual, SpriteVisual)> {
    let c = &gfx.compositor;
    let seconds = script.seconds;
    let duration = TimeSpan {
        Duration: (seconds as f64 * 1e7) as i64,
    };
    let at = |t: f32| (t / seconds).clamp(0.0, 1.0);
    let linear: CompositionEasingFunction = c.CreateLinearEasingFunction()?.cast()?;
    let out: CompositionEasingFunction = c
        .CreateCubicBezierEasingFunction(Vector2::new(0.2, 0.9), Vector2::new(0.3, 1.0))?
        .cast()?;
    let cell_px = cell as u32;

    let root = c.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let path = c.CreateContainerVisual()?;
    path.SetSize(Vector2::new(crab.0, crab_top + crab.1))?;
    root.Children()?.InsertAtTop(&path)?;

    // The walk: the left edge along the path, linear between its points.
    let slide = c.CreateVector3KeyFrameAnimation()?;
    slide.SetDuration(duration)?;
    for &(time, x) in &script.path {
        slide.InsertKeyFrameWithEasingFunction(at(time), Vector3::new(x, 0.0, 0.0), &linear)?;
    }
    if script.path.len() == 1 {
        path.SetOffset(Vector3::new(script.path[0].1, 0.0, 0.0))?;
    } else {
        path.StartAnimation(h!("Offset"), &slide)?;
    }

    // Hops, trembling, and the pop in and out (scale from the feet).
    let hop = c.CreateContainerVisual()?;
    hop.SetSize(Vector2::new(crab.0, crab.1))?;
    hop.SetOffset(Vector3::new(0.0, crab_top, 0.0))?;
    hop.SetCenterPoint(Vector3::new(crab.0 / 2.0, crab.1, 0.0))?;
    path.Children()?.InsertAtTop(&hop)?;

    let lift = c.CreateVector3KeyFrameAnimation()?;
    lift.SetDuration(duration)?;
    lift.InsertKeyFrame(0.0, Vector3::new(0.0, crab_top, 0.0))?;
    for &(start, height) in &script.hops {
        let up = Vector3::new(0.0, crab_top - height * cell, 0.0);
        let down = Vector3::new(0.0, crab_top, 0.0);
        lift.InsertKeyFrameWithEasingFunction(at(start), down, &linear)?;
        lift.InsertKeyFrameWithEasingFunction(at(start + HOP * 0.45), up, &out)?;
        lift.InsertKeyFrameWithEasingFunction(at(start + HOP), down, &linear)?;
    }
    if let Some((from, to)) = script.shake {
        // A cell either way, twelve times a second.
        let mut t = from;
        let mut side = 1.0;
        lift.InsertKeyFrameWithEasingFunction(at(from), Vector3::new(0.0, crab_top, 0.0), &linear)?;
        while t < to {
            t += 1.0 / 12.0;
            lift.InsertKeyFrameWithEasingFunction(
                at(t),
                Vector3::new(side * cell * 0.5, crab_top, 0.0),
                &linear,
            )?;
            side = -side;
        }
        lift.InsertKeyFrameWithEasingFunction(
            at(t + 0.05),
            Vector3::new(0.0, crab_top, 0.0),
            &linear,
        )?;
    }
    lift.InsertKeyFrame(1.0, Vector3::new(0.0, crab_top, 0.0))?;
    hop.StartAnimation(h!("Offset"), &lift)?;

    let pop = c.CreateVector3KeyFrameAnimation()?;
    pop.SetDuration(duration)?;
    pop.InsertKeyFrame(0.0, Vector3::new(0.0, 0.0, 1.0))?;
    pop.InsertKeyFrameWithEasingFunction(at(POP * 0.7), Vector3::new(1.15, 1.15, 1.0), &out)?;
    pop.InsertKeyFrameWithEasingFunction(at(POP), Vector3::new(1.0, 1.0, 1.0), &linear)?;
    pop.InsertKeyFrameWithEasingFunction(at(seconds - POP), Vector3::new(1.0, 1.0, 1.0), &linear)?;
    pop.InsertKeyFrameWithEasingFunction(at(seconds), Vector3::new(0.0, 0.0, 1.0), &linear)?;
    hop.StartAnimation(h!("Scale"), &pop)?;

    // Faces and steps: every pose the act uses is a sprite, shown only
    // while it is current. Sampled at the step rate, cut, not faded.
    let mut poses: Vec<(Face, bool)> = Vec::new();
    let samples = (seconds * STEP_FPS).ceil() as u32 + 1;
    let timeline: Vec<(f32, (Face, bool))> = (0..samples)
        .map(|index| {
            let t = (index as f32 / STEP_FPS).min(seconds);
            (t, (script.face_at(t), script.step_at(t)))
        })
        .collect();
    for (_, pose) in &timeline {
        if !poses.contains(pose) {
            poses.push(*pose);
        }
    }
    // Pointer reactions have their own transform, so they cannot replace the
    // act's entry/exit Scale animation or leave the mascot visible at the end.
    let reaction = c.CreateContainerVisual()?;
    reaction.SetSize(Vector2::new(crab.0, crab.1))?;
    reaction.SetCenterPoint(Vector3::new(crab.0 / 2.0, crab.1, 0.0))?;
    hop.Children()?.InsertAtTop(&reaction)?;
    let poses_layer = reaction.Children()?;
    for pose in &poses {
        let sprite = sprite(gfx, &crab_image(pose.0, pose.1, cell_px))?;
        poses_layer.InsertAtTop(&sprite)?;
        let shown = c.CreateScalarKeyFrameAnimation()?;
        shown.SetDuration(duration)?;
        let mut current = timeline.first().map(|(_, first)| first == pose);
        shown.InsertKeyFrame(0.0, if current == Some(true) { 1.0 } else { 0.0 })?;
        for (t, at_pose) in &timeline {
            let visible = at_pose == pose;
            if Some(visible) != current {
                let value = if visible { 1.0 } else { 0.0 };
                shown.InsertKeyFrameWithEasingFunction(
                    (at(*t) - 0.0005).max(0.0),
                    if visible { 0.0 } else { 1.0 },
                    &linear,
                )?;
                shown.InsertKeyFrameWithEasingFunction(at(*t), value, &linear)?;
                current = Some(visible);
            }
        }
        sprite.StartAnimation(h!("Opacity"), &shown)?;
    }

    // The emote rises from above the head and fades.
    if let Some((emote, from, to)) = script.emote {
        let sprite = sprite(gfx, &emote_image(emote, cell_px))?;
        let size = sprite.Size()?;
        let x = (crab.0 - size.X) / 2.0
            + if emote == Emote::Sleep {
                4.0 * cell
            } else {
                0.0
            };
        let start = crab_top - size.Y - cell;
        sprite.SetOffset(Vector3::new(x, start, 0.0))?;
        path.Children()?.InsertAtTop(&sprite)?;
        let rise = c.CreateVector3KeyFrameAnimation()?;
        rise.SetDuration(duration)?;
        rise.InsertKeyFrame(0.0, Vector3::new(x, start, 0.0))?;
        rise.InsertKeyFrameWithEasingFunction(at(from), Vector3::new(x, start, 0.0), &linear)?;
        rise.InsertKeyFrameWithEasingFunction(
            at(to),
            Vector3::new(x, start - 5.0 * cell, 0.0),
            &out,
        )?;
        rise.InsertKeyFrame(1.0, Vector3::new(x, start - 5.0 * cell, 0.0))?;
        sprite.StartAnimation(h!("Offset"), &rise)?;
        let fade = c.CreateScalarKeyFrameAnimation()?;
        fade.SetDuration(duration)?;
        fade.InsertKeyFrame(0.0, 0.0)?;
        fade.InsertKeyFrameWithEasingFunction(at(from), 0.0, &linear)?;
        fade.InsertKeyFrameWithEasingFunction(at(from + 0.15), 1.0, &linear)?;
        fade.InsertKeyFrameWithEasingFunction(at(to - 0.3), 1.0, &linear)?;
        fade.InsertKeyFrameWithEasingFunction(at(to), 0.0, &linear)?;
        fade.InsertKeyFrame(1.0, 0.0)?;
        sprite.StartAnimation(h!("Opacity"), &fade)?;
    }

    let poke_heart = sprite(gfx, &emote_image(Emote::Heart, cell_px))?;
    poke_heart.SetOpacity(0.0)?;
    path.Children()?.InsertAtTop(&poke_heart)?;
    Ok((root, reaction, poke_heart))
}

/// A poke: a quick hop and a heart above the head.
fn react(
    gfx: &Gfx,
    reaction: &ContainerVisual,
    heart: &SpriteVisual,
    cell: f32,
    crab_top: f32,
) -> Result<()> {
    let c = &gfx.compositor;
    let duration = TimeSpan {
        Duration: 6_500_000,
    };
    // The hop container's own animation keeps running; this one adds a
    // short squash and stretch on top, through the scale.
    let squash = c.CreateVector3KeyFrameAnimation()?;
    squash.SetDuration(TimeSpan {
        Duration: 3_500_000,
    })?;
    squash.InsertKeyFrame(0.0, Vector3::new(1.0, 1.0, 1.0))?;
    squash.InsertKeyFrame(0.25, Vector3::new(1.12, 0.86, 1.0))?;
    squash.InsertKeyFrame(0.6, Vector3::new(0.92, 1.12, 1.0))?;
    squash.InsertKeyFrame(1.0, Vector3::new(1.0, 1.0, 1.0))?;
    reaction.StartAnimation(h!("Scale"), &squash)?;

    let size = heart.Size()?;
    let hop_size = reaction.Size()?;
    let x = (hop_size.X - size.X) / 2.0;
    let start = crab_top - size.Y - cell;
    let rise = c.CreateVector3KeyFrameAnimation()?;
    rise.SetDuration(duration)?;
    rise.InsertKeyFrame(0.0, Vector3::new(x, start, 0.0))?;
    rise.InsertKeyFrame(1.0, Vector3::new(x, start - 6.0 * cell, 0.0))?;
    heart.StartAnimation(h!("Offset"), &rise)?;
    let fade = c.CreateScalarKeyFrameAnimation()?;
    fade.SetDuration(duration)?;
    fade.InsertKeyFrame(0.0, 1.0)?;
    fade.InsertKeyFrame(0.65, 1.0)?;
    fade.InsertKeyFrame(1.0, 0.0)?;
    heart.StartAnimation(h!("Opacity"), &fade)?;
    Ok(())
}

/// A sprite showing `image` at its exact size, never smoothed.
fn sprite(gfx: &Gfx, image: &IconImage) -> Result<SpriteVisual> {
    let c = &gfx.compositor;
    let brush = c.CreateSurfaceBrushWithSurface(&gfx.image_surface(image)?)?;
    brush.SetStretch(CompositionStretch::None)?;
    brush.SetBitmapInterpolationMode(
        windows::UI::Composition::CompositionBitmapInterpolationMode::NearestNeighbor,
    )?;
    let sprite = c.CreateSpriteVisual()?;
    sprite.SetBrush(&brush)?;
    sprite.SetSize(Vector2::new(image.width as f32, image.height as f32))?;
    let _: &Visual = &sprite.cast()?;
    Ok(sprite)
}

fn create_window(rect: RECT) -> Result<HWND> {
    static CLASS: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    let class = w!("MuelleMascot");
    // SAFETY: The class is registered once; the window is destroyed by
    // `Mascot::drop`, or by the caller if building the act fails.
    unsafe {
        let instance = GetModuleHandleW(None)?;
        CLASS.get_or_init(|| {
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(mascot_proc),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            });
        });
        // Click-through: the crab never takes a click from what is under it.
        Ok(CreateWindowExW(
            WS_EX_LAYERED
                | WS_EX_TRANSPARENT
                | WS_EX_NOREDIRECTIONBITMAP
                | WS_EX_TOPMOST
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE,
            class,
            &HSTRING::new(),
            WS_POPUP,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            Some(instance.into()),
            None,
        )?)
    }
}

unsafe extern "system" fn mascot_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: Default handling for our own window.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprites_should_be_rectangular_and_faces_inside_the_body() {
        for row in BODY.iter().chain(LEGS.iter().flatten()) {
            assert!(row.len() <= WIDTH + 1, "{row}");
        }
        for face in Face::ALL {
            for &(row, column, _) in face.marks() {
                assert!(row < BODY.len() && column < WIDTH, "{face:?}");
                assert_ne!(BODY[row].as_bytes()[column], b'.', "{face:?} off the body");
            }
        }
    }

    #[test]
    fn script_should_place_the_crab_along_its_path() {
        let script = Script::new(Act::Charging, 1000.0, 44.0);
        assert_eq!(script.x_at(0.0), 0.0);
        assert!((script.x_at(script.seconds) - (1000.0 - 44.0)).abs() < 0.01);
        assert_eq!(script.face_at(2.5), Face::Love);
        assert!(!script.step_at(2.5), "it stands still while hopping");
    }

    #[test]
    #[ignore = "Requires Windows desktop composition"]
    fn compositor_accepts_every_act_and_the_independent_pointer_reaction() -> Result<()> {
        use windows::Win32::System::WinRT::{
            CreateDispatcherQueueController, DQTAT_COM_STA, DQTYPE_THREAD_CURRENT,
            DispatcherQueueOptions,
        };
        // SAFETY: The test owns the dispatcher for this thread.
        let _queue = unsafe {
            CreateDispatcherQueueController(DispatcherQueueOptions {
                dwSize: size_of::<DispatcherQueueOptions>() as u32,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_STA,
            })?
        };
        let gfx = Gfx::new()?;
        for act in [
            Act::Fresh,
            Act::Nervous,
            Act::Exhausted,
            Act::Charging,
            Act::Unplugged,
            Act::Sleepy,
        ] {
            let script = Script::new(act, 800.0, 44.0);
            let (_root, reaction, heart) = choreograph(&gfx, &script, (44.0, 30.0), 28.0, 2.0)?;
            react(&gfx, &reaction, &heart, 2.0, 28.0)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod turn_tests {
    use super::*;

    /// The scene's rim (bottom) lands on the dock side of the window, and
    /// `to_scene` undoes the window transform.
    #[test]
    fn a_turned_scene_stands_on_the_dock_and_maps_back() {
        for edge in [DockEdge::Left, DockEdge::Right] {
            let turn = Turn {
                edge,
                width: 600.0,
                height: 80.0,
            };
            let m = turn.matrix().unwrap_or_default();
            let to_window =
                |x: f32, y: f32| (x * m.M11 + y * m.M21 + m.M41, x * m.M12 + y * m.M22 + m.M42);
            // Feet (scene bottom) at the dock: left side of a left dock's
            // window, right side of a right dock's.
            let (feet_x, _) = to_window(300.0, 80.0);
            assert_eq!(feet_x, if edge == DockEdge::Left { 0.0 } else { 80.0 });
            for (x, y) in [(0.0, 0.0), (123.0, 40.0), (600.0, 80.0)] {
                let (wx, wy) = to_window(x, y);
                let (sx, sy) = turn.to_scene(wx, wy);
                assert!((sx - x).abs() < 1e-3 && (sy - y).abs() < 1e-3, "{edge:?}");
            }
        }
    }
}
