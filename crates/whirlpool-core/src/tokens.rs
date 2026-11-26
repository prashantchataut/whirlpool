//! One place for the numbers every Whirlpool surface draws from, and the size
//! budgets the tests hold them to. Logical pixels (before display scaling).
//!
//! The budgets are not guesses: they were set against the Windows 11 flyouts
//! (see `docs/BASELINES.md`) and may only get smaller.

/// Dark glass fill, RGBA 0-255: `rgba(20, 20, 24, 0.76)`.
pub const GLASS: [u8; 4] = [20, 20, 24, 194];
/// One-pixel inner highlight on the rim: `rgba(255, 255, 255, 0.08)`.
pub const HIGHLIGHT: [u8; 4] = [255, 255, 255, 20];
/// The single accent colour, RGB 0-255 (a calm blue).
pub const ACCENT: [u8; 3] = [96, 176, 255];
/// Layout grid; a 4 px sub-grid is allowed for tight spots.
pub const GRID: f32 = 8.0;
pub const SUBGRID: f32 = 4.0;

/// Type sizes: body and controls, then secondary labels.
pub const TEXT_BODY: (f32, f32) = (12.0, 13.0);
pub const TEXT_SECONDARY: f32 = 11.0;

/// A size with the largest it may ever be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    pub width: f32,
    pub height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Budget {
    #[must_use]
    pub fn allows(self, width: f32, height: f32) -> bool {
        width <= self.max_width && height <= self.max_height
    }
}

/// Volume and brightness pill: an icon, a track and a number.
pub const HUD_PILL: Budget = Budget {
    width: 200.0,
    height: 40.0,
    max_width: 210.0,
    max_height: 44.0,
};
pub const HUD_ICON: f32 = 16.0;
pub const HUD_TRACK: f32 = 4.0;
pub const HUD_NUMBER: f32 = 12.0;

/// Wi-Fi and Bluetooth pill: an icon and a small indicator, no slider.
pub const HUD_SMALL_PILL: (f32, f32) = (132.0, 40.0);

/// Quick settings: width, and the tile height range.
pub const QUICK_WIDTH: (f32, f32) = (320.0, 328.0);
pub const QUICK_TILE_HEIGHT: (f32, f32) = (44.0, 48.0);

/// Dock defaults for a profile that has never saved a size.
pub const DOCK_ICON: f32 = 32.0;
pub const DOCK_MAGNIFICATION: (f32, f32) = (1.2, 1.3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pill_budget_is_met_and_is_tighter_than_the_old_300_by_56() {
        assert!(HUD_PILL.allows(HUD_PILL.width, HUD_PILL.height));
        assert!(HUD_PILL.max_width <= 210.0 && HUD_PILL.max_height <= 44.0);
    }

    #[test]
    fn sizes_sit_on_the_grid() {
        for value in [HUD_PILL.width, HUD_PILL.height, QUICK_WIDTH.0, DOCK_ICON] {
            assert_eq!(value % SUBGRID, 0.0, "{value} is off the 4 px grid");
        }
        assert_eq!(QUICK_TILE_HEIGHT.0 % SUBGRID, 0.0);
        assert_eq!(QUICK_TILE_HEIGHT.1 % SUBGRID, 0.0);
    }

    #[test]
    fn the_glass_recipe_is_the_documented_one() {
        assert_eq!(&GLASS[..3], &[20, 20, 24]);
        assert!((f32::from(GLASS[3]) / 255.0 - 0.76).abs() < 0.01);
        assert!((f32::from(HIGHLIGHT[3]) / 255.0 - 0.08).abs() < 0.01);
    }
}
