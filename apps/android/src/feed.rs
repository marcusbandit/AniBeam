//! The feed on Home: the desktop's Feed page, a card per show with files, sorted by the
//! core (`ListFeed`) as Recently released or Coming soon and cut into the same sections.
//! The sort is the core's own `feed_sort` preference, so it is the one the desktop keeps.

use std::cell::RefCell;

use anibeam_companion::format;
use anibeam_core::{Call, FeedCard, FeedReason, FeedSort, Reply};
use slint::ComponentHandle;

use crate::app::Shelves;
use crate::phone::{cube_codes, secs};
use crate::remote::Art;
use crate::{AppWindow, FeedData, Tile};

/// Nav.page of Home.
const HOME_PAGE: i32 = 0;
const WEEK: i64 = 7 * 86400;
/// The feed is read again at least this often, so the countdowns and the "ago" lines move.
const REBUILD_EVERY: i64 = 30;
/// How many tiles a section shows; a longer run is cut so a huge library stays light.
const SECTION: usize = 60;

struct State {
    sort: FeedSort,
    /// The sort has been read from the preferences.
    sort_read: bool,
    cards: Vec<FeedCard>,
    /// Read the cards again on the next tick.
    dirty: bool,
    built_at: i64,
    shelves: Shelves,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn reason_at(card: &FeedCard) -> i64 {
    match &card.reason {
        FeedReason::Aired { at, .. } | FeedReason::Downloaded { at } => secs(*at),
        _ => 0,
    }
}

fn scheduled_at(card: &FeedCard) -> Option<i64> {
    match &card.reason {
        FeedReason::Scheduled { at, .. } => Some(secs(*at)),
        _ => None,
    }
}

/// The cards of each section, by position, under the section title's message ID (the
/// shelves look it up): Recently released is this week's over the earlier ones; Coming soon
/// is the scheduled ones over everything else.
fn cut(cards: &[FeedCard], sort: FeedSort, now: i64) -> Vec<(&'static str, Vec<usize>)> {
    let fresh = |c: &FeedCard| {
        let at = reason_at(c);
        at > 0 && now - at <= WEEK
    };
    let all = 0..cards.len();
    match sort {
        FeedSort::Recent => vec![
            (
                "feed-this-week",
                all.clone().filter(|&i| fresh(&cards[i])).collect(),
            ),
            ("feed-earlier", all.filter(|&i| !fresh(&cards[i])).collect()),
        ],
        FeedSort::Upcoming => vec![
            (
                "feed-coming-soon",
                all.clone()
                    .filter(|&i| scheduled_at(&cards[i]).is_some())
                    .collect(),
            ),
            (
                "feed-everything-else",
                all.filter(|&i| scheduled_at(&cards[i]).is_none()).collect(),
            ),
        ],
    }
}

fn short_countdown(secs: i64) -> String {
    let t = secs.max(0);
    let (d, h, m) = (t / 86400, t % 86400 / 3600, t % 3600 / 60);
    if d > 0 {
        tr!("phone-feed-countdown-days", d = d, h = h)
    } else if h > 0 {
        tr!("phone-feed-countdown-hours", h = h, m = m)
    } else {
        tr!("phone-feed-countdown-minutes", m = m.max(1))
    }
}

/// This sort's one line: what is new this week, or what airs next.
fn summary(cards: &[FeedCard], sort: FeedSort, now: i64) -> String {
    match sort {
        FeedSort::Upcoming => cards
            .iter()
            .filter_map(|c| scheduled_at(c).map(|at| (at, c)))
            .filter(|(at, _)| *at > now)
            .min_by_key(|(at, _)| *at)
            .map(|(at, c)| {
                tr!(
                    "phone-feed-next",
                    title = &c.series.title,
                    countdown = short_countdown(at - now)
                )
            })
            .unwrap_or_else(|| tr!("phone-feed-nothing-scheduled")),
        FeedSort::Recent => {
            let fresh = cards
                .iter()
                .filter(|c| {
                    let at = reason_at(c);
                    at > 0 && now - at <= WEEK
                })
                .count();
            match fresh {
                0 => tr!("phone-feed-nothing-new"),
                n => tr!("phone-feed-new-episodes", count = n),
            }
        }
    }
}

fn tile(card: &FeedCard, art: &Art, now: i64) -> Tile {
    let s = &card.series;
    let (poster, has_poster) = art.poster(Some(s.id));
    let mut t = Tile {
        id: i32::try_from(s.id).unwrap_or(-1),
        title: s.title.clone().into(),
        poster,
        has_poster,
        initials: format::initials(&s.title).into(),
        cubes: std::rc::Rc::new(slint::VecModel::from(cube_codes(&s.cubes))).into(),
        ..Default::default()
    };
    match &card.reason {
        FeedReason::Aired { episode, at } => {
            t.badge = format::episode_code(None, *episode).into();
            t.ledger = tr!("phone-feed-aired", ago = format::ago(now - secs(*at))).into();
        }
        FeedReason::Downloaded { at } => {
            t.ledger = tr!("phone-feed-downloaded", ago = format::ago(now - secs(*at))).into();
        }
        FeedReason::Scheduled {
            episode,
            at,
            season,
            ..
        } => {
            t.badge = format::episode_code(*season, *episode).into();
            t.ledger = format::countdown(secs(*at) - now).into();
            t.ledger_tone = 1;
        }
        FeedReason::None => {}
    }
    if let Some(d) = s.downloading.first() {
        t.flag = format!("{}%", (d.progress * 100.0).round() as i64).into();
    }
    t
}

fn read_sort() -> Option<FeedSort> {
    match crate::local::core()?.call(Call::GetPreferences) {
        Ok(Reply::Preferences { preferences }) => Some(preferences.feed_sort),
        _ => None,
    }
}

/// The sort goes into the preferences the desktop reads too.
fn keep_sort(sort: FeedSort) {
    let Some(core) = crate::local::core() else {
        return;
    };
    if let Ok(Reply::Preferences { mut preferences }) = core.call(Call::GetPreferences) {
        preferences.feed_sort = sort;
        if let Err(e) = core.call(Call::SetPreferences { preferences }) {
            crate::local::log(&format!("feed: sort not kept: {e}"));
        }
    }
}

/// Wires the sort. Once, at startup.
pub fn install(window: &AppWindow) {
    let shelves = Shelves::new();
    window
        .global::<FeedData>()
        .set_shelves(shelves.shelves.clone().into());
    STATE.with(|s| {
        *s.borrow_mut() = Some(State {
            sort: FeedSort::Recent,
            sort_read: false,
            cards: Vec::new(),
            dirty: true,
            built_at: 0,
            shelves,
        });
    });
    let window = window.as_weak();
    window.unwrap().global::<FeedData>().on_pick_sort(move |i| {
        let sort = if i == 1 {
            FeedSort::Upcoming
        } else {
            FeedSort::Recent
        };
        STATE.with(|s| {
            if let Some(s) = s.borrow_mut().as_mut() {
                s.sort = sort;
                s.sort_read = true;
                s.dirty = true;
            }
        });
        if let Some(window) = window.upgrade() {
            window.global::<FeedData>().set_sort(i);
        }
        keep_sort(sort);
    });
}

/// The language changed: the tiles' lines and the summary are written again at the next tick.
pub fn relocalize() {
    STATE.with(|s| {
        if let Some(s) = s.borrow_mut().as_mut() {
            s.built_at = 0;
        }
    });
}

/// Called each second with whether the core said something: reads the feed again when it
/// did, or when the sort changed, and rebuilds the tiles every half minute so the lines
/// that count time move.
pub fn tick(window: &AppWindow, changed: bool, art: &Art, now: i64) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let Some(s) = s.as_mut() else { return };
        let data = window.global::<FeedData>();
        // Only Home shows the feed, so the reads wait until it is open.
        if window.global::<crate::Nav>().get_page() != HOME_PAGE {
            if changed {
                s.dirty = true;
            }
            return;
        }
        if !s.sort_read
            && let Some(sort) = read_sort()
        {
            s.sort = sort;
            s.sort_read = true;
            s.dirty = true;
        }
        if !(changed || s.dirty || now - s.built_at >= REBUILD_EVERY) {
            return;
        }
        if changed || s.dirty {
            let Some(core) = crate::local::core() else {
                return;
            };
            match core.call(Call::ListFeed { sort: s.sort }) {
                Ok(Reply::Feed { cards }) => s.cards = cards,
                Ok(_) => {}
                Err(e) => crate::local::log(&format!("feed: {e}")),
            }
            s.dirty = false;
        }
        s.built_at = now;
        data.set_sort(i32::from(s.sort == FeedSort::Upcoming));
        data.set_summary(summary(&s.cards, s.sort, now).into());
        let rows = cut(&s.cards, s.sort, now)
            .into_iter()
            .map(|(title, idx)| {
                let tiles = idx
                    .into_iter()
                    .take(SECTION)
                    .map(|i| tile(&s.cards[i], art, now))
                    .collect();
                (title, tiles)
            })
            .collect();
        s.shelves.set(rows);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn a_countdown_reads_in_its_two_biggest_units() {
        assert_eq!(short_countdown(2 * 86400 + 3 * 3600 + 5), "2d 3h");
        assert_eq!(short_countdown(3 * 3600 + 120), "3h 2m");
        assert_eq!(short_countdown(10), "1m");
    }

    #[test]
    fn the_times_are_unix_seconds() {
        assert_eq!(secs(UNIX_EPOCH + Duration::from_secs(90)), 90);
    }
}
