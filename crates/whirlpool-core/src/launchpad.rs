//! Launchpad-style Start grid: how many icons fit, which page an app is on, where a slot is and
//! what a click hit, and search. Pure arithmetic; drawing and input belong to the caller.

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    #[must_use]
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// One page of icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub columns: u32,
    pub rows: u32,
}

impl Grid {
    #[must_use]
    pub const fn per_page(self) -> usize {
        (self.columns * self.rows) as usize
    }
}

/// Picks a grid for an area. `cell` is the icon cell edge (icon plus label) and `gap` the space
/// between cells, both in the caller's units. At least 2 x 2, at most `max_columns` wide.
#[must_use]
pub fn fit(width: f32, height: f32, cell: f32, gap: f32, max_columns: u32) -> Grid {
    let per = |extent: f32| ((extent + gap) / (cell + gap)).floor().max(2.0) as u32;
    Grid {
        columns: per(width).min(max_columns.max(2)),
        rows: per(height),
    }
}

#[must_use]
pub fn page_count(items: usize, grid: Grid) -> usize {
    items.div_ceil(grid.per_page().max(1)).max(1)
}

/// Indices of the items shown on `page` (clamped to the last page).
#[must_use]
pub fn page_range(page: usize, items: usize, grid: Grid) -> Range<usize> {
    let per = grid.per_page().max(1);
    let page = page.min(page_count(items, grid) - 1);
    let start = (page * per).min(items);
    start..(start + per).min(items)
}

/// The cell for `slot` (0-based within the page), with the grid centred in `area`.
#[must_use]
pub fn slot_rect(grid: Grid, area: Rect, cell: f32, gap: f32, slot: usize) -> Rect {
    let (columns, rows) = (grid.columns as f32, grid.rows as f32);
    let used_w = columns * cell + (columns - 1.0) * gap;
    let used_h = rows * cell + (rows - 1.0) * gap;
    let left = area.x + ((area.w - used_w) / 2.0).max(0.0);
    let top = area.y + ((area.h - used_h) / 2.0).max(0.0);
    let (column, row) = (
        (slot as u32 % grid.columns) as f32,
        (slot as u32 / grid.columns) as f32,
    );
    Rect {
        x: left + column * (cell + gap),
        y: top + row * (cell + gap),
        w: cell,
        h: cell,
    }
}

/// The slot under a point, if any (the gaps between cells hit nothing).
#[must_use]
pub fn hit(grid: Grid, area: Rect, cell: f32, gap: f32, x: f32, y: f32) -> Option<usize> {
    (0..grid.per_page()).find(|&slot| slot_rect(grid, area, cell, gap, slot).contains(x, y))
}

/// The page after a horizontal swipe: a drag of more than a third of the width turns the page.
#[must_use]
pub fn swiped_page(current: usize, pages: usize, drag_x: f32, width: f32) -> usize {
    let last = pages.saturating_sub(1);
    if drag_x <= -width / 3.0 {
        (current + 1).min(last)
    } else if drag_x >= width / 3.0 {
        current.saturating_sub(1)
    } else {
        current.min(last)
    }
}

/// Indices of `names` that match `query`, best first: names that start with it, then words that
/// start with it, then names that contain it. Case-insensitive, stable within each tier. An empty
/// query matches everything in the original order.
#[must_use]
pub fn search(names: &[&str], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return (0..names.len()).collect();
    }
    let mut tiers: [Vec<usize>; 3] = Default::default();
    for (index, name) in names.iter().enumerate() {
        let name = name.to_lowercase();
        if name.starts_with(&query) {
            tiers[0].push(index);
        } else if name.split_whitespace().any(|word| word.starts_with(&query)) {
            tiers[1].push(index);
        } else if name.contains(&query) {
            tiers[2].push(index);
        }
    }
    tiers.into_iter().flatten().collect()
}

/// A key that moves the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

/// The page an item is on.
#[must_use]
pub fn page_of(index: usize, grid: Grid) -> usize {
    index / grid.per_page().max(1)
}

/// Where the selection goes when `nav` is pressed. Left and right walk the items in order, so they
/// run on to the next page; up and down move a row and cross to the neighbouring page from the top or
/// bottom row; a short last row is reached by going down from the row above it. Never leaves
/// `0..count`.
#[must_use]
pub fn navigate(current: usize, count: usize, grid: Grid, nav: Nav) -> usize {
    if count == 0 {
        return 0;
    }
    let last = count - 1;
    let current = current.min(last);
    let columns = grid.columns.max(1) as usize;
    let per = grid.per_page().max(1);
    let slot = current % per;
    let page = current / per;
    match nav {
        Nav::Left => current.saturating_sub(1),
        Nav::Right => (current + 1).min(last),
        Nav::Home => 0,
        Nav::End => last,
        Nav::Up => {
            if slot >= columns {
                current - columns
            } else if page > 0 {
                // Bottom row of the previous page, same column.
                (page - 1) * per + (per - columns) + slot % columns
            } else {
                current
            }
        }
        Nav::Down => {
            let below = current + columns;
            if slot + columns < per {
                if below <= last {
                    below
                } else if last / columns > current / columns {
                    last
                } else {
                    current
                }
            } else if (page + 1) * per <= last {
                // Top row of the next page, same column.
                ((page + 1) * per + slot % columns).min(last)
            } else {
                current
            }
        }
        Nav::PageUp => {
            if page > 0 {
                (current - per).min(last)
            } else {
                0
            }
        }
        Nav::PageDown => {
            if (page + 1) * per <= last {
                (current + per).min(last)
            } else {
                last
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1000.0,
        h: 600.0,
    };

    #[test]
    fn the_grid_fits_the_area_and_never_collapses() {
        let grid = fit(1000.0, 600.0, 120.0, 24.0, 8);
        assert_eq!(
            grid,
            Grid {
                columns: 7,
                rows: 4
            }
        );
        assert_eq!(
            fit(50.0, 50.0, 120.0, 24.0, 8),
            Grid {
                columns: 2,
                rows: 2
            }
        );
        assert_eq!(fit(4000.0, 600.0, 120.0, 24.0, 6).columns, 6);
    }

    #[test]
    fn paging_covers_every_item_exactly_once() {
        let grid = Grid {
            columns: 7,
            rows: 4,
        };
        assert_eq!(page_count(0, grid), 1);
        assert_eq!(page_count(28, grid), 1);
        assert_eq!(page_count(29, grid), 2);
        let mut seen = Vec::new();
        for page in 0..page_count(60, grid) {
            seen.extend(page_range(page, 60, grid));
        }
        assert_eq!(seen, (0..60).collect::<Vec<_>>());
        assert_eq!(page_range(99, 60, grid), 56..60, "past the end: last page");
        assert_eq!(page_range(0, 0, grid), 0..0);
    }

    #[test]
    fn a_click_finds_the_slot_that_was_drawn_there() {
        let grid = Grid {
            columns: 7,
            rows: 4,
        };
        for slot in 0..grid.per_page() {
            let r = slot_rect(grid, AREA, 120.0, 24.0, slot);
            assert_eq!(
                hit(grid, AREA, 120.0, 24.0, r.x + r.w / 2.0, r.y + r.h / 2.0),
                Some(slot)
            );
        }
        let first = slot_rect(grid, AREA, 120.0, 24.0, 0);
        let second = slot_rect(grid, AREA, 120.0, 24.0, 1);
        let between = (first.x + first.w + second.x) / 2.0;
        assert_eq!(hit(grid, AREA, 120.0, 24.0, between, first.y + 10.0), None);
        assert_eq!(hit(grid, AREA, 120.0, 24.0, -5.0, -5.0), None);
    }

    #[test]
    fn the_grid_is_centred() {
        let grid = Grid {
            columns: 7,
            rows: 4,
        };
        let first = slot_rect(grid, AREA, 120.0, 24.0, 0);
        let last = slot_rect(grid, AREA, 120.0, 24.0, 6);
        assert!((first.x - (AREA.w - (last.x + last.w))).abs() < 0.01);
    }

    #[test]
    fn swipes_turn_pages_only_when_far_enough() {
        assert_eq!(swiped_page(0, 3, -400.0, 900.0), 1);
        assert_eq!(swiped_page(1, 3, 400.0, 900.0), 0);
        assert_eq!(swiped_page(0, 3, -100.0, 900.0), 0);
        assert_eq!(
            swiped_page(2, 3, -900.0, 900.0),
            2,
            "no page after the last"
        );
        assert_eq!(swiped_page(0, 3, 900.0, 900.0), 0, "none before the first");
    }

    #[test]
    fn search_ranks_prefix_then_word_then_substring() {
        let names = [
            "Notepad",
            "Windows Terminal",
            "Terminal Services",
            "Cortana",
            "Pinterest",
        ];
        assert_eq!(search(&names, "ter"), vec![2, 1, 4]);
        assert_eq!(search(&names, ""), vec![0, 1, 2, 3, 4]);
        assert_eq!(search(&names, "  NOTE "), vec![0]);
        assert!(search(&names, "zzz").is_empty());
    }

    #[test]
    fn keyboard_navigation_walks_rows_pages_and_short_rows() {
        let grid = Grid {
            columns: 4,
            rows: 3,
        }; // 12 per page
        let count = 30; // pages of 12, 12 and 6 (rows of 4 and 2)
        let go = |from, nav| navigate(from, count, grid, nav);
        assert_eq!(go(0, Nav::Left), 0, "stays at the start");
        assert_eq!(go(11, Nav::Right), 12, "runs on to the next page");
        assert_eq!(go(29, Nav::Right), 29, "stays at the end");
        assert_eq!(go(1, Nav::Down), 5);
        assert_eq!(go(5, Nav::Up), 1);
        assert_eq!(go(2, Nav::Up), 2, "top of the first page");
        assert_eq!(
            go(10, Nav::Down),
            14,
            "bottom row goes to the next page, same column"
        );
        assert_eq!(
            go(13, Nav::Up),
            9,
            "top row goes back to the previous page's bottom row"
        );
        // Last page holds 24..30: row 0 is 24..28, row 1 is 28..30.
        assert_eq!(
            go(27, Nav::Down),
            29,
            "a short last row is reached by its last item"
        );
        assert_eq!(go(29, Nav::Down), 29);
        assert_eq!(go(5, Nav::PageDown), 17);
        assert_eq!(go(26, Nav::PageDown), 29, "last page: to the very end");
        assert_eq!(go(17, Nav::PageUp), 5);
        assert_eq!(go(3, Nav::PageUp), 0);
        assert_eq!(go(7, Nav::Home), 0);
        assert_eq!(go(7, Nav::End), 29);
        assert_eq!(
            navigate(0, 0, grid, Nav::Down),
            0,
            "an empty list has no selection to move"
        );
        assert_eq!(
            navigate(99, 5, grid, Nav::Left),
            3,
            "a stale index is clamped first"
        );
        assert_eq!(page_of(11, grid), 0);
        assert_eq!(page_of(12, grid), 1);
        // Every key from every position stays in range.
        for from in 0..count {
            for nav in [
                Nav::Left,
                Nav::Right,
                Nav::Up,
                Nav::Down,
                Nav::Home,
                Nav::End,
                Nav::PageUp,
                Nav::PageDown,
            ] {
                assert!(go(from, nav) < count);
            }
        }
    }
}
