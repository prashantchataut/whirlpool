//! Focus timer: focus, short break, long break. A pure state machine; the caller passes elapsed
//! time, so nothing here reads a clock and nothing ticks unless the timer is running.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub focus_min: u32,
    pub short_min: u32,
    pub long_min: u32,
    /// A long break after this many focus sessions.
    pub rounds: u32,
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            focus_min: 25,
            short_min: 5,
            long_min: 15,
            rounds: 4,
        }
    }
}

impl Plan {
    /// A plan from minutes typed in Settings; each length is kept between 1 and 180 minutes.
    #[must_use]
    pub fn from_minutes(focus: u32, short: u32, long: u32) -> Self {
        let keep = |m: u32| m.clamp(1, 180);
        Self {
            focus_min: keep(focus),
            short_min: keep(short),
            long_min: keep(long),
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A phase finished and the next one is ready (the timer pauses; it never rolls on by itself).
    Finished { was: Mode, next: Mode },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timer {
    pub plan: Plan,
    pub mode: Mode,
    pub running: bool,
    pub remaining_ms: u64,
    /// Focus sessions completed in this cycle.
    pub done: u32,
}

impl Timer {
    #[must_use]
    pub fn new(plan: Plan) -> Self {
        let mut t = Self {
            plan,
            mode: Mode::Focus,
            running: false,
            remaining_ms: 0,
            done: 0,
        };
        t.remaining_ms = t.length_ms(Mode::Focus);
        t
    }

    #[must_use]
    pub fn length_ms(&self, mode: Mode) -> u64 {
        let min = match mode {
            Mode::Focus => self.plan.focus_min,
            Mode::ShortBreak => self.plan.short_min,
            Mode::LongBreak => self.plan.long_min,
        };
        u64::from(min.max(1)) * 60_000
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
    }

    pub fn reset(&mut self) {
        self.running = false;
        self.remaining_ms = self.length_ms(self.mode);
    }

    fn next_mode(&self) -> Mode {
        match self.mode {
            Mode::Focus if (self.done + 1) % self.plan.rounds.max(1) == 0 => Mode::LongBreak,
            Mode::Focus => Mode::ShortBreak,
            _ => Mode::Focus,
        }
    }

    fn enter(&mut self, mode: Mode) {
        self.mode = mode;
        self.remaining_ms = self.length_ms(mode);
        self.running = false;
    }

    /// Skips to the next phase without counting a finished session.
    pub fn skip(&mut self) {
        let next = self.next_mode();
        self.enter(next);
    }

    /// Advances by `elapsed_ms` while running. Returns an event when a phase ends.
    pub fn tick(&mut self, elapsed_ms: u64) -> Option<Event> {
        if !self.running {
            return None;
        }
        if elapsed_ms < self.remaining_ms {
            self.remaining_ms -= elapsed_ms;
            return None;
        }
        let was = self.mode;
        let next = self.next_mode();
        if was == Mode::Focus {
            self.done += 1;
        }
        if was == Mode::LongBreak {
            self.done = 0;
        }
        self.enter(next);
        Some(Event::Finished { was, next })
    }

    /// 0.0 at the start of a phase to 1.0 at its end.
    #[must_use]
    pub fn progress(&self) -> f32 {
        let total = self.length_ms(self.mode) as f32;
        (1.0 - self.remaining_ms as f32 / total).clamp(0.0, 1.0)
    }

    /// `24:59`, or `1:05:00` for an hour or more.
    #[must_use]
    pub fn label(&self) -> String {
        let secs = self.remaining_ms.div_ceil(1000);
        let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
        if h > 0 {
            format!("{h}:{m:02}:{s:02}")
        } else {
            format!("{m:02}:{s:02}")
        }
    }
}

/// The timer as text for a file, with the moment it was saved.
#[must_use]
pub fn encode(timer: &Timer, now_ms: u64) -> String {
    let mode = match timer.mode {
        Mode::Focus => "focus",
        Mode::ShortBreak => "short",
        Mode::LongBreak => "long",
    };
    format!(
        "mode={mode}\ndone={}\nrunning={}\nremaining_ms={}\nsaved_ms={now_ms}\n",
        timer.done,
        u8::from(timer.running),
        timer.remaining_ms
    )
}

/// Reads a saved timer. A running timer keeps counting through the time the file sat there, so
/// closing the panel does not stop it. At most one phase change is applied for that gap.
#[must_use]
pub fn decode(text: &str, plan: Plan, now_ms: u64) -> Option<(Timer, Option<Event>)> {
    let get = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
    };
    let mut timer = Timer::new(plan);
    timer.mode = match get("mode")? {
        "focus" => Mode::Focus,
        "short" => Mode::ShortBreak,
        "long" => Mode::LongBreak,
        _ => return None,
    };
    timer.done = get("done")?.parse().ok()?;
    timer.running = get("running")? == "1";
    timer.remaining_ms = get("remaining_ms")?.parse().ok()?;
    let saved: u64 = get("saved_ms")?.parse().ok()?;
    let event = timer.tick(now_ms.saturating_sub(saved));
    Some((timer, event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_move_while_paused() {
        let mut t = Timer::new(Plan::default());
        assert_eq!(t.tick(60_000), None);
        assert_eq!(t.remaining_ms, 25 * 60_000);
        assert_eq!(t.label(), "25:00");
    }

    #[test]
    fn counts_down_and_labels_round_up() {
        let mut t = Timer::new(Plan::default());
        t.toggle();
        assert_eq!(t.tick(1_000), None);
        assert_eq!(t.label(), "24:59");
        t.tick(500);
        assert_eq!(t.label(), "24:59", "24:58.5 shows as 24:59");
        t.tick(500);
        assert_eq!(t.label(), "24:58");
        assert!(t.progress() > 0.0 && t.progress() < 0.01);
    }

    #[test]
    fn a_full_cycle_ends_in_a_long_break_and_pauses_between_phases() {
        let mut t = Timer::new(Plan::default());
        for round in 1..=4 {
            t.toggle();
            let ev = t.tick(25 * 60_000).expect("focus ends");
            let next = if round == 4 {
                Mode::LongBreak
            } else {
                Mode::ShortBreak
            };
            assert_eq!(
                ev,
                Event::Finished {
                    was: Mode::Focus,
                    next
                }
            );
            assert!(!t.running, "it waits for the user");
            assert_eq!(t.done, round);
            t.toggle();
            let after = t.tick(60 * 60_000).expect("break ends");
            assert_eq!(
                after,
                Event::Finished {
                    was: next,
                    next: Mode::Focus
                }
            );
        }
        assert_eq!(t.done, 0, "a long break closes the cycle");
    }

    #[test]
    fn skip_and_reset() {
        let mut t = Timer::new(Plan {
            focus_min: 1,
            short_min: 1,
            long_min: 2,
            rounds: 2,
        });
        t.toggle();
        t.tick(30_000);
        t.reset();
        assert!(!t.running);
        assert_eq!(t.remaining_ms, 60_000);
        t.skip();
        assert_eq!(t.mode, Mode::ShortBreak);
        assert_eq!(t.done, 0);
    }

    #[test]
    fn a_saved_running_timer_keeps_counting() {
        let mut t = Timer::new(Plan::default());
        t.toggle();
        t.tick(60_000);
        let saved = encode(&t, 1_000);
        let (back, event) = decode(&saved, Plan::default(), 1_000 + 120_000).expect("reads");
        assert_eq!(event, None);
        assert_eq!(back.remaining_ms, 25 * 60_000 - 180_000);
        assert!(back.running);
        let (done, event) = decode(&saved, Plan::default(), 1_000 + 3_600_000).expect("reads");
        assert!(matches!(
            event,
            Some(Event::Finished {
                was: Mode::Focus,
                ..
            })
        ));
        assert!(!done.running && done.mode == Mode::ShortBreak && done.done == 1);
        assert!(decode("junk", Plan::default(), 0).is_none());
    }

    #[test]
    fn hour_labels() {
        let t = Timer::new(Plan {
            focus_min: 90,
            ..Plan::default()
        });
        assert_eq!(t.label(), "1:30:00");
    }
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    #[test]
    fn plan_from_minutes_is_kept_in_range() {
        let p = Plan::from_minutes(0, 5, 100_000);
        assert_eq!((p.focus_min, p.short_min, p.long_min), (1, 5, 180));
        assert_eq!(p.rounds, 4);
        assert_eq!(Timer::new(Plan::from_minutes(50, 5, 15)).label(), "50:00");
    }
}
