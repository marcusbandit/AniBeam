//! Moving the list kept on this phone up to a tracker (#749). A shown that has a status or a
//! score only here is not on the tracker; once a tracker connects and its list is read, the
//! sheet offers to move them, one row a show, what is kept here beside what the tracker has
//! now. `PreviewListMove` reads that off the cache and sends nothing; `MoveLists` is the one
//! call that sends the list anywhere, and only when Move is pressed.

use std::cell::RefCell;

use anibeam_core::{Call, Event, EventBody, JobKind, ListStatus, MoveRow, Reply, Tracker};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::{AppWindow, MoveData, MoveLine};

struct Page {
    window: Weak<AppWindow>,
    /// Trackers whose first read list should open the sheet: set when a sign-in starts.
    offered: [bool; 2],
    /// The sheet's tracker and rows while it is up.
    open: Option<(Tracker, Vec<MoveRow>)>,
    /// How many shows wait for each tracker, AniList then MyAnimeList.
    waiting: [usize; 2],
    /// A count is on its way; another waits for it.
    counting: bool,
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

fn slot(t: Tracker) -> usize {
    usize::from(t == Tracker::Mal)
}

fn site(t: Tracker) -> &'static str {
    if t == Tracker::Mal {
        "MyAnimeList"
    } else {
        "AniList"
    }
}

pub fn install(window: &AppWindow) {
    PAGE.with(|cell| {
        *cell.borrow_mut() = Some(Page {
            window: window.as_weak(),
            offered: [false; 2],
            open: None,
            waiting: [0; 2],
            counting: false,
        })
    });
    let d = window.global::<MoveData>();
    d.on_read(|| with(Page::count));
    d.on_review(|| {
        with(|p| {
            let to = if p.waiting[0] > 0 {
                Tracker::Anilist
            } else {
                Tracker::Mal
            };
            p.preview(to, false);
        })
    });
    d.on_move(|| with(Page::move_up));
    d.on_later(|| with(Page::close));
}

/// A sign-in to `tracker` started: its first read list is when the list kept here is
/// offered to it. One tracker's offer never takes another's place.
pub fn offer(tracker: Tracker) {
    with(|p| p.offered[slot(tracker)] = true);
}

/// Every event the core sends, from whichever thread sent it.
pub fn core_event(event: &Event) {
    match &event.body {
        EventBody::ProgressRefreshed { tracker } => {
            let tracker = *tracker;
            let _ = slint::invoke_from_event_loop(move || {
                with(|p| {
                    if std::mem::take(&mut p.offered[slot(tracker)]) {
                        p.preview(tracker, true);
                    }
                    p.count();
                })
            });
        }
        // A sign-in that did not finish leaves no offer waiting for the next list read.
        EventBody::JobFailed { .. }
            if event
                .job
                .as_ref()
                .is_some_and(|j| j.kind == JobKind::ConnectTracker) =>
        {
            let _ = slint::invoke_from_event_loop(|| with(|p| p.offered = [false; 2]));
        }
        EventBody::ListsMoved {
            tracker,
            moved,
            failed,
        } => {
            let (tracker, moved, failed) = (*tracker, *moved, *failed);
            let _ = slint::invoke_from_event_loop(move || {
                with(|p| {
                    p.note(&moved_line(tracker, moved, failed));
                    p.count();
                })
            });
        }
        _ => {}
    }
}

impl Page {
    fn data<R>(&self, f: impl FnOnce(MoveData) -> R) -> Option<R> {
        self.window.upgrade().map(|w| f(w.global::<MoveData>()))
    }

    fn note(&self, text: &str) {
        self.data(|d| d.set_note(text.into()));
    }

    /// Reads what moving to `to` would change and opens the sheet when it is anything.
    /// `offer` says the sheet came up by itself, so an empty answer stays quiet.
    fn preview(&mut self, to: Tracker, offer: bool) {
        // A test hook for captures, like the seed list: the rows as JSON.
        if let Some(rows) = std::fs::read("/data/local/tmp/anibeam-demo/move.json")
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<MoveRow>>(&b).ok())
        {
            return self.show(to, rows);
        }
        let Some(core) = crate::local::core() else {
            return;
        };
        let _ = std::thread::Builder::new()
            .name("listmove".into())
            .spawn(move || {
                let answer = core.call(Call::PreviewListMove { tracker: to });
                let _ = slint::invoke_from_event_loop(move || {
                    with(|p| match answer {
                        Ok(Reply::ListMove { rows }) if !rows.is_empty() => p.show(to, rows),
                        // The offer after a sign-in stays quiet when there is nothing to move.
                        Ok(_) if offer => {}
                        Ok(_) => p.note("Nothing left to move"),
                        Err(e) if !offer => p.note(&e.to_string()),
                        Err(_) => {}
                    })
                });
            });
    }

    fn show(&mut self, to: Tracker, rows: Vec<MoveRow>) {
        let lines: Vec<MoveLine> = rows
            .iter()
            .map(|r| MoveLine {
                title: r.title.as_str().into(),
                here: side(r.status, r.score, "Nothing").into(),
                there: side(r.tracker_status, r.tracker_score, "Not on your list").into(),
            })
            .collect();
        let n = rows.len();
        self.data(|d| {
            d.set_site(site(to).into());
            d.set_lead(lead(n, site(to)).into());
            d.set_move_label(move_label(n).into());
            d.set_rows(ModelRc::new(VecModel::from(lines)));
            d.set_open(true);
        });
        self.open = Some((to, rows));
    }

    fn close(&mut self) {
        self.open = None;
        self.data(|d| d.set_open(false));
    }

    fn move_up(&mut self) {
        let Some((tracker, rows)) = self.open.take() else {
            return;
        };
        self.data(|d| d.set_open(false));
        let Some(core) = crate::local::core() else {
            return;
        };
        let ids: Vec<u64> = rows.iter().map(|r| r.series).collect();
        let n = ids.len();
        match core.call(Call::MoveLists {
            tracker,
            series: ids,
        }) {
            Ok(_) => self.note(&format!(
                "Moving {} to {}",
                if n == 1 {
                    "1 show".to_string()
                } else {
                    format!("{n} shows")
                },
                site(tracker)
            )),
            Err(e) => self.note(&e.to_string()),
        }
    }

    /// Counts what waits for each connected tracker, for Accounts' row.
    fn count(&mut self) {
        // The capture hook of `preview`: those rows wait for AniList.
        if let Some(rows) = std::fs::read("/data/local/tmp/anibeam-demo/move.json")
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<MoveRow>>(&b).ok())
        {
            self.waiting = [rows.len(), 0];
            return self
                .data(|d| {
                    d.set_waiting(rows.len() as i32);
                    d.set_waiting_line(waiting_line(rows.len(), "AniList").into());
                })
                .unwrap_or(());
        }
        let Some(core) = crate::local::core() else {
            return;
        };
        if std::mem::replace(&mut self.counting, true) {
            return;
        }
        let _ = std::thread::Builder::new()
            .name("listmove".into())
            .spawn(move || {
                let mut counts = [0usize; 2];
                if let Ok(Reply::Trackers { state }) = core.call(Call::GetTrackers) {
                    for (t, on) in [
                        (Tracker::Anilist, state.anilist.connected),
                        (Tracker::Mal, state.mal.connected),
                    ] {
                        if on
                            && let Ok(Reply::ListMove { rows }) =
                                core.call(Call::PreviewListMove { tracker: t })
                        {
                            counts[slot(t)] = rows.len();
                        }
                    }
                }
                let _ = slint::invoke_from_event_loop(move || {
                    with(|p| {
                        p.counting = false;
                        p.waiting = counts;
                        let (n, to) = if counts[0] > 0 {
                            (counts[0], Tracker::Anilist)
                        } else {
                            (counts[1], Tracker::Mal)
                        };
                        p.data(|d| {
                            d.set_waiting(n as i32);
                            d.set_waiting_line(waiting_line(n, site(to)).into());
                        });
                    })
                });
            });
    }
}

fn status_word(s: ListStatus) -> &'static str {
    match s {
        ListStatus::Watching => "Watching",
        ListStatus::Planning => "Planning",
        ListStatus::Completed => "Completed",
        ListStatus::Paused => "On hold",
        ListStatus::Dropped => "Dropped",
        ListStatus::Repeating => "Rewatching",
    }
}

/// A score as the desktop writes it: one decimal, a quarter point rounded away from even.
pub(crate) fn score(x: f64) -> String {
    let quarters = x * 4.0;
    if quarters.fract() == 0.0 && (quarters as i64).rem_euclid(2) != 0 {
        format!("{:.1}", (x * 10.0).round() / 10.0)
    } else {
        format!("{x:.1}")
    }
}

/// One side of a row: the status and the score, or `empty` for neither.
fn side(status: Option<ListStatus>, score_of: Option<f64>, empty: &str) -> String {
    let mut bits = Vec::new();
    if let Some(s) = status {
        bits.push(status_word(s).to_string());
    }
    if let Some(x) = score_of {
        bits.push(format!("score {}", score(x)));
    }
    if bits.is_empty() {
        empty.to_string()
    } else {
        bits.join(", ")
    }
}

fn lead(n: usize, site: &str) -> String {
    format!(
        "{} a status or a score only on this phone. Moving them puts them on your {site} list, where your computer and your profile see them. For these shows, what {site} has now is replaced. Nothing is sent until you press Move.",
        if n == 1 {
            "One show has".to_string()
        } else {
            format!("{n} shows have")
        }
    )
}

fn move_label(n: usize) -> String {
    if n == 1 {
        "Move 1 show".into()
    } else {
        format!("Move {n} shows")
    }
}

fn waiting_line(n: usize, site: &str) -> String {
    format!(
        "{} a status or a score only here. Move them to {site} to see them on your computer and your profile.",
        if n == 1 {
            "One show has".to_string()
        } else {
            format!("{n} shows have")
        }
    )
}

fn moved_line(t: Tracker, moved: u32, failed: u32) -> String {
    let shows = |n: u32| {
        if n == 1 {
            "1 show".to_string()
        } else {
            format!("{n} shows")
        }
    };
    if failed > 0 {
        format!(
            "{} did not take {}; see the activity log",
            site(t),
            shows(failed)
        )
    } else if moved > 0 {
        format!("Moved {} to {}", shows(moved), site(t))
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_side_says_status_and_score_or_the_empty_words() {
        assert_eq!(
            side(Some(ListStatus::Paused), Some(8.0), "Nothing"),
            "On hold, score 8.0"
        );
        assert_eq!(side(None, None, "Not on your list"), "Not on your list");
        assert_eq!(side(Some(ListStatus::Repeating), None, ""), "Rewatching");
    }

    #[test]
    fn the_lead_counts_the_shows() {
        assert!(lead(1, "AniList").starts_with("One show has a status"));
        assert!(lead(3, "AniList").starts_with("3 shows have a status"));
        assert_eq!(move_label(1), "Move 1 show");
        assert_eq!(move_label(4), "Move 4 shows");
    }

    #[test]
    fn a_move_that_failed_says_so_and_one_that_landed_says_where() {
        assert_eq!(
            moved_line(Tracker::Mal, 2, 1),
            "MyAnimeList did not take 1 show; see the activity log"
        );
        assert_eq!(
            moved_line(Tracker::Anilist, 3, 0),
            "Moved 3 shows to AniList"
        );
        assert_eq!(moved_line(Tracker::Anilist, 0, 0), "");
    }
}
