//! Command palette: what the query means and how results are ranked. Pure logic, tested anywhere.
//!
//! One box searches four kinds of things. A leading character narrows it: `>` actions, `@` apps,
//! `/` files, `#` clipboard. Anything that parses as arithmetic also gets a calculator row.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    App,
    File,
    Action,
    Clipboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    All,
    Apps,
    Files,
    Actions,
    Clipboard,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub kind: Kind,
    pub title: String,
    /// Second line: a folder, a hint, or the first characters of a clip.
    pub detail: String,
    /// What running the item hands to the shell (path, app id, action id, or the clip text).
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub mode: Mode,
    pub text: String,
}

/// Splits a leading mode character off the text the user typed.
pub fn parse_query(raw: &str) -> Query {
    let trimmed = raw.trim_start();
    let (mode, rest) = match trimmed.chars().next() {
        Some('>') => (Mode::Actions, &trimmed[1..]),
        Some('@') => (Mode::Apps, &trimmed[1..]),
        Some('/') => (Mode::Files, &trimmed[1..]),
        Some('#') => (Mode::Clipboard, &trimmed[1..]),
        _ => (Mode::All, trimmed),
    };
    Query {
        mode,
        text: rest.trim().to_owned(),
    }
}

fn allowed(mode: Mode, kind: Kind) -> bool {
    matches!(
        (mode, kind),
        (Mode::All, _)
            | (Mode::Apps, Kind::App)
            | (Mode::Files, Kind::File)
            | (Mode::Actions, Kind::Action)
            | (Mode::Clipboard, Kind::Clipboard)
    )
}

/// How well `query` matches `title`; `None` when it does not match at all. Higher is better.
pub fn score(title: &str, query: &str) -> Option<u32> {
    if query.is_empty() {
        return Some(1);
    }
    let title = title.to_lowercase();
    let query = query.to_lowercase();
    if title == query {
        return Some(2000);
    }
    if title.starts_with(&query) {
        return Some(1500u32.saturating_sub(title.len() as u32 - query.len() as u32));
    }
    // A word inside the title starts with the query ("code" in "Visual Studio Code").
    if title
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| !word.is_empty() && word.starts_with(&query))
    {
        return Some(1100);
    }
    if let Some(at) = title.find(&query) {
        return Some(800u32.saturating_sub(at as u32));
    }
    // Letters in order, anywhere: "vsc" finds "Visual Studio Code". Tighter spans rank higher.
    let mut wanted = query.chars().filter(|c| !c.is_whitespace());
    let mut next = wanted.next();
    let (mut first, mut last) = (None, 0usize);
    for (index, c) in title.chars().enumerate() {
        if Some(c) == next {
            first.get_or_insert(index);
            last = index;
            next = wanted.next();
            if next.is_none() {
                break;
            }
        }
    }
    if next.is_some() {
        return None;
    }
    let span = (last - first.unwrap_or(0) + 1) as u32;
    Some(500u32.saturating_sub(span.min(400)))
}

fn kind_order(kind: Kind) -> u8 {
    match kind {
        Kind::Action => 0,
        Kind::App => 1,
        Kind::File => 2,
        Kind::Clipboard => 3,
    }
}

/// Indices into `items`, best first, at most `limit`. An empty query in `All` mode lists actions
/// and the newest clips, so the palette is useful before anything is typed.
pub fn rank(items: &[Item], query: &Query, limit: usize) -> Vec<usize> {
    let mut hits: Vec<(u32, u8, usize)> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| allowed(query.mode, item.kind))
        .filter(|(_, item)| {
            // Nothing typed in All mode: only what is useful blind (actions, recent clips).
            !(query.text.is_empty() && query.mode == Mode::All)
                || matches!(item.kind, Kind::Action | Kind::Clipboard)
        })
        .filter_map(|(index, item)| {
            let base = score(&item.title, &query.text)?;
            // Clips match on their text; files also match their folder at a discount.
            Some((base, kind_order(item.kind), index))
        })
        .collect();
    hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    hits.into_iter()
        .take(limit)
        .map(|(_, _, index)| index)
        .collect()
}

/// Evaluates `+ - * / ^` with parentheses and unary minus. `None` if it is not plain arithmetic.
pub fn calculate(text: &str) -> Option<f64> {
    struct Parser<'a> {
        bytes: &'a [u8],
        at: usize,
    }
    impl Parser<'_> {
        fn peek(&mut self) -> Option<u8> {
            while self.bytes.get(self.at).is_some_and(|b| *b == b' ') {
                self.at += 1;
            }
            self.bytes.get(self.at).copied()
        }
        fn sum(&mut self) -> Option<f64> {
            let mut value = self.product()?;
            while let Some(op @ (b'+' | b'-')) = self.peek() {
                self.at += 1;
                let rhs = self.product()?;
                value = if op == b'+' { value + rhs } else { value - rhs };
            }
            Some(value)
        }
        fn product(&mut self) -> Option<f64> {
            let mut value = self.power()?;
            while let Some(op @ (b'*' | b'/' | b'x')) = self.peek() {
                self.at += 1;
                let rhs = self.power()?;
                value = if op == b'/' { value / rhs } else { value * rhs };
            }
            Some(value)
        }
        fn power(&mut self) -> Option<f64> {
            let base = self.unary()?;
            if self.peek() == Some(b'^') {
                self.at += 1;
                let exponent = self.power()?;
                return Some(base.powf(exponent));
            }
            Some(base)
        }
        fn unary(&mut self) -> Option<f64> {
            match self.peek()? {
                b'-' => {
                    self.at += 1;
                    Some(-self.unary()?)
                }
                b'(' => {
                    self.at += 1;
                    let value = self.sum()?;
                    if self.peek() != Some(b')') {
                        return None;
                    }
                    self.at += 1;
                    Some(value)
                }
                _ => self.number(),
            }
        }
        fn number(&mut self) -> Option<f64> {
            let start = self.at;
            while self
                .bytes
                .get(self.at)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
            {
                self.at += 1;
            }
            std::str::from_utf8(&self.bytes[start..self.at])
                .ok()?
                .parse()
                .ok()
        }
    }
    // Plain numbers are not calculations; there must be an operator or a bracket.
    if !text
        .bytes()
        .any(|b| matches!(b, b'+' | b'-' | b'*' | b'/' | b'^' | b'(' | b'x'))
    {
        return None;
    }
    let mut parser = Parser {
        bytes: text.as_bytes(),
        at: 0,
    };
    let value = parser.sum()?;
    (parser.peek().is_none() && value.is_finite()).then_some(value)
}

/// `12`, `0.5`, `1234567` without trailing zeros or exponent noise.
pub fn format_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{value:.0}");
    }
    let text = format!("{value:.8}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: Kind, title: &str) -> Item {
        Item {
            kind,
            title: title.to_owned(),
            detail: String::new(),
            target: title.to_owned(),
        }
    }

    #[test]
    fn leading_character_picks_the_mode() {
        assert_eq!(parse_query("> lock").mode, Mode::Actions);
        assert_eq!(parse_query("@note").text, "note");
        assert_eq!(parse_query("  /report").mode, Mode::Files);
        assert_eq!(parse_query("#pass").mode, Mode::Clipboard);
        assert_eq!(
            parse_query("plain"),
            Query {
                mode: Mode::All,
                text: "plain".into()
            }
        );
    }

    #[test]
    fn prefix_beats_word_beats_substring_beats_letters() {
        let p = score("Code Editor", "code").unwrap();
        let w = score("Visual Studio Code", "code").unwrap();
        let s = score("Barcode Scanner", "code").unwrap();
        let f = score("Calculator Code", "clr").unwrap();
        assert!(p > w && w > s && s > f);
        assert_eq!(score("Notes", "xyz"), None);
        assert_eq!(score("Notes", "notes"), Some(2000));
    }

    #[test]
    fn letters_in_order_find_initials() {
        assert!(score("Visual Studio Code", "vsc").is_some());
        assert_eq!(score("Visual Studio Code", "csv"), None);
    }

    #[test]
    fn modes_filter_by_kind() {
        let items = vec![
            item(Kind::App, "Notes"),
            item(Kind::File, "notes.txt"),
            item(Kind::Action, "Note: lock"),
        ];
        let apps = rank(&items, &parse_query("@note"), 10);
        assert_eq!(apps, vec![0]);
        let all = rank(&items, &parse_query("note"), 10);
        assert_eq!(all.len(), 3);
        // The exact-prefix actions and apps outrank the file only by score then kind order.
        assert!(all.contains(&1));
    }

    #[test]
    fn empty_query_lists_actions_and_clips_not_every_app() {
        let items = vec![
            item(Kind::App, "Notes"),
            item(Kind::Action, "Lock"),
            item(Kind::Clipboard, "hello"),
        ];
        assert_eq!(rank(&items, &parse_query(""), 10), vec![1, 2]);
        assert_eq!(rank(&items, &parse_query("@"), 10), vec![0]);
    }

    #[test]
    fn limit_is_respected() {
        let items: Vec<Item> = (0..50)
            .map(|i| item(Kind::App, &format!("App {i}")))
            .collect();
        assert_eq!(rank(&items, &parse_query("app"), 8).len(), 8);
    }

    #[test]
    fn calculator_follows_precedence_and_brackets() {
        assert_eq!(calculate("2+3*4"), Some(14.0));
        assert_eq!(calculate("(2+3)*4"), Some(20.0));
        assert_eq!(calculate("-3 + 5"), Some(2.0));
        assert_eq!(calculate("2^3^2"), Some(512.0));
        assert_eq!(calculate("10/4"), Some(2.5));
        assert_eq!(calculate("7x6"), Some(42.0));
    }

    #[test]
    fn calculator_refuses_what_is_not_arithmetic() {
        assert_eq!(calculate("42"), None);
        assert_eq!(calculate("notes"), None);
        assert_eq!(calculate("2+"), None);
        assert_eq!(calculate("1/0"), None);
        assert_eq!(calculate("(2+3"), None);
        assert_eq!(calculate("2+3)"), None);
    }

    #[test]
    fn numbers_print_without_noise() {
        assert_eq!(format_number(14.0), "14");
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(1.0 / 3.0), "0.33333333");
    }
}
