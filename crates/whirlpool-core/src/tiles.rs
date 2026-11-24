//! Editable tile grid for the quick settings panel: packing, drag reorder, resize, add and remove.
//!
//! The grid has two columns. A tile spans one column (small) or both (wide). Tiles are placed in list
//! order, left to right; a wide tile that does not fit in the current row starts the next one.

/// Columns of the grid.
pub const COLUMNS: usize = 2;

/// Where one tile sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// Position of the tile in the list.
    pub index: usize,
    pub col: usize,
    pub row: usize,
    /// Columns covered, 1 or 2.
    pub span: usize,
}

/// Places tiles given the column span of each. Spans are clamped to `1..=COLUMNS`.
pub fn pack(spans: &[usize]) -> Vec<Slot> {
    let mut slots = Vec::with_capacity(spans.len());
    let (mut col, mut row) = (0, 0);
    for (index, &span) in spans.iter().enumerate() {
        let span = span.clamp(1, COLUMNS);
        if col + span > COLUMNS {
            col = 0;
            row += 1;
        }
        slots.push(Slot {
            index,
            col,
            row,
            span,
        });
        col += span;
        if col >= COLUMNS {
            col = 0;
            row += 1;
        }
    }
    slots
}

/// Number of rows the tiles occupy.
pub fn rows(spans: &[usize]) -> usize {
    pack(spans)
        .iter()
        .map(|slot| slot.row + 1)
        .max()
        .unwrap_or(0)
}

/// Rectangle of a slot in grid-local pixels: `(x, y, width, height)`.
pub fn slot_rect(slot: &Slot, cell_w: f32, cell_h: f32, gap: f32) -> (f32, f32, f32, f32) {
    (
        slot.col as f32 * (cell_w + gap),
        slot.row as f32 * (cell_h + gap),
        slot.span as f32 * cell_w + (slot.span as f32 - 1.0) * gap,
        cell_h,
    )
}

/// Moves the item at `from` so that it ends at `to`. Out-of-range indices are ignored.
pub fn move_item<T>(items: &mut Vec<T>, from: usize, to: usize) {
    if from >= items.len() || to >= items.len() || from == to {
        return;
    }
    let item = items.remove(from);
    items.insert(to, item);
}

/// The list position a tile dragged to grid-local point `(x, y)` should take: the slot whose centre
/// is nearest. Below the last row it is the last position; an empty grid has none.
pub fn drop_index(
    spans: &[usize],
    x: f32,
    y: f32,
    cell_w: f32,
    cell_h: f32,
    gap: f32,
) -> Option<usize> {
    let slots = pack(spans);
    let last = slots.last()?;
    let bottom = (last.row + 1) as f32 * (cell_h + gap);
    if y > bottom {
        return Some(slots.len() - 1);
    }
    slots
        .iter()
        .map(|slot| {
            let (sx, sy, sw, sh) = slot_rect(slot, cell_w, cell_h, gap);
            let (dx, dy) = (x - (sx + sw / 2.0), y - (sy + sh / 2.0));
            (slot.index, dx * dx + dy * dy)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(index, _)| index)
}

/// Keys of `all` that are not in `used`, in the order of `all`: what "Add tile" can offer.
pub fn missing<'a>(all: &[&'a str], used: &[&str]) -> Vec<&'a str> {
    all.iter().copied().filter(|k| !used.contains(k)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_should_fill_rows_left_to_right() {
        let slots = pack(&[1, 1, 1]);
        assert_eq!((slots[1].col, slots[1].row), (1, 0));
        assert_eq!((slots[2].col, slots[2].row), (0, 1));
    }

    #[test]
    fn pack_should_start_a_new_row_for_a_wide_tile_after_a_single() {
        let slots = pack(&[1, 2, 1]);
        assert_eq!((slots[1].col, slots[1].row, slots[1].span), (0, 1, 2));
        assert_eq!((slots[2].col, slots[2].row), (0, 2));
        assert_eq!(rows(&[1, 2, 1]), 3);
    }

    #[test]
    fn pack_should_clamp_spans_and_handle_empty() {
        assert!(pack(&[]).is_empty());
        assert_eq!(pack(&[0, 9])[0].span, 1);
        assert_eq!(pack(&[0, 9])[1].span, 2);
        assert_eq!(rows(&[]), 0);
    }

    #[test]
    fn move_item_should_reorder_and_ignore_bad_indices() {
        let mut v = vec!['a', 'b', 'c', 'd'];
        move_item(&mut v, 0, 2);
        assert_eq!(v, ['b', 'c', 'a', 'd']);
        move_item(&mut v, 9, 0);
        move_item(&mut v, 1, 9);
        assert_eq!(v, ['b', 'c', 'a', 'd']);
    }

    #[test]
    fn drop_index_should_pick_the_nearest_slot() {
        // Cells 100x50 with a gap of 10: slot 0 at (0,0), slot 1 at (110,0), slot 2 at (0,60).
        let spans = [1, 1, 1];
        assert_eq!(drop_index(&spans, 20.0, 10.0, 100.0, 50.0, 10.0), Some(0));
        assert_eq!(drop_index(&spans, 200.0, 20.0, 100.0, 50.0, 10.0), Some(1));
        assert_eq!(drop_index(&spans, 30.0, 80.0, 100.0, 50.0, 10.0), Some(2));
        assert_eq!(drop_index(&spans, 30.0, 900.0, 100.0, 50.0, 10.0), Some(2));
        assert_eq!(drop_index(&[], 0.0, 0.0, 100.0, 50.0, 10.0), None);
    }

    #[test]
    fn slot_rect_should_span_the_gap_for_wide_tiles() {
        let slot = Slot {
            index: 0,
            col: 0,
            row: 1,
            span: 2,
        };
        assert_eq!(
            slot_rect(&slot, 100.0, 50.0, 10.0),
            (0.0, 60.0, 210.0, 50.0)
        );
    }

    #[test]
    fn missing_should_list_unused_kinds_in_order() {
        assert_eq!(missing(&["a", "b", "c"], &["b"]), ["a", "c"]);
    }
}
