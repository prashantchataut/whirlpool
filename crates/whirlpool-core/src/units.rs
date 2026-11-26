//! Unit conversion for the palette: `56 KB to MB`, `100 Mbps to MB/s`, `5'11" to cm`, `30 mpg to L/100km`,
//! `72 F to C`. Pure and dependency-free. kB, MB, GB use decimal prefixes (1000); KiB, MiB, GiB use binary
//! ones (1024). An upper-case `B` is a byte and a lower-case `b` is a bit.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dim {
    Length,
    Mass,
    Volume,
    Data,
    Rate,
    Time,
    Speed,
    Temperature,
    Economy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Unit {
    dim: Dim,
    /// Multiply by this to reach the base unit of the dimension.
    scale: f64,
    /// Added after scaling (Celsius and Fahrenheit only).
    offset: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Conversion {
    pub value: f64,
    pub from: String,
    pub to: String,
    pub result: f64,
}

impl Conversion {
    /// `56 KB = 0.056 MB`
    pub fn display(&self) -> String {
        format!(
            "{} {} = {} {}",
            sig(self.value),
            self.from,
            sig(self.result),
            self.to
        )
    }
}

/// Up to six significant digits, no trailing zeros, no exponent for ordinary sizes.
pub fn sig(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0".into();
    }
    let digits = 6 - 1 - v.abs().log10().floor() as i32;
    let places = digits.clamp(0, 12) as usize;
    let text = format!("{v:.places$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    };
    crate::calc::group_thousands(&text)
}

fn u(dim: Dim, scale: f64) -> Option<Unit> {
    Some(Unit {
        dim,
        scale,
        offset: 0.0,
    })
}

/// Data units: prefix (k, M, G, T with optional `i`) then `B` (byte) or `b` (bit), optionally `/s` or `ps`.
fn data_unit(name: &str) -> Option<Unit> {
    let (body, rate) = if let Some(b) = name.strip_suffix("/s") {
        (b, true)
    } else if let Some(b) = name.strip_suffix("ps") {
        (b, true)
    } else {
        (name, false)
    };
    let (prefix, unit) = body.split_at(body.len().checked_sub(1)?);
    let bytes = match unit {
        "B" => 1.0,
        "b" => 0.125,
        _ => return None,
    };
    let (prefix, binary) = match prefix.strip_suffix('i') {
        Some(p) => (p, true),
        None => (prefix, false),
    };
    let step: f64 = if binary { 1024.0 } else { 1000.0 };
    let power = match prefix {
        "" => 0,
        "k" | "K" => 1,
        "M" => 2,
        "G" => 3,
        "T" => 4,
        _ => return None,
    };
    if binary && power == 0 {
        return None;
    }
    // "Mbps" means megabits per second: a rate unit must carry its time part.
    let scale = bytes * step.powi(power);
    if rate {
        u(Dim::Rate, scale)
    } else {
        u(Dim::Data, scale)
    }
}

fn lookup(name: &str) -> Option<Unit> {
    if let Some(unit) = data_unit(name) {
        return Some(unit);
    }
    let n = name.to_lowercase().replace(['.', '°', ' '], "");
    match n.as_str() {
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => u(Dim::Length, 0.001),
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => u(Dim::Length, 0.01),
        "m" | "meter" | "meters" | "metre" | "metres" => u(Dim::Length, 1.0),
        "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => u(Dim::Length, 1000.0),
        "in" | "inch" | "inches" => u(Dim::Length, 0.0254),
        "ft" | "foot" | "feet" => u(Dim::Length, 0.3048),
        "yd" | "yard" | "yards" => u(Dim::Length, 0.9144),
        "mi" | "mile" | "miles" => u(Dim::Length, 1609.344),
        "mg" | "milligram" | "milligrams" => u(Dim::Mass, 1e-6),
        "g" | "gram" | "grams" => u(Dim::Mass, 0.001),
        "kg" | "kilogram" | "kilograms" | "kilo" | "kilos" => u(Dim::Mass, 1.0),
        "t" | "tonne" | "tonnes" => u(Dim::Mass, 1000.0),
        "oz" | "ounce" | "ounces" => u(Dim::Mass, 0.028_349_523_125),
        "lb" | "lbs" | "pound" | "pounds" => u(Dim::Mass, 0.453_592_37),
        "st" | "stone" => u(Dim::Mass, 6.350_293_18),
        "ml" | "milliliter" | "milliliters" => u(Dim::Volume, 0.001),
        "l" | "liter" | "liters" | "litre" | "litres" => u(Dim::Volume, 1.0),
        "tsp" | "teaspoon" | "teaspoons" => u(Dim::Volume, 0.004_928_921_593_75),
        "tbsp" | "tablespoon" | "tablespoons" => u(Dim::Volume, 0.014_786_764_781_25),
        "floz" => u(Dim::Volume, 0.029_573_529_562_5),
        "cup" | "cups" => u(Dim::Volume, 0.236_588_236_5),
        "pt" | "pint" | "pints" => u(Dim::Volume, 0.473_176_473),
        "qt" | "quart" | "quarts" => u(Dim::Volume, 0.946_352_946),
        "gal" | "gallon" | "gallons" => u(Dim::Volume, 3.785_411_784),
        "ms" => u(Dim::Time, 0.001),
        "s" | "sec" | "secs" | "second" | "seconds" => u(Dim::Time, 1.0),
        "min" | "mins" | "minute" | "minutes" => u(Dim::Time, 60.0),
        "h" | "hr" | "hrs" | "hour" | "hours" => u(Dim::Time, 3600.0),
        "d" | "day" | "days" => u(Dim::Time, 86_400.0),
        "wk" | "week" | "weeks" => u(Dim::Time, 604_800.0),
        "m/s" | "mps" => u(Dim::Speed, 1.0),
        "km/h" | "kmh" | "kph" => u(Dim::Speed, 1.0 / 3.6),
        "mph" => u(Dim::Speed, 0.447_04),
        "kn" | "kt" | "knot" | "knots" => u(Dim::Speed, 0.514_444),
        "c" | "celsius" => Some(Unit {
            dim: Dim::Temperature,
            scale: 1.0,
            offset: 0.0,
        }),
        "f" | "fahrenheit" => Some(Unit {
            dim: Dim::Temperature,
            scale: 5.0 / 9.0,
            offset: -32.0 * 5.0 / 9.0,
        }),
        "k" | "kelvin" => Some(Unit {
            dim: Dim::Temperature,
            scale: 1.0,
            offset: -273.15,
        }),
        // Economy base: litres per 100 km. mpg and km/L are inverses of it.
        "l/100km" => u(Dim::Economy, 1.0),
        "mpg" => u(Dim::Economy, -1.0),
        "km/l" | "kmpl" => u(Dim::Economy, -2.0),
        _ => None,
    }
}

fn to_base(value: f64, unit: Unit) -> f64 {
    if unit.dim == Dim::Economy {
        return match unit.scale as i32 {
            1 => value,
            -1 => 235.214_583 / value,
            _ => 100.0 / value,
        };
    }
    value * unit.scale + unit.offset
}

fn from_base(value: f64, unit: Unit) -> f64 {
    if unit.dim == Dim::Economy {
        return match unit.scale as i32 {
            1 => value,
            -1 => 235.214_583 / value,
            _ => 100.0 / value,
        };
    }
    (value - unit.offset) / unit.scale
}

/// `5'11"`, `5' 11`, `6ft` style heights in metres, with the text that follows them.
fn feet_inches(text: &str) -> Option<(f64, &str)> {
    let (feet, rest) = text.split_once('\'')?;
    let feet: f64 = feet.trim().parse().ok()?;
    let rest = rest.trim_start();
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(rest.len());
    let inches: f64 = if end == 0 {
        0.0
    } else {
        rest[..end].parse().ok()?
    };
    let rest = rest[end..]
        .trim_start()
        .strip_prefix('"')
        .unwrap_or(&rest[end..]);
    Some(((feet * 12.0 + inches) * 0.0254, rest))
}

/// Parses `<number> <unit> to <unit>` (also `in`, `as`, `->`, `=`).
pub fn convert(text: &str) -> Option<Conversion> {
    let text = text.trim();
    let (value, from_label, rest, from) = if text.contains('\'') {
        let (metres, rest) = feet_inches(text)?;
        (metres, "m".to_owned(), rest, lookup("m")?)
    } else {
        let end = text
            .find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | ',' | '-' | '+')))
            .unwrap_or(text.len());
        if end == 0 {
            return None;
        }
        let value: f64 = text[..end].replace(',', "").parse().ok()?;
        let rest = text[end..].trim_start();
        let sep = rest
            .match_indices(" to ")
            .chain(rest.match_indices(" in "))
            .chain(rest.match_indices(" as "))
            .chain(rest.match_indices("->"))
            .chain(rest.match_indices(" = "))
            .map(|(i, _)| i)
            .min()?;
        let from_label = rest[..sep].trim().to_owned();
        let from = lookup(&from_label)?;
        (value, from_label, &rest[sep..], from)
    };
    let to_label = rest
        .trim_start()
        .trim_start_matches("->")
        .trim_start_matches('=')
        .trim_start()
        .trim();
    let to_label = ["to ", "in ", "as "]
        .iter()
        .find_map(|p| to_label.strip_prefix(p))
        .unwrap_or(to_label)
        .trim();
    let to = lookup(to_label)?;
    if from.dim != to.dim {
        return None;
    }
    let result = from_base(to_base(value, from), to);
    if !result.is_finite() {
        return None;
    }
    let from_label = if text.contains('\'') {
        text.split(|c: char| c.is_alphabetic())
            .next()
            .unwrap_or("")
            .trim()
            .to_owned()
    } else {
        from_label
    };
    Some(Conversion {
        value,
        from: from_label,
        to: to_label.to_owned(),
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(text: &str) -> f64 {
        convert(text)
            .unwrap_or_else(|| panic!("no conversion for {text}"))
            .result
    }
    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() <= b.abs() * 1e-4 + 1e-9
    }

    #[test]
    fn data_uses_decimal_and_binary_prefixes() {
        assert!(near(res("56 KB to MB"), 0.056));
        assert!(near(res("1 MiB to KiB"), 1024.0));
        assert!(near(res("1 GB to MB"), 1000.0));
        assert!(near(res("8 Mb to MB"), 1.0));
    }

    #[test]
    fn rates_separate_bits_from_bytes() {
        assert!(near(res("100 Mbps to MB/s"), 12.5));
        assert!(near(res("1 GB/s to MB/s"), 1000.0));
    }

    #[test]
    fn heights_in_feet_and_inches() {
        assert!(near(res("5'11\" to cm"), 180.34));
        assert!(near(res("6' to cm"), 182.88));
    }

    #[test]
    fn everyday_units() {
        assert!(near(res("10 km to mi"), 6.213_71));
        assert!(near(res("1 lb to kg"), 0.453_592));
        assert!(near(res("2 gal to L"), 7.570_82));
        assert!(near(res("90 min to h"), 1.5));
        assert!(near(res("60 mph to km/h"), 96.560_6));
    }

    #[test]
    fn temperatures_are_affine() {
        assert!(near(res("212 F to C"), 100.0));
        assert!(near(res("0 C to F"), 32.0));
        assert!(near(res("0 C to K"), 273.15));
        assert!(near(res("-40 C to F"), -40.0));
    }

    #[test]
    fn fuel_economy_inverts() {
        assert!(near(res("30 mpg to L/100km"), 7.840_49));
        assert!(near(res("10 km/L to L/100km"), 10.0));
    }

    #[test]
    fn mismatched_or_unknown_units_do_not_convert() {
        assert!(convert("5 kg to km").is_none());
        assert!(convert("5 foo to bar").is_none());
        assert!(convert("kg to lb").is_none());
        assert!(convert("2+2").is_none());
        assert!(convert("notes").is_none());
    }

    #[test]
    fn display_reads_naturally() {
        assert_eq!(
            convert("56 KB to MB").unwrap().display(),
            "56 KB = 0.056 MB"
        );
        assert_eq!(sig(1234567.0), "1,234,567");
        assert_eq!(sig(0.000_123_456_789), "0.000123457");
    }
}
