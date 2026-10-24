//! What's new: the phone's CHANGELOG.md, built into the APK, read through the core's rules
//! and laid out as the rows the page draws. build.sh refuses an APK whose newest notes are
//! not its own version.

use anibeam_core::changelog::{self, Seen, View};

use crate::NoteLine;

const NOTES: &str = include_str!("../CHANGELOG.md");

/// This build's version, the one its notes are for.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What's new for this build, given what the phone has read.
pub fn view(seen: &Seen, used_before: bool) -> View {
    changelog::view(&changelog::parse(NOTES), VERSION, seen, used_before)
}

/// The page's rows, top to bottom: each version, its sections and their changes, with the
/// line after the last version that is new.
pub fn lines(view: &View) -> Vec<NoteLine> {
    let mut out = Vec::new();
    let fresh = view.releases.iter().filter(|r| r.fresh).count();
    for (i, r) in view.releases.iter().enumerate() {
        if i == fresh && fresh > 0 && !view.line.is_empty() {
            out.push(row(3, &view.line, "", true));
        }
        // Dimmer under the line; with no line every version reads alike.
        let lit = r.fresh || fresh == 0;
        out.push(row(0, &r.version, &r.day, lit));
        for s in &r.sections {
            out.push(row(1, &s.kind, "", lit));
            out.extend(s.lines.iter().map(|l| row(2, l, "", lit)));
        }
    }
    if fresh > 0 && fresh == view.releases.len() && !view.line.is_empty() {
        out.push(row(3, &view.line, "", true));
    }
    out
}

fn row(kind: i32, text: &str, note: &str, lit: bool) -> NoteLine {
    NoteLine { kind, text: text.into(), note: note.into(), lit }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_notes_open_on_this_version() {
        let v = view(&Seen::default(), true);
        assert_eq!(v.releases[0].version, VERSION);
        assert_eq!(v.note, format!("Updated to {VERSION}"));
    }

    #[test]
    fn the_line_sits_after_the_last_new_version() {
        let seen = Seen { version: "2.0.4".into(), from: String::new() };
        let rows = lines(&view(&seen, true));
        let line = rows.iter().position(|r| r.kind == 3).expect("a line");
        assert_eq!(rows[line].text, "You were on 2.0.4");
        assert!(rows[..line].iter().all(|r| r.lit));
        assert_eq!((rows[line + 1].kind, rows[line + 1].text.as_str()), (0, "2.0.4"));
        assert!(rows[line + 1..].iter().all(|r| !r.lit));
    }

    #[test]
    fn notes_read_on_a_fresh_install_have_no_line_and_none_dimmed() {
        let seen = Seen::default().started(VERSION, false).unwrap();
        assert!(lines(&view(&seen, false)).iter().all(|r| r.kind != 3 && r.lit));
    }
}
