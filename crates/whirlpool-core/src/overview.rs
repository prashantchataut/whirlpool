//! Window overview: where every window's card goes on the screen, and how the arrow keys move between
//! them. Pure geometry; the panel in `whirlpool-settings` draws the cards and DWM draws the live
//! previews into them.

/// One card: a preview of `aspect` (width over height) with a caption row of `caption` points below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The cards for `count` windows and the number of columns they were packed in.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub slots: Vec<Slot>,
    pub columns: usize,
}

/// Packs `count` cards into `width` by `height`, as large as fit, never wider than `max_width`.
/// Rows are centred, the last (shorter) row too. The result is in reading order.
#[must_use]
pub fn layout(
    count: usize,
    width: f32,
    height: f32,
    aspect: f32,
    caption: f32,
    gap: f32,
    max_width: f32,
) -> Grid {
    if count == 0 || width <= 0.0 || height <= 0.0 || aspect <= 0.0 {
        return Grid {
            slots: Vec::new(),
            columns: 0,
        };
    }
    let mut best: Option<(usize, f32)> = None;
    for columns in 1..=count {
        let rows = count.div_ceil(columns);
        let by_width = (width - gap * (columns as f32 - 1.0)) / columns as f32;
        let by_height = ((height - gap * (rows as f32 - 1.0)) / rows as f32 - caption) * aspect;
        let card = by_width.min(by_height).min(max_width);
        if card > 0.0 && best.is_none_or(|(_, size)| card > size + 0.01) {
            best = Some((columns, card));
        }
    }
    let (columns, card) = best.unwrap_or((1, 1.0));
    let rows = count.div_ceil(columns);
    let card_height = card / aspect + caption;
    let total_height = rows as f32 * card_height + gap * (rows as f32 - 1.0);
    let top = (height - total_height) / 2.0;
    let mut slots = Vec::with_capacity(count);
    for index in 0..count {
        let row = index / columns;
        let in_row = if row + 1 == rows && count % columns != 0 {
            count % columns
        } else {
            columns
        };
        let row_width = in_row as f32 * card + gap * (in_row as f32 - 1.0);
        let left = (width - row_width) / 2.0;
        let col = index % columns;
        slots.push(Slot {
            x: left + col as f32 * (card + gap),
            y: top + row as f32 * (card_height + gap),
            width: card,
            height: card_height,
        });
    }
    Grid { slots, columns }
}

/// The card selected after an arrow key. Left and right wrap around the list; up and down move a row
/// and stay put at the edges.
#[must_use]
pub fn step(selected: usize, count: usize, columns: usize, dx: i32, dy: i32) -> usize {
    if count == 0 {
        return 0;
    }
    let selected = selected.min(count - 1);
    let mut next = selected as i64;
    if dx != 0 {
        next = (next + i64::from(dx)).rem_euclid(count as i64);
    }
    if dy != 0 && columns > 0 {
        let moved = next + i64::from(dy) * columns as i64;
        if (0..count as i64).contains(&moved) {
            next = moved;
        }
    }
    next as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(grid: &Grid, width: f32, height: f32) -> bool {
        grid.slots.iter().all(|slot| {
            slot.x >= -0.01
                && slot.y >= -0.01
                && slot.x + slot.width <= width + 0.01
                && slot.y + slot.height <= height + 0.01
        })
    }

    fn overlap(a: &Slot, b: &Slot) -> bool {
        a.x < b.x + b.width - 0.01
            && b.x < a.x + a.width - 0.01
            && a.y < b.y + b.height - 0.01
            && b.y < a.y + a.height - 0.01
    }

    #[test]
    fn no_windows_no_cards() {
        assert!(
            layout(0, 1000.0, 700.0, 1.6, 30.0, 20.0, 480.0)
                .slots
                .is_empty()
        );
    }

    #[test]
    fn every_count_fits_and_nothing_overlaps() {
        for count in 1..=40 {
            let grid = layout(count, 1280.0, 720.0, 1.6, 34.0, 18.0, 520.0);
            assert_eq!(grid.slots.len(), count, "count {count}");
            assert!(
                inside(&grid, 1280.0, 720.0),
                "count {count} leaves the area"
            );
            for (i, a) in grid.slots.iter().enumerate() {
                for b in &grid.slots[i + 1..] {
                    assert!(!overlap(a, b), "count {count}: cards overlap");
                }
            }
        }
    }

    #[test]
    fn one_window_is_capped_not_stretched() {
        let grid = layout(1, 1920.0, 1080.0, 1.6, 34.0, 18.0, 520.0);
        assert!((grid.slots[0].width - 520.0).abs() < 0.01);
        // Centred.
        assert!((grid.slots[0].x - (1920.0 - 520.0) / 2.0).abs() < 0.01);
    }

    #[test]
    fn more_windows_make_smaller_cards() {
        let few = layout(2, 1280.0, 720.0, 1.6, 34.0, 18.0, 900.0).slots[0].width;
        let many = layout(12, 1280.0, 720.0, 1.6, 34.0, 18.0, 900.0).slots[0].width;
        assert!(many < few);
    }

    #[test]
    fn short_last_row_is_centred() {
        let grid = layout(5, 1200.0, 800.0, 1.6, 30.0, 20.0, 400.0);
        let last = grid.slots[4];
        let first_row_start = grid.slots[0].x;
        if grid.columns > 1 && 5 % grid.columns != 0 {
            assert!(last.x > first_row_start);
        }
    }

    #[test]
    fn arrows_wrap_sideways_and_stop_vertically() {
        assert_eq!(step(0, 5, 3, -1, 0), 4);
        assert_eq!(step(4, 5, 3, 1, 0), 0);
        assert_eq!(step(0, 5, 3, 0, 1), 3);
        assert_eq!(step(3, 5, 3, 0, 1), 3, "no row below");
        assert_eq!(step(1, 5, 3, 0, -1), 1, "no row above");
        assert_eq!(step(9, 5, 3, 0, 0), 4, "stale selection is clamped");
        assert_eq!(step(0, 0, 3, 1, 0), 0);
    }
}
