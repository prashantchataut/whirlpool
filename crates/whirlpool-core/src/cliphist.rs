//! Clipboard history and the file shelf: bounded lists with pins, search and expiry.
//! Memory only: nothing here writes to disk, and the history is lost when Whirlpool quits.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub text: String,
    pub pinned: bool,
    pub at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    pub items: Vec<Clip>,
    /// Unpinned entries kept; pinned ones are never dropped.
    pub cap: usize,
}

/// Longest text kept. A huge copy (a whole file) is ignored, not truncated.
pub const MAX_CHARS: usize = 20_000;

impl History {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            items: Vec::new(),
            cap: cap.max(1),
        }
    }

    /// Adds a copy at the top. Blank or oversized text is ignored; a repeat moves to the top and
    /// keeps its pin. Returns whether the list changed.
    pub fn push(&mut self, text: &str, now_ms: u64) -> bool {
        if text.trim().is_empty() || text.chars().count() > MAX_CHARS {
            return false;
        }
        let pinned = match self.items.iter().position(|c| c.text == text) {
            Some(i) => self.items.remove(i).pinned,
            None => false,
        };
        let at = self.items.iter().take_while(|c| c.pinned).count();
        let at = if pinned { 0 } else { at };
        self.items.insert(
            at,
            Clip {
                text: text.to_owned(),
                pinned,
                at_ms: now_ms,
            },
        );
        let mut kept = 0;
        self.items.retain(|c| {
            if c.pinned {
                return true;
            }
            kept += 1;
            kept <= self.cap
        });
        true
    }

    pub fn toggle_pin(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        let mut clip = self.items.remove(index);
        clip.pinned = !clip.pinned;
        let at = if clip.pinned {
            0
        } else {
            self.items.iter().take_while(|c| c.pinned).count()
        };
        self.items.insert(at, clip);
    }

    pub fn clear_unpinned(&mut self) {
        self.items.retain(|c| c.pinned);
    }

    /// Entries containing every word of `query` (case-insensitive), in list order.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<usize> {
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                let t = c.text.to_lowercase();
                words.iter().all(|w| t.contains(w))
            })
            .map(|(i, _)| i)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Link,
    Text,
}

#[must_use]
pub fn kind(text: &str) -> Kind {
    let t = text.trim();
    if !t.contains(char::is_whitespace) && (t.starts_with("http://") || t.starts_with("https://")) {
        Kind::Link
    } else {
        Kind::Text
    }
}

/// One line for a list row: whitespace collapsed, cut with an ellipsis.
#[must_use]
pub fn preview(text: &str, max_chars: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() <= max_chars {
        one
    } else {
        let cut: String = one.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}\u{2026}", cut.trim_end())
    }
}

/// Files held on the notch shelf until the user drags them out or clears them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shelf {
    pub files: Vec<(String, u64)>,
}

pub const SHELF_CAP: usize = 24;

impl Shelf {
    /// Adds paths (newest first, no duplicates). Returns how many were new.
    pub fn add<I: IntoIterator<Item = String>>(&mut self, paths: I, now_ms: u64) -> usize {
        let mut added = 0;
        for p in paths {
            if p.is_empty() || self.files.iter().any(|(q, _)| q.eq_ignore_ascii_case(&p)) {
                continue;
            }
            self.files.insert(0, (p, now_ms));
            added += 1;
        }
        self.files.truncate(SHELF_CAP);
        added
    }

    pub fn remove(&mut self, path: &str) {
        self.files.retain(|(p, _)| p != path);
    }

    /// Drops entries older than `max_age_ms` (0 keeps everything) and entries `exists` rejects.
    pub fn prune(&mut self, now_ms: u64, max_age_ms: u64, exists: impl Fn(&str) -> bool) {
        self.files.retain(|(p, at)| {
            (max_age_ms == 0 || now_ms.saturating_sub(*at) <= max_age_ms) && exists(p)
        });
    }

    #[must_use]
    pub fn file_name(path: &str) -> &str {
        path.rsplit(['\\', '/']).next().unwrap_or(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_and_repeats_move_up() {
        let mut h = History::new(10);
        assert!(h.push("a", 1));
        assert!(h.push("b", 2));
        assert!(h.push("a", 3));
        let texts: Vec<_> = h.items.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, ["a", "b"]);
        assert_eq!(h.items[0].at_ms, 3);
    }

    #[test]
    fn ignores_blank_and_huge() {
        let mut h = History::new(3);
        assert!(!h.push("   \n", 1));
        assert!(!h.push(&"x".repeat(MAX_CHARS + 1), 1));
        assert!(h.push(&"x".repeat(MAX_CHARS), 1));
    }

    #[test]
    fn cap_spares_pins() {
        let mut h = History::new(2);
        h.push("keep", 1);
        h.toggle_pin(0);
        for (i, t) in ["a", "b", "c", "d"].iter().enumerate() {
            h.push(t, 10 + i as u64);
        }
        let texts: Vec<_> = h.items.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, ["keep", "d", "c"]);
        h.push("keep", 99);
        assert!(h.items[0].pinned && h.items[0].text == "keep" && h.items.len() == 3);
        h.clear_unpinned();
        assert_eq!(h.items.len(), 1);
    }

    #[test]
    fn pins_sort_to_the_top_and_unpin_returns() {
        let mut h = History::new(5);
        for t in ["a", "b", "c"] {
            h.push(t, 1);
        }
        // order: c b a
        h.toggle_pin(2); // a
        assert_eq!(h.items[0].text, "a");
        h.toggle_pin(0);
        assert!(!h.items[0].pinned);
    }

    #[test]
    fn search_matches_all_words() {
        let mut h = History::new(5);
        h.push("Hello World", 1);
        h.push("hello there", 2);
        assert_eq!(h.search("hello"), vec![0, 1]);
        assert_eq!(h.search("WORLD hel"), vec![1]);
        assert!(h.search("zzz").is_empty());
        assert_eq!(h.search("  ").len(), 2);
    }

    #[test]
    fn kinds_and_previews() {
        assert_eq!(kind(" https://a.b/c "), Kind::Link);
        assert_eq!(kind("see https://a.b"), Kind::Text);
        assert_eq!(preview("a   b\n c", 20), "a b c");
        assert_eq!(preview("abcdefghij", 5), "abcd\u{2026}");
    }

    #[test]
    fn shelf_dedupes_caps_and_prunes() {
        let mut s = Shelf::default();
        assert_eq!(
            s.add(
                [
                    "C:\\a.txt".to_owned(),
                    "c:\\A.txt".to_owned(),
                    "C:\\b.txt".to_owned()
                ],
                5
            ),
            2
        );
        assert_eq!(s.files[0].0, "C:\\b.txt");
        assert_eq!(Shelf::file_name("C:\\x\\y.png"), "y.png");
        s.prune(100, 50, |_| true);
        assert!(s.files.is_empty());
        s.add((0..40).map(|i| format!("f{i}")), 1);
        assert_eq!(s.files.len(), SHELF_CAP);
        s.prune(2, 0, |p| p != "f39");
        assert_eq!(s.files.len(), SHELF_CAP - 1);
        s.remove("f38");
        assert_eq!(s.files.len(), SHELF_CAP - 2);
    }
}
