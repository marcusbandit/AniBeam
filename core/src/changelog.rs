//! Patch notes: an app's CHANGELOG.md read into its versions, newest first, and where What's
//! new draws the line between what a person has read and what is new to them. Each shell
//! keeps a [`Seen`] between runs and draws the [`WhatsNew`] this hands back; every rule for
//! both lives here, so the desktop and the phone open, mark, dim and close the notes the same
//! way.
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
//! A heading per version, newest first, with the day it was made and, when the version has
//! one, a title to remember it by between the two: `## 2.5.0 · Quality of life · 2026-11-02`.
//! The Release workflow names the GitHub release after it. Under each heading one or more of
//! the sections New, Changed, Fixed and Removed, in that order; under each, one line per
//! change. The `# ` title and `<!-- -->` comments are skipped.

use serde::{Deserialize, Serialize};

/// The section names a version may use, in the order a version lists them.
pub const KINDS: [&str; 4] = ["New", "Changed", "Fixed", "Removed"];

/// The most characters a line may have: two lines on a phone.
pub const MAX_LINE: usize = 80;

/// The most characters a version's title may have: one line beside its number.
pub const MAX_TITLE: usize = 40;

/// One version's notes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Notes {
    pub version: String,
    /// `Quality of life`, the version's own name; empty for a version known by its number.
    pub title: String,
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
pub fn parse(text: &str) -> Vec<Notes> {
    let mut out: Vec<Notes> = Vec::new();
    for (_, line) in read(text) {
        match line {
            Line::Heading {
                version,
                title,
                date,
                ..
            } => out.push(Notes {
                version,
                title,
                date,
                sections: Vec::new(),
            }),
            Line::Section(kind) => {
                if let Some(n) = out.last_mut() {
                    n.sections.push(Section {
                        kind,
                        lines: Vec::new(),
                    });
                }
            }
            Line::Item(text) => {
                if let Some(s) = out.last_mut().and_then(|n| n.sections.last_mut()) {
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
    // The newest version may have no sections yet: main moves to it the moment the last
    // one ships, before anything is written under it.
    let mut headings = 0;
    for (n, line) in read(text) {
        match line {
            Line::Title => {}
            Line::Heading {
                version: v,
                title,
                date,
                exact,
            } => {
                close(&mut problems, open.take());
                if headings > 1 && kinds.is_empty() {
                    problems.push(format!("line {heading_at}: a version with no sections"));
                }
                headings += 1;
                heading_at = n;
                kinds.clear();
                if !exact {
                    problems.push(format!(
                        "line {n}: write a heading as \"## 2.0.8 · 2026-10-24\", or with a title as \"## 2.0.8 · Quality of life · 2026-10-24\""
                    ));
                }
                let len = title.chars().count();
                if len > MAX_TITLE {
                    problems.push(format!(
                        "line {n}: a title of {len} characters, {MAX_TITLE} at most: \"{title}\""
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
                let rank = KINDS.iter().position(|k| *k == kind);
                match rank {
                    None => problems.push(format!(
                        "line {n}: \"### {kind}\" is not one of {}",
                        KINDS.join(", ")
                    )),
                    Some(r) => {
                        if kinds.contains(&kind) {
                            problems
                                .push(format!("line {n}: a second {kind} section in one version"));
                        } else if let Some(last) = kinds.last()
                            && KINDS.iter().position(|k| *k == *last) > Some(r)
                        {
                            problems.push(format!(
                                "line {n}: {kind} comes before {last}; the order is {}",
                                KINDS.join(", ")
                            ));
                        }
                    }
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
        Some(_) if headings > 1 && kinds.is_empty() => {
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
    pub sections: Vec<Section>,
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
                sections: n.sections.clone(),
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
    /// `## 2.0.8 · 2026-10-24` or `## 2.5.0 · Quality of life · 2026-11-02`, read
    /// leniently; `exact` when it is written just so.
    Heading {
        version: String,
        title: String,
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

/// Whether `version` has anything to show: main moves to a new version before a line is
/// written under it, and notes with nothing in them never open What's new by themselves.
pub fn has_lines(notes: &[Notes], version: &str) -> bool {
    notes
        .iter()
        .find(|n| n.version == version)
        .is_some_and(|n| n.sections.iter().any(|s| !s.lines.is_empty()))
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

    fn lit(w: &WhatsNew) -> Vec<bool> {
        w.versions.iter().map(|v| v.lit).collect()
    }

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
        assert_eq!(notes[0].title, "");
    }

    #[test]
    fn a_heading_may_name_its_version() {
        let named = "## 2.5.0 · Quality of life · 2026-11-02\n### New\n- One\n";
        let notes = parse(named);
        assert_eq!(notes[0].version, "2.5.0");
        assert_eq!(notes[0].title, "Quality of life");
        assert_eq!(notes[0].date, "2026-11-02");
        assert_eq!(lint(named, "2.5.0"), Vec::<String>::new());
        let w = whats_new(&notes, "2.5.0", &Seen::default(), false);
        assert_eq!(w.versions[0].title, "Quality of life");

        let long = "## 2.5.0 · A title that goes on far longer than any name should · 2026-11-02\n### New\n- One\n";
        assert!(lint(long, "2.5.0")[0].contains("40 at most"));
        let sloppy = "## 2.5.0 ·Quality of life· 2026-11-02\n### New\n- One\n";
        assert!(lint(sloppy, "2.5.0")[0].contains("write a heading as"));
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
## 2.0.8 · 2026-02-30
### Fixed
- One
### New
- Two
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
        assert!(has("line 8: \"2026-02-30\" is not a day"), "{problems:?}");
        assert!(has("line 11: New comes before Fixed"), "{problems:?}");
    }

    #[test]
    fn only_the_newest_version_may_be_empty() {
        let just_begun = "## 2.1.0 · 2026-10-30\n## 2.0.8 · 2026-10-24\n### Fixed\n- One\n";
        assert_eq!(lint(just_begun, "2.1.0"), Vec::<String>::new());
        let hollow = "## 2.1.0 · 2026-10-30\n### New\n- One\n## 2.0.8 · 2026-10-24\n";
        assert_eq!(
            lint(hollow, "2.1.0"),
            ["line 4: a version with no sections"]
        );
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
