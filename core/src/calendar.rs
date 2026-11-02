//! The calendar feed: with the switch on, the core keeps an iCal file of the
//! airing times of the shows on the user's lists at `<data_dir>/anibeam.ics`,
//! so a calendar app can subscribe to it.
//!
//! Nothing here is on a timer. The file is written when the switch goes on,
//! and again whenever a job that can move the schedule or a list status
//! finishes; turning the switch off deletes it. `render` is the pure part:
//! the entries in, the file's text out.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use crate::contract::*;
use crate::core::Core;
use crate::events::Subscription;
use crate::library::snapshot::Snapshot;
use crate::library::titles;
use crate::prefs;
use crate::sync::recover;
use crate::time;

/// How far back an episode still goes in the file: one that aired in the
/// last few days stays on the calendar rather than vanishing the moment it
/// starts.
const KEEP_PAST_SECS: i64 = 3 * 86_400;

/// The length of an episode whose series names no runtime.
const DEFAULT_MINUTES: u32 = 24;

/// Lines longer than this many octets are folded, as RFC 5545 asks.
const FOLD_OCTETS: usize = 75;

/// The calendar's state on the core: the listener that rewrites the file
/// when a job finishes, and the lock that keeps two writes (or a write and a
/// delete) from crossing on the same file.
#[derive(Default)]
pub(crate) struct Calendar {
    listener: Mutex<Option<Arc<Subscription>>>,
    file: Mutex<()>,
}

/// One episode on the calendar.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub series: u64,
    pub title: String,
    pub number: u32,
    /// Unix seconds, UTC.
    pub at: i64,
    pub minutes: u32,
}

/// Whether a show with this list status belongs on the calendar: the lists
/// a person is following or means to, not the ones they finished or left.
fn wanted(status: Option<ListStatus>) -> bool {
    matches!(
        status,
        Some(
            ListStatus::Watching
                | ListStatus::Planning
                | ListStatus::Paused
                | ListStatus::Repeating
        )
    )
}

/// The episodes the file holds: every listed, unhidden show's airing rows
/// from `KEEP_PAST_SECS` ago onward, sorted by start time.
pub fn entries(snap: &Snapshot, now: i64) -> Vec<Entry> {
    let from = now - KEEP_PAST_SECS;
    let mut out = Vec::new();
    for row in &snap.series {
        if row.hidden || !wanted(snap.tracker(row).and_then(|t| t.status)) {
            continue;
        }
        let media = row.media.as_ref();
        let title = titles::resolve(
            snap.lang,
            media.and_then(|m| m.title_romaji.as_deref()),
            media.and_then(|m| m.title_english.as_deref()),
            media.and_then(|m| m.title_native.as_deref()),
            &row.folder_name,
        );
        let minutes = media
            .and_then(|m| m.duration)
            .filter(|&d| d > 0)
            .unwrap_or(DEFAULT_MINUTES);
        for ep in snap.airing_of(row) {
            let Some(at) = ep.aired_at.filter(|&at| at >= from) else {
                continue;
            };
            out.push(Entry {
                series: row.id,
                title: title.clone(),
                number: ep.number,
                at,
                minutes,
            });
        }
    }
    out.sort_by_key(|e| (e.at, e.series, e.number));
    out
}

/// The whole file: a VCALENDAR with one VEVENT per entry, CRLF line ends,
/// long lines folded. `now` stamps every event.
pub fn render(entries: &[Entry], now: i64) -> String {
    let stamp = utc(now);
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//AniBeam//EN".to_string(),
        "CALSCALE:GREGORIAN".to_string(),
        "X-WR-CALNAME:AniBeam".to_string(),
    ];
    for e in entries {
        lines.push("BEGIN:VEVENT".to_string());
        lines.push(format!("UID:anibeam-{}-{}@anibeam", e.series, e.number));
        lines.push(format!("DTSTAMP:{stamp}"));
        lines.push(format!("DTSTART:{}", utc(e.at)));
        lines.push(format!("DURATION:PT{}M", e.minutes));
        lines.push(format!(
            "SUMMARY:{}",
            escape(&format!("{} - Episode {}", e.title, e.number))
        ));
        lines.push("END:VEVENT".to_string());
    }
    lines.push("END:VCALENDAR".to_string());
    let mut out = String::new();
    for line in &lines {
        out.push_str(&fold(line));
        out.push_str("\r\n");
    }
    out
}

/// A time as iCal's UTC form, `YYYYMMDDTHHMMSSZ`.
fn utc(secs: i64) -> String {
    let t = ::time::OffsetDateTime::from_unix_timestamp(secs)
        .unwrap_or(::time::OffsetDateTime::UNIX_EPOCH);
    let mut s = String::with_capacity(16);
    let _ = write!(
        s,
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        t.year(),
        u8::from(t.month()),
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    );
    s
}

/// A TEXT value with its specials escaped: backslash, semicolon, comma and
/// line breaks.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

/// One content line folded at `FOLD_OCTETS` octets, never inside a
/// character: each continuation starts with a space, which counts toward
/// its own line's length. Joined with CRLF, without the final one.
fn fold(line: &str) -> String {
    if line.len() <= FOLD_OCTETS {
        return line.to_string();
    }
    let mut out = String::with_capacity(line.len() + line.len() / FOLD_OCTETS * 3);
    let mut used = 0;
    for c in line.chars() {
        if used + c.len_utf8() > FOLD_OCTETS {
            out.push_str("\r\n ");
            used = 1;
        }
        out.push(c);
        used += c.len_utf8();
    }
    out
}

fn enabled(core: &Core) -> bool {
    core.store
        .read(prefs::load_preferences)
        .map(|p| p.calendar_feed)
        .unwrap_or(false)
}

fn tmp_sibling(target: &Path) -> PathBuf {
    let mut name = target
        .file_name()
        .map(std::ffi::OsString::from)
        .unwrap_or_else(|| std::ffi::OsString::from("anibeam.ics"));
    name.push(".tmp");
    target.with_file_name(name)
}

fn write_file(core: &Core) -> Result<(), CoreError> {
    let images_dir = core.paths.images_dir();
    let now = time::now_secs();
    let text = core.store.read(|c| {
        let snap = Snapshot::load(c, &images_dir, time::now(), None)?;
        Ok(render(&entries(&snap, now), now))
    })?;
    let target = core.paths.calendar_path();
    if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| CoreError::io_at(parent.to_string_lossy(), e))?;
    }
    let tmp = tmp_sibling(&target);
    std::fs::write(&tmp, text).map_err(|e| CoreError::io_at(tmp.to_string_lossy(), e))?;
    std::fs::rename(&tmp, &target).map_err(|e| CoreError::io_at(target.to_string_lossy(), e))?;
    Ok(())
}

/// Rewrites the file if the switch is on, on the calling thread. A failure
/// is one warning, never an error to the caller: the calendar is a side
/// effect of whatever changed the schedule.
pub(crate) fn write(core: &Core) {
    let _file = recover(&core.calendar.file);
    if !enabled(core) {
        return;
    }
    match write_file(core) {
        Ok(()) => {
            core.bus
                .debug(Stage::System, "calendar written", EventBody::Notice);
        }
        Err(e) => {
            core.bus.warn(
                Stage::System,
                format!("the calendar file could not be written: {e}"),
                EventBody::Notice,
            );
        }
    }
}

/// Deletes the file, as turning the switch off does. A file that is not
/// there is already the wanted state.
pub(crate) fn remove(core: &Core) {
    let _file = recover(&core.calendar.file);
    let path = core.paths.calendar_path();
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            core.bus.warn(
                Stage::System,
                format!("the calendar file could not be removed: {e}"),
                EventBody::Notice,
            );
        }
    }
}

/// Follows a change of the switch: on writes the file now, off deletes it.
pub(crate) fn set_enabled(core: &Core, on: bool) {
    if on { write(core) } else { remove(core) }
}

/// The job kinds whose end can move an airing time or a list status, and so
/// what the file shows.
fn changes_schedule(kind: JobKind) -> bool {
    use JobKind::*;
    matches!(
        kind,
        RefreshAiring
            | Refresh
            | RefreshAll
            | AutoMatch
            | ApplyMatch
            | ConnectTracker
            | Mark
            | SetProgress
            | RefreshProgress
            | RefreshWatching
            | OnHold
            | SetStatus
            | Import
    )
}

/// Subscribes the listener that rewrites the file when such a job ends.
/// Called once, by `Core::open`; `shutdown` drops it.
pub(crate) fn listen(core: &Arc<Core>) {
    let listener = Arc::new(AfterJob {
        core: Arc::downgrade(core),
    });
    *recover(&core.calendar.listener) = Some(core.subscribe(listener));
}

/// Unsubscribes the listener. Taken out of the mutex before it is dropped,
/// since the drop takes the bus's own lock.
pub(crate) fn stop(core: &Core) {
    let sub = recover(&core.calendar.listener).take();
    drop(sub);
}

/// Holds a `Weak`, so the bus never keeps the core alive through it.
struct AfterJob {
    core: Weak<Core>,
}

impl EventListener for AfterJob {
    fn on_event(&self, event: Event) {
        let Some(job) = event.job.as_ref() else {
            return;
        };
        if job.phase != JobPhase::Finished || !changes_schedule(job.kind) {
            return;
        }
        let Some(core) = self.core.upgrade() else {
            return;
        };
        // Off the emitting thread: the bus calls listeners inline, and a
        // job's runner should not wait on a file write.
        let handle = core.handle.clone();
        handle.spawn_blocking(move || write(&core));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str, number: u32, at: i64) -> Entry {
        Entry {
            series: 7,
            title: title.to_string(),
            number,
            at,
            minutes: 24,
        }
    }

    #[test]
    fn utc_is_the_basic_form_with_a_z() {
        // 2026-11-02 13:05:09 UTC.
        assert_eq!(utc(1_793_624_709), "20261102T130509Z");
        assert_eq!(utc(0), "19700101T000000Z");
    }

    #[test]
    fn text_specials_are_escaped() {
        assert_eq!(escape(r"a\b;c,d"), r"a\\b\;c\,d");
        assert_eq!(escape("one\r\ntwo"), r"one\ntwo");
    }

    #[test]
    fn long_lines_fold_at_75_octets_on_char_boundaries() {
        let line = format!("SUMMARY:{}", "葬送のフリーレン".repeat(6));
        let folded = fold(&line);
        let parts: Vec<&str> = folded.split("\r\n").collect();
        assert!(parts.len() > 1);
        for (i, p) in parts.iter().enumerate() {
            assert!(p.len() <= FOLD_OCTETS, "line {i} is {} octets", p.len());
            if i > 0 {
                assert!(p.starts_with(' '));
            }
        }
        let unfolded: String = parts
            .iter()
            .enumerate()
            .map(|(i, p)| if i == 0 { *p } else { &p[1..] })
            .collect();
        assert_eq!(unfolded, line);
        assert_eq!(fold("SHORT:line"), "SHORT:line");
    }

    #[test]
    fn render_writes_one_event_per_entry_with_crlf() {
        let now = 1_793_624_709;
        let text = render(&[entry("Frieren, Part 2", 3, now + 3600)], now);
        assert!(text.starts_with("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n"));
        assert!(text.ends_with("END:VCALENDAR\r\n"));
        assert!(!text.replace("\r\n", "").contains('\n'));
        assert!(text.contains("UID:anibeam-7-3@anibeam\r\n"));
        assert!(text.contains("DTSTAMP:20261102T130509Z\r\n"));
        assert!(text.contains("DTSTART:20261102T140509Z\r\n"));
        assert!(text.contains("DURATION:PT24M\r\n"));
        assert!(text.contains("SUMMARY:Frieren\\, Part 2 - Episode 3\r\n"));
        assert!(text.contains("X-WR-CALNAME:AniBeam\r\n"));
    }
}
