//! Dynamic island presets: sizes and placement as pure numbers.
//!
//! `Classic` is exactly what the menu bar uses today (145 / 440 / 118 logical pixels), so wiring
//! a preset into the renderer later changes nothing for people who keep the default.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preset {
    /// Narrow, one line; for small screens and people who want it out of the way.
    Compact,
    /// The shipped look.
    #[default]
    Classic,
    /// More room for long track titles and several usage meters.
    Wide,
    /// Tall panel with larger artwork and spacing.
    Expressive,
}

/// Logical pixels (before DPI scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// The collapsed island with one provider.
    pub single_width: f32,
    /// The opened panel.
    pub expanded_width: f32,
    /// Height of one row of the opened panel.
    pub row_height: f32,
    /// Corner radius of the opened panel.
    pub corner_radius: f32,
}

impl Preset {
    pub const ALL: [Self; 4] = [Self::Compact, Self::Classic, Self::Wide, Self::Expressive];

    #[must_use]
    pub const fn metrics(self) -> Metrics {
        match self {
            Self::Compact => Metrics {
                single_width: 112.0,
                expanded_width: 340.0,
                row_height: 96.0,
                corner_radius: 14.0,
            },
            Self::Classic => Metrics {
                single_width: 145.0,
                expanded_width: 440.0,
                row_height: 118.0,
                corner_radius: 18.0,
            },
            Self::Wide => Metrics {
                single_width: 180.0,
                expanded_width: 540.0,
                row_height: 118.0,
                corner_radius: 20.0,
            },
            Self::Expressive => Metrics {
                single_width: 160.0,
                expanded_width: 480.0,
                row_height: 148.0,
                corner_radius: 26.0,
            },
        }
    }
}

impl Metrics {
    /// The metrics in device pixels.
    #[must_use]
    pub fn scaled(self, scale: f32) -> Self {
        Self {
            single_width: (self.single_width * scale).round(),
            expanded_width: (self.expanded_width * scale).round(),
            row_height: (self.row_height * scale).round(),
            corner_radius: (self.corner_radius * scale).round(),
        }
    }

    /// Width and height while opening: `progress` 0 is the collapsed island (`collapsed_height`
    /// tall), 1 is the full panel with `rows` rows. Eased, so it starts and ends gently.
    #[must_use]
    pub fn opening(self, progress: f32, collapsed_height: f32, rows: u32) -> (f32, f32) {
        let t = progress.clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        let full = self.row_height * rows.max(1) as f32;
        (
            self.single_width + (self.expanded_width - self.single_width) * eased,
            collapsed_height + (full - collapsed_height) * eased,
        )
    }
}

/// Left edge for a panel `width` wide centred on `anchor`, kept inside `[bounds_left,
/// bounds_right]`. A panel wider than the bounds sits at the left edge.
#[must_use]
pub fn panel_left(anchor: f32, width: f32, bounds_left: f32, bounds_right: f32) -> f32 {
    let wanted = anchor - width / 2.0;
    let latest = bounds_right - width;
    if latest <= bounds_left {
        return bounds_left;
    }
    wanted.clamp(bounds_left, latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_is_what_ships_today() {
        let m = Preset::default().metrics();
        assert_eq!(
            (m.single_width, m.expanded_width, m.row_height),
            (145.0, 440.0, 118.0)
        );
    }

    #[test]
    fn presets_grow_from_compact_to_wide() {
        let w = |p: Preset| p.metrics().expanded_width;
        assert!(w(Preset::Compact) < w(Preset::Classic));
        assert!(w(Preset::Classic) < w(Preset::Wide));
        for p in Preset::ALL {
            let m = p.metrics();
            assert!(
                m.single_width < m.expanded_width,
                "{p:?} must open wider than it rests"
            );
        }
    }

    #[test]
    fn scaling_rounds_to_whole_pixels() {
        let m = Preset::Classic.metrics().scaled(1.25);
        assert_eq!(m.single_width, 181.0);
        assert_eq!(m.expanded_width, 550.0);
    }

    #[test]
    fn opening_runs_from_collapsed_to_full_without_overshoot() {
        let m = Preset::Classic.metrics();
        assert_eq!(m.opening(0.0, 30.0, 2), (145.0, 30.0));
        assert_eq!(m.opening(1.0, 30.0, 2), (440.0, 236.0));
        let mut last = (0.0, 0.0);
        for i in 0..=20 {
            let now = m.opening(i as f32 / 20.0, 30.0, 2);
            assert!(now.0 >= last.0 && now.1 >= last.1);
            assert!(now.0 <= 440.0 && now.1 <= 236.0);
            last = now;
        }
        assert_eq!(
            m.opening(7.0, 30.0, 0),
            m.opening(1.0, 30.0, 1),
            "clamped input"
        );
    }

    #[test]
    fn the_panel_stays_on_screen() {
        assert_eq!(panel_left(500.0, 200.0, 0.0, 1000.0), 400.0);
        assert_eq!(panel_left(50.0, 200.0, 0.0, 1000.0), 0.0);
        assert_eq!(panel_left(990.0, 200.0, 0.0, 1000.0), 800.0);
        assert_eq!(
            panel_left(100.0, 600.0, 0.0, 500.0),
            0.0,
            "too wide: left edge"
        );
    }
}
