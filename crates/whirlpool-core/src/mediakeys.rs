//! The three volume keys and what one press does. Pure logic, so the hook in
//! `whirlpool-shell` stays a thin shell around it.
//!
//! Only these three virtual-key codes are ever recognised. Everything else is
//! passed on by the caller before it is looked at twice.

/// `VK_VOLUME_MUTE`.
pub const VK_MUTE: u32 = 0xAD;
/// `VK_VOLUME_DOWN`.
pub const VK_DOWN: u32 = 0xAE;
/// `VK_VOLUME_UP`.
pub const VK_UP: u32 = 0xAF;

/// Percent of the master volume one key press moves.
pub const STEP_PERCENT: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeKey {
    Up,
    Down,
    Mute,
}

impl VolumeKey {
    /// `Some` only for the three volume keys.
    #[must_use]
    pub fn from_vk(vk: u32) -> Option<Self> {
        match vk {
            VK_UP => Some(Self::Up),
            VK_DOWN => Some(Self::Down),
            VK_MUTE => Some(Self::Mute),
            _ => None,
        }
    }

    /// Signed percent change for a press; zero for mute.
    #[must_use]
    pub fn delta(self) -> i32 {
        match self {
            Self::Up => STEP_PERCENT,
            Self::Down => -STEP_PERCENT,
            Self::Mute => 0,
        }
    }
}

/// The level after `presses` (positive up, negative down), kept in 0 to 100.
#[must_use]
pub fn stepped(percent: u8, presses_delta: i32) -> u8 {
    (i32::from(percent) + presses_delta).clamp(0, 100) as u8
}

/// Packs a reading for a window message: percent in the low byte, bit 8 for muted.
#[must_use]
pub fn pack(percent: u8, muted: bool) -> isize {
    isize::from(percent.min(100)) | if muted { 0x100 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_three_volume_keys_are_recognised() {
        for vk in 0..=0x1FFu32 {
            let known = matches!(vk, 0xAD..=0xAF);
            assert_eq!(VolumeKey::from_vk(vk).is_some(), known, "vk {vk:#x}");
        }
        assert_eq!(VolumeKey::from_vk(0xAF), Some(VolumeKey::Up));
        assert_eq!(VolumeKey::from_vk(0xAE), Some(VolumeKey::Down));
        assert_eq!(VolumeKey::from_vk(0xAD), Some(VolumeKey::Mute));
        // The neighbours: next/previous track and play/pause belong to the media session.
        assert_eq!(VolumeKey::from_vk(0xB0), None);
        assert_eq!(VolumeKey::from_vk(0xB3), None);
    }

    #[test]
    fn one_press_moves_two_percent() {
        assert_eq!(stepped(50, VolumeKey::Up.delta()), 52);
        assert_eq!(stepped(50, VolumeKey::Down.delta()), 48);
        assert_eq!(stepped(50, VolumeKey::Mute.delta()), 50);
    }

    #[test]
    fn the_level_stays_between_zero_and_one_hundred() {
        assert_eq!(stepped(99, 2), 100);
        assert_eq!(stepped(100, 2), 100);
        assert_eq!(stepped(1, -2), 0);
        assert_eq!(stepped(0, -10), 0);
        assert_eq!(stepped(40, 20), 60);
    }

    #[test]
    fn a_reading_packs_into_one_message_value() {
        assert_eq!(pack(42, false), 42);
        assert_eq!(pack(42, true), 0x100 | 42);
        assert_eq!(pack(250, false), 100);
    }
}
