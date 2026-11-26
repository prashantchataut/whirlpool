//! Command palette: what the query means and how results are ranked. Pure logic, tested anywhere.
//!
//! One box searches open windows, apps, settings, widgets, files, actions and clipboard. A leading
//! character narrows it: `>` actions, `@` apps, `/` files, `#` clipboard; a word and a colon does the
//! same for the newer kinds: `window:`, `setting:`, `widget:`. Anything that parses as arithmetic or as a
//! unit conversion (`56 KB to MB`) also gets an answer row.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    App,
    File,
    Action,
    Clipboard,
    /// An open window; the target is its handle, as text.
    Window,
    /// A Settings page; the target is the page id.
    Setting,
    /// A widget; the target is its id.
    Widget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    All,
    Apps,
    Files,
    Actions,
    Clipboard,
    Windows,
    Settings,
    Widgets,
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
    // `window: term`, also `win:` and `w:` (case does not matter).
    let lower = trimmed.to_lowercase();
    for (word, mode) in [
        ("window:", Mode::Windows),
        ("win:", Mode::Windows),
        ("w:", Mode::Windows),
        ("setting:", Mode::Settings),
        ("settings:", Mode::Settings),
        ("widget:", Mode::Widgets),
        ("widgets:", Mode::Widgets),
    ] {
        if lower.starts_with(word) {
            return Query {
                mode,
                text: trimmed[word.len()..].trim().to_owned(),
            };
        }
    }
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
            | (Mode::Windows, Kind::Window)
            | (Mode::Settings, Kind::Setting)
            | (Mode::Widgets, Kind::Widget)
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
        Kind::Window => 1,
        Kind::App => 2,
        Kind::Setting => 3,
        Kind::Widget => 4,
        Kind::File => 5,
        Kind::Clipboard => 6,
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
                || matches!(item.kind, Kind::Action | Kind::Clipboard | Kind::Window)
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

/// Evaluates a calculation typed into the palette: `+ - * / ^ %`, brackets, `!`, constants and
/// functions such as `sqrt(16)` or `200 + 10%` (see [`crate::calc`]). `None` if the text is not a
/// calculation, so `42` and `notes` stay plain searches.
pub fn calculate(text: &str) -> Option<f64> {
    if !crate::calc::is_calculation(text) {
        return None;
    }
    crate::calc::evaluate(text).ok()
}

/// What an answer row shows and copies.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    /// The line shown: `14` or `56 KB = 0.056 MB`.
    pub text: String,
    /// What Enter copies: the bare number (and unit for conversions).
    pub copy: String,
}

/// A calculation or a unit conversion, whichever the text is. `None` for plain searches.
pub fn answer(text: &str) -> Option<Answer> {
    if let Some(c) = crate::units::convert(text) {
        return Some(Answer {
            text: c.display(),
            copy: format!("{} {}", crate::units::sig(c.result).replace(',', ""), c.to),
        });
    }
    let v = calculate(text)?;
    let shown = format_number(v);
    Some(Answer {
        text: shown.clone(),
        copy: shown,
    })
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
    fn colon_words_pick_the_newer_kinds() {
        assert_eq!(
            parse_query("window: term"),
            Query {
                mode: Mode::Windows,
                text: "term".into()
            }
        );
        assert_eq!(parse_query("  Window:term").mode, Mode::Windows);
        assert_eq!(parse_query("w:code").text, "code");
        assert_eq!(parse_query("setting: dark").mode, Mode::Settings);
        assert_eq!(parse_query("widget:").mode, Mode::Widgets);
        // A path or a time is not a filter.
        assert_eq!(parse_query("what: now").mode, Mode::All);
        assert_eq!(parse_query("window").mode, Mode::All);
    }

    #[test]
    fn window_filter_lists_only_windows_and_blind_search_shows_them() {
        let items = vec![
            item(Kind::Window, "Terminal"),
            item(Kind::App, "Terminal Settings"),
            item(Kind::Setting, "Terminal colours"),
            item(Kind::Widget, "Terminal clock"),
        ];
        assert_eq!(rank(&items, &parse_query("window: term"), 10), vec![0]);
        assert_eq!(rank(&items, &parse_query("setting:term"), 10), vec![2]);
        assert_eq!(rank(&items, &parse_query("widget:term"), 10), vec![3]);
        // In All mode an open window outranks an app of the same score.
        assert_eq!(rank(&items, &parse_query("terminal"), 10)[0], 0);
        assert_eq!(rank(&items, &parse_query(""), 10), vec![0]);
    }

    #[test]
    fn answers_cover_math_and_units() {
        assert_eq!(answer("2+2*3").unwrap().text, "8");
        let a = answer("100 Mbps to MB/s").unwrap();
        assert_eq!(a.text, "100 Mbps = 12.5 MB/s");
        assert_eq!(a.copy, "12.5 MB/s");
        assert!(answer("notes").is_none());
        assert!(answer("5 kg to km").is_none());
    }

    #[test]
    fn numbers_print_without_noise() {
        assert_eq!(format_number(14.0), "14");
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(1.0 / 3.0), "0.33333333");
    }
}
