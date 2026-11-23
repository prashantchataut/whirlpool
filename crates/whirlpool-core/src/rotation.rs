//! Which wallpaper comes next. Pure arithmetic so it is tested without a desktop.

/// Index into a list of `len` pictures for monitor `monitor` on rotation step `tick`.
///
/// Sequential walks the list in order. Shuffle walks a permutation that is re-drawn every full
/// cycle, so every picture shows once per cycle and none repeats back to back. Monitors on the
/// same tick get different pictures (offset by their index) whenever `len` allows.
#[must_use]
pub fn pick(len: usize, shuffle: bool, tick: u64, monitor: usize, seed: u64) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let step = tick.wrapping_add(monitor as u64);
    let position = (step % len as u64) as usize;
    if !shuffle {
        return Some(position);
    }
    let cycle = step / len as u64;
    Some(permutation(len, seed ^ cycle.wrapping_mul(0x9E37_79B9_7F4A_7C15))[position])
}

fn permutation(len: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    let mut state = seed | 1;
    for i in (1..len).rev() {
        // xorshift64*
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let r = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        order.swap(i, (r % (i as u64 + 1)) as usize);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_has_no_pick() {
        assert_eq!(pick(0, false, 3, 0, 1), None);
    }

    #[test]
    fn sequential_walks_in_order_and_wraps() {
        let seen: Vec<_> = (0..7)
            .map(|t| pick(3, false, t, 0, 0).unwrap_or(99))
            .collect();
        assert_eq!(seen, [0, 1, 2, 0, 1, 2, 0]);
    }

    #[test]
    fn shuffle_shows_every_picture_once_per_cycle() {
        for seed in [1, 7, 12345] {
            let mut cycle: Vec<_> = (0..10)
                .map(|t| pick(10, true, t, 0, seed).unwrap_or(99))
                .collect();
            cycle.sort_unstable();
            assert_eq!(cycle, (0..10).collect::<Vec<_>>());
        }
    }

    #[test]
    fn shuffle_is_deterministic_and_changes_between_cycles() {
        let first: Vec<_> = (0..8).map(|t| pick(8, true, t, 0, 42)).collect();
        assert_eq!(
            first,
            (0..8).map(|t| pick(8, true, t, 0, 42)).collect::<Vec<_>>()
        );
        let second: Vec<_> = (8..16).map(|t| pick(8, true, t, 0, 42)).collect();
        assert_ne!(first, second);
    }

    #[test]
    fn monitors_get_different_pictures() {
        for shuffle in [false, true] {
            for tick in 0..20 {
                let a = pick(6, shuffle, tick, 0, 9);
                let b = pick(6, shuffle, tick, 1, 9);
                let c = pick(6, shuffle, tick, 2, 9);
                assert!(a != b && b != c && a != c, "tick {tick} shuffle {shuffle}");
            }
        }
    }

    #[test]
    fn a_single_picture_always_repeats() {
        assert_eq!(pick(1, true, 5, 3, 8), Some(0));
    }
}
