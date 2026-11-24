//! Notification history: a bounded, deduplicating list with a do-not-disturb switch.
//!
//! Windows does not let a classic desktop program read other apps' toasts (that needs a packaged app
//! identity), so the history holds what Whirlpool itself raises (updates, charging, battery, copied
//! text) and what scripts push with `whirlpool --notify`. This module is the pure store; the file and
//! the window live elsewhere.

/// Most entries kept; the oldest go first.
pub const MAX_ENTRIES: usize = 200;
/// A repeat of the same source, title and body inside this window only bumps a counter.
pub const DEDUPE_MS: u64 = 60_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub at_ms: u64,
    pub source: String,
    pub title: String,
    pub body: String,
    /// How many times the same notification arrived inside the dedupe window.
    pub count: u32,
    pub read: bool,
}

/// Adds a notification at the front (newest first). Returns `true` when it was stored as a new row,
/// `false` when it was folded into a recent identical one. Empty titles are ignored.
pub fn push(list: &mut Vec<Entry>, now_ms: u64, source: &str, title: &str, body: &str) -> bool {
    let title = title.trim();
    if title.is_empty() {
        return false;
    }
    let (source, body) = (clip(source.trim(), 40), clip(body.trim(), 400));
    let title = clip(title, 120);
    if let Some(top) = list.iter_mut().find(|e| {
        now_ms.saturating_sub(e.at_ms) <= DEDUPE_MS
            && e.source == source
            && e.title == title
            && e.body == body
    }) {
        top.count = top.count.saturating_add(1);
        top.at_ms = now_ms;
        top.read = false;
        return false;
    }
    list.insert(
        0,
        Entry {
            at_ms: now_ms,
            source,
            title,
            body,
            count: 1,
            read: false,
        },
    );
    list.truncate(MAX_ENTRIES);
    true
}

fn clip(text: &str, max: usize) -> String {
    text.chars()
        .map(|c| if c == '\t' { ' ' } else { c })
        .filter(|c| !c.is_control() || *c == '\n')
        .take(max)
        .collect()
}

/// Entries not yet opened.
pub fn unread(list: &[Entry]) -> usize {
    list.iter().filter(|e| !e.read).count()
}

pub fn mark_all_read(list: &mut [Entry]) {
    list.iter_mut().for_each(|e| e.read = true);
}

/// Groups of the same source in order of their newest entry: `(source, indices)`.
pub fn group_by_source(list: &[Entry]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, entry) in list.iter().enumerate() {
        match groups.iter_mut().find(|(s, _)| *s == entry.source) {
            Some((_, rows)) => rows.push(i),
            None => groups.push((entry.source.clone(), vec![i])),
        }
    }
    groups
}

/// "now", "5 min ago", "2 h ago", "3 d ago".
pub fn ago(now_ms: u64, at_ms: u64) -> String {
    let s = now_ms.saturating_sub(at_ms) / 1000;
    match s {
        0..=44 => "now".to_owned(),
        45..=3_599 => format!("{} min ago", (s / 60).max(1)),
        3_600..=86_399 => format!("{} h ago", s / 3_600),
        _ => format!("{} d ago", s / 86_400),
    }
}

/// Tab-separated, one entry per line; newlines inside a body become `\n`. Stable and tiny.
pub fn encode(list: &[Entry]) -> String {
    list.iter()
        .map(|e| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                e.at_ms,
                e.count,
                u8::from(e.read),
                e.source.replace(['\t', '\n'], " "),
                e.title.replace(['\t', '\n'], " "),
                e.body.replace('\t', " ").replace('\n', "\\n"),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reads what [`encode`] wrote; damaged lines are skipped.
pub fn decode(text: &str) -> Vec<Entry> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            let at_ms = f.next()?.parse().ok()?;
            let count = f.next()?.parse().ok()?;
            let read = f.next()? == "1";
            let source = f.next()?.to_owned();
            let title = f.next()?.to_owned();
            let body = f.next()?.replace("\\n", "\n");
            Some(Entry {
                at_ms,
                source,
                title,
                body,
                count,
                read,
            })
        })
        .take(MAX_ENTRIES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_should_put_newest_first_and_ignore_empty_titles() {
        let mut list = Vec::new();
        assert!(push(&mut list, 1_000, "Updates", "A", ""));
        assert!(push(&mut list, 2_000, "Charging", "B", ""));
        assert!(!push(&mut list, 3_000, "x", "   ", "body"));
        assert_eq!(list[0].title, "B");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn push_should_fold_repeats_inside_the_window_only() {
        let mut list = Vec::new();
        push(&mut list, 0, "Battery", "Low", "10%");
        assert!(!push(&mut list, 30_000, "Battery", "Low", "10%"));
        assert_eq!((list.len(), list[0].count), (1, 2));
        assert!(push(
            &mut list,
            30_000 + DEDUPE_MS + 1,
            "Battery",
            "Low",
            "10%"
        ));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn push_should_cap_the_list() {
        let mut list = Vec::new();
        for i in 0..(MAX_ENTRIES as u64 + 20) {
            push(&mut list, i * 100_000, "s", &format!("t{i}"), "");
        }
        assert_eq!(list.len(), MAX_ENTRIES);
        assert_eq!(list[0].title, format!("t{}", MAX_ENTRIES + 19));
    }

    #[test]
    fn unread_and_mark_all_read_should_agree() {
        let mut list = Vec::new();
        push(&mut list, 0, "a", "1", "");
        push(&mut list, 1, "a", "2", "");
        assert_eq!(unread(&list), 2);
        mark_all_read(&mut list);
        assert_eq!(unread(&list), 0);
    }

    #[test]
    fn group_by_source_should_order_by_newest() {
        let mut list = Vec::new();
        push(&mut list, 0, "a", "1", "");
        push(&mut list, 1, "b", "2", "");
        push(&mut list, 2, "a", "3", "");
        let groups = group_by_source(&list);
        assert_eq!(groups[0], ("a".to_owned(), vec![0, 2]));
        assert_eq!(groups[1], ("b".to_owned(), vec![1]));
    }

    #[test]
    fn ago_should_read_naturally() {
        assert_eq!(ago(10_000, 0), "now");
        assert_eq!(ago(300_000, 0), "5 min ago");
        assert_eq!(ago(7_300_000, 0), "2 h ago");
        assert_eq!(ago(3 * 86_400_000, 0), "3 d ago");
        assert_eq!(ago(0, 5_000), "now");
    }

    #[test]
    fn encode_then_decode_should_round_trip_and_skip_damage() {
        let mut list = Vec::new();
        push(
            &mut list,
            5,
            "Updates",
            "New\tversion",
            "line one\nline two",
        );
        let mut text = encode(&list);
        text.push_str("\ngarbage line\n");
        let back = decode(&text);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].title, "New version");
        assert_eq!(back[0].body, "line one\nline two");
    }

    #[test]
    fn push_should_strip_control_characters() {
        let mut list = Vec::new();
        push(&mut list, 0, "s", "Hi\u{7}there", "");
        assert_eq!(list[0].title, "Hithere");
    }
}
