//! The clock: formats, presets, a table of time zones and tick scheduling. Pure logic; Windows
//! supplies the time (`whirlpool_shell::system`), and localised month and weekday names.

/// A broken-down local time. `weekday` counts from Sunday = 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub weekday: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// 12 or 24 hours; `System` follows the regional setting Windows reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HourCycle {
    #[default]
    System,
    H12,
    H24,
}

/// How much of the date goes next to the time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateStyle {
    Off,
    #[default]
    Short,
    Long,
    Numeric,
}

impl DateStyle {
    /// The Windows date picture for this style (see `GetDateFormatEx`), `None` for `Off`.
    #[must_use]
    pub fn pattern(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            Self::Short => Some("ddd d MMM"),
            Self::Long => Some("dddd, d MMMM"),
            Self::Numeric => Some("yyyy-MM-dd"),
        }
    }
}

/// Typeface families that ship with Windows 10 and 11, so no font file is bundled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClockFont {
    /// The interface font.
    #[default]
    System,
    /// Fixed-width digits: the time never jitters while seconds tick.
    Mono,
    /// Narrow digits for a tight bar.
    Condensed,
    Serif,
}

impl ClockFont {
    pub const ALL: [Self; 4] = [Self::System, Self::Mono, Self::Condensed, Self::Serif];

    /// The family name to ask DirectWrite for; `None` keeps the interface font.
    #[must_use]
    pub fn family(self) -> Option<&'static str> {
        match self {
            Self::System => None,
            Self::Mono => Some("Consolas"),
            Self::Condensed => Some("Bahnschrift SemiCondensed"),
            Self::Serif => Some("Georgia"),
        }
    }

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Mono => "mono",
            Self::Condensed => "condensed",
            Self::Serif => "serif",
        }
    }
}

/// Stroke weight, as DirectWrite numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClockWeight {
    Light,
    #[default]
    Regular,
    Semibold,
}

impl ClockWeight {
    pub const ALL: [Self; 3] = [Self::Light, Self::Regular, Self::Semibold];

    #[must_use]
    pub fn value(self) -> i32 {
        match self {
            Self::Light => 300,
            Self::Regular => 400,
            Self::Semibold => 600,
        }
    }
}

/// A named combination of font, weight, date and seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockPreset {
    Classic,
    Quiet,
    Bold,
    Digital,
    Editorial,
    Compact,
}

/// What a preset sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresetLook {
    pub font: ClockFont,
    pub weight: ClockWeight,
    pub date: DateStyle,
    pub seconds: bool,
}

impl ClockPreset {
    pub const ALL: [Self; 6] = [
        Self::Classic,
        Self::Quiet,
        Self::Bold,
        Self::Digital,
        Self::Editorial,
        Self::Compact,
    ];

    #[must_use]
    pub fn look(self) -> PresetLook {
        let (font, weight, date, seconds) = match self {
            Self::Classic => (
                ClockFont::System,
                ClockWeight::Regular,
                DateStyle::Short,
                false,
            ),
            Self::Quiet => (ClockFont::System, ClockWeight::Light, DateStyle::Off, false),
            Self::Bold => (
                ClockFont::System,
                ClockWeight::Semibold,
                DateStyle::Short,
                false,
            ),
            Self::Digital => (
                ClockFont::Mono,
                ClockWeight::Regular,
                DateStyle::Numeric,
                true,
            ),
            Self::Editorial => (
                ClockFont::Serif,
                ClockWeight::Regular,
                DateStyle::Long,
                false,
            ),
            Self::Compact => (
                ClockFont::Condensed,
                ClockWeight::Semibold,
                DateStyle::Off,
                false,
            ),
        };
        PresetLook {
            font,
            weight,
            date,
            seconds,
        }
    }

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Quiet => "quiet",
            Self::Bold => "bold",
            Self::Digital => "digital",
            Self::Editorial => "editorial",
            Self::Compact => "compact",
        }
    }

    /// The preset whose settings equal these, if any (so Settings can highlight it).
    #[must_use]
    pub fn matching(
        font: ClockFont,
        weight: ClockWeight,
        date: DateStyle,
        seconds: bool,
    ) -> Option<Self> {
        Self::ALL.into_iter().find(|p| {
            p.look()
                == PresetLook {
                    font,
                    weight,
                    date,
                    seconds,
                }
        })
    }
}

/// `19:42`, `19:42:05`, `7:42 PM`, `7:42:05 PM`.
#[must_use]
pub fn format_time(t: &LocalTime, cycle: HourCycle, system_is_24h: bool, seconds: bool) -> String {
    let h24 = match cycle {
        HourCycle::H24 => true,
        HourCycle::H12 => false,
        HourCycle::System => system_is_24h,
    };
    let sec = if seconds {
        format!(":{:02}", t.second)
    } else {
        String::new()
    };
    if h24 {
        format!("{:02}:{:02}{sec}", t.hour, t.minute)
    } else {
        let hour = match t.hour % 12 {
            0 => 12,
            h => h,
        };
        let suffix = if t.hour < 12 { "AM" } else { "PM" };
        format!("{hour}:{:02}{sec} {suffix}", t.minute)
    }
}

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// An English date, for tests and for when Windows cannot format one. The app asks Windows first,
/// which localises the names.
#[must_use]
pub fn english_date(t: &LocalTime, style: DateStyle) -> String {
    let wd = WEEKDAYS[usize::from(t.weekday % 7)];
    let mo = MONTHS[usize::from(t.month.clamp(1, 12) - 1)];
    match style {
        DateStyle::Off => String::new(),
        DateStyle::Short => format!("{} {} {}", &wd[..3], t.day, &mo[..3]),
        DateStyle::Long => format!("{wd}, {} {mo}", t.day),
        DateStyle::Numeric => format!("{:04}-{:02}-{:02}", t.year, t.month, t.day),
    }
}

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
#[must_use]
pub fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// How many calendar days `other` is ahead of (`+1`) or behind (`-1`) `home`.
#[must_use]
pub fn day_offset(home: &LocalTime, other: &LocalTime) -> i64 {
    days_from_civil(other.year, other.month, other.day)
        - days_from_civil(home.year, home.month, home.day)
}

/// A built-in zone: Windows key, city shown to the user, three-letter tag for the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub key: &'static str,
    pub city: &'static str,
    pub tag: &'static str,
}

/// Places people ask for, by Windows time zone key (the names Windows itself uses).
pub const ZONES: &[Zone] = &[
    Zone {
        key: "UTC",
        city: "UTC",
        tag: "UTC",
    },
    Zone {
        key: "Pacific Standard Time",
        city: "Los Angeles",
        tag: "LAX",
    },
    Zone {
        key: "Mountain Standard Time",
        city: "Denver",
        tag: "DEN",
    },
    Zone {
        key: "Central Standard Time",
        city: "Chicago",
        tag: "CHI",
    },
    Zone {
        key: "Eastern Standard Time",
        city: "New York",
        tag: "NYC",
    },
    Zone {
        key: "SA Eastern Standard Time",
        city: "Sao Paulo",
        tag: "SAO",
    },
    Zone {
        key: "GMT Standard Time",
        city: "London",
        tag: "LON",
    },
    Zone {
        key: "W. Europe Standard Time",
        city: "Berlin",
        tag: "BER",
    },
    Zone {
        key: "Romance Standard Time",
        city: "Paris",
        tag: "PAR",
    },
    Zone {
        key: "E. Europe Standard Time",
        city: "Bucharest",
        tag: "BUH",
    },
    Zone {
        key: "Russian Standard Time",
        city: "Moscow",
        tag: "MOW",
    },
    Zone {
        key: "Arab Standard Time",
        city: "Riyadh",
        tag: "RUH",
    },
    Zone {
        key: "Arabian Standard Time",
        city: "Dubai",
        tag: "DXB",
    },
    Zone {
        key: "India Standard Time",
        city: "Mumbai",
        tag: "BOM",
    },
    Zone {
        key: "SE Asia Standard Time",
        city: "Jakarta",
        tag: "JKT",
    },
    Zone {
        key: "Singapore Standard Time",
        city: "Singapore",
        tag: "SIN",
    },
    Zone {
        key: "China Standard Time",
        city: "Shanghai",
        tag: "SHA",
    },
    Zone {
        key: "Tokyo Standard Time",
        city: "Tokyo",
        tag: "TYO",
    },
    Zone {
        key: "Korea Standard Time",
        city: "Seoul",
        tag: "SEL",
    },
    Zone {
        key: "AUS Eastern Standard Time",
        city: "Sydney",
        tag: "SYD",
    },
    Zone {
        key: "New Zealand Standard Time",
        city: "Auckland",
        tag: "AKL",
    },
    Zone {
        key: "South Africa Standard Time",
        city: "Johannesburg",
        tag: "JNB",
    },
    Zone {
        key: "Egypt Standard Time",
        city: "Cairo",
        tag: "CAI",
    },
    Zone {
        key: "Hawaiian Standard Time",
        city: "Honolulu",
        tag: "HNL",
    },
];

#[must_use]
pub fn zone(key: &str) -> Option<&'static Zone> {
    ZONES.iter().find(|z| z.key == key)
}

/// The tag for a stored key, or the first three letters of an unknown key.
#[must_use]
pub fn zone_tag(key: &str) -> String {
    zone(key).map_or_else(
        || key.chars().take(3).collect::<String>().to_uppercase(),
        |z| z.tag.to_owned(),
    )
}

/// The bar text: optional date, the time, then up to `max_zones` other zones as `TYO 08:42`.
#[must_use]
pub fn compose_bar(
    date: Option<&str>,
    time: &str,
    zones: &[(String, String)],
    max_zones: usize,
) -> String {
    let mut out = String::new();
    if let Some(date) = date.filter(|d| !d.is_empty()) {
        out.push_str(date);
        out.push_str("  ");
    }
    out.push_str(time);
    for (tag, t) in zones.iter().take(max_zones) {
        out.push_str("  \u{b7}  ");
        out.push_str(tag);
        out.push(' ');
        out.push_str(t);
    }
    out
}

/// Milliseconds until the next redraw. With seconds, the next whole second; without, the next
/// minute. A few milliseconds of slack keep the timer from firing just before the boundary.
#[must_use]
pub fn next_tick_ms(second: u32, millisecond: u32, seconds: bool) -> u32 {
    let ms = millisecond.min(999);
    if seconds {
        1000 - ms + 20
    } else {
        let elapsed = second.min(59) * 1000 + ms;
        60_000 - elapsed + 50
    }
}

/// Whether a ticking clock should keep a one-second timer running. Seconds tick only while the
/// clock can be seen: the bar is on screen, no full-screen program covers it, the session is not
/// locked and the display is on. Anything else drops back to one wake-up per minute.
#[must_use]
pub fn should_tick_seconds(
    wants_seconds: bool,
    bar_visible: bool,
    fullscreen: bool,
    locked_or_off: bool,
) -> bool {
    wants_seconds && bar_visible && !fullscreen && !locked_or_off
}

/// Everything the bar needs to write the clock.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClockOptions {
    pub hour: HourCycle,
    pub seconds: bool,
    /// `DateStyle::Off` shows no date.
    pub date: DateStyleOpt,
    pub zones: Vec<String>,
    pub font: ClockFont,
    pub weight: ClockWeight,
}

/// A date style that defaults to "off" (the options struct starts with no date).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateStyleOpt(pub DateStyle);

impl Default for DateStyleOpt {
    fn default() -> Self {
        Self(DateStyle::Off)
    }
}

impl ClockFont {
    #[must_use]
    pub fn from_id(id: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|f| f.id() == id)
            .unwrap_or_default()
    }
}

impl ClockWeight {
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Regular => "regular",
            Self::Semibold => "semibold",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|w| w.id() == id)
            .unwrap_or_default()
    }
}

impl DateStyle {
    pub const ALL: [Self; 4] = [Self::Off, Self::Short, Self::Long, Self::Numeric];

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Short => "short",
            Self::Long => "long",
            Self::Numeric => "numeric",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|d| d.id() == id)
            .unwrap_or_default()
    }
}

impl ClockPreset {
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(h: u8, m: u8, s: u8) -> LocalTime {
        LocalTime {
            year: 2026,
            month: 11,
            day: 13,
            weekday: 5,
            hour: h,
            minute: m,
            second: s,
        }
    }

    #[test]
    fn twenty_four_and_twelve_hours() {
        assert_eq!(
            format_time(&t(19, 42, 5), HourCycle::H24, false, false),
            "19:42"
        );
        assert_eq!(
            format_time(&t(19, 42, 5), HourCycle::H24, false, true),
            "19:42:05"
        );
        assert_eq!(
            format_time(&t(19, 42, 5), HourCycle::H12, true, false),
            "7:42 PM"
        );
        assert_eq!(
            format_time(&t(0, 7, 9), HourCycle::H12, true, true),
            "12:07:09 AM"
        );
        assert_eq!(
            format_time(&t(12, 0, 0), HourCycle::H12, true, false),
            "12:00 PM"
        );
        assert_eq!(
            format_time(&t(9, 3, 0), HourCycle::System, true, false),
            "09:03"
        );
        assert_eq!(
            format_time(&t(9, 3, 0), HourCycle::System, false, false),
            "9:03 AM"
        );
    }

    #[test]
    fn dates() {
        let d = t(1, 1, 1);
        assert_eq!(english_date(&d, DateStyle::Short), "Fri 13 Nov");
        assert_eq!(english_date(&d, DateStyle::Long), "Friday, 13 November");
        assert_eq!(english_date(&d, DateStyle::Numeric), "2026-11-13");
        assert_eq!(english_date(&d, DateStyle::Off), "");
        assert!(DateStyle::Off.pattern().is_none());
    }

    #[test]
    fn day_offsets_across_month_and_year() {
        let home = LocalTime {
            year: 2026,
            month: 12,
            day: 31,
            weekday: 4,
            hour: 22,
            minute: 0,
            second: 0,
        };
        let ahead = LocalTime {
            year: 2027,
            month: 1,
            day: 1,
            weekday: 5,
            hour: 7,
            minute: 0,
            second: 0,
        };
        assert_eq!(day_offset(&home, &ahead), 1);
        assert_eq!(day_offset(&ahead, &home), -1);
        assert_eq!(day_offset(&home, &home), 0);
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(
            days_from_civil(2000, 3, 1) - days_from_civil(2000, 2, 28),
            2,
            "2000 is a leap year"
        );
        assert_eq!(
            days_from_civil(2100, 3, 1) - days_from_civil(2100, 2, 28),
            1,
            "2100 is not"
        );
    }

    #[test]
    fn ticks_align_to_the_boundary() {
        assert_eq!(next_tick_ms(59, 500, false), 500 + 50);
        assert_eq!(next_tick_ms(0, 0, false), 60_050);
        assert_eq!(next_tick_ms(10, 250, true), 770);
        assert!(next_tick_ms(10, 999, true) >= 20);
    }

    #[test]
    fn seconds_tick_only_while_visible() {
        assert!(should_tick_seconds(true, true, false, false));
        assert!(!should_tick_seconds(false, true, false, false));
        assert!(!should_tick_seconds(true, false, false, false));
        assert!(!should_tick_seconds(true, true, true, false));
        assert!(!should_tick_seconds(true, true, false, true));
    }

    #[test]
    fn presets_are_distinct_and_recognised() {
        for a in ClockPreset::ALL {
            for b in ClockPreset::ALL {
                assert_eq!(a == b, a.look() == b.look(), "{a:?} vs {b:?}");
            }
            let l = a.look();
            assert_eq!(
                ClockPreset::matching(l.font, l.weight, l.date, l.seconds),
                Some(a)
            );
        }
        assert_eq!(
            ClockPreset::matching(ClockFont::Serif, ClockWeight::Light, DateStyle::Off, true),
            None
        );
    }

    #[test]
    fn zones_are_unique_and_tagged() {
        for (i, a) in ZONES.iter().enumerate() {
            assert_eq!(a.tag.len(), 3, "{}", a.key);
            for b in &ZONES[i + 1..] {
                assert_ne!(a.key, b.key);
                assert_ne!(a.tag, b.tag);
            }
        }
        assert_eq!(zone_tag("Tokyo Standard Time"), "TYO");
        assert_eq!(zone_tag("Nowhere Time"), "NOW");
    }

    #[test]
    fn ids_round_trip_and_unknown_ids_read_as_defaults() {
        for f in ClockFont::ALL {
            assert_eq!(ClockFont::from_id(f.id()), f);
        }
        for w in ClockWeight::ALL {
            assert_eq!(ClockWeight::from_id(w.id()), w);
        }
        for d in DateStyle::ALL {
            assert_eq!(DateStyle::from_id(d.id()), d);
        }
        for p in ClockPreset::ALL {
            assert_eq!(ClockPreset::from_id(p.id()), Some(p));
        }
        assert_eq!(ClockFont::from_id("wingdings"), ClockFont::System);
        assert_eq!(ClockPreset::from_id("x"), None);
        assert_eq!(ClockFont::System.family(), None);
        assert_eq!(ClockWeight::Semibold.value(), 600);
    }

    #[test]
    fn bar_text_composes() {
        let zones = vec![
            ("TYO".to_owned(), "08:42".to_owned()),
            ("NYC".to_owned(), "19:42".to_owned()),
        ];
        assert_eq!(
            compose_bar(Some("Fri 13 Nov"), "19:42", &zones, 1),
            "Fri 13 Nov  19:42  \u{b7}  TYO 08:42"
        );
        assert_eq!(compose_bar(None, "19:42", &[], 3), "19:42");
        assert_eq!(compose_bar(Some(""), "19:42", &zones, 0), "19:42");
    }
}
