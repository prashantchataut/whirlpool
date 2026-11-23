//! Motion for a window being dragged: it follows the pointer on a spring, leans into the
//! direction of travel and rises slightly while held, then settles when released.
//!
//! This is the model only (a pose per frame from pointer samples). It uses the same critically
//! damped [`Spring`] as the dock, so it is stable at any frame duration and never overshoots.

use crate::spring::Spring;

/// What to draw this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    /// Degrees; positive leans right. Zero at rest.
    pub tilt: f32,
    /// 1.0 at rest, slightly larger while held.
    pub scale: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct DragMotion {
    x: Spring,
    y: Spring,
    scale: Spring,
    tilt: Spring,
    held: bool,
    last: Option<(f32, f32)>,
}

/// Pixels per second of horizontal speed that gives the full lean.
const FULL_LEAN_SPEED: f32 = 2400.0;
const MAX_TILT_DEGREES: f32 = 4.0;
const HELD_SCALE: f32 = 1.02;

impl DragMotion {
    /// `follow` is how tightly the window trails the pointer (angular frequency; about 18 is
    /// soft, 40 is tight).
    #[must_use]
    pub fn new(x: f32, y: f32, follow: f32) -> Self {
        Self {
            x: Spring::at(x, follow),
            y: Spring::at(y, follow),
            scale: Spring::at(1.0, 22.0),
            tilt: Spring::at(0.0, 14.0),
            held: false,
            last: None,
        }
    }

    /// The window was picked up at this point.
    pub fn grab(&mut self, x: f32, y: f32) {
        self.x.value = x;
        self.x.target = x;
        self.y.value = y;
        self.y.target = y;
        self.x.velocity = 0.0;
        self.y.velocity = 0.0;
        self.scale.target = HELD_SCALE;
        self.held = true;
        self.last = Some((x, y));
    }

    /// The pointer moved to (x, y); `dt` seconds since the last sample.
    pub fn drag(&mut self, x: f32, y: f32, dt: f32) {
        if !self.held {
            return;
        }
        if let Some((last_x, _)) = self.last
            && dt > 0.0
        {
            let speed = (x - last_x) / dt;
            self.tilt.target = (speed / FULL_LEAN_SPEED).clamp(-1.0, 1.0) * MAX_TILT_DEGREES;
        }
        self.x.target = x;
        self.y.target = y;
        self.last = Some((x, y));
    }

    /// Let go: the window glides to where the pointer was and levels out.
    pub fn release(&mut self) {
        self.held = false;
        self.scale.target = 1.0;
        self.tilt.target = 0.0;
    }

    /// Advances one frame. A pointer that stopped moving relaxes the lean on its own.
    pub fn step(&mut self, dt: f32) {
        if self.held {
            self.tilt.target *= (-8.0 * dt).exp();
        }
        for spring in [&mut self.x, &mut self.y, &mut self.scale, &mut self.tilt] {
            spring.step(dt);
        }
    }

    #[must_use]
    pub fn pose(&self) -> Pose {
        Pose {
            x: self.x.value,
            y: self.y.value,
            tilt: self.tilt.value,
            scale: self.scale.value,
        }
    }

    /// Nothing left to animate: stop the frame timer.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        !self.held
            && self.x.is_settled()
            && self.y.is_settled()
            && self.scale.is_settled()
            && self.tilt.is_settled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut DragMotion, frames: usize, dt: f32) {
        for _ in 0..frames {
            m.step(dt);
        }
    }

    #[test]
    fn the_window_trails_the_pointer_without_overshooting() {
        let mut m = DragMotion::new(0.0, 0.0, 24.0);
        m.grab(0.0, 0.0);
        m.drag(300.0, 120.0, 0.016);
        let mut last = m.pose().x;
        for _ in 0..240 {
            m.step(1.0 / 120.0);
            let x = m.pose().x;
            assert!(
                x >= last && x <= 300.0 + 0.001,
                "monotonic, no overshoot: {x}"
            );
            last = x;
        }
        assert!((m.pose().x - 300.0).abs() < 0.5);
        assert!((m.pose().y - 120.0).abs() < 0.5);
    }

    #[test]
    fn it_leans_into_the_direction_of_travel_and_never_too_far() {
        let mut right = DragMotion::new(0.0, 0.0, 24.0);
        right.grab(0.0, 0.0);
        right.drag(60.0, 0.0, 0.016);
        run(&mut right, 6, 0.016);
        assert!(right.pose().tilt > 0.0);

        let mut left = DragMotion::new(500.0, 0.0, 24.0);
        left.grab(500.0, 0.0);
        left.drag(440.0, 0.0, 0.016);
        run(&mut left, 6, 0.016);
        assert!(left.pose().tilt < 0.0);

        let mut flick = DragMotion::new(0.0, 0.0, 24.0);
        flick.grab(0.0, 0.0);
        flick.drag(100_000.0, 0.0, 0.001);
        run(&mut flick, 60, 0.016);
        assert!(flick.pose().tilt.abs() <= MAX_TILT_DEGREES + 0.001);
    }

    #[test]
    fn it_rises_when_held_and_settles_after_release() {
        let mut m = DragMotion::new(10.0, 10.0, 24.0);
        assert!(m.is_settled());
        m.grab(10.0, 10.0);
        run(&mut m, 120, 0.016);
        assert!((m.pose().scale - HELD_SCALE).abs() < 0.001);
        assert!(!m.is_settled(), "held is never settled");
        m.drag(200.0, 50.0, 0.016);
        m.release();
        run(&mut m, 600, 0.016);
        assert!(m.is_settled());
        let pose = m.pose();
        assert_eq!((pose.scale, pose.tilt), (1.0, 0.0));
        assert!((pose.x - 200.0).abs() < 0.01);
    }

    #[test]
    fn a_pointer_that_stops_straightens_the_window() {
        let mut m = DragMotion::new(0.0, 0.0, 24.0);
        m.grab(0.0, 0.0);
        m.drag(80.0, 0.0, 0.016);
        run(&mut m, 5, 0.016);
        let leaning = m.pose().tilt;
        run(&mut m, 120, 0.016);
        assert!(m.pose().tilt.abs() < leaning.abs() * 0.1);
    }

    #[test]
    fn a_huge_frame_does_not_blow_up() {
        let mut m = DragMotion::new(0.0, 0.0, 40.0);
        m.grab(0.0, 0.0);
        m.drag(500.0, 500.0, 0.016);
        m.step(5.0);
        let pose = m.pose();
        assert!(pose.x.is_finite() && pose.x <= 500.001 && pose.y <= 500.001);
    }

    #[test]
    fn dragging_without_a_grab_does_nothing() {
        let mut m = DragMotion::new(5.0, 5.0, 24.0);
        m.drag(900.0, 900.0, 0.016);
        m.step(0.016);
        assert_eq!((m.pose().x, m.pose().y), (5.0, 5.0));
        assert!(m.is_settled());
    }
}
