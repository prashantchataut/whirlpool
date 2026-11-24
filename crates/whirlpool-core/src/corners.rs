//! Hot corners: a pointer that rests in a screen corner fires once; it fires again only after the
//! pointer has left the corner area. Pure state machine; the dock feeds it cursor samples.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Pixels from the screen edge that count as "in the corner" on each axis.
pub const HOT_PX: i32 = 3;
/// The pointer must be farther than this from the corner to re-arm it.
pub const REARM_PX: i32 = 40;
/// How long the pointer must rest before it fires.
pub const DWELL_MS: u64 = 350;

/// Which corner (if any) the point is in. `screen` is `(left, top, right, bottom)`, right and bottom
/// exclusive like a Windows `RECT`.
pub fn corner_at(screen: (i32, i32, i32, i32), x: i32, y: i32) -> Option<Corner> {
    let (l, t, r, b) = screen;
    if x < l || y < t || x >= r || y >= b {
        return None;
    }
    let left = x - l < HOT_PX;
    let right = r - 1 - x < HOT_PX;
    let top = y - t < HOT_PX;
    let bottom = b - 1 - y < HOT_PX;
    match (left, right, top, bottom) {
        (true, _, true, _) => Some(Corner::TopLeft),
        (_, true, true, _) => Some(Corner::TopRight),
        (true, _, _, true) => Some(Corner::BottomLeft),
        (_, true, _, true) => Some(Corner::BottomRight),
        _ => None,
    }
}

fn near_any_corner(screen: (i32, i32, i32, i32), x: i32, y: i32) -> bool {
    let (l, t, r, b) = screen;
    let near_x = (x - l) < REARM_PX || (r - 1 - x) < REARM_PX;
    let near_y = (y - t) < REARM_PX || (b - 1 - y) < REARM_PX;
    near_x && near_y
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Tracker {
    inside: Option<(Corner, u64)>,
    fired: bool,
}

impl Tracker {
    /// Feed one cursor sample. Returns the corner exactly once per visit, after the dwell.
    pub fn sample(
        &mut self,
        now_ms: u64,
        screen: (i32, i32, i32, i32),
        x: i32,
        y: i32,
    ) -> Option<Corner> {
        match corner_at(screen, x, y) {
            Some(corner) => {
                let since = match self.inside {
                    Some((c, since)) if c == corner => since,
                    _ => {
                        self.inside = Some((corner, now_ms));
                        self.fired = false;
                        now_ms
                    }
                };
                if !self.fired && now_ms.saturating_sub(since) >= DWELL_MS {
                    self.fired = true;
                    return Some(corner);
                }
                None
            }
            None => {
                if !near_any_corner(screen, x, y) {
                    self.inside = None;
                    self.fired = false;
                }
                None
            }
        }
    }

    /// Whether the dock still needs to sample quickly: the pointer is in a corner and has not fired.
    pub fn waiting(&self) -> bool {
        self.inside.is_some() && !self.fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const S: (i32, i32, i32, i32) = (0, 0, 1920, 1080);

    #[test]
    fn corner_at_should_find_all_four_and_nothing_else() {
        assert_eq!(corner_at(S, 0, 0), Some(Corner::TopLeft));
        assert_eq!(corner_at(S, 1919, 0), Some(Corner::TopRight));
        assert_eq!(corner_at(S, 0, 1079), Some(Corner::BottomLeft));
        assert_eq!(corner_at(S, 1919, 1079), Some(Corner::BottomRight));
        assert_eq!(corner_at(S, 960, 0), None);
        assert_eq!(corner_at(S, 0, 540), None);
        assert_eq!(corner_at(S, 1920, 0), None);
    }

    #[test]
    fn corner_at_should_work_on_a_second_monitor() {
        let second = (1920, 0, 3840, 1080);
        assert_eq!(corner_at(second, 1920, 0), Some(Corner::TopLeft));
        assert_eq!(corner_at(second, 3839, 1079), Some(Corner::BottomRight));
    }

    #[test]
    fn tracker_should_fire_once_after_the_dwell() {
        let mut t = Tracker::default();
        assert_eq!(t.sample(0, S, 0, 0), None);
        assert!(t.waiting());
        assert_eq!(t.sample(DWELL_MS - 1, S, 0, 0), None);
        assert_eq!(t.sample(DWELL_MS, S, 0, 0), Some(Corner::TopLeft));
        assert_eq!(t.sample(DWELL_MS * 3, S, 1, 1), None);
        assert!(!t.waiting());
    }

    #[test]
    fn tracker_should_not_fire_for_a_pass_through() {
        let mut t = Tracker::default();
        t.sample(0, S, 0, 0);
        t.sample(100, S, 200, 200);
        assert_eq!(t.sample(600, S, 200, 200), None);
        assert!(!t.waiting());
    }

    #[test]
    fn tracker_should_rearm_only_after_leaving_the_area() {
        let mut t = Tracker::default();
        t.sample(0, S, 0, 0);
        assert!(t.sample(400, S, 0, 0).is_some());
        // Nudged just outside the hot pixels but still near the corner: no second fire.
        t.sample(500, S, 10, 10);
        assert_eq!(t.sample(1_500, S, 0, 0), None);
        // Gone far away, then back.
        t.sample(2_000, S, 500, 500);
        t.sample(2_100, S, 0, 0);
        assert!(t.sample(2_100 + DWELL_MS, S, 0, 0).is_some());
    }
}
