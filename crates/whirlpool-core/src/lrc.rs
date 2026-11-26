//! Synced lyrics: the LRC format and the request for the public LRCLIB service. This is the only
//! place a network request is described, and it only runs when the user turns lyrics on.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub at_ms: u64,
    pub text: String,
}

fn timestamp(tag: &str) -> Option<u64> {
    let (m, rest) = tag.split_once(':')?;
    let (s, frac) = rest.split_once(['.', ':']).unwrap_or((rest, "0"));
    let (m, s): (u64, u64) = (m.trim().parse().ok()?, s.trim().parse().ok()?);
    if s >= 60 || frac.is_empty() || !frac.chars().all(|c| c.is_ascii_digit()) || frac.len() > 3 {
        return None;
    }
    let ms = match frac.len() {
        1 => frac.parse::<u64>().ok()? * 100,
        2 => frac.parse::<u64>().ok()? * 10,
        _ => frac.parse::<u64>().ok()?,
    };
    Some(m * 60_000 + s * 1000 + ms)
}

/// Parses LRC text: `[01:23.45] words`, several stamps per line, and an `[offset:+250]` tag.
/// Lines come back sorted by time. Metadata tags (`[ar:...]`) are skipped.
#[must_use]
pub fn parse(src: &str) -> Vec<Line> {
    let mut out = Vec::new();
    let mut offset: i64 = 0;
    for raw in src.lines() {
        let mut rest = raw.trim();
        let mut stamps = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some(end) = stripped.find(']') else { break };
            let tag = &stripped[..end];
            if let Some(ms) = timestamp(tag) {
                stamps.push(ms);
            } else if let Some(value) = tag.strip_prefix("offset:") {
                offset = value.trim().parse().unwrap_or(0);
            }
            rest = stripped[end + 1..].trim_start();
        }
        for at in stamps {
            let shifted = (at as i64 - offset).max(0) as u64;
            out.push(Line {
                at_ms: shifted,
                text: rest.trim().to_owned(),
            });
        }
    }
    out.sort_by_key(|l| l.at_ms);
    out
}

/// The index of the line to show at `position_ms`; `None` before the first line.
#[must_use]
pub fn line_at(lines: &[Line], position_ms: u64) -> Option<usize> {
    match lines.partition_point(|l| l.at_ms <= position_ms) {
        0 => None,
        n => Some(n - 1),
    }
}

/// Milliseconds until the line after the current one starts, so a caller can sleep exactly that
/// long instead of polling. `None` once the last line is showing (or there are no lines).
#[must_use]
pub fn next_change_ms(lines: &[Line], position_ms: u64) -> Option<u64> {
    let next = lines.partition_point(|l| l.at_ms <= position_ms);
    lines.get(next).map(|l| l.at_ms - position_ms)
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The LRCLIB lookup URL. It carries the track title, artist, album and length, nothing else.
#[must_use]
pub fn request_url(title: &str, artist: &str, album: &str, duration_secs: u32) -> String {
    let mut url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}",
        encode(title.trim()),
        encode(artist.trim())
    );
    if !album.trim().is_empty() {
        url.push_str(&format!("&album_name={}", encode(album.trim())));
    }
    if duration_secs > 0 {
        url.push_str(&format!("&duration={duration_secs}"));
    }
    url
}

/// Pulls the `syncedLyrics` string out of an LRCLIB JSON answer without a JSON dependency here.
#[must_use]
pub fn extract_synced(json: &str) -> Option<String> {
    let key = "\"syncedLyrics\"";
    let at = json.find(key)? + key.len();
    let rest = json[at..].trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return if out.is_empty() { None } else { Some(out) },
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                'r' => {}
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(
                        char::from_u32(u32::from_str_radix(&hex, 16).ok()?).unwrap_or('\u{fffd}'),
                    );
                }
                other => out.push(other),
            },
            c => out.push(c),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[ar:Someone]\n[offset:+500]\n[00:12.00] first\n[00:05.5]zero\n[01:00.250][00:30.00] twice\n\nplain";

    #[test]
    fn parses_stamps_offsets_and_sorting() {
        let lines = parse(SAMPLE);
        let times: Vec<u64> = lines.iter().map(|l| l.at_ms).collect();
        assert_eq!(times, vec![5_000, 11_500, 29_500, 59_750]);
        assert_eq!(lines[0].text, "zero");
        assert_eq!(lines[2].text, "twice");
    }

    #[test]
    fn finds_the_current_line() {
        let lines = parse("[00:01.00]a\n[00:03.00]b\n[00:05.00]c");
        assert_eq!(line_at(&lines, 500), None);
        assert_eq!(line_at(&lines, 1_000), Some(0));
        assert_eq!(line_at(&lines, 4_999), Some(1));
        assert_eq!(line_at(&lines, 99_000), Some(2));
        assert_eq!(line_at(&[], 5), None);
    }

    #[test]
    fn rejects_bad_stamps() {
        assert!(parse("[00:75.00] no").is_empty());
        assert!(parse("[xx:yy] no").is_empty());
    }

    #[test]
    fn url_carries_only_the_track() {
        let url = request_url("Don't Stop", "A & B", "", 215);
        assert_eq!(
            url,
            "https://lrclib.net/api/get?track_name=Don%27t%20Stop&artist_name=A%20%26%20B&duration=215"
        );
        assert!(request_url("x", "y", "Album Z", 0).ends_with("&album_name=Album%20Z"));
    }

    #[test]
    fn extracts_synced_lyrics_from_json() {
        let json = r#"{"id":1,"plainLyrics":"a","syncedLyrics":"[00:01.00] hi \"you\"\n[00:02.00] caf\u00e9"}"#;
        let s = extract_synced(json).expect("present");
        assert_eq!(s, "[00:01.00] hi \"you\"\n[00:02.00] caf\u{e9}");
        assert_eq!(extract_synced(r#"{"syncedLyrics":null}"#), None);
        assert_eq!(extract_synced("{}"), None);
    }
}

#[cfg(test)]
mod wait_tests {
    use super::*;

    #[test]
    fn waits_until_the_next_line() {
        let lines = parse("[00:01.00] a\n[00:03.50] b");
        assert_eq!(next_change_ms(&lines, 0), Some(1000));
        assert_eq!(next_change_ms(&lines, 1000), Some(2500));
        assert_eq!(next_change_ms(&lines, 3500), None);
        assert_eq!(next_change_ms(&[], 10), None);
    }
}
