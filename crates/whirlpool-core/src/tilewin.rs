//! Tiling chosen windows on one display: side by side, stacked or in a grid, with undo.
//! Pure geometry; `whirlpool-shell` moves the windows.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    SideBySide,
    Stacked,
    Grid,
}

impl Layout {
    pub fn from_id(id: &str) -> Option<Layout> {
        match id {
            "side" | "sideBySide" => Some(Layout::SideBySide),
            "stacked" => Some(Layout::Stacked),
            "grid" => Some(Layout::Grid),
            _ => None,
        }
    }
}

/// Splits `total` into `n` parts that add up exactly (the first parts take the remainder).
fn split(total: i32, n: i32, gap: i32) -> Vec<(i32, i32)> {
    let room = (total - gap * (n - 1)).max(n);
    let (each, extra) = (room / n, room % n);
    let mut at = 0;
    (0..n)
        .map(|i| {
            let len = each + i32::from(i < extra);
            let start = at;
            at += len + gap;
            (start, len)
        })
        .collect()
}

/// One rectangle per window inside `area`, in the order the windows were given.
/// A grid uses `ceil(sqrt(n))` columns; a short last row widens its tiles to fill the width.
pub fn tile(layout: Layout, area: Rect, count: usize, gap: i32) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }
    let n = count as i32;
    match layout {
        Layout::SideBySide => split(area.w, n, gap)
            .into_iter()
            .map(|(x, w)| Rect {
                x: area.x + x,
                y: area.y,
                w,
                h: area.h,
            })
            .collect(),
        Layout::Stacked => split(area.h, n, gap)
            .into_iter()
            .map(|(y, h)| Rect {
                x: area.x,
                y: area.y + y,
                w: area.w,
                h,
            })
            .collect(),
        Layout::Grid => {
            let cols = (n as f64).sqrt().ceil() as i32;
            let rows = (n + cols - 1) / cols;
            let mut out = Vec::new();
            for (r, (y, h)) in split(area.h, rows, gap).into_iter().enumerate() {
                let in_row = if r as i32 == rows - 1 {
                    n - cols * (rows - 1)
                } else {
                    cols
                };
                for (x, w) in split(area.w, in_row, gap) {
                    out.push(Rect {
                        x: area.x + x,
                        y: area.y + y,
                        w,
                        h,
                    });
                }
            }
            out
        }
    }
}

/// Where each window was before a tiling change, newest last. `T` is whatever names a window.
#[derive(Clone, Debug, Default)]
pub struct History<T: Clone> {
    steps: Vec<Vec<(T, Rect)>>,
}

impl<T: Clone> History<T> {
    const KEEP: usize = 20;

    pub fn new() -> Self {
        History { steps: Vec::new() }
    }

    pub fn record(&mut self, before: Vec<(T, Rect)>) {
        if before.is_empty() {
            return;
        }
        self.steps.push(before);
        if self.steps.len() > Self::KEEP {
            self.steps.remove(0);
        }
    }

    /// The positions to restore, or `None` when there is nothing to undo.
    pub fn undo(&mut self) -> Option<Vec<(T, Rect)>> {
        self.steps.pop()
    }

    pub fn can_undo(&self) -> bool {
        !self.steps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect {
        x: 0,
        y: 40,
        w: 1000,
        h: 700,
    };

    fn covered(rects: &[Rect]) -> i64 {
        rects.iter().map(|r| r.w as i64 * r.h as i64).sum()
    }

    #[test]
    fn side_by_side_fills_the_width() {
        let r = tile(Layout::SideBySide, AREA, 3, 0);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].x, 0);
        assert_eq!(r[2].x + r[2].w, 1000);
        assert!(r.iter().all(|t| t.h == 700 && t.y == 40));
        assert_eq!(covered(&r), 1000 * 700);
    }

    #[test]
    fn stacked_fills_the_height_with_gaps() {
        let r = tile(Layout::Stacked, AREA, 2, 10);
        assert_eq!(r[0].h + r[1].h + 10, 700);
        assert_eq!(r[1].y, r[0].y + r[0].h + 10);
    }

    #[test]
    fn grid_of_five_has_two_rows_and_a_wide_last_row() {
        let r = tile(Layout::Grid, AREA, 5, 0);
        assert_eq!(r.len(), 5);
        // 3 columns, 2 rows: three on top, two below that fill the width.
        assert_eq!(r.iter().filter(|t| t.y == 40).count(), 3);
        assert_eq!(r[3].w + r[4].w, 1000);
        assert_eq!(covered(&r), 1000 * 700);
    }

    #[test]
    fn grid_of_four_is_two_by_two_and_one_window_fills_the_area() {
        let r = tile(Layout::Grid, AREA, 4, 0);
        assert!(r.iter().all(|t| t.w == 500 && t.h == 350));
        assert_eq!(tile(Layout::Grid, AREA, 1, 0), vec![AREA]);
        assert!(tile(Layout::Grid, AREA, 0, 0).is_empty());
    }

    #[test]
    fn tiles_never_overlap() {
        for n in 1..=9 {
            for layout in [Layout::SideBySide, Layout::Stacked, Layout::Grid] {
                let r = tile(layout, AREA, n, 8);
                for (i, a) in r.iter().enumerate() {
                    for b in &r[i + 1..] {
                        let apart = a.x + a.w <= b.x
                            || b.x + b.w <= a.x
                            || a.y + a.h <= b.y
                            || b.y + b.h <= a.y;
                        assert!(apart, "{layout:?} {n}: {a:?} overlaps {b:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn undo_returns_the_last_positions_first() {
        let mut h: History<u32> = History::new();
        assert!(!h.can_undo());
        h.record(vec![(1, AREA)]);
        h.record(vec![(1, Rect { x: 5, ..AREA })]);
        assert_eq!(h.undo().unwrap()[0].1.x, 5);
        assert_eq!(h.undo().unwrap()[0].1.x, 0);
        assert!(h.undo().is_none());
    }

    #[test]
    fn history_is_bounded() {
        let mut h: History<u32> = History::new();
        for i in 0..50 {
            h.record(vec![(i, AREA)]);
        }
        let mut n = 0;
        while h.undo().is_some() {
            n += 1;
        }
        assert_eq!(n, 20);
    }
}
