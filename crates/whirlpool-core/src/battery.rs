//! Battery alerts at chosen levels. One alert per level per discharge: the state resets when the
//! charger comes back, and an alert never repeats for a level already announced.

#[derive(Clone, Debug)]
pub struct Alerts {
    levels: Vec<u8>,
    lowest_announced: u8,
}

impl Alerts {
    /// `levels` are percentages (1 to 99); duplicates and nonsense are dropped, highest first.
    pub fn new(levels: &[u8]) -> Self {
        let mut levels: Vec<u8> = levels
            .iter()
            .copied()
            .filter(|l| (1..100).contains(l))
            .collect();
        levels.sort_unstable_by(|a, b| b.cmp(a));
        levels.dedup();
        Alerts {
            levels,
            lowest_announced: 101,
        }
    }

    /// `percent`: 0..=100, or `None` when unknown (no battery, or Windows reports 255).
    /// Returns the level to announce now, if any: the lowest level the charge has reached that has
    /// not been announced yet.
    pub fn observe(&mut self, percent: Option<u8>, on_ac: bool) -> Option<u8> {
        let percent = percent.filter(|p| *p <= 100)?;
        if on_ac {
            self.lowest_announced = 101;
            return None;
        }
        let level = self
            .levels
            .iter()
            .copied()
            .filter(|l| percent <= *l && (*l as u16) < self.lowest_announced as u16)
            .min()?;
        self.lowest_announced = level as u16 as u8;
        Some(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announces_each_level_once_while_draining() {
        let mut a = Alerts::new(&[20, 10, 5]);
        assert_eq!(a.observe(Some(40), false), None);
        assert_eq!(a.observe(Some(20), false), Some(20));
        assert_eq!(a.observe(Some(19), false), None);
        assert_eq!(a.observe(Some(10), false), Some(10));
        assert_eq!(a.observe(Some(10), false), None);
        assert_eq!(a.observe(Some(4), false), Some(5));
    }

    #[test]
    fn a_big_drop_announces_only_the_lowest_level_reached() {
        let mut a = Alerts::new(&[25, 15, 5]);
        assert_eq!(a.observe(Some(12), false), Some(15));
        assert_eq!(a.observe(Some(3), false), Some(5));
    }

    #[test]
    fn charging_resets_the_alerts() {
        let mut a = Alerts::new(&[20]);
        assert_eq!(a.observe(Some(15), false), Some(20));
        assert_eq!(a.observe(Some(30), true), None);
        assert_eq!(a.observe(Some(18), false), Some(20));
    }

    #[test]
    fn unknown_or_invalid_readings_say_nothing() {
        let mut a = Alerts::new(&[20, 0, 100, 20]);
        assert_eq!(a.observe(None, false), None);
        assert_eq!(a.observe(Some(255), false), None);
        assert_eq!(a.levels, vec![20]);
    }
}
