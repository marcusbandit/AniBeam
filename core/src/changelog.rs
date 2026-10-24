//! Patch notes: an app's CHANGELOG.md read into its versions, newest first, and where What's
//! new draws the line between what a person has read and what is new to them. Each shell
//! keeps a [`Seen`] between runs and draws the [`View`] this hands back; the rules for both
//! live here, so the desktop and the phone open, mark and close the notes the same way.
//!
//! The file, which [`lint`] holds every app's copy to:
//!
//! ```text
//! ## 2.0.8 · 2026-10-24
//!
//! ### Fixed
//! - The seek bar previews in the bubble while you drag
//! ```
//!
//! A heading per version, newest first, with the day it was made; under it one or more of
//! the sections New, Changed, Fixed and Removed; under each, one line per change. The `# `
//! title and `<!-- -->` comments are skipped.

use serde::{Deserialize, Serialize};

/// The section names a version may use, in the order they read best.
pub const KINDS: [&str; 4] = ["New", "Changed", "Fixed", "Removed"];

/// The most characters a line may have: two lines on a phone.
pub const MAX_LINE: usize = 80;

/// One version's notes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Release {
    pub version: String,
    /// `2026-10-24`, as the heading has it; empty when it has none.
    pub date: String,
    pub sections: Vec<Section>,
}

/// One section of a version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Section {
    /// One of [`KINDS`], as the file spells it.
    pub kind: String,
    pub lines: Vec<String>,
}

/// The versions in `text`, in the file's order. Forgiving: a line it cannot place is
/// skipped, so a slip in the file costs that line, never the sheet. [`lint`] is the strict
/// reading.
pub fn parse(text: &str) -> Vec<Release> {
    let mut out: Vec<Release> = Vec::new();
    for (_, line) in read(text) {
        match line {
            Line::Heading { version, date, .. } => out.push(Release {
                version,
                date,
                sections: Vec::new(),
            }),
            Line::Section(kind) => {
                if let Some(r) = out.last_mut() {
                    r.sections.push(Section {
                        kind,
                        lines: Vec::new(),
                    });
                }
            }
            Line::Item(text) => {
                if let Some(s) = out.last_mut().and_then(|r| r.sections.last_mut()) {
                    s.lines.push(text);
                }
            }
            Line::Title | Line::Other(_) => {}
        }
    }
    out
}

/// Each way `text` breaks the file's shape, one sentence naming the line; empty when it is
/// fine. `version` is the app's own, which the newest heading has to be.
pub fn lint(text: &str, version: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        if raw.contains(['\u{2014}', '\u{2013}']) {
            problems.push(format!(
                "line {}: a long dash; use a comma or end the sentence",
                n + 1
            ));
        }
    }
    // The heading above, its sections so far, and the open section's line count.
    let mut above: Option<String> = None;
    let mut kinds: Vec<String> = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let mut first = true;
    let close = |problems: &mut Vec<String>, open: Option<(usize, usize)>| {
        if let Some((at, 0)) = open {
            problems.push(format!("line {at}: a section with no lines under it"));
        }
    };
    let mut heading_at = 0;
    for (n, line) in read(text) {
        match line {
            Line::Title => {}
            Line::Heading {
                version: v,
                date,
                exact,
            } => {
                close(&mut problems, open.take());
                if above.is_some() && kinds.is_empty() {
                    problems.push(format!("line {heading_at}: a version with no sections"));
                }
                heading_at = n;
                kinds.clear();
                if !exact {
                    problems.push(format!(
                        "line {n}: write a heading as \"## 2.0.8 · 2026-10-24\""
                    ));
                }
                if parts(&v).is_none() {
                    problems.push(format!("line {n}: \"{v}\" is not a version like 2.0.8"));
                }
                if !date.is_empty() && day_parts(&date).is_none() {
                    problems.push(format!("line {n}: \"{date}\" is not a day like 2026-10-24"));
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
            Line::Section(kind) => {
                close(&mut problems, open.take());
                if above.is_none() {
                    problems.push(format!("line {n}: a section before any version"));
                }
                if !KINDS.contains(&kind.as_str()) {
                    problems.push(format!(
                        "line {n}: \"### {kind}\" is not one of {}",
                        KINDS.join(", ")
                    ));
                }
                if kinds.contains(&kind) {
                    problems.push(format!("line {n}: a second {kind} section in one version"));
                }
                kinds.push(kind);
                open = Some((n, 0));
            }
            Line::Item(item) => {
                match open.as_mut() {
                    Some((_, count)) => *count += 1,
                    None => problems.push(format!("line {n}: a line before any section")),
                }
                let len = item.chars().count();
                if len > MAX_LINE {
                    problems.push(format!(
                        "line {n}: {len} characters, {MAX_LINE} at most: \"{item}\""
                    ));
                }
                if item.ends_with('.') {
                    problems.push(format!("line {n}: no full stop at the end of a line"));
                }
            }
            // Words above the first version are the file's own; under one, nothing but
            // sections and their lines.
            Line::Other(text) => {
                if above.is_some() {
                    problems.push(format!(
                        "line {n}: \"{text}\" is not a heading, a section or a \"- \" line"
                    ));
                }
            }
        }
    }
    close(&mut problems, open);
    match above {
        None => problems.push("no version heading".to_string()),
        Some(_) if kinds.is_empty() => {
            problems.push(format!("line {heading_at}: a version with no sections"))
        }
        Some(_) => {}
    }
    problems
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
pub struct View {
    /// Under the title: "2.0.4 → 2.0.8", "Updated to 2.0.8", or "Version 2.0.8".
    pub note: String,
    /// The words on the line under what is new, "You were on 2.0.4"; empty for no line.
    pub line: String,
    /// The versions, newest first, none newer than the app.
    pub releases: Vec<Shown>,
}

/// One version as What's new lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Shown {
    pub version: String,
    /// "24 Oct 2026", or empty.
    pub day: String,
    /// Above the line: new since the version the person came from.
    pub fresh: bool,
    pub sections: Vec<Section>,
}

/// What's new for `current`, read with `seen` and whether this install ran before the
/// notes existed. While the notes are due the line sits at the version last closed; once
/// they are closed it stays at the version the last update came from.
pub fn view(notes: &[Release], current: &str, seen: &Seen, used_before: bool) -> View {
    let due = seen.due(current, used_before);
    let from = if due {
        seen.version.as_str()
    } else {
        seen.from.as_str()
    };
    let releases: Vec<&Release> = notes
        .iter()
        .filter(|r| !newer(&r.version, current))
        .collect();
    let fresh = if !from.is_empty() {
        releases
            .iter()
            .take_while(|r| newer(&r.version, from))
            .count()
    } else if due {
        // Updated from a version that kept no record: only this one's notes are new.
        usize::from(releases.first().is_some_and(|r| r.version == current))
    } else {
        0
    };
    let (note, line) = match (fresh, from) {
        (0, _) => (format!("Version {current}"), String::new()),
        (_, "") => (
            format!("Updated to {current}"),
            "Before this update".to_string(),
        ),
        (_, from) => (format!("{from} → {current}"), format!("You were on {from}")),
    };
    View {
        note,
        line,
        releases: releases
            .into_iter()
            .enumerate()
            .map(|(i, r)| Shown {
                version: r.version.clone(),
                day: day(&r.date),
                fresh: i < fresh,
                sections: r.sections.clone(),
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
    /// `# AniBeam for Android`
    Title,
    /// `## 2.0.8 · 2026-10-24`, read leniently; `exact` when it is written just so.
    Heading {
        version: String,
        date: String,
        exact: bool,
    },
    /// `### Fixed`
    Section(String),
    /// `- The seek bar previews in the bubble`
    Item(String),
    Other(String),
}

/// The file's lines with their numbers, counted from 1, less the blank ones and comments.
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
            continue;
        }
        let read = if let Some(rest) = line.strip_prefix("### ") {
            Line::Section(rest.trim().to_string())
        } else if let Some(rest) = line.strip_prefix("## ") {
            heading(rest.trim())
        } else if line.starts_with("# ") {
            Line::Title
        } else if let Some(rest) = line.strip_prefix("- ") {
            Line::Item(rest.trim().to_string())
        } else {
            Line::Other(line.to_string())
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
    let exact = !date.is_empty() && text == format!("{version} · {date}");
    Line::Heading {
        version,
        date,
        exact,
    }
}

fn parts(v: &str) -> Option<[u64; 3]> {
    let mut numbers = v.split('.');
    let mut out = [0; 3];
    for slot in &mut out {
        let n = numbers.next()?;
        if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = n.parse().ok()?;
    }
    numbers.next().is_none().then_some(out)
}

fn day_parts(date: &str) -> Option<(i32, u32, u32)> {
    let mut it = date.split('-');
    let (y, m, d) = (it.next()?, it.next()?, it.next()?);
    if it.next().is_some() || y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let (y, m, d) = (y.parse().ok()?, m.parse().ok()?, d.parse().ok()?);
    ((1..=12).contains(&m) && (1..=31).contains(&d)).then_some((y, m, d))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTES: &str = "# AniBeam for Android

<!--
How to write a line.
-->

## 2.0.8 · 2026-10-24

### Fixed
- Letting go near the start does not seek

## 2.0.7 · 2026-10-24

### New
- A thumbnail over the seek bar while you drag

### Fixed
- Countdowns read in 3d, not in in 3d

## 2.0.4 · 2026-10-24

### Removed
- The preloaded sources
";

    #[test]
    fn a_file_reads_into_its_versions_sections_and_lines() {
        let notes = parse(NOTES);
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].version, "2.0.8");
        assert_eq!(notes[0].date, "2026-10-24");
        assert_eq!(notes[1].sections.len(), 2);
        assert_eq!(notes[1].sections[0].kind, "New");
        assert_eq!(
            notes[1].sections[1].lines,
            ["Countdowns read in 3d, not in in 3d"]
        );
        assert_eq!(notes[2].sections[0].kind, "Removed");
    }

    #[test]
    fn the_house_file_has_no_problems() {
        assert_eq!(lint(NOTES, "2.0.8"), Vec::<String>::new());
    }

    #[test]
    fn lint_names_each_slip() {
        let bad = "## 2.0.9 (2026-10-25)
### Fixes
- This line runs on and on well past the eighty characters that two lines on a phone hold.
- Ends with a full stop.
wraps onto a second line
## 2.0.9 · 2026-10-25
### New
";
        let problems = lint(bad, "2.1.0");
        let has = |s: &str| problems.iter().any(|p| p.contains(s));
        assert!(has("line 1: write a heading as"), "{problems:?}");
        assert!(
            has("the newest heading is 2.0.9, but the app is 2.1.0"),
            "{problems:?}"
        );
        assert!(
            has("\"### Fixes\" is not one of New, Changed, Fixed, Removed"),
            "{problems:?}"
        );
        assert!(has("line 3: 88 characters, 80 at most"), "{problems:?}");
        assert!(has("line 4: no full stop"), "{problems:?}");
        assert!(
            has("line 5: \"wraps onto a second line\" is not a heading"),
            "{problems:?}"
        );
        assert!(has("line 6: 2.0.9 is not older than 2.0.9"), "{problems:?}");
        assert!(has("line 7: a section with no lines"), "{problems:?}");
    }

    #[test]
    fn lint_refuses_long_dashes_and_an_empty_file() {
        assert!(
            lint(
                "## 2.0.8 · 2026-10-24\n### Fixed\n- Fast \u{2014} and done\n",
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
        let v = view(&notes, "2.0.8", &seen, true);
        assert_eq!(v.note, "2.0.4 → 2.0.8");
        assert_eq!(v.line, "You were on 2.0.4");
        assert_eq!(
            v.releases.iter().map(|r| r.fresh).collect::<Vec<_>>(),
            [true, true, false]
        );
        assert_eq!(v.releases[0].day, "24 Oct 2026");

        // Closed: it does not open again, and opened by hand it still shows the update.
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
        let again = view(&notes, "2.0.8", &kept, true);
        assert_eq!(again.line, "You were on 2.0.4");
        assert_eq!(again.releases.iter().filter(|r| r.fresh).count(), 2);
    }

    #[test]
    fn a_version_from_before_the_notes_marks_only_the_newest_as_new() {
        let notes = parse(NOTES);
        let v = view(&notes, "2.0.8", &Seen::default(), true);
        assert_eq!(v.note, "Updated to 2.0.8");
        assert_eq!(v.line, "Before this update");
        assert_eq!(v.releases.iter().filter(|r| r.fresh).count(), 1);
    }

    #[test]
    fn a_fresh_install_opened_by_hand_shows_no_line() {
        let notes = parse(NOTES);
        let kept = Seen::default().started("2.0.8", false).unwrap();
        let v = view(&notes, "2.0.8", &kept, false);
        assert_eq!(v.note, "Version 2.0.8");
        assert_eq!(v.line, "");
        assert!(v.releases.iter().all(|r| !r.fresh));
    }

    #[test]
    fn notes_newer_than_the_app_stay_out_and_an_unlisted_version_still_places_the_line() {
        let notes = parse(NOTES);
        let v = view(
            &notes,
            "2.0.7",
            &Seen {
                version: "2.0.6".into(),
                from: String::new(),
            },
            true,
        );
        assert_eq!(v.releases[0].version, "2.0.7");
        assert_eq!(v.releases.iter().filter(|r| r.fresh).count(), 1);
        assert_eq!(v.line, "You were on 2.0.6");
    }

    #[test]
    fn a_downgrade_opens_nothing_and_keeps_what_was_read() {
        let seen = Seen {
            version: "2.0.8".into(),
            from: "2.0.4".into(),
        };
        assert!(!seen.due("2.0.7", true));
        assert_eq!(seen.closed("2.0.7"), seen);
    }
}
