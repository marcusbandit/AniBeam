//! Release notes: an app's CHANGELOG.md read into its versions, newest first, and where What's
//! new draws the line between what a person has read and what is new to them. Each shell
//! keeps a [`Seen`] between runs and draws the [`WhatsNew`] this hands back; every rule for
//! both lives here, so the desktop and the phone open, mark, dim and close the notes the same
//! way.
//!
//! Each release is curated for the person using the app, not dumped PR by PR:
//!
//! ```text
//! ## 2.5.0 · Watch together · 2026-11-21
//!
//! We're happy to bring Watch together to Anibeam, along with 13 languages and a simpler layout.
//!
//! - **Watch together.** Start a room from the player and invite friends with a code or link.
//! - **13 languages.** Pick your language during setup or in Settings > Appearance.
//!
//! ### Fixes & polish
//!
//! Fixes for multi-season tracking, Jikan outages, casting, and narrow-window layouts.
//! ```
//!
//! A heading per version, newest first, with the day it was made and an optional title between
//! the two (`## 2.5.0 · Watch together · 2026-11-21`). Under each heading, freeform paragraphs,
//! bullet lists (with optional `**bold**` lead-ins and inline `` `code` ``), and optional `### `
//! subheadings appear in the order written. `# ` titles and `<!-- -->` comments are skipped.

use serde::{Deserialize, Serialize};

/// The most characters a version's title may have: one line beside its number.
pub const MAX_TITLE: usize = 40;

/// The most bullet items one version may list before it starts reading like a commit log.
pub const MAX_ITEMS_PER_VERSION: usize = 15;

/// One version's notes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Notes {
    pub version: String,
    /// `Watch together`, the version's own name; empty for a version known by its number.
    pub title: String,
    /// `2026-11-21`, as the heading has it; empty when it has none.
    pub date: String,
    /// Ordered content blocks: prose paragraphs, optional subheadings, and bullet lists.
    pub blocks: Vec<Block>,
}

/// One block inside a version's notes, in the order written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Block {
    /// A freeform paragraph, such as an opening note or a short fixes roundup.
    Paragraph {
        /// Plain text with Markdown emphasis and backticks removed.
        text: String,
        /// Inline HTML (`<b>`, `<code>`, escaped) for rich text renderers.
        rich: String,
    },
    /// An optional short subheading, such as `Heads up` or `Fixes & polish`.
    Heading { text: String },
    /// A bullet list of notable changes.
    List { items: Vec<Item> },
}

/// One bullet item inside a [`Block::List`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Item {
    /// Optional bold lead-in parsed from `- **Lead.** Body` or `- **Lead:** Body`, with
    /// trailing punctuation removed (e.g. `Watch together`); empty when the item has none.
    pub lead: String,
    /// Plain text after the lead-in, or the full plain text when `lead` is empty.
    pub body: String,
    /// Full plain text of the item, with Markdown emphasis and backticks removed.
    pub text: String,
    /// Full inline HTML (`<b>`, `<code>`, escaped) for rich text renderers.
    pub rich: String,
}

/// The versions in `text`, in the file's order. Forgiving: anything it cannot place before the
/// first version heading is skipped, so a slip never costs the sheet. [`lint`] is the strict
/// reading.
pub fn parse(text: &str) -> Vec<Notes> {
    let mut out: Vec<Notes> = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let mut list: Vec<String> = Vec::new();

    let flush_para = |out: &mut Vec<Notes>, para: &mut Vec<String>| {
        if para.is_empty() {
            return;
        }
        let raw = para.join(" ");
        para.clear();
        if let Some(n) = out.last_mut() {
            let (text, rich) = inline(&raw);
            n.blocks.push(Block::Paragraph { text, rich });
        }
    };
    let flush_list = |out: &mut Vec<Notes>, list: &mut Vec<String>| {
        if list.is_empty() {
            return;
        }
        let items: Vec<Item> = list.drain(..).map(|raw| make_item(&raw)).collect();
        if let Some(n) = out.last_mut() {
            n.blocks.push(Block::List { items });
        }
    };

    for (_, line) in read(text) {
        match line {
            Line::Blank => {
                flush_para(&mut out, &mut para);
                flush_list(&mut out, &mut list);
            }
            Line::Title => {}
            Line::Heading {
                version,
                title,
                date,
                ..
            } => {
                flush_para(&mut out, &mut para);
                flush_list(&mut out, &mut list);
                out.push(Notes {
                    version,
                    title,
                    date,
                    blocks: Vec::new(),
                });
            }
            Line::Subheading(heading) => {
                if out.is_empty() {
                    continue;
                }
                flush_para(&mut out, &mut para);
                flush_list(&mut out, &mut list);
                if !heading.is_empty()
                    && let Some(n) = out.last_mut()
                {
                    n.blocks.push(Block::Heading { text: heading });
                }
            }
            Line::Item(item) => {
                if out.is_empty() {
                    continue;
                }
                flush_para(&mut out, &mut para);
                if !item.is_empty() {
                    list.push(item);
                }
            }
            Line::Text { text, indented } => {
                if out.is_empty() {
                    continue;
                }
                if indented && let Some(last) = list.last_mut() {
                    last.push(' ');
                    last.push_str(&text);
                } else {
                    flush_list(&mut out, &mut list);
                    para.push(text);
                }
            }
        }
    }
    flush_para(&mut out, &mut para);
    flush_list(&mut out, &mut list);
    out
}

/// Each way `text` breaks the file's shape, one sentence naming the line; empty when it is
/// fine. `version` is the app's own, which the newest heading has to be.
pub fn lint(text: &str, version: &str) -> Vec<String> {
    let mut problems = dashes(text);
    let mut above: Option<String> = None;
    let mut first = true;
    let mut headings = 0usize;
    let mut heading_at = 0usize;
    let mut has_content = false;
    let mut list_items = 0usize;
    let mut open_sub: Option<usize> = None;
    let mut in_list = false;

    let close_sub = |problems: &mut Vec<String>, open_sub: &mut Option<usize>| {
        if let Some(at) = open_sub.take() {
            problems.push(format!("line {at}: a subheading with nothing under it"));
        }
    };
    let close_version = |problems: &mut Vec<String>,
                         headings: usize,
                         heading_at: usize,
                         has_content: bool,
                         list_items: usize| {
        if headings > 1 && !has_content {
            problems.push(format!("line {heading_at}: a version with no notes"));
        }
        if list_items > MAX_ITEMS_PER_VERSION {
            problems.push(format!(
                "line {heading_at}: {list_items} list items, {MAX_ITEMS_PER_VERSION} at most; group related changes and sum up small fixes in a line or two"
            ));
        }
    };

    for (n, line) in read(text) {
        match line {
            Line::Blank => {
                in_list = false;
            }
            Line::Title => {
                in_list = false;
            }
            Line::Heading {
                version: v,
                title,
                date,
                exact,
            } => {
                in_list = false;
                close_sub(&mut problems, &mut open_sub);
                if headings > 0 {
                    close_version(&mut problems, headings, heading_at, has_content, list_items);
                }
                headings += 1;
                heading_at = n;
                has_content = false;
                list_items = 0;
                if !exact {
                    problems.push(format!(
                        "line {n}: write a heading as \"## 2.5.0 · 2026-11-21\", or with a title as \"## 2.5.0 · Watch together · 2026-11-21\""
                    ));
                }
                let len = title.chars().count();
                if len > MAX_TITLE {
                    problems.push(format!(
                        "line {n}: a title of {len} characters, {MAX_TITLE} at most: \"{title}\""
                    ));
                }
                if parts(&v).is_none() {
                    problems.push(format!("line {n}: \"{v}\" is not a version like 2.5.0"));
                }
                if !date.is_empty() && day_parts(&date).is_none() {
                    problems.push(format!("line {n}: \"{date}\" is not a day like 2026-11-21"));
                }
                if first && v != version {
                    problems.push(format!(
                        "line {n}: the newest heading is {v}, but the app is {version}"
                    ));
                }
                if let Some(prev) = &above
                    && !newer(prev, &v)
                {
                    problems.push(format!("line {n}: {v} is not older than {prev} above it"));
                }
                first = false;
                above = Some(v);
            }
            Line::Subheading(heading) => {
                in_list = false;
                close_sub(&mut problems, &mut open_sub);
                if above.is_none() {
                    problems.push(format!("line {n}: a subheading before any version"));
                } else if heading.is_empty() {
                    problems.push(format!("line {n}: an empty subheading"));
                } else {
                    open_sub = Some(n);
                }
            }
            Line::Item(item) => {
                if above.is_none() {
                    problems.push(format!("line {n}: a list item before any version"));
                    continue;
                }
                if item.is_empty() {
                    problems.push(format!("line {n}: an empty list item"));
                    continue;
                }
                open_sub = None;
                has_content = true;
                list_items += 1;
                in_list = true;
            }
            Line::Text { indented, .. } => {
                // Words above the first version heading are the file's own.
                if above.is_none() {
                    continue;
                }
                if !(indented && in_list) {
                    in_list = false;
                }
                open_sub = None;
                has_content = true;
            }
        }
    }
    close_sub(&mut problems, &mut open_sub);
    if headings == 0 {
        problems.push("no version heading".to_string());
    } else {
        close_version(&mut problems, headings, heading_at, has_content, list_items);
    }
    problems
}

fn dashes(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        if raw.contains(['\u{2014}', '\u{2013}']) {
            problems.push(format!(
                "line {}: a long dash; use a comma or end the sentence",
                n + 1
            ));
        }
    }
    problems
}

/// Turns a raw bullet line into an [`Item`], extracting an optional leading `**Lead.**` or
/// `**Lead:**` span for shells that render lead-ins separately (such as Slint on phones).
fn make_item(raw: &str) -> Item {
    let (text, rich) = inline(raw);
    let (lead, body) = split_lead(raw).unwrap_or_else(|| (String::new(), text.clone()));
    Item {
        lead,
        body,
        text,
        rich,
    }
}

fn split_lead(raw: &str) -> Option<(String, String)> {
    let rest = raw.strip_prefix("**")?;
    let end = rest.find("**")?;
    let lead_raw = &rest[..end];
    if lead_raw.is_empty() {
        return None;
    }
    let after = &rest[end + 2..];
    // Optional trailing punctuation right outside the closing `**`, then whitespace + body.
    let after = after.strip_prefix(['.', ':']).unwrap_or(after);
    let first_char = after.chars().next()?;
    if !first_char.is_whitespace() {
        return None;
    }
    let body_raw = after.trim();
    if body_raw.is_empty() {
        return None;
    }
    let (lead_plain, _) = inline(lead_raw);
    let lead = lead_plain.trim().trim_end_matches(['.', ':']).trim();
    if lead.is_empty() {
        return None;
    }
    let (body, _) = inline(body_raw);
    Some((lead.to_string(), body))
}

/// Converts inline Markdown (`**bold**` and `` `code` ``) into `(plain, rich)` where `rich` is
/// HTML-escaped and uses `<b>` and `<code>` tags for Qt `Text.StyledText`.
fn inline(raw: &str) -> (String, String) {
    let mut plain = String::with_capacity(raw.len());
    let mut rich = String::with_capacity(raw.len() + 16);
    let mut i = 0;
    let bytes = raw.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'`'
            && let Some(rel) = raw[i + 1..].find('`')
            && rel > 0
        {
            let span = &raw[i + 1..i + 1 + rel];
            plain.push_str(span);
            rich.push_str("<code>");
            push_escaped(&mut rich, span);
            rich.push_str("</code>");
            i += rel + 2;
            continue;
        }
        if raw[i..].starts_with("**")
            && let Some(rel) = raw[i + 2..].find("**")
            && rel > 0
        {
            let span = &raw[i + 2..i + 2 + rel];
            let (inner_plain, inner_rich) = inline(span);
            plain.push_str(&inner_plain);
            rich.push_str("<b>");
            rich.push_str(&inner_rich);
            rich.push_str("</b>");
            i += rel + 4;
            continue;
        }
        let ch = raw[i..].chars().next().unwrap();
        plain.push(ch);
        push_escaped_char(&mut rich, ch);
        i += ch.len_utf8();
    }
    (plain, rich)
}

fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        push_escaped_char(out, ch);
    }
}

fn push_escaped_char(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        _ => out.push(ch),
    }
}

/// Whether `a` is a later version than `b`. Versions are three numbers, 2.0.8; one that is
/// not is never the later.
pub fn newer(a: &str, b: &str) -> bool {
    match (parts(a), parts(b)) {
        (Some(a), Some(b)) => a > b,
        (Some(_), None) => true,
        _ => false,
    }
}

/// The app version in one the core logged as it started: `2.1.0` out of `2.1.0`,
/// `2.1.0.r6.gf1d3824` or `2.1.0.dirty`, and `2.1.0` out of a nightly's
/// `2.1.0-nightly.20261030.12`. None when it does not start with three numbers, as a build
/// described from one of the phone's tags (`android.v2.0.4`) does not.
pub fn version_of(logged: &str) -> Option<String> {
    let base = logged.split('-').next()?;
    let mut numbers = base.splitn(4, '.');
    let version = format!(
        "{}.{}.{}",
        numbers.next()?,
        numbers.next()?,
        numbers.next()?
    );
    parts(&version).map(|_| version)
}

/// What a shell keeps between runs, so the notes know what this person has read: the newest
/// version whose notes they closed, and the version they had before it. Both empty on an
/// install that has never closed them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Seen {
    pub version: String,
    pub from: String,
}

impl Seen {
    /// Whether the notes open by themselves as `current` starts: it is newer than the notes
    /// last closed, or nothing was ever closed on an install in use before the notes
    /// existed. A fresh install never opens them; see [`Seen::started`].
    pub fn due(&self, current: &str, used_before: bool) -> bool {
        if self.version.is_empty() {
            used_before
        } else {
            newer(current, &self.version)
        }
    }

    /// What to keep once the notes close on `current`. Closing them again changes nothing.
    pub fn closed(&self, current: &str) -> Seen {
        if self.version.is_empty() || newer(current, &self.version) {
            Seen {
                version: current.to_string(),
                from: self.version.clone(),
            }
        } else {
            self.clone()
        }
    }

    /// What to keep as `current` starts, when that changes: a fresh install counts its own
    /// notes as read, so they wait for its first update.
    pub fn started(&self, current: &str, used_before: bool) -> Option<Seen> {
        (self.version.is_empty() && !used_before).then(|| self.closed(current))
    }
}

/// The notes as What's new shows them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WhatsNew {
    /// Under the title: "2.0.4 → 2.0.8", "Updated to 2.0.8", or "Version 2.0.8".
    pub note: String,
    /// The words on the line, "You were on 2.0.4"; empty for no line.
    pub line: String,
    /// How many versions sit above the line, which goes under the last of them; 0 for none.
    pub above: usize,
    /// The versions, newest first, none newer than the app.
    pub versions: Vec<Shown>,
}

/// One version as What's new lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Shown {
    pub version: String,
    /// The version's title, or empty.
    pub title: String,
    /// "24 Oct 2026", or empty.
    pub day: String,
    /// Drawn at full strength: every version, except those under a line that marks where
    /// the person came from, which they have read.
    pub lit: bool,
    pub blocks: Vec<Block>,
}

/// What's new for `current`, read with `seen` and whether this install ran before the
/// notes existed. While the notes are due the line sits at the version last closed. Once
/// they are closed it stays at the version that update came from, when that was known; an
/// update from a version that kept no record, or a downgrade, has no line once closed.
pub fn whats_new(notes: &[Notes], current: &str, seen: &Seen, used_before: bool) -> WhatsNew {
    let versions: Vec<&Notes> = notes
        .iter()
        .filter(|n| !newer(&n.version, current))
        .collect();
    let due = seen.due(current, used_before);
    let from = if due {
        seen.version.as_str()
    } else if newer(&seen.version, current) {
        ""
    } else {
        seen.from.as_str()
    };
    let (above, known) = if !from.is_empty() {
        let n = versions
            .iter()
            .take_while(|n| newer(&n.version, from))
            .count();
        (n, n > 0)
    } else if due {
        // From a version that kept no record: this version's notes go above the line, and
        // nothing under it is dimmed, since nobody knows what was read.
        let n = usize::from(versions.first().is_some_and(|n| n.version == current));
        (n, false)
    } else {
        (0, false)
    };
    let (note, line) = match (above, known) {
        (0, _) => (format!("Version {current}"), String::new()),
        (_, false) => (
            format!("Updated to {current}"),
            "Before this update".to_string(),
        ),
        (_, true) => (format!("{from} → {current}"), format!("You were on {from}")),
    };
    WhatsNew {
        note,
        line,
        above,
        versions: versions
            .into_iter()
            .enumerate()
            .map(|(i, n)| Shown {
                version: n.version.clone(),
                title: n.title.clone(),
                day: day(&n.date),
                lit: !known || i < above,
                blocks: n.blocks.clone(),
            })
            .collect(),
    }
}

/// `2026-10-24` as a person reads it: `24 Oct 2026`. Anything else comes back as it was.
pub fn day(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    match day_parts(date) {
        Some((y, m, d)) => format!("{d} {} {y}", MONTHS[m as usize - 1]),
        None => date.to_string(),
    }
}

enum Line {
    Blank,
    /// `# Anibeam for Android`
    Title,
    /// `## 2.0.8 · 2026-10-24` or `## 2.5.0 · Watch together · 2026-11-21`, read leniently;
    /// `exact` when it is written just so.
    Heading {
        version: String,
        title: String,
        date: String,
        exact: bool,
    },
    /// `### Fixes & polish`
    Subheading(String),
    /// `- **Watch together.** Start a room...`
    Item(String),
    /// A prose line, or an indented continuation of a bullet item.
    Text {
        text: String,
        indented: bool,
    },
}

/// The file's lines with their numbers, counted from 1, skipping HTML comments while keeping
/// blank lines so paragraph and list breaks stay intact.
fn read(text: &str) -> Vec<(usize, Line)> {
    let mut out = Vec::new();
    let mut comment = false;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if comment {
            comment = !line.contains("-->");
            continue;
        }
        if line.starts_with("<!--") {
            comment = !line.contains("-->");
            continue;
        }
        if line.is_empty() {
            out.push((i + 1, Line::Blank));
            continue;
        }
        let read = if let Some(rest) = line.strip_prefix("###")
            && (rest.is_empty() || rest.starts_with(' '))
        {
            Line::Subheading(rest.trim().to_string())
        } else if let Some(rest) = line.strip_prefix("## ") {
            heading(rest.trim())
        } else if line.starts_with("# ") {
            Line::Title
        } else if line == "-" || line.starts_with("- ") {
            Line::Item(line.strip_prefix('-').unwrap_or("").trim().to_string())
        } else {
            let indented = raw.starts_with("  ") || raw.starts_with('\t');
            Line::Text {
                text: line.to_string(),
                indented,
            }
        };
        out.push((i + 1, read));
    }
    out
}

fn heading(text: &str) -> Line {
    let mut words = text.split_whitespace();
    let version = words
        .next()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();
    let date = text
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_ascii_digit()))
        .rfind(|w| w.len() == 10 && w.as_bytes()[4] == b'-')
        .unwrap_or("")
        .to_string();
    // The title is whatever sits between the number and the day, each set off by a " · "
    let fields: Vec<&str> = text.split(" · ").map(str::trim).collect();
    let title = match fields.as_slice() {
        [_, title, _] => title.to_string(),
        _ => String::new(),
    };
    let exact = !date.is_empty()
        && if title.is_empty() {
            text == format!("{version} · {date}")
        } else {
            text == format!("{version} · {title} · {date}")
        };
    Line::Heading {
        version,
        title,
        date,
        exact,
    }
}

fn parts(v: &str) -> Option<[u64; 3]> {
    digits::<3>(v)
}

/// Exactly `N` dot-separated whole numbers: `digits::<3>("2.1.0")`. The update check reads
/// a nightly's day and run with it too.
pub(crate) fn digits<const N: usize>(s: &str) -> Option<[u64; N]> {
    let mut parts = s.split('.');
    let mut out = [0; N];
    for slot in &mut out {
        let n = parts.next()?;
        if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = n.parse().ok()?;
    }
    parts.next().is_none().then_some(out)
}

/// Whether `version` has anything to show: main moves to a new version before its notes are
/// written, and an empty heading never opens What's new by itself.
pub fn has_lines(notes: &[Notes], version: &str) -> bool {
    notes
        .iter()
        .find(|n| n.version == version)
        .is_some_and(|n| !n.blocks.is_empty())
}

fn day_parts(date: &str) -> Option<(i32, u32, u32)> {
    let mut it = date.split('-');
    let (y, m, d) = (it.next()?, it.next()?, it.next()?);
    if it.next().is_some() || y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let (y, m, d): (i32, u32, u32) = (y.parse().ok()?, m.parse().ok()?, d.parse().ok()?);
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => return None,
    };
    (1..=days).contains(&d).then_some((y, m, d))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTES: &str = "# Anibeam for Android

<!--
How to write release notes.
-->

## 2.0.8 · 2026-10-24

Smoother seek bar dragging that waits until you let go before jumping.

## 2.0.7 · Seek previews · 2026-10-24

We're happy to bring frame previews to the player seek bar.

- **Seek thumbnails.** Drag along the seek bar to preview the frame under your finger.
- **Smoother scrubbing.** Run `anibeam` & watch the video follow your drag.

### Fixes & polish

Fixed duplicate \"in\" in episode countdowns.

## 2.0.4 · 2026-10-24

- Delete episodes right from a series page.
- Removed the preloaded sources.
";

    fn lit(w: &WhatsNew) -> Vec<bool> {
        w.versions.iter().map(|v| v.lit).collect()
    }

    #[test]
    fn a_file_reads_into_versions_paragraphs_headings_and_items() {
        let notes = parse(NOTES);
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].version, "2.0.8");
        assert_eq!(notes[0].title, "");
        assert_eq!(notes[0].date, "2026-10-24");
        assert_eq!(
            notes[0].blocks,
            vec![Block::Paragraph {
                text: "Smoother seek bar dragging that waits until you let go before jumping."
                    .into(),
                rich: "Smoother seek bar dragging that waits until you let go before jumping."
                    .into(),
            }]
        );

        assert_eq!(notes[1].version, "2.0.7");
        assert_eq!(notes[1].title, "Seek previews");
        assert_eq!(notes[1].blocks.len(), 4);
        assert_eq!(
            notes[1].blocks[0],
            Block::Paragraph {
                text: "We're happy to bring frame previews to the player seek bar.".into(),
                rich: "We're happy to bring frame previews to the player seek bar.".into(),
            }
        );
        assert_eq!(
            notes[1].blocks[1],
            Block::List {
                items: vec![
                    Item {
                        lead: "Seek thumbnails".into(),
                        body: "Drag along the seek bar to preview the frame under your finger."
                            .into(),
                        text: "Seek thumbnails. Drag along the seek bar to preview the frame under your finger."
                            .into(),
                        rich: "<b>Seek thumbnails.</b> Drag along the seek bar to preview the frame under your finger."
                            .into(),
                    },
                    Item {
                        lead: "Smoother scrubbing".into(),
                        body: "Run anibeam & watch the video follow your drag.".into(),
                        text: "Smoother scrubbing. Run anibeam & watch the video follow your drag."
                            .into(),
                        rich: "<b>Smoother scrubbing.</b> Run <code>anibeam</code> &amp; watch the video follow your drag."
                            .into(),
                    },
                ],
            }
        );
        assert_eq!(
            notes[1].blocks[2],
            Block::Heading {
                text: "Fixes & polish".into()
            }
        );
        assert_eq!(
            notes[1].blocks[3],
            Block::Paragraph {
                text: "Fixed duplicate \"in\" in episode countdowns.".into(),
                rich: "Fixed duplicate \"in\" in episode countdowns.".into(),
            }
        );

        assert_eq!(notes[2].blocks.len(), 1);
        match &notes[2].blocks[0] {
            Block::List { items } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].lead, "");
                assert_eq!(items[0].body, "Delete episodes right from a series page.");
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn wrapped_paragraphs_and_bullets_join_cleanly() {
        let md = "## 2.5.0 · 2026-11-21\n\nFirst line of a paragraph\nthat wraps cleanly.\n\n- **Heads up:** First line of a bullet\n  continued on the next line with `a < b`.\n";
        let notes = parse(md);
        assert_eq!(lint(md, "2.5.0"), Vec::<String>::new());
        assert_eq!(
            notes[0].blocks[0],
            Block::Paragraph {
                text: "First line of a paragraph that wraps cleanly.".into(),
                rich: "First line of a paragraph that wraps cleanly.".into(),
            }
        );
        match &notes[0].blocks[1] {
            Block::List { items } => {
                assert_eq!(items[0].lead, "Heads up");
                assert_eq!(
                    items[0].body,
                    "First line of a bullet continued on the next line with a < b."
                );
                assert_eq!(
                    items[0].rich,
                    "<b>Heads up:</b> First line of a bullet continued on the next line with <code>a &lt; b</code>."
                );
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn code_spans_keep_asterisks_literal() {
        let (plain, rich) = inline("Use `a**b` and **real bold** here.");
        assert_eq!(plain, "Use a**b and real bold here.");
        assert_eq!(rich, "Use <code>a**b</code> and <b>real bold</b> here.");
    }

    #[test]
    fn a_heading_may_name_its_version() {
        let named = "## 2.5.0 · Watch together · 2026-11-21\n\nWatch with friends.\n";
        let notes = parse(named);
        assert_eq!(notes[0].version, "2.5.0");
        assert_eq!(notes[0].title, "Watch together");
        assert_eq!(notes[0].date, "2026-11-21");
        assert_eq!(lint(named, "2.5.0"), Vec::<String>::new());
        let w = whats_new(&notes, "2.5.0", &Seen::default(), false);
        assert_eq!(w.versions[0].title, "Watch together");

        let long = "## 2.5.0 · A title that goes on far longer than any name should · 2026-11-21\n\nNote.\n";
        assert!(lint(long, "2.5.0")[0].contains("40 at most"));
        let sloppy = "## 2.5.0 ·Watch together· 2026-11-21\n\nNote.\n";
        assert!(lint(sloppy, "2.5.0")[0].contains("write a heading as"));
    }

    #[test]
    fn the_house_file_has_no_problems() {
        assert_eq!(lint(NOTES, "2.0.8"), Vec::<String>::new());
    }

    #[test]
    fn lint_catches_structural_slips_and_bloat() {
        let mut many = String::from(
            "## 2.0.9 (2026-10-25)\n### Dangling\n## 2.0.9 · 2026-10-25\n- \n## 2.0.8 · 2026-02-30\n",
        );
        for i in 1..=16 {
            many.push_str(&format!("- Item {i}.\n"));
        }
        let problems = lint(&many, "2.1.0");
        let has = |s: &str| problems.iter().any(|p| p.contains(s));
        assert!(has("line 1: write a heading as"), "{problems:?}");
        assert!(
            has("the newest heading is 2.0.9, but the app is 2.1.0"),
            "{problems:?}"
        );
        assert!(
            has("line 2: a subheading with nothing under it"),
            "{problems:?}"
        );
        assert!(has("line 3: 2.0.9 is not older than 2.0.9"), "{problems:?}");
        assert!(has("line 4: an empty list item"), "{problems:?}");
        assert!(has("line 3: a version with no notes"), "{problems:?}");
        assert!(has("line 5: \"2026-02-30\" is not a day"), "{problems:?}");
        assert!(has("16 list items, 15 at most"), "{problems:?}");
    }

    #[test]
    fn only_the_newest_version_may_be_empty() {
        let just_begun = "## 2.1.0 · 2026-10-30\n\n## 2.0.8 · 2026-10-24\n\nOne fix.\n";
        assert_eq!(lint(just_begun, "2.1.0"), Vec::<String>::new());
        assert!(!has_lines(&parse(just_begun), "2.1.0"));
        assert!(has_lines(&parse(just_begun), "2.0.8"));
        let hollow = "## 2.1.0 · 2026-10-30\n\nReady.\n\n## 2.0.8 · 2026-10-24\n";
        assert_eq!(lint(hollow, "2.1.0"), ["line 5: a version with no notes"]);
    }

    #[test]
    fn lint_refuses_long_dashes_and_an_empty_file() {
        assert!(
            lint(
                "## 2.0.8 · 2026-10-24\n\nFast \u{2014} and done.\n",
                "2.0.8"
            )[0]
            .contains("long dash")
        );
        assert_eq!(lint("# Title only\n", "2.0.8"), ["no version heading"]);
    }

    #[test]
    fn versions_compare_by_their_numbers() {
        assert!(newer("2.0.10", "2.0.9"));
        assert!(newer("2.1.0", "2.0.99"));
        assert!(!newer("2.0.8", "2.0.8"));
        assert!(newer("2.0.8", ""));
        assert!(!newer("", "2.0.8"));
        assert!(!newer("2.0", "1.0.0"));
    }

    #[test]
    fn a_logged_version_reads_as_the_app_version_it_starts_with() {
        assert_eq!(version_of("2.1.0").as_deref(), Some("2.1.0"));
        assert_eq!(version_of("2.1.0.r6.gf1d3824").as_deref(), Some("2.1.0"));
        assert_eq!(version_of("2.1.0.dirty").as_deref(), Some("2.1.0"));
        assert_eq!(
            version_of("2.1.0-nightly.20261030.12").as_deref(),
            Some("2.1.0")
        );
        assert_eq!(version_of("2.1.0-pr.280.3").as_deref(), Some("2.1.0"));
        assert_eq!(version_of("android.v2.0.4"), None);
        assert_eq!(version_of("2.1"), None);
    }

    #[test]
    fn a_fresh_install_waits_for_its_first_update() {
        let seen = Seen::default();
        assert!(!seen.due("2.0.8", false));
        let kept = seen.started("2.0.8", false).unwrap();
        assert_eq!(
            kept,
            Seen {
                version: "2.0.8".into(),
                from: String::new()
            }
        );
        assert!(!kept.due("2.0.8", false));
        assert!(kept.due("2.1.0", false));
        assert_eq!(kept.started("2.1.0", true), None);
    }

    #[test]
    fn an_update_opens_once_and_draws_the_line_at_the_version_it_came_from() {
        let notes = parse(NOTES);
        let seen = Seen {
            version: "2.0.4".into(),
            from: "2.0.2".into(),
        };
        assert!(seen.due("2.0.8", true));
        let w = whats_new(&notes, "2.0.8", &seen, true);
        assert_eq!(w.note, "2.0.4 → 2.0.8");
        assert_eq!(w.line, "You were on 2.0.4");
        assert_eq!(w.above, 2);
        assert_eq!(lit(&w), [true, true, false]);
        assert_eq!(w.versions[0].day, "24 Oct 2026");

        let kept = seen.closed("2.0.8");
        assert_eq!(
            kept,
            Seen {
                version: "2.0.8".into(),
                from: "2.0.4".into()
            }
        );
        assert!(!kept.due("2.0.8", true));
        assert_eq!(kept.closed("2.0.8"), kept);
        let again = whats_new(&notes, "2.0.8", &kept, true);
        assert_eq!(again.line, "You were on 2.0.4");
        assert_eq!(lit(&again), [true, true, false]);
    }

    #[test]
    fn a_version_from_before_the_notes_marks_the_newest_and_dims_nothing() {
        let notes = parse(NOTES);
        let w = whats_new(&notes, "2.0.8", &Seen::default(), true);
        assert_eq!(w.note, "Updated to 2.0.8");
        assert_eq!(w.line, "Before this update");
        assert_eq!(w.above, 1);
        assert_eq!(lit(&w), [true, true, true]);
    }

    #[test]
    fn a_fresh_install_opened_by_hand_shows_no_line() {
        let notes = parse(NOTES);
        let kept = Seen::default().started("2.0.8", false).unwrap();
        let w = whats_new(&notes, "2.0.8", &kept, false);
        assert_eq!(w.note, "Version 2.0.8");
        assert_eq!((w.line.as_str(), w.above), ("", 0));
        assert_eq!(lit(&w), [true, true, true]);
    }

    #[test]
    fn notes_newer_than_the_app_stay_out_and_an_unlisted_version_still_places_the_line() {
        let notes = parse(NOTES);
        let seen = Seen {
            version: "2.0.6".into(),
            from: String::new(),
        };
        let w = whats_new(&notes, "2.0.7", &seen, true);
        assert_eq!(w.versions[0].version, "2.0.7");
        assert_eq!(w.above, 1);
        assert_eq!(w.line, "You were on 2.0.6");
    }

    #[test]
    fn a_downgrade_opens_nothing_keeps_what_was_read_and_draws_no_line() {
        let notes = parse(NOTES);
        let seen = Seen {
            version: "2.0.8".into(),
            from: "2.0.4".into(),
        };
        assert!(!seen.due("2.0.7", true));
        assert_eq!(seen.closed("2.0.7"), seen);
        let w = whats_new(&notes, "2.0.7", &seen, true);
        assert_eq!(w.note, "Version 2.0.7");
        assert_eq!((w.line.as_str(), w.above), ("", 0));
        assert_eq!(lit(&w), [true, true]);
    }
}
