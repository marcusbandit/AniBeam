//! The feed: recently released and coming soon. Both readings are pure
//! functions over a `Snapshot`, ported from Electron's `FeedPage.tsx`
//! (`buildRecentEntry`, `buildRecentFeed`, `buildUpcomingFeed`) and
//! `airingUtils.ts` (`findNextUpcomingEpisode`). `ListFeed` persists
//! nothing: the shell writes `Preferences.feed_sort` through
//! `SetPreferences` on its own.

use std::path::Path;
use std::time::SystemTime;

use rusqlite::Connection;

use crate::contract::*;
use crate::library::cards::Snapshot;
use crate::time;

/// One series's place on the Recent feed: the reason it is there, the
/// highest episode number the series can claim (on disk, or the on-disk
/// episode `best_aired` names, whichever is greater), and the instant the
/// list sorts on. A series with no files at all has no entry.
///
/// `best_aired` is the latest on-disk episode with a known past air date.
/// The instant is that episode's own past air date (`Aired`), else the
/// newest file's `mtime` (`Downloaded`), else `best_aired`'s own date
/// (`Aired`), else no entry at all. A film always reads `Downloaded` from
/// its newest file's `mtime`, this plan's decision, since a film's single
/// file is never usefully described as "aired".
pub fn recent_entry(snapshot: &Snapshot, series_id: u64, now: SystemTime) -> Option<(FeedReason, f64, i64)> {
    let row = snapshot.row(series_id)?;
    let files = snapshot.files.get(&series_id).map(Vec::as_slice).unwrap_or(&[]);
    if files.is_empty() {
        return None;
    }
    let now_secs = time::to_secs(now);
    let newest_mtime = files.iter().map(|f| f.mtime).max();

    let on_disk_numbers: Vec<f64> = files.iter().filter(|f| f.is_episode).map(|f| f.number).collect();
    let highest_on_disk = on_disk_numbers.iter().copied().fold(0.0_f64, f64::max);

    let airing = row.anilist_id.and_then(|id| snapshot.airing.get(&id)).map(Vec::as_slice).unwrap_or(&[]);
    let mut best_aired: Option<(i64, u32)> = None;
    for a in airing {
        let Some(at) = a.aired_at else { continue };
        if at > now_secs || !on_disk_numbers.contains(&f64::from(a.number)) {
            continue;
        }
        if best_aired.is_none_or(|(t, _)| at > t) {
            best_aired = Some((at, a.number));
        }
    }

    let highest = highest_on_disk.max(f64::from(best_aired.map_or(0, |(_, e)| e)));

    if row.kind == SeriesKind::Movie {
        let at = newest_mtime?;
        return Some((FeedReason::Downloaded { at: time::from_secs(at) }, highest, at));
    }

    let shown_air = airing.iter().find(|a| f64::from(a.number) == highest).and_then(|a| a.aired_at).filter(|t| *t <= now_secs);
    if let Some(at) = shown_air {
        return Some((FeedReason::Aired { episode: highest as u32, at: time::from_secs(at) }, highest, at));
    }
    if let Some(at) = newest_mtime {
        return Some((FeedReason::Downloaded { at: time::from_secs(at) }, highest, at));
    }
    if let Some((at, episode)) = best_aired {
        return Some((FeedReason::Aired { episode, at: time::from_secs(at) }, highest, at));
    }
    None
}

/// The earliest future `anilist_episodes` row for this series, matched or
/// not on disk: coming soon lists what is next, not what is already owned.
pub fn upcoming(snapshot: &Snapshot, series_id: u64, now: SystemTime) -> Option<(u32, i64)> {
    let row = snapshot.row(series_id)?;
    let anilist_id = row.anilist_id?;
    let airing = snapshot.airing.get(&anilist_id)?;
    let now_secs = time::to_secs(now);
    airing.iter().filter_map(|a| a.aired_at.map(|at| (a.number, at))).filter(|(_, at)| *at > now_secs).min_by_key(|(_, at)| *at)
}

/// A `FeedCard` for one series, or `None` when the id somehow no longer
/// resolves against the snapshot (never happens for an id taken from the
/// snapshot's own series list, but a lookup beats an `unwrap`).
fn feed_card(snapshot: &Snapshot, series_id: u64, reason: FeedReason, highest: f64) -> Option<FeedCard> {
    let series = snapshot.card(series_id)?;
    Some(FeedCard { series, reason, highest_on_disk: (highest > 0.0).then_some(highest) })
}

/// One series's Recent entry, kept alongside its id so the Upcoming sort
/// can split the set into scheduled and rest without losing it.
struct Recent {
    series_id: u64,
    reason: FeedReason,
    highest: f64,
    instant: i64,
}

/// The feed: series with files, not hidden, not missing, one snapshot load
/// for the whole read. `Recent` orders every entry newest first. `Upcoming`
/// floats the series with a scheduled next episode to the top, soonest
/// first, each carrying `Scheduled { episode, at }` and badged with its
/// highest episode on disk; the rest follow in Recent order with their
/// Recent reason. The shell draws its divider at the first non-`Scheduled`
/// card, so the two groups are simply concatenated here.
pub fn list(conn: &Connection, images_dir: &Path, sort: FeedSort, now: SystemTime) -> Result<Vec<FeedCard>, CoreError> {
    let snap = Snapshot::load(conn, images_dir, now, None)?;

    // `snap.series` is loaded `ORDER BY s.id`, so building straight off it
    // rather than through a `HashMap` keeps every sort below deterministic
    // before it even runs: two series sharing an instant (a batch import
    // often lands several files in the same second) must break the tie the
    // same way on every call, not by whichever order a hasher happened to
    // iterate in.
    let mut recents: Vec<Recent> = Vec::new();
    for row in &snap.series {
        if row.hidden || row.missing {
            continue;
        }
        if let Some((reason, highest, instant)) = recent_entry(&snap, row.id, now) {
            recents.push(Recent { series_id: row.id, reason, highest, instant });
        }
    }

    let cards = match sort {
        FeedSort::Recent => {
            recents.sort_by(|a, b| b.instant.cmp(&a.instant).then_with(|| a.series_id.cmp(&b.series_id)));
            recents.into_iter().filter_map(|r| feed_card(&snap, r.series_id, r.reason, r.highest)).collect()
        }
        FeedSort::Upcoming => {
            let mut scheduled: Vec<(u64, u32, i64, f64)> = Vec::new();
            let mut rest: Vec<Recent> = Vec::new();
            for r in recents {
                match upcoming(&snap, r.series_id, now) {
                    Some((episode, at)) => scheduled.push((r.series_id, episode, at, r.highest)),
                    None => rest.push(r),
                }
            }
            scheduled.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));
            rest.sort_by(|a, b| b.instant.cmp(&a.instant).then_with(|| a.series_id.cmp(&b.series_id)));

            scheduled
                .into_iter()
                .filter_map(|(series_id, episode, at, highest)| {
                    feed_card(&snap, series_id, FeedReason::Scheduled { episode, at: time::from_secs(at) }, highest)
                })
                .chain(rest.into_iter().filter_map(|r| feed_card(&snap, r.series_id, r.reason, r.highest)))
                .collect()
        }
    };
    Ok(cards)
}
