//! The calendar feed: with the switch on, the core keeps an iCal file of the
//! airing times of the shows on the user's lists at `<data_dir>/anibeam.ics`,
//! so a calendar app can subscribe to it.
//!
//! Nothing here is on a timer. The file is written when the switch goes on,
//! and again after any change that can move the schedule, a list status or
//! a title, unless the new text matches what is already there; turning the
//! switch off deletes it. `render` is the pure part:
//! the entries in, the file's text out.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::atomic_file;
use crate::contract::*;
use crate::core::Core;
use crate::events::Subscription;
use crate::library::snapshot::{AiringRow, SeriesRow, Snapshot};
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
/// after a change, and the lock that keeps two writes (or a write and a
/// delete) from crossing on the same file.
#[derive(Default)]
pub(crate) struct Calendar {
    listener: Mutex<Option<Arc<Subscription>>>,
    file: Mutex<()>,
    /// A rewrite is waiting out `SETTLE`.
    queued: AtomicBool,
    /// The last write failed and said so; the next failure stays quiet.
    failing: AtomicBool,
}

/// One episode on the calendar.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// Stable across rewrites and machines: the show's record and the
    /// episode, so two folders matched to one show share one event.
    pub uid: String,
    pub title: String,
    /// TVmaze's season, whose numbers restart every season; None for an
    /// AniList schedule, which carries none.
    pub season: Option<u32>,
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

/// The record an event's UID is named for: the matched show where there is
/// one, the local series otherwise.
fn record_key(row: &SeriesRow) -> String {
    match (row.provider, row.tvmaze_id, row.anilist_id) {
        (Some(Provider::Tvmaze), Some(id), _) => format!("tvmaze-{id}"),
        (_, _, Some(id)) => format!("anilist-{id}"),
        _ => format!("series-{}", row.id),
    }
}

/// The episodes the file holds: every listed show that is neither hidden
/// nor gone from disk, with its airing rows from `KEEP_PAST_SECS` ago
/// onward, every TVmaze season included, sorted by start time. A show two
/// folders match lands once.
pub fn entries(snap: &Snapshot, now: i64) -> Vec<Entry> {
    let from = now - KEEP_PAST_SECS;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in &snap.series {
        if row.hidden || row.missing || !wanted(snap.tracker(row).and_then(|t| t.status)) {
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
        let key = record_key(row);
        let episodes: Vec<(Option<u32>, &AiringRow)> = if row.provider == Some(Provider::Tvmaze) {
            snap.tv_seasons_of(row)
                .flat_map(|(season, eps)| eps.iter().map(move |a| (Some(season), a)))
                .collect()
        } else {
            snap.airing_of(row).iter().map(|a| (None, a)).collect()
        };
        for (season, ep) in episodes {
            let Some(at) = ep.aired_at.filter(|&at| at >= from) else {
                continue;
            };
            let uid = match season {
                Some(s) => format!("anibeam-{key}-s{s}e{}@anibeam", ep.number),
                None => format!("anibeam-{key}-{}@anibeam", ep.number),
            };
            if !seen.insert(uid.clone()) {
                continue;
            }
            out.push(Entry {
                uid,
                title: title.clone(),
                season,
                number: ep.number,
                at,
                minutes,
            });
        }
    }
    out.sort_by(|a, b| (a.at, &a.uid).cmp(&(b.at, &b.uid)));
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
        lines.push(format!("UID:{}", e.uid));
        lines.push(format!("DTSTAMP:{stamp}"));
        lines.push(format!("DTSTART:{}", utc(e.at)));
        lines.push(format!("DURATION:PT{}M", e.minutes));
        lines.push(format!("SUMMARY:{}", escape(&summary(e))));
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

/// What the event is called: the show and the episode, with the season
/// where the schedule has one.
fn summary(e: &Entry) -> String {
    match e.season {
        Some(s) => format!("{} - Season {s} Episode {}", e.title, e.number),
        None => format!("{} - Episode {}", e.title, e.number),
    }
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

/// The file's text with the DTSTAMP lines left out: what a calendar app
/// compares, so a rewrite that only moves the stamp is no rewrite at all.
fn without_stamps(text: &str) -> String {
    text.split("\r\n")
        .filter(|line| !line.starts_with("DTSTAMP:"))
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// Writes the file unless what is on disk already says the same. Ok(true)
/// when it wrote.
fn write_file(core: &Core) -> Result<bool, CoreError> {
    let images_dir = core.paths.images_dir();
    let now = time::now_secs();
    let text = core.store.read(|c| {
        let snap = Snapshot::load(c, &images_dir, time::now(), None)?;
        Ok(render(&entries(&snap, now), now))
    })?;
    let target = core.paths.calendar_path();
    if let Ok(old) = std::fs::read_to_string(&target)
        && without_stamps(&old) == without_stamps(&text)
    {
        return Ok(false);
    }
    atomic_file::write(&target, text)?;
    Ok(true)
}

/// Rewrites the file if the switch is on, on the calling thread. A failure
/// is a warning the first time only, never an error to the caller: the
/// calendar is a side effect of whatever changed the schedule, and a disk
/// that stays full should not fill the activity log too. A success says
/// nothing on the bus, since every event there invalidates the shell's
/// caches.
pub(crate) fn write(core: &Core) {
    let _file = recover(&core.calendar.file);
    if !enabled(core) {
        return;
    }
    match write_file(core) {
        Ok(wrote) => {
            core.calendar.failing.store(false, Ordering::SeqCst);
            if wrote {
                tracing::debug!("calendar written");
            }
        }
        Err(e) => {
            if !core.calendar.failing.swap(true, Ordering::SeqCst) {
                core.bus.warn(
                    Stage::System,
                    format!("the calendar file could not be written: {e}"),
                    EventBody::Notice,
                );
            }
        }
    }
}

/// Deletes the file, as turning the switch off does. A file that is not
/// there is already the wanted state.
pub(crate) fn remove(core: &Core) {
    let _file = recover(&core.calendar.file);
    core.calendar.failing.store(false, Ordering::SeqCst);
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

/// Whether an event can change what the file shows. Any job that ends, and
/// the changes that land without a job: a show hidden, unmatched or gone, a
/// source removed, a tracker connected, dropped or made main, the
/// preferences (the title language names every event). A broad net on
/// purpose: a rewrite that changes nothing is skipped, so a false alarm
/// costs one read.
fn may_change(event: &Event) -> bool {
    if event
        .job
        .as_ref()
        .is_some_and(|j| j.phase == JobPhase::Finished)
    {
        return true;
    }
    matches!(
        event.body,
        EventBody::SeriesChanged { .. }
            | EventBody::SeriesRemoved { .. }
            | EventBody::SourceRemoved { .. }
            | EventBody::TrackersChanged { .. }
            | EventBody::PreferencesChanged { .. }
    )
}

/// How long a burst of changes is gathered before one rewrite: a scan or a
/// sync emits many events, and the file needs writing once after them.
const SETTLE: Duration = Duration::from_secs(1);

/// Subscribes the listener that rewrites the file after a change. Called
/// once, by `Core::open`; `shutdown` drops it.
pub(crate) fn listen(core: &Arc<Core>) {
    let listener = Arc::new(AfterChange {
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
struct AfterChange {
    core: Weak<Core>,
}

impl EventListener for AfterChange {
    fn on_event(&self, event: Event) {
        if !may_change(&event) {
            return;
        }
        let Some(core) = self.core.upgrade() else {
            return;
        };
        // One rewrite queued at a time: the events of a burst join it.
        if core.calendar.queued.swap(true, Ordering::SeqCst) {
            return;
        }
        // Off the emitting thread: the bus calls listeners inline, and a
        // job's runner should not wait on a file write.
        let weak = self.core.clone();
        core.handle.spawn(async move {
            tokio::time::sleep(SETTLE).await;
            let Some(core) = weak.upgrade() else {
                return;
            };
            // Cleared before the write, so a change during it queues the next.
            core.calendar.queued.store(false, Ordering::SeqCst);
            let handle = core.handle.clone();
            let _ = handle.spawn_blocking(move || write(&core)).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str, number: u32, at: i64) -> Entry {
        Entry {
            uid: format!("anibeam-anilist-7-{number}@anibeam"),
            title: title.to_string(),
            season: None,
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
        assert!(text.contains("UID:anibeam-anilist-7-3@anibeam\r\n"));
        assert!(text.contains("DTSTAMP:20261102T130509Z\r\n"));
        assert!(text.contains("DTSTART:20261102T140509Z\r\n"));
        assert!(text.contains("DURATION:PT24M\r\n"));
        assert!(text.contains("SUMMARY:Frieren\\, Part 2 - Episode 3\r\n"));
        assert!(text.contains("X-WR-CALNAME:AniBeam\r\n"));
    }

    #[test]
    fn a_tvmaze_season_is_named_in_the_summary() {
        let mut e = entry("Bluey", 4, 0);
        e.season = Some(3);
        assert_eq!(summary(&e), "Bluey - Season 3 Episode 4");
        e.season = None;
        assert_eq!(summary(&e), "Bluey - Episode 4");
    }

    #[test]
    fn a_new_stamp_alone_is_the_same_file() {
        let at = 1_793_624_709;
        let e = [entry("Frieren", 3, at)];
        let first = render(&e, at);
        let later = render(&e, at + 600);
        assert_ne!(first, later);
        assert_eq!(without_stamps(&first), without_stamps(&later));
        let moved = render(&[entry("Frieren", 3, at + 60)], at);
        assert_ne!(without_stamps(&first), without_stamps(&moved));
    }
}
