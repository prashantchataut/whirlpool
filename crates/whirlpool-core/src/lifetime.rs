//! When a heavy panel (overview, Start grid, clipboard) should exit after it was hidden. The dock keeps
//! only a tiny resident process; a panel that is hidden and idle for `grace` is closed, unless a job
//! (a copy, a download listing) is still running.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Visible,
    /// Hidden since this many milliseconds (monotonic clock of the caller).
    HiddenSince(u64),
}

#[derive(Clone, Copy, Debug)]
pub struct Lifetime {
    state: State,
    grace_ms: u64,
}

impl Lifetime {
    pub fn new(grace_ms: u64) -> Self {
        Lifetime {
            state: State::Visible,
            grace_ms,
        }
    }

    pub fn shown(&mut self) {
        self.state = State::Visible;
    }

    pub fn hidden(&mut self, now_ms: u64) {
        if self.state == State::Visible {
            self.state = State::HiddenSince(now_ms);
        }
    }

    /// True once the panel has been hidden for the grace period and no job is running.
    pub fn should_exit(&self, now_ms: u64, jobs_running: bool) -> bool {
        match self.state {
            State::Visible => false,
            State::HiddenSince(at) => !jobs_running && now_ms.saturating_sub(at) >= self.grace_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exits_after_the_grace_period() {
        let mut l = Lifetime::new(20_000);
        assert!(!l.should_exit(1_000_000, false));
        l.hidden(1_000);
        assert!(!l.should_exit(10_000, false));
        assert!(l.should_exit(21_000, false));
    }

    #[test]
    fn showing_again_cancels_and_jobs_hold_it_open() {
        let mut l = Lifetime::new(5_000);
        l.hidden(0);
        assert!(!l.should_exit(60_000, true));
        l.shown();
        assert!(!l.should_exit(60_000, false));
        l.hidden(60_000);
        l.hidden(61_000); // a second hide does not restart the clock
        assert!(l.should_exit(65_000, false));
    }
}
