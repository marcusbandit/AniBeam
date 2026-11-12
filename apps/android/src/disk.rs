//! What the phone's episodes take: the desktop's Downloads usage line (watched against not
//! watched, and Delete watched), the shows hidden from the lists, and Hide itself. The
//! figures come from the library cards the core already counts (`bytes_on_disk`,
//! `watched_bytes`, `watched_on_disk`, `finished_watched`).

use std::cell::RefCell;
use std::collections::HashSet;

use anibeam_companion::format;
use anibeam_core::{Call, Direction, Reply, SeriesAction, SeriesCard, Sort, Tab};
use slint::{ComponentHandle, Model, ModelRc, VecModel};

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
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { dirty: true, ..State::default() });
}

fn plural(n: u64, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
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
        2 | 3 => format!(
            "{} and {}",
            names[..names.len() - 1].join(", "),
            names[names.len() - 1]
        ),
        n => format!("{} and {} more", names[..3].join(", "), n - 3),
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
                    ledger: format!(
                        "{}, {}",
                        plural(c.episodes_on_disk, "episode", "episodes"),
                        format::bytes(c.bytes_on_disk)
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
        format!(
            "{}, {}",
            plural(u.watched_episodes, "episode", "episodes"),
            format::bytes(u.watched_bytes)
        )
        .into(),
    );
    d.set_finished_line(
        format!(
            "{}, {}",
            plural(u.finished_shows, "show", "shows"),
            format::bytes(u.finished_bytes)
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
            let (u, names) = STATE.with(|s| {
                let s = s.borrow();
                let done: Vec<&SeriesCard> = s.disk.iter().filter(|c| c.finished_watched).collect();
                (usage(&s.disk), titles(&done))
            });
            if finished && u.finished_shows == 0 {
                d.set_sheet(0);
                d.set_line("No finished show is watched all the way through".into());
                return;
            }
            if !finished && u.watched_episodes == 0 {
                d.set_sheet(0);
                d.set_line("Nothing watched is on this phone".into());
                return;
            }
            STATE.with(|s| s.borrow_mut().finished = finished);
            if finished {
                d.set_sheet_title(
                    format!(
                        "Delete {}, {}?",
                        plural(u.finished_shows, "finished show", "finished shows"),
                        format::bytes(u.finished_bytes)
                    )
                    .into(),
                );
                d.set_sheet_body(
                    format!(
                        "{names}. Only shows the provider says have finished, every episode watched; a show still airing, or whose end is a guess, stays. Every file of them is deleted from this phone, extras too. Their downloads leave too, so they can be downloaded again."
                    )
                    .into(),
                );
            } else {
                d.set_sheet_title(
                    format!(
                        "Delete {}, {}?",
                        plural(u.watched_episodes, "watched episode", "watched episodes"),
                        format::bytes(u.watched_bytes)
                    )
                    .into(),
                );
                d.set_sheet_body(
                    "Every episode you have watched, in every show. Episodes not watched yet and extras stay. The files and their subtitles are deleted from this phone for good."
                        .into(),
                );
            }
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
                            }) => format!(
                                "Deleted {}, {}",
                                plural(episodes, "episode", "episodes"),
                                format::bytes(bytes)
                            ),
                            Ok(_) => String::new(),
                            Err(e) => format!("Nothing was deleted: {e}"),
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
    fn counts_read_one_and_many() {
        assert_eq!(plural(1, "show", "shows"), "1 show");
        assert_eq!(plural(0, "show", "shows"), "0 shows");
        assert_eq!(plural(12, "episode", "episodes"), "12 episodes");
    }

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
