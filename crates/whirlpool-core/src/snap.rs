//! Snap zones and the numbers behind the drag shadow, as pure arithmetic.
//!
//! Windows cannot transform another program's window, so drag motion is made of things Whirlpool can
//! draw itself: a shadow behind the dragged window, a preview of where it would snap, and a spring
//! that carries it to the snapped (or rescued) position after release.

/// A rectangle in device pixels, edges as Windows reports them (`right` and `bottom` exclusive).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Area {
    #[must_use]
    pub const fn width(self) -> i32 {
        self.right - self.left
    }
    #[must_use]
    pub const fn height(self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Left,
    Right,
    /// The top edge: the window fills the work area.
    Maximize,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// The zone the pointer is in, if any. `edge` is how close to a side counts as touching it and
/// `corner` how far along a side a corner reaches. Left and right win over the top edge, as in
/// Windows, and the corners are the ends of the sides.
#[must_use]
pub fn zone_at(x: i32, y: i32, work: Area, edge: i32, corner: i32) -> Option<Zone> {
    let at_left = x < work.left + edge;
    let at_right = x >= work.right - edge;
    let near_top = y < work.top + corner;
    let near_bottom = y >= work.bottom - corner;
    match (at_left, at_right) {
        (true, _) if near_top => Some(Zone::TopLeft),
        (true, _) if near_bottom => Some(Zone::BottomLeft),
        (true, _) => Some(Zone::Left),
        (_, true) if near_top => Some(Zone::TopRight),
        (_, true) if near_bottom => Some(Zone::BottomRight),
        (_, true) => Some(Zone::Right),
        _ if y < work.top + edge => Some(Zone::Maximize),
        _ => None,
    }
}

/// Where a window in `zone` goes.
#[must_use]
pub fn zone_rect(zone: Zone, work: Area) -> Area {
    let mid_x = work.left + work.width() / 2;
    let mid_y = work.top + work.height() / 2;
    let (left, top, right, bottom) = match zone {
        Zone::Maximize => (work.left, work.top, work.right, work.bottom),
        Zone::Left => (work.left, work.top, mid_x, work.bottom),
        Zone::Right => (mid_x, work.top, work.right, work.bottom),
        Zone::TopLeft => (work.left, work.top, mid_x, mid_y),
        Zone::TopRight => (mid_x, work.top, work.right, mid_y),
        Zone::BottomLeft => (work.left, mid_y, mid_x, work.bottom),
        Zone::BottomRight => (mid_x, mid_y, work.right, work.bottom),
    };
    Area {
        left,
        top,
        right,
        bottom,
    }
}

/// The same-sized rectangle moved the least that leaves the window reachable: at least `grab` pixels
/// of it inside the work area horizontally, and its title bar (the top) inside vertically. A window
/// that is already reachable comes back unchanged.
#[must_use]
pub fn rescue(rect: Area, work: Area, grab: i32, title: i32) -> Area {
    let grab = grab.min(rect.width());
    let left = rect
        .left
        .clamp(work.left - (rect.width() - grab), work.right - grab);
    let top = rect
        .top
        .clamp(work.top, (work.bottom - title).max(work.top));
    Area {
        left,
        top,
        right: left + rect.width(),
        bottom: top + rect.height(),
    }
}

/// One edge of a blurred rectangle's shadow along one axis: 0 far outside, 1 deep inside, a smooth
/// ramp of width about `sigma` at each end. A rectangle blurred by a Gaussian is the product of this
/// along x and along y, which is how the shadow bitmap is made without a blur pass.
#[must_use]
pub fn shadow_profile(position: f32, start: f32, end: f32, sigma: f32) -> f32 {
    let step = |t: f32| 1.0 / (1.0 + (-1.702 * t / sigma.max(0.001)).exp());
    (step(position - start) - step(position - end)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Area = Area {
        left: 0,
        top: 0,
        right: 1000,
        bottom: 800,
    };

    #[test]
    fn zones_follow_the_pointer_to_sides_corners_and_top() {
        let zone = |x, y| zone_at(x, y, WORK, 8, 120);
        assert_eq!(zone(500, 400), None);
        assert_eq!(zone(2, 400), Some(Zone::Left));
        assert_eq!(zone(997, 400), Some(Zone::Right));
        assert_eq!(zone(500, 3), Some(Zone::Maximize));
        assert_eq!(zone(2, 30), Some(Zone::TopLeft));
        assert_eq!(zone(995, 30), Some(Zone::TopRight));
        assert_eq!(zone(2, 790), Some(Zone::BottomLeft));
        assert_eq!(zone(999, 790), Some(Zone::BottomRight));
        assert_eq!(
            zone(-40, 400),
            Some(Zone::Left),
            "past the edge still counts"
        );
        assert_eq!(zone(500, 790), None, "the bottom edge snaps nothing");
    }

    #[test]
    fn zone_rects_tile_the_work_area() {
        let r = |zone| zone_rect(zone, WORK);
        assert_eq!(r(Zone::Maximize), WORK);
        assert_eq!(r(Zone::Left).right, r(Zone::Right).left);
        assert_eq!(r(Zone::Left).width() + r(Zone::Right).width(), 1000);
        assert_eq!(r(Zone::TopLeft).bottom, r(Zone::BottomLeft).top);
        assert_eq!(r(Zone::TopRight).left, r(Zone::TopLeft).right);
        assert_eq!(r(Zone::BottomRight).right, 1000);
        assert_eq!(r(Zone::BottomRight).bottom, 800);
    }

    #[test]
    fn a_reachable_window_is_left_alone_and_a_lost_one_is_pulled_back() {
        let window = Area {
            left: 100,
            top: 80,
            right: 700,
            bottom: 500,
        };
        assert_eq!(rescue(window, WORK, 80, 48), window);
        let above = Area {
            top: -200,
            bottom: 220,
            ..window
        };
        assert_eq!(
            rescue(above, WORK, 80, 48).top,
            0,
            "title bar comes back under the bar"
        );
        let right = Area {
            left: 990,
            right: 1590,
            ..window
        };
        let back = rescue(right, WORK, 80, 48);
        assert_eq!(back.left, 920);
        assert_eq!(back.width(), 600, "size is never changed");
        let left = Area {
            left: -900,
            right: -300,
            ..window
        };
        assert_eq!(rescue(left, WORK, 80, 48).right, 80);
        let low = Area {
            top: 900,
            bottom: 1320,
            ..window
        };
        assert_eq!(rescue(low, WORK, 80, 48).top, 752);
    }

    #[test]
    fn the_shadow_is_dark_inside_and_fades_out() {
        let at = |x| shadow_profile(x, 40.0, 440.0, 12.0);
        assert!(at(240.0) > 0.99);
        assert!(at(-50.0) < 0.001);
        assert!(at(40.0) > 0.4 && at(40.0) < 0.6, "half way at the edge");
        assert!(at(20.0) < at(40.0) && at(40.0) < at(60.0));
        assert!((at(40.0) - at(440.0)).abs() < 0.01, "symmetric");
    }
}
