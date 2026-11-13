//! Activity (#750): the core's own log on the phone. The core words each event for a person
//! (`plain`); this shows those lines, newest first, the way the desktop's activity drawer
//! does: runs of identical lines folded into one row, Everything or just Problems, Copy and
//! Clear. The page reads the log when it opens and follows new lines after that.

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Write;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use anibeam_core::{Call, Event, Level, Reply};
use chrono::{DateTime, Local};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::{ActivityData, AppWindow, LogRow};

/// The lines the page keeps, as the desktop keeps them.
const KEPT: usize = 2000;

/// One logged line, as the page needs it.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    seq: u64,
    at: SystemTime,
    level: Level,
    plain: String,
}

/// One row on the page: a line, or a folded run of identical ones.
#[derive(Clone, Debug, PartialEq)]
struct Row {
    time: String,
    msg: String,
    level: i32,
    count: u32,
    /// The oldest line of the run's seq, which stays the same as the run grows at the top.
    key: u64,
    /// Set on the first line of an open run: the key of the run it folds back up.
    fold: Option<u64>,
}

struct Page {
    window: Weak<AppWindow>,
    entries: Vec<Entry>,
    loaded: bool,
    /// Lines that came while the first read was on its way.
    early: Vec<Entry>,
    problems_only: bool,
    expanded: HashSet<u64>,
    rows: Vec<Row>,
    model: Rc<VecModel<LogRow>>,
    /// A burst of lines makes one redraw.
    redraw: slint::Timer,
}

thread_local! {
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Page)) {
    PAGE.with(|cell| {
        if let Some(p) = cell.borrow_mut().as_mut() {
            f(p);
        }
    });
}

pub fn install(window: &AppWindow) {
    let model = Rc::new(VecModel::default());
    window
        .global::<ActivityData>()
        .set_rows(ModelRc::from(model.clone()));
    PAGE.with(|cell| {
        *cell.borrow_mut() = Some(Page {
            window: window.as_weak(),
            entries: Vec::new(),
            loaded: false,
            early: Vec::new(),
            problems_only: false,
            expanded: HashSet::new(),
            rows: Vec::new(),
            model,
            redraw: slint::Timer::default(),
        })
    });
    let d = window.global::<ActivityData>();
    d.on_read(|| with(Page::read));
    d.on_filter(|problems| {
        with(|p| {
            p.problems_only = problems;
            p.show();
        })
    });
    d.on_copy_all(|| with(Page::copy_all));
    d.on_copy_line(|i| with(|p| p.copy_line(i)));
    d.on_clear(|| with(Page::clear));
    d.on_expand(|i| {
        with(|p| {
            if let Some(r) = usize::try_from(i).ok().and_then(|i| p.rows.get(i)) {
                p.expanded.insert(r.key);
                p.show();
            }
        })
    });
    d.on_fold(|i| {
        with(|p| {
            if let Some(key) = usize::try_from(i)
                .ok()
                .and_then(|i| p.rows.get(i))
                .and_then(|r| r.fold)
            {
                p.expanded.remove(&key);
                p.show();
            }
        })
    });
}

/// The language changed: the rows are built again, with their times written the new way.
pub fn relocalize() {
    with(Page::show);
}

/// Every event the core sends, from whichever thread sent it.
pub fn core_event(event: &Event) {
    if matches!(event.level, Level::Debug) {
        return;
    }
    let entry = entry(event);
    let _ = slint::invoke_from_event_loop(move || with(|p| p.add(entry)));
}

fn entry(e: &Event) -> Entry {
    Entry {
        seq: e.seq,
        at: e.at,
        level: e.level,
        plain: e.plain.clone().unwrap_or_default(),
    }
}

impl Page {
    /// The page opened: the core's log, newest last in the call and newest first here.
    fn read(&mut self) {
        let Some(core) = crate::local::core() else {
            return;
        };
        let _ = std::thread::Builder::new()
            .name("activity".into())
            .spawn(move || {
                let answer = core.call(Call::RecentEvents { limit: KEPT as u64 });
                let _ = slint::invoke_from_event_loop(move || {
                    with(|p| match answer {
                        Ok(Reply::Events { events }) => {
                            p.entries = events
                                .iter()
                                .rev()
                                .filter(|e| !matches!(e.level, Level::Debug))
                                .map(entry)
                                .collect();
                            // What came meanwhile and the snapshot lacks goes in front.
                            let newest = p.entries.first().map_or(0, |e| e.seq);
                            for e in std::mem::take(&mut p.early) {
                                if e.seq > newest {
                                    p.entries.insert(0, e);
                                }
                            }
                            p.loaded = true;
                            p.show();
                        }
                        Ok(_) => {}
                        Err(e) => crate::local::log(&format!("activity: {e}")),
                    })
                });
            });
    }

    /// A new line while the log is open.
    fn add(&mut self, entry: Entry) {
        if !self.loaded {
            self.early.push(entry);
            self.early.truncate(KEPT);
            return;
        }
        if self.entries.first().is_some_and(|e| e.seq >= entry.seq) {
            return;
        }
        self.entries.insert(0, entry);
        self.entries.truncate(KEPT);
        let this = self.window.clone();
        self.redraw.start(
            slint::TimerMode::SingleShot,
            Duration::from_millis(250),
            move || {
                if this.upgrade().is_some() {
                    with(Page::show);
                }
            },
        );
    }

    fn show(&mut self) {
        self.rows = rows(
            &self.entries,
            self.problems_only,
            &self.expanded,
            Local::now(),
        );
        let lines: Vec<LogRow> = self
            .rows
            .iter()
            .map(|r| LogRow {
                time: r.time.as_str().into(),
                msg: r.msg.as_str().into(),
                level: r.level,
                count: r.count as i32,
                fold: r.fold.is_some(),
            })
            .collect();
        self.model.set_vec(lines);
        if let Some(w) = self.window.upgrade() {
            w.global::<ActivityData>().set_loaded(self.loaded);
        }
    }

    fn copy_all(&mut self) {
        let text = self.rows.iter().map(line).collect::<Vec<_>>().join("\n");
        let ok = crate::android::copy_text("AniBeam activity", &text);
        if let Some(w) = self.window.upgrade() {
            w.global::<ActivityData>().set_copied(ok);
        }
    }

    fn copy_line(&mut self, i: i32) {
        if let Some(r) = usize::try_from(i).ok().and_then(|i| self.rows.get(i)) {
            crate::android::copy_text("AniBeam activity", &line(r));
        }
    }

    fn clear(&mut self) {
        let Some(core) = crate::local::core() else {
            return;
        };
        match core.call(Call::ClearEvents) {
            Ok(_) => {
                self.entries.clear();
                self.expanded.clear();
                self.show();
            }
            Err(e) => crate::local::log(&format!("activity: {e}")),
        }
    }
}

/// A row as one line of text, for Copy.
fn line(r: &Row) -> String {
    let count = if r.count > 1 {
        format!("  ×{}", r.count)
    } else {
        String::new()
    };
    format!("{}  {}{}", r.time, r.msg, count)
}

/// Today's lines say the time; older ones say the day too, so yesterday's error does not
/// read as a fresh one.
fn stamp(at: SystemTime, now: DateTime<Local>) -> String {
    let d: DateTime<Local> = at.into();
    let (id, fallback) = if d.date_naive() == now.date_naive() {
        ("activity-time-today", "%H:%M:%S")
    } else {
        ("activity-time-other", "%-d %b %H:%M")
    };
    // A pattern chrono cannot read falls back rather than panicking.
    let mut out = String::new();
    let locale = crate::tr::chrono_locale(crate::tr::lang());
    if write!(out, "{}", d.format_localized(&tr!(id), locale)).is_err() {
        out.clear();
        let _ = write!(out, "{}", d.format_localized(fallback, locale));
    }
    out
}

/// The page's rows: the lines worded for a person, only problems when asked, with runs of
/// identical consecutive lines folded. `entries` is newest first, so a run's newest line
/// is met first and its oldest last; the run is keyed by the oldest, which never changes
/// as the run grows at the top, so an open run stays open.
fn rows(
    entries: &[Entry],
    problems_only: bool,
    expanded: &HashSet<u64>,
    now: DateTime<Local>,
) -> Vec<Row> {
    let mut runs: Vec<(Row, Vec<Row>)> = Vec::new();
    for e in entries {
        if e.plain.is_empty() || (problems_only && matches!(e.level, Level::Info)) {
            continue;
        }
        let level = match e.level {
            Level::Error => 2,
            Level::Warn => 1,
            _ => 0,
        };
        let one = Row {
            time: stamp(e.at, now),
            msg: e.plain.clone(),
            level,
            count: 1,
            key: e.seq,
            fold: None,
        };
        match runs.last_mut() {
            Some((run, items)) if run.level == level && run.msg == one.msg => {
                run.count += 1;
                run.key = e.seq;
                items.push(one);
            }
            _ => runs.push((one.clone(), vec![one])),
        }
    }
    let mut out = Vec::new();
    for (run, items) in runs {
        if run.count > 1 && expanded.contains(&run.key) {
            for (n, mut item) in items.into_iter().enumerate() {
                item.key = 0;
                item.fold = (n == 0).then_some(run.key);
                out.push(item);
            }
        } else {
            out.push(run);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(h: u32, m: u32, s: u32) -> SystemTime {
        Local
            .with_ymd_and_hms(2026, 11, 12, h, m, s)
            .unwrap()
            .into()
    }

    fn e(seq: u64, level: Level, plain: &str) -> Entry {
        Entry {
            seq,
            at: at(9, 0, seq as u32),
            level,
            plain: plain.into(),
        }
    }

    fn now() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 11, 12, 12, 0, 0).unwrap()
    }

    #[test]
    fn identical_lines_in_a_row_fold_and_key_on_the_oldest() {
        let log = [
            e(5, Level::Info, "Scanned"),
            e(4, Level::Info, "Scanned"),
            e(3, Level::Warn, "Slow"),
            e(2, Level::Info, "Scanned"),
        ];
        let r = rows(&log, false, &HashSet::new(), now());
        assert_eq!(r.len(), 3);
        assert_eq!((r[0].count, r[0].key), (2, 4));
        assert_eq!((r[1].level, r[2].count), (1, 1));
    }

    #[test]
    fn lines_without_a_plain_word_and_info_when_problems_only_are_left_out() {
        let log = [
            e(3, Level::Info, "Done"),
            e(2, Level::Error, "Failed"),
            e(1, Level::Info, ""),
        ];
        assert_eq!(rows(&log, false, &HashSet::new(), now()).len(), 2);
        let r = rows(&log, true, &HashSet::new(), now());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].level, 2);
    }

    #[test]
    fn an_open_run_lists_each_line_and_keeps_a_fold_on_its_first() {
        let log = [e(3, Level::Info, "Scanned"), e(2, Level::Info, "Scanned")];
        let open: HashSet<u64> = [2].into_iter().collect();
        let r = rows(&log, false, &open, now());
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].fold, Some(2));
        assert_eq!(r[1].fold, None);
        assert!(r.iter().all(|r| r.count == 1));
    }

    #[test]
    fn a_line_from_another_day_says_the_day() {
        crate::tr::set_lang(crate::tr::Lang::En);
        let old: SystemTime = Local
            .with_ymd_and_hms(2026, 11, 10, 8, 5, 3)
            .unwrap()
            .into();
        assert_eq!(stamp(old, now()), "10 Nov 08:05");
        assert_eq!(stamp(at(9, 1, 2), now()), "09:01:02");
    }

    #[test]
    fn copy_writes_the_time_the_line_and_its_count() {
        let r = Row {
            time: "09:00:01".into(),
            msg: "Scanned".into(),
            level: 0,
            count: 3,
            key: 1,
            fold: None,
        };
        assert_eq!(line(&r), "09:00:01  Scanned  ×3");
    }
}
