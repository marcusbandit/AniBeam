//! Subtitles for a receiver: WebVTT is the one text format every Cast receiver renders, so
//! ASS and SRT, from the file or beside it, become plain cues. ASS styling does not survive;
//! drawings and karaoke timing are dropped, the words stay.

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    /// ms
    pub start: u64,
    pub end: u64,
    pub text: String,
}

/// The words of one ASS event as Matroska stores it: `ReadOrder,Layer,Style,Name,MarginL,
/// MarginR,MarginV,Effect,Text`. None for a drawing, which has no words.
pub fn ass_text(block: &str) -> Option<String> {
    let text = block.splitn(9, ',').nth(8)?;
    ass_clean(text)
}

/// One ASS text field as plain lines: override blocks gone, `\N` a line break, `\h` a space.
pub fn ass_clean(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                let mut tag = String::new();
                for t in chars.by_ref() {
                    if t == '}' {
                        break;
                    }
                    tag.push(t);
                }
                // \p1 and up switch to vector drawing until \p0: no words in it.
                if tag.contains("\\p1")
                    || tag.contains("\\p2")
                    || tag.contains("\\p3")
                    || tag.contains("\\p4")
                {
                    return None;
                }
            }
            '\\' => match chars.peek() {
                Some('N') | Some('n') => {
                    chars.next();
                    out.push('\n');
                }
                Some('h') => {
                    chars.next();
                    out.push(' ');
                }
                _ => out.push('\\'),
            },
            _ => out.push(c),
        }
    }
    let lines: Vec<&str> = out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    Some(lines.join("\n"))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn stamp(ms: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

/// A WebVTT file of `cues`. Events at the same time and the same words, which ASS uses for
/// outlines and shadows, show once.
pub fn render(cues: &[Cue]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    let mut last: Option<&Cue> = None;
    for c in cues {
        if last.is_some_and(|l| l.start == c.start && l.end == c.end && l.text == c.text) {
            continue;
        }
        if c.end <= c.start {
            continue;
        }
        out.push_str(&format!(
            "{} --> {}\n{}\n\n",
            stamp(c.start),
            stamp(c.end),
            escape(&c.text)
        ));
        last = Some(c);
    }
    out
}

/// The same cues as SRT. A DLNA renderer plays the file from its start time, so these keep
/// the episode's own times.
pub fn render_srt(cues: &[Cue]) -> String {
    let stamp = |ms: u64| {
        format!(
            "{:02}:{:02}:{:02},{:03}",
            ms / 3_600_000,
            ms / 60_000 % 60,
            ms / 1000 % 60,
            ms % 1000
        )
    };
    let mut out = String::new();
    for (i, c) in cues.iter().filter(|c| c.end > c.start).enumerate() {
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            i + 1,
            stamp(c.start),
            stamp(c.end),
            c.text
        ));
    }
    out
}

fn parse_time(s: &str) -> Option<u64> {
    // h:mm:ss.cc (ASS) or hh:mm:ss,mmm (SRT).
    let s = s.trim().replace(',', ".");
    let mut parts = s.split(':');
    let h: u64 = parts.next()?.trim().parse().ok()?;
    let m: u64 = parts.next()?.parse().ok()?;
    let rest = parts.next()?;
    let (sec, frac) = rest.split_once('.').unwrap_or((rest, "0"));
    let sec: u64 = sec.parse().ok()?;
    let frac_ms = match frac.len() {
        0 => 0,
        1 => frac.parse::<u64>().ok()? * 100,
        2 => frac.parse::<u64>().ok()? * 10,
        _ => frac[..3].parse::<u64>().ok()?,
    };
    Some(((h * 60 + m) * 60 + sec) * 1000 + frac_ms)
}

/// Cues from a whole subtitle file beside the episode, ASS or SRT by its contents.
pub fn parse_file(text: &str) -> Vec<Cue> {
    let text = text.trim_start_matches('\u{feff}');
    if text.contains("[Events]") {
        parse_ass(text)
    } else {
        parse_srt(text)
    }
}

fn parse_ass(text: &str) -> Vec<Cue> {
    let mut fields: Vec<String> = Vec::new();
    let mut cues = Vec::new();
    let mut in_events = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_events = line.eq_ignore_ascii_case("[events]");
            continue;
        }
        if !in_events {
            continue;
        }
        if let Some(f) = line.strip_prefix("Format:") {
            fields = f.split(',').map(|s| s.trim().to_lowercase()).collect();
        } else if let Some(d) = line.strip_prefix("Dialogue:") {
            if fields.is_empty() {
                continue;
            }
            let parts: Vec<&str> = d.splitn(fields.len(), ',').collect();
            let get = |name: &str| {
                fields
                    .iter()
                    .position(|f| f == name)
                    .and_then(|i| parts.get(i))
                    .copied()
            };
            let (Some(start), Some(end), Some(t)) = (get("start"), get("end"), get("text")) else {
                continue;
            };
            let (Some(start), Some(end)) = (parse_time(start), parse_time(end)) else {
                continue;
            };
            if let Some(text) = ass_clean(t).filter(|t| !t.is_empty()) {
                cues.push(Cue { start, end, text });
            }
        }
    }
    cues.sort_by_key(|c| c.start);
    cues
}

fn parse_srt(text: &str) -> Vec<Cue> {
    let mut cues = Vec::new();
    for block in text.replace("\r\n", "\n").split("\n\n") {
        let mut lines = block.lines().filter(|l| !l.trim().is_empty());
        let mut first = lines.next();
        if first.is_some_and(|l| !l.contains("-->")) {
            first = lines.next();
        }
        let Some((a, b)) = first.and_then(|l| l.split_once("-->")) else {
            continue;
        };
        let (Some(start), Some(end)) = (
            parse_time(a),
            parse_time(b.split_whitespace().next().unwrap_or("")),
        ) else {
            continue;
        };
        let text: Vec<String> = lines.map(strip_html).collect();
        let text = text.join("\n").trim().to_string();
        if !text.is_empty() {
            cues.push(Cue { start, end, text });
        }
    }
    cues
}

fn strip_html(line: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in line.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ass_events_lose_their_tags_and_keep_their_words() {
        assert_eq!(
            ass_text("12,0,Default,,0,0,0,,{\\an8\\fs40}Hello,\\Nworld\\hagain").as_deref(),
            Some("Hello,\nworld again")
        );
        assert_eq!(ass_text("3,0,Sign,,0,0,0,,{\\p1}m 0 0 l 10 10{\\p0}"), None);
    }

    #[test]
    fn webvtt_escapes_and_drops_doubles() {
        let c = Cue {
            start: 1000,
            end: 2500,
            text: "a < b".into(),
        };
        let out = render(&[c.clone(), c]);
        assert_eq!(out, "WEBVTT\n\n00:00:01.000 --> 00:00:02.500\na &lt; b\n\n");
    }

    #[test]
    fn sidecar_ass_and_srt_parse() {
        let ass = "[Script Info]\nTitle: x\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.50,0:00:03.00,Default,,0,0,0,,Hi, there\n";
        assert_eq!(
            parse_file(ass),
            vec![Cue {
                start: 1500,
                end: 3000,
                text: "Hi, there".into()
            }]
        );
        let srt = "1\n00:00:01,000 --> 00:00:02,000\n<i>One</i>\n\n2\n00:01:00,250 --> 00:01:01,000\nTwo\nlines\n";
        assert_eq!(
            parse_file(srt),
            vec![
                Cue {
                    start: 1000,
                    end: 2000,
                    text: "One".into()
                },
                Cue {
                    start: 60250,
                    end: 61000,
                    text: "Two\nlines".into()
                },
            ]
        );
    }
}
