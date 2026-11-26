//! A local `.ics` calendar export for the meetings widget. Reads events, not recurrence rules: a
//! repeating series shows only the occurrences the file lists. Times are unix seconds; a time without
//! a zone (floating) is read in the caller's local offset.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub title: String,
    pub start: i64,
    pub end: i64,
    pub all_day: bool,
    pub location: String,
    /// A meeting link, only for hosts we recognise.
    pub join: Option<String>,
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `20260314T093000Z`, `20260314T093000`, `20260314` -> (unix seconds, all_day).
fn parse_time(value: &str, offset_secs: i64) -> Option<(i64, bool)> {
    let v = value.trim();
    let digits = |s: &str| s.parse::<i64>().ok();
    if v.len() < 8 || !v.is_char_boundary(8) {
        return None;
    }
    let (y, m, d) = (digits(&v[0..4])?, digits(&v[4..6])?, digits(&v[6..8])?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let day = days_from_civil(y, m, d) * 86_400;
    if v.len() == 8 {
        return Some((day - offset_secs, true));
    }
    let t = v.get(9..15)?;
    let secs = digits(&t[0..2])? * 3600 + digits(&t[2..4])? * 60 + digits(&t[4..6])?;
    let utc = v.ends_with('Z');
    Some((day + secs - if utc { 0 } else { offset_secs }, false))
}

fn unescape(text: &str) -> String {
    text.replace("\\n", " ")
        .replace("\\N", " ")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}

/// Hosts that get a Join action. Anything else stays text.
pub fn join_link(text: &str) -> Option<String> {
    for word in text.split(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '(' | ')'))
    {
        let Some(rest) = word.strip_prefix("https://") else {
            continue;
        };
        let host = rest
            .split(['/', '?', '#'])
            .next()
            .unwrap_or("")
            .to_lowercase();
        let known = [
            "meet.google.com",
            "teams.microsoft.com",
            "teams.live.com",
            "zoom.us",
            "webex.com",
        ];
        if known
            .iter()
            .any(|k| host == *k || host.ends_with(&format!(".{k}")))
        {
            return Some(word.to_owned());
        }
    }
    None
}

pub fn parse(text: &str, offset_secs: i64) -> Vec<Event> {
    // Unfold continuation lines (a line starting with a space or tab continues the previous one).
    let mut lines: Vec<String> = Vec::new();
    for raw in text.lines() {
        if (raw.starts_with(' ') || raw.starts_with('\t')) && !lines.is_empty() {
            if let Some(last) = lines.last_mut() {
                last.push_str(&raw[1..]);
            }
        } else {
            lines.push(raw.trim_end().to_owned());
        }
    }
    let mut events = Vec::new();
    let mut current: Option<(String, Option<(i64, bool)>, Option<i64>, String, String)> = None;
    for line in lines {
        if line == "BEGIN:VEVENT" {
            current = Some((String::new(), None, None, String::new(), String::new()));
        } else if line == "END:VEVENT" {
            if let Some((title, start, end, location, desc)) = current.take()
                && let Some((start, all_day)) = start
            {
                let end = end.unwrap_or(if all_day { start + 86_400 } else { start });
                let join = join_link(&location).or_else(|| join_link(&desc));
                events.push(Event {
                    title: if title.is_empty() {
                        "(no title)".into()
                    } else {
                        title
                    },
                    start,
                    end,
                    all_day,
                    location,
                    join,
                });
            }
        } else if let Some(cur) = current.as_mut() {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            let key = name.split(';').next().unwrap_or("");
            match key {
                "SUMMARY" => cur.0 = unescape(value),
                "DTSTART" => cur.1 = parse_time(value, offset_secs),
                "DTEND" => cur.2 = parse_time(value, offset_secs).map(|t| t.0),
                "LOCATION" => cur.3 = unescape(value),
                "DESCRIPTION" => cur.4 = unescape(value),
                _ => {}
            }
        }
    }
    events.sort_by_key(|e| e.start);
    events
}

/// Events that have not ended at `now`, soonest first, at most `limit`.
pub fn upcoming(events: &[Event], now: i64, limit: usize) -> Vec<&Event> {
    events
        .iter()
        .filter(|e| e.end.max(e.start) >= now)
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Design review\r\nDTSTART:20260314T093000Z\r\nDTEND:20260314T100000Z\r\nDESCRIPTION:Join https://meet.google.com/abc-defg-hij\\nbring notes\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Holiday\r\nDTSTART;VALUE=DATE:20260320\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Lunch\\, with Sam\r\nDTSTART:20260314T120000\r\nLOCATION:https://evil.example/zoom.us\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[test]
    fn reads_titles_times_and_join_links() {
        let e = parse(SAMPLE, 0);
        assert_eq!(e.len(), 3);
        assert_eq!(e[0].title, "Design review");
        assert_eq!(e[0].end - e[0].start, 1800);
        assert_eq!(
            e[0].join.as_deref(),
            Some("https://meet.google.com/abc-defg-hij")
        );
        // 2026-03-14T09:30:00Z
        assert_eq!(e[0].start, 1_773_480_600);
    }

    #[test]
    fn floating_times_use_the_local_offset_and_all_day_is_flagged() {
        let e = parse(SAMPLE, 3600);
        let lunch = e.iter().find(|x| x.title == "Lunch, with Sam").unwrap();
        assert_eq!(lunch.start, 1_773_489_600 - 3600);
        let holiday = e.iter().find(|x| x.all_day).unwrap();
        assert_eq!(holiday.end - holiday.start, 86_400);
    }

    #[test]
    fn only_known_hosts_get_a_join_action() {
        assert_eq!(
            join_link("see https://zoom.us/j/123"),
            Some("https://zoom.us/j/123".into())
        );
        assert_eq!(
            join_link("https://us02web.zoom.us/j/1"),
            Some("https://us02web.zoom.us/j/1".into())
        );
        assert_eq!(join_link("https://evil.example/zoom.us"), None);
        assert_eq!(join_link("https://notzoom.us/j/1"), None);
        assert_eq!(join_link("http://zoom.us/j/1"), None);
    }

    #[test]
    fn upcoming_skips_finished_events() {
        let e = parse(SAMPLE, 0);
        let now = e[0].end + 1;
        let next = upcoming(&e, now, 5);
        assert!(next.iter().all(|x| x.title != "Design review"));
        assert_eq!(upcoming(&e, 0, 1).len(), 1);
    }

    #[test]
    fn garbage_is_ignored() {
        assert!(parse("not a calendar", 0).is_empty());
        assert!(parse("BEGIN:VEVENT\nDTSTART:soon\nEND:VEVENT", 0).is_empty());
    }
}
