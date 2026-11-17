//! What the phone's episodes take: the desktop's Downloads usage line (watched against not
//! watched, and Delete watched), the shows hidden from the lists, and Hide itself. The
//! figures come from the library cards the core already counts (`bytes_on_disk`,
//! `watched_bytes`, `watched_on_disk`, `finished_watched`).

use std::cell::RefCell;
use std::collections::HashSet;

use anibeam_companion::format;
use anibeam_core::{Call, Direction, Reply, SeriesAction, SeriesCard, Sort, Tab};
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};

use crate::{AppWindow, DiskData, HiddenRow};

/// Nav.page of Downloads.
const DOWNLOADS_PAGE: i32 = 2;

#[derive(Default)]
struct State {
    /// Every show with a file on the phone, the watched and hidden ones too.
    disk: Vec<SeriesCard>,
    hidden: Vec<SeriesCard>,
    /// Read again at the next tick.
    dirty: bool,
    deleting: bool,
    /// What the open sheet will delete.
    finished: bool,
    window: Weak<AppWindow>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { dirty: true, ..State::default() });
}

/// The figures behind the usage line and the sheet.
#[derive(Debug, PartialEq)]
struct Usage {
    bytes: u64,
    watched_bytes: u64,
    watched_episodes: u64,
    finished_shows: u64,
    finished_bytes: u64,
}

fn usage(disk: &[SeriesCard]) -> Usage {
    let finished = || disk.iter().filter(|c| c.finished_watched);
    Usage {
        bytes: disk.iter().map(|c| c.bytes_on_disk).sum(),
        watched_bytes: disk.iter().map(|c| c.watched_bytes).sum(),
        watched_episodes: disk.iter().map(|c| c.watched_on_disk).sum(),
        finished_shows: finished().count() as u64,
        finished_bytes: finished().map(|c| c.bytes_on_disk).sum(),
    }
}

fn titles(cards: &[&SeriesCard]) -> String {
    let mut t: Vec<&SeriesCard> = cards.to_vec();
    t.sort_by_key(|c| std::cmp::Reverse(c.bytes_on_disk));
    let names: Vec<&str> = t.iter().map(|c| c.title.as_str()).collect();
    match names.len() {
        0 => String::new(),
        1 => names[0].to_string(),
        2 => tr!("disk-list-two", a = names[0], b = names[1]),
        3 => tr!("disk-list-three", a = names[0], b = names[1], c = names[2]),
        n => tr!(
            "disk-list-more",
            a = names[0],
            b = names[1],
            c = names[2],
            count = n - 3
        ),
    }
}

fn list(core: &anibeam_core::Core, tab: Tab) -> Vec<SeriesCard> {
    match core.call(Call::ListLibrary {
        tab,
        query: String::new(),
        sort: Sort::LastViewed,
        direction: Direction::Desc,
        reveal_hidden: false,
    }) {
        Ok(Reply::Library { series, .. }) => series,
        _ => Vec::new(),
    }
}

/// Reads the disk and the hidden shows again.
fn read(all: &[SeriesCard]) {
    let Some(core) = crate::local::core() else {
        return;
    };
    let mut seen = HashSet::new();
    let mut disk = Vec::new();
    let hidden = list(&core, Tab::Hidden);
    for c in all
        .iter()
        .cloned()
        .chain(list(&core, Tab::Watched))
        .chain(hidden.iter().cloned())
    {
        if c.bytes_on_disk > 0 && seen.insert(c.id) {
            disk.push(c);
        }
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.disk = disk;
        s.hidden = hidden;
        s.dirty = false;
    });
}

fn show(window: &AppWindow) {
    let (u, hidden, deleting) = STATE.with(|s| {
        let s = s.borrow();
        (
            usage(&s.disk),
            s.hidden
                .iter()
                .map(|c| HiddenRow {
                    id: i32::try_from(c.id).unwrap_or(-1),
                    title: c.title.clone().into(),
                    ledger: tr!(
                        "disk-episodes-size",
                        count = c.episodes_on_disk,
                        size = format::bytes(c.bytes_on_disk)
                    )
                    .into(),
                })
                .collect::<Vec<_>>(),
            s.deleting,
        )
    });
    let d = window.global::<DiskData>();
    d.set_show(u.bytes > 0);
    d.set_total_text(format::bytes(u.bytes).into());
    d.set_watched_text(format::bytes(u.watched_bytes).into());
    d.set_unwatched_text(format::bytes(u.bytes.saturating_sub(u.watched_bytes)).into());
    d.set_watched_share(if u.bytes > 0 {
        (u.watched_bytes as f64 / u.bytes as f64).clamp(0.0, 1.0) as f32
    } else {
        0.0
    });
    d.set_can_delete(u.watched_bytes > 0);
    d.set_busy(deleting);
    d.set_all_line(
        tr!(
            "disk-episodes-size",
            count = u.watched_episodes,
            size = format::bytes(u.watched_bytes)
        )
        .into(),
    );
    d.set_finished_line(
        tr!(
            "disk-shows-size",
            count = u.finished_shows,
            size = format::bytes(u.finished_bytes)
        )
        .into(),
    );
    let model = d.get_hidden();
    if model.row_count() == hidden.len()
        && (0..hidden.len()).all(|i| model.row_data(i).as_ref() == Some(&hidden[i]))
    {
        return;
    }
    d.set_hidden(ModelRc::new(VecModel::from(hidden)));
}

/// The language changed: the figures' lines are written again at the next tick, and an open
/// sheet is worded again now.
pub fn relocalize() {
    let (window, finished) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.dirty = true;
        (s.window.upgrade(), s.finished)
    });
    if let Some(window) = window {
        let d = window.global::<DiskData>();
        if d.get_sheet() >= 2 {
            let u = STATE.with(|s| usage(&s.borrow().disk));
            sheet_text(&d, finished, &u);
        }
    }
}

/// The open sheet's title and body: what the delete takes, and its names.
fn sheet_text(d: &DiskData, finished: bool, u: &Usage) {
    if finished {
        let names = STATE.with(|s| {
            let s = s.borrow();
            let done: Vec<&SeriesCard> = s.disk.iter().filter(|c| c.finished_watched).collect();
            titles(&done)
        });
        d.set_sheet_title(
            tr!(
                "disk-delete-finished-title",
                count = u.finished_shows,
                size = format::bytes(u.finished_bytes)
            )
            .into(),
        );
        d.set_sheet_body(tr!("disk-delete-finished-body", names = names).into());
    } else {
        d.set_sheet_title(
            tr!(
                "disk-delete-watched-title",
                count = u.watched_episodes,
                size = format::bytes(u.watched_bytes)
            )
            .into(),
        );
        d.set_sheet_body(tr!("disk-delete-watched-body").into());
    }
}

/// Called each second with whether the core said something and the library's cards.
pub fn tick(window: &AppWindow, changed: bool, cards: &[SeriesCard]) {
    if changed {
        STATE.with(|s| s.borrow_mut().dirty = true);
    }
    // Only Downloads shows these figures, so the extra reads wait until it is open.
    if window.global::<crate::Nav>().get_page() != DOWNLOADS_PAGE {
        return;
    }
    if STATE.with(|s| s.borrow().dirty) && crate::local::core().is_some() {
        read(cards);
        show(window);
    }
}

fn act_hide(id: u64, hidden: bool) {
    let Some(core) = crate::local::core() else {
        return;
    };
    if let Err(e) = core.call(Call::ActOnSeries {
        series: vec![id],
        action: SeriesAction::Hide { hidden },
    }) {
        crate::local::log(&format!("hide: {e}"));
    }
    crate::local::touch();
}

/// Wires the usage line, the sheet and Hide. Once, at startup.
pub fn install(window: &AppWindow) {
    STATE.with(|s| s.borrow_mut().window = window.as_weak());
    let d = window.global::<DiskData>();
    d.on_hide(|id| {
        if let Ok(id) = u64::try_from(id) {
            act_hide(id, true);
        }
    });
    d.on_unhide(|id| {
        if let Ok(id) = u64::try_from(id) {
            act_hide(id, false);
        }
    });
    d.on_ask({
        let window = window.as_weak();
        move |kind| {
            let Some(window) = window.upgrade() else {
                return;
            };
            let d = window.global::<DiskData>();
            if kind == 0 {
                d.set_sheet(1);
                return;
            }
            let finished = kind == 2;
            let u = STATE.with(|s| usage(&s.borrow().disk));
            if finished && u.finished_shows == 0 {
                d.set_sheet(0);
                d.set_line(tr!("disk-no-finished").into());
                return;
            }
            if !finished && u.watched_episodes == 0 {
                d.set_sheet(0);
                d.set_line(tr!("disk-nothing-watched").into());
                return;
            }
            STATE.with(|s| s.borrow_mut().finished = finished);
            sheet_text(&d, finished, &u);
            d.set_sheet(if finished { 3 } else { 2 });
        }
    });
    d.on_confirm({
        let window = window.as_weak();
        move || {
            let Some(core) = crate::local::core() else {
                return;
            };
            let (finished, busy) = STATE.with(|s| {
                let mut s = s.borrow_mut();
                let busy = s.deleting;
                s.deleting = true;
                (s.finished, busy)
            });
            if busy {
                return;
            }
            if let Some(window) = window.upgrade() {
                let d = window.global::<DiskData>();
                d.set_busy(true);
                d.set_line("".into());
            }
            let window = window.clone();
            let _ = std::thread::Builder::new()
                .name("delete-watched".into())
                .spawn(move || {
                    let answer = core.call(Call::DeleteWatched {
                        finished_shows: finished,
                    });
                    let _ = slint::invoke_from_event_loop(move || {
                        STATE.with(|s| {
                            let mut s = s.borrow_mut();
                            s.deleting = false;
                            s.dirty = true;
                        });
                        crate::local::touch();
                        let Some(window) = window.upgrade() else {
                            return;
                        };
                        let line = match answer {
                            Ok(Reply::Deleted {
                                episodes, bytes, ..
                            }) => tr!(
                                "disk-deleted",
                                count = episodes,
                                size = format::bytes(bytes)
                            ),
                            Ok(_) => String::new(),
                            Err(e) => tr!("disk-delete-failed", error = e.to_string()),
                        };
                        let d = window.global::<DiskData>();
                        d.set_busy(false);
                        d.set_line(line.into());
                    });
                });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cards_take_no_space() {
        assert_eq!(
            usage(&[]),
            Usage {
                bytes: 0,
                watched_bytes: 0,
                watched_episodes: 0,
                finished_shows: 0,
                finished_bytes: 0
            }
        );
        assert_eq!(titles(&[]), "");
    }
}
