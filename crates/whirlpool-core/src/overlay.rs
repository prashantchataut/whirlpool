//! Geometry and hotkey parsing for the system overlay. Pure, so it is tested without a desktop.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Top-left origin of a `size` box in `corner` of `work` (left, top, right, bottom), `margin` in.
#[must_use]
pub fn corner_origin(
    corner: Corner,
    work: (i32, i32, i32, i32),
    size: (i32, i32),
    margin: i32,
) -> (i32, i32) {
    let x = match corner {
        Corner::TopLeft | Corner::BottomLeft => work.0 + margin,
        Corner::TopRight | Corner::BottomRight => work.2 - size.0 - margin,
    };
    let y = match corner {
        Corner::TopLeft | Corner::TopRight => work.1 + margin,
        Corner::BottomLeft | Corner::BottomRight => work.3 - size.1 - margin,
    };
    (x, y)
}

/// The corner a dropped box should snap to: the one whose origin is within `reach` pixels on both
/// axes. `None` leaves the box where the user dropped it.
#[must_use]
pub fn snap(
    position: (i32, i32),
    work: (i32, i32, i32, i32),
    size: (i32, i32),
    margin: i32,
    reach: i32,
) -> Option<Corner> {
    [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ]
    .into_iter()
    .filter_map(|corner| {
        let origin = corner_origin(corner, work, size, margin);
        let (dx, dy) = ((position.0 - origin.0).abs(), (position.1 - origin.1).abs());
        (dx <= reach && dy <= reach).then_some((dx + dy, corner))
    })
    .min_by_key(|(distance, _)| *distance)
    .map(|(_, corner)| corner)
}

/// Keeps a box fully inside `work`, whatever the monitors did since it was placed.
#[must_use]
pub fn clamp_inside(
    position: (i32, i32),
    work: (i32, i32, i32, i32),
    size: (i32, i32),
) -> (i32, i32) {
    let max_x = (work.2 - size.0).max(work.0);
    let max_y = (work.3 - size.1).max(work.1);
    (
        position.0.clamp(work.0, max_x),
        position.1.clamp(work.1, max_y),
    )
}

pub const MOD_ALT: u32 = 0x1;
pub const MOD_CONTROL: u32 = 0x2;
pub const MOD_SHIFT: u32 = 0x4;
pub const MOD_WIN: u32 = 0x8;

/// Parses `Ctrl+Shift+F10` into (modifiers, virtual-key code). A key alone, or modifiers alone,
/// is rejected: a global hotkey without a modifier would swallow ordinary typing.
#[must_use]
pub fn parse_hotkey(text: &str) -> Option<(u32, u32)> {
    let (mut modifiers, mut key) = (0, None);
    for part in text.split('+').map(str::trim) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= MOD_CONTROL,
            "alt" => modifiers |= MOD_ALT,
            "shift" => modifiers |= MOD_SHIFT,
            "win" | "super" => modifiers |= MOD_WIN,
            other => {
                if key.is_some() {
                    return None;
                }
                key = Some(virtual_key(other)?);
            }
        }
    }
    (modifiers != 0).then_some((modifiers, key?))
}

fn virtual_key(name: &str) -> Option<u32> {
    let bytes = name.as_bytes();
    if name == "space" {
        return Some(0x20);
    }
    match bytes {
        [c @ b'a'..=b'z'] => Some(u32::from(c.to_ascii_uppercase())),
        [c @ b'0'..=b'9'] => Some(u32::from(*c)),
        [b'f', rest @ ..] => {
            let n: u32 = std::str::from_utf8(rest).ok()?.parse().ok()?;
            (1..=24).contains(&n).then(|| 0x70 + n - 1)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: (i32, i32, i32, i32) = (0, 0, 1920, 1040);

    #[test]
    fn corners() {
        assert_eq!(
            corner_origin(Corner::TopLeft, WORK, (200, 100), 12),
            (12, 12)
        );
        assert_eq!(
            corner_origin(Corner::BottomRight, WORK, (200, 100), 12),
            (1708, 928)
        );
        // A work area that does not start at zero (second monitor, taskbar on the left).
        assert_eq!(
            corner_origin(Corner::TopRight, (-1920, 40, 0, 1080), (200, 100), 8),
            (-208, 48)
        );
    }

    #[test]
    fn snaps_only_when_close() {
        assert_eq!(
            snap((30, 20), WORK, (200, 100), 12, 80),
            Some(Corner::TopLeft)
        );
        assert_eq!(
            snap((1690, 900), WORK, (200, 100), 12, 80),
            Some(Corner::BottomRight)
        );
        assert_eq!(snap((900, 500), WORK, (200, 100), 12, 80), None);
    }

    #[test]
    fn clamps_after_a_monitor_change() {
        assert_eq!(clamp_inside((3000, 2000), WORK, (200, 100)), (1720, 940));
        assert_eq!(clamp_inside((-50, -50), WORK, (200, 100)), (0, 0));
        assert_eq!(clamp_inside((10, 10), (0, 0, 100, 50), (200, 100)), (0, 0));
    }

    #[test]
    fn parses_hotkeys() {
        assert_eq!(
            parse_hotkey("Ctrl+Shift+F10"),
            Some((MOD_CONTROL | MOD_SHIFT, 0x79))
        );
        assert_eq!(parse_hotkey("alt + g"), Some((MOD_ALT, 0x47)));
        assert_eq!(parse_hotkey("Win+1"), Some((MOD_WIN, 0x31)));
        assert_eq!(
            parse_hotkey("Ctrl+Alt+Space"),
            Some((MOD_CONTROL | MOD_ALT, 0x20))
        );
        assert_eq!(parse_hotkey("F10"), None);
        assert_eq!(parse_hotkey("Ctrl+Shift"), None);
        assert_eq!(parse_hotkey("Ctrl+F25"), None);
        assert_eq!(parse_hotkey("Ctrl+A+B"), None);
        assert_eq!(parse_hotkey(""), None);
    }
}
