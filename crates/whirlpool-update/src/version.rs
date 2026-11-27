//! Semantic versions with pre-release ordering (`4.2.0-rc1 < 4.2.0-rc2 < 4.2.0`).

use std::{cmp::Ordering, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pre: Vec<Ident>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ident {
    Num(u64),
    Text(String),
}

impl Version {
    /// Parses `1.2.3`, `v1.2.3`, `1.2.3-rc.1`, `1.2.3+build`. Missing minor or patch count as zero.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().trim_start_matches(['v', 'V']);
        let text = text.split('+').next()?;
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (text, None),
        };
        let mut numbers = core.split('.');
        let major = numbers.next()?.parse().ok()?;
        let minor = numbers.next().map_or(Some(0), |n| n.parse().ok())?;
        let patch = numbers.next().map_or(Some(0), |n| n.parse().ok())?;
        if numbers.next().is_some() {
            return None;
        }
        let pre = match pre {
            None => Vec::new(),
            Some("") => return None,
            Some(pre) => pre
                .split('.')
                .map(|part| {
                    part.parse()
                        .map_or_else(|_| Ident::Text(part.to_owned()), Ident::Num)
                })
                .collect(),
        };
        Some(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    #[must_use]
    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        for (index, part) in self.pre.iter().enumerate() {
            f.write_str(if index == 0 { "-" } else { "." })?;
            match part {
                Ident::Num(n) => write!(f, "{n}")?,
                Ident::Text(t) => f.write_str(t)?,
            }
        }
        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => compare_pre(&self.pre, &other.pre),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders `rc9 < rc10`: letters first, then a trailing number by value (plain text order puts `rc10` before `rc9`).
fn natural(a: &str, b: &str) -> Ordering {
    fn split(text: &str) -> (&str, Option<u64>) {
        let digits = text.len() - text.trim_end_matches(|c: char| c.is_ascii_digit()).len();
        let (head, tail) = text.split_at(text.len() - digits);
        (head, tail.parse().ok())
    }
    let (ah, an) = split(a);
    let (bh, bn) = split(b);
    ah.cmp(bh).then_with(|| an.cmp(&bn)).then_with(|| a.cmp(b))
}

fn compare_pre(a: &[Ident], b: &[Ident]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let order = match (x, y) {
            (Ident::Num(x), Ident::Num(y)) => x.cmp(y),
            (Ident::Num(_), Ident::Text(_)) => Ordering::Less,
            (Ident::Text(_), Ident::Num(_)) => Ordering::Greater,
            (Ident::Text(x), Ident::Text(y)) => natural(x, y),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect("valid version")
    }

    #[test]
    fn orders_numbers_numerically() {
        assert!(v("4.10.0") > v("4.9.9"));
        assert!(v("v5") > v("4.99.99"));
    }

    #[test]
    fn prerelease_sorts_before_release() {
        assert!(v("4.2.0-rc1") < v("4.2.0"));
        assert!(v("4.2.0-rc1") < v("4.2.0-rc2"));
        assert!(v("4.2.0-rc.2") < v("4.2.0-rc.10"));
        assert!(v("4.2.0-alpha") < v("4.2.0-alpha.1"));
        assert!(v("4.2.0-1") < v("4.2.0-alpha"));
        assert!(v("4.1.9") < v("4.2.0-rc1"));
    }

    #[test]
    fn rc_numbers_order_by_value() {
        assert!(v("4.2.0-rc9") < v("4.2.0-rc10"));
        assert!(v("4.2.0-rc10") < v("4.2.0-rc11"));
        assert!(v("4.2.0-rc99") < v("4.2.0-rc100"));
        assert!(v("4.2.0-beta3") < v("4.2.0-rc1"));
        assert!(v("4.2.0-rc11") < v("4.2.0"));
    }

    #[test]
    fn rejects_garbage() {
        assert!(Version::parse("").is_none());
        assert!(Version::parse("latest").is_none());
        assert!(Version::parse("1.2.3.4").is_none());
        assert!(Version::parse("1.2.3-").is_none());
    }

    #[test]
    fn round_trips() {
        assert_eq!(v("v4.2.0-rc1+abc").to_string(), "4.2.0-rc1");
        assert!(v("4.2.0-rc1").is_prerelease());
        assert!(!v("4.2.0").is_prerelease());
    }
}
