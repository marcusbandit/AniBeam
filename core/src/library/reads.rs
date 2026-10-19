//! The library's read side: the six calls a shell makes to draw the home
//! grid, the airing rail, a series page, the metadata table, and to turn a
//! path back into an id. Every one of them runs on the reader connection,
//! so a list never waits behind a long write.

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params};

use crate::contract::*;
use crate::core::Core;
use crate::images;
use crate::library::cards;
use crate::library::snapshot::Snapshot;
use crate::library::sort;
use crate::prefs;
use crate::time;

/// A film to the tabs: the folder was classified as one, or the match says
/// the title is one. Carried from `MetadataTab.tsx`'s `isMovie`.
fn is_movie(c: &SeriesCard) -> bool {
    cards::is_film(c)
}

/// Hidden series live in their own tab and are never mixed into the others;
/// a missing series appears under no tab at all. Watched series (`cards::
/// watched_out`) sit in their own tab too, and only rejoin the main tabs
/// when the preference says so — the Watched and Hidden tabs themselves
/// always list what they hold.
fn in_tab(c: &SeriesCard, tab: Tab, show_watched: bool) -> bool {
    if c.missing {
        return false;
    }
    let done = cards::watched_out(c);
    match tab {
        Tab::Hidden => c.hidden,
        Tab::Watched => !c.hidden && done,
        Tab::All => !c.hidden && (show_watched || !done),
        Tab::Series => !c.hidden && !is_movie(c) && (show_watched || !done),
        Tab::Movies => !c.hidden && is_movie(c) && (show_watched || !done),
    }
}

/// One whole-library read: the cards, the series whose poster is missing,
/// and the cover url behind each poster actually handed out.
#[derive(Clone)]
pub(crate) struct Loaded {
    cards: Vec<SeriesCard>,
    gaps: Vec<u64>,
    posters: HashMap<u64, String>,
}

/// The last whole-library load, kept for the reads that follow it in the
/// same state generation when a shell has turned the cache on
/// (`Core::set_snapshot_cache`). A list, the rail and the pool reading in
/// one beat with different arguments then pay for one snapshot.
pub(crate) struct KeptLoad {
    generation: u64,
    at: std::time::Instant,
    loaded: Loaded,
}

/// As long as a kept load may stand with nothing changed: the cards follow
/// the time of day (an episode airs, the rail and the tabs move).
const KEEP_FOR: std::time::Duration = std::time::Duration::from_secs(30);

/// Every card, with what a torrent is still writing laid over them (the
/// overlay is live state, so it goes on after the cache, never into it).
fn load_all(core: &Core) -> Result<Loaded, CoreError> {
    let mut loaded = load_kept(core)?;
    crate::downloads::overlay(core, &mut loaded.cards);
    Ok(loaded)
}

fn load_kept(core: &Core) -> Result<Loaded, CoreError> {
    if !core
        .snapshot_cache
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return load_fresh(core);
    }
    // Read before the load, so a change that lands during it leaves the
    // load stamped with the generation it may not have seen.
    let generation = core.generation();
    if let Some(kept) = core.kept_load.lock().unwrap().as_ref()
        && kept.generation == generation
        && kept.at.elapsed() < KEEP_FOR
    {
        return Ok(kept.loaded.clone());
    }
    let loaded = load_fresh(core)?;
    *core.kept_load.lock().unwrap() = Some(KeptLoad {
        generation,
        at: std::time::Instant::now(),
        loaded: loaded.clone(),
    });
    Ok(loaded)
}

fn load_fresh(core: &Core) -> Result<Loaded, CoreError> {
    let images_dir = core.paths.images_dir();
    let now = time::now();
    core.store.read(|c| {
        let snap = Snapshot::load(c, &images_dir, now, None)?;
        let posters = snap
            .series
            .iter()
            .filter(|r| r.poster_path.is_some())
            .filter_map(|r| Some((r.id, r.media.as_ref()?.cover_url.clone()?)))
            .collect();
        Ok(Loaded {
            gaps: snap.gaps.clone(),
            posters,
            cards: snap.cards(),
        })
    })
}

fn load_cards(core: &Core) -> Result<Vec<SeriesCard>, CoreError> {
    Ok(load_all(core)?.cards)
}

/// What every card read owes the image cache: the posters it handed out are
/// marked used, so the sweep can tell a live image from a forgotten one,
/// and a read that found a gap asks for a fill. The bump is queued on the
/// writer thread with no reply waited for, and the fill is coalesced and
/// rate-gated, so neither costs the read anything.
fn after_read(core: &Core, gaps: &[u64], used: Vec<String>) {
    core.images.bump_used(used);
    if gaps.is_empty() || !core.images.fill_is_due() {
        return;
    }
    if let Some(owner) = core.arc() {
        images::start_fill(&owner);
    }
}

/// The home grid. `reveal_hidden` is the shell's own tab visibility: the
/// Hidden tab always lists what it holds.
pub fn list_series(
    core: &Core,
    tab: Tab,
    query: &str,
    sort_key: Sort,
    direction: Direction,
) -> Result<Reply, CoreError> {
    let show_watched = core
        .store
        .read(|c| prefs::load_preferences(c).map(|p| p.library_show_watched))?;
    let Loaded {
        mut cards,
        gaps,
        posters,
    } = load_all(core)?;
    cards.retain(|c| in_tab(c, tab, show_watched) && sort::matches_query(c, query));
    sort::sort_cards(&mut cards, sort_key, direction);
    after_read(
        core,
        &gaps,
        cards
            .iter()
            .filter_map(|c| posters.get(&c.id).cloned())
            .collect(),
    );
    Ok(Reply::Series { series: cards })
}

/// The airing rail: what is still releasing and has something on disk,
/// freshest first. A series the core counts as watched-out is out of here
/// even while a stale status still says releasing — the tracker is the
/// firmer answer, and the airing refresh's status write clears the rest.
/// A dropped series is out too: its next episode is nothing to wait for.
pub fn list_airing(core: &Core, offset: u64, limit: u64) -> Result<Reply, CoreError> {
    let series = airing_rail(load_cards(core)?)
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    Ok(Reply::Series { series })
}

fn airing_rail(mut cards: Vec<SeriesCard>) -> Vec<SeriesCard> {
    cards.retain(|c| {
        c.status == Some(AiringStatus::Releasing)
            && c.episodes_on_disk > 0
            && !c.hidden
            && !c.missing
            && !cards::watched_out(c)
            && c.list_status != Some(ListStatus::Dropped)
    });
    cards.sort_by_key(|c| std::cmp::Reverse(c.latest_activity_at));
    cards
}

/// The library page in one read: `list_series`'s answer, the whole airing
/// rail and every tab's count, all off one snapshot rather than the four or
/// five whole-library reads the page used to make per reload.
pub fn list_library(
    core: &Core,
    tab: Tab,
    query: &str,
    sort_key: Sort,
    direction: Direction,
) -> Result<Reply, CoreError> {
    let show_watched = core
        .store
        .read(|c| prefs::load_preferences(c).map(|p| p.library_show_watched))?;
    let Loaded {
        cards: all,
        gaps,
        posters,
    } = load_all(core)?;
    let count = |t: Tab| all.iter().filter(|c| in_tab(c, t, show_watched)).count() as u64;
    let counts = TabCounts {
        all: count(Tab::All),
        series: count(Tab::Series),
        movies: count(Tab::Movies),
        watched: count(Tab::Watched),
        hidden: count(Tab::Hidden),
    };
    let mut series: Vec<SeriesCard> = all
        .iter()
        .filter(|c| in_tab(c, tab, show_watched) && sort::matches_query(c, query))
        .cloned()
        .collect();
    sort::sort_cards(&mut series, sort_key, direction);
    let airing = airing_rail(all);
    let mut used: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for c in series.iter().chain(airing.iter()) {
        if seen.insert(c.id)
            && let Some(url) = posters.get(&c.id)
        {
            used.push(url.clone());
        }
    }
    after_read(core, &gaps, used);
    Ok(Reply::Library {
        series,
        airing,
        counts,
    })
}

pub fn get_series(core: &Core, series: u64) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let now = time::now();
    let (detail, gaps, poster) = core.store.read(|c| {
        let snap = Snapshot::load(c, &images_dir, now, Some(&[series]))?;
        let poster = snap
            .row(series)
            .filter(|r| r.poster_path.is_some())
            .and_then(|r| r.media.as_ref()?.cover_url.clone());
        Ok((snap.detail(c, series)?, snap.gaps.clone(), poster))
    })?;
    match detail {
        Some(mut detail) => {
            crate::downloads::overlay_detail(core, &mut detail);
            after_read(core, &gaps, poster.into_iter().collect());
            Ok(Reply::SeriesDetail {
                detail: Box::new(detail),
            })
        }
        None => Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        }),
    }
}

pub fn set_hidden(core: &Core, series: u64, hidden: bool) -> Result<Reply, CoreError> {
    let changed = core.store.write(move |c| {
        Ok(c.execute(
            "UPDATE series SET hidden = ?1 WHERE id = ?2",
            params![i64::from(hidden), series as i64],
        )?)
    })?;
    if changed == 0 {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        });
    }
    let images_dir = core.paths.images_dir();
    let cards = core
        .store
        .read(|c| cards::cards_for(c, &images_dir, &[series]))?;
    let title = cards.first().map_or_else(String::new, |c| c.title.clone());
    let what = if hidden { "hidden" } else { "shown" };
    core.bus.debug(
        Stage::Library,
        format!("{title} {what}"),
        EventBody::SeriesChanged { series: cards },
    );
    Ok(Reply::Ok)
}

/// The metadata table. The counts are over the visible set, so they answer
/// "how many of what you can see", and only this call reveals hidden rows.
pub fn list_metadata(
    core: &Core,
    filter: MetadataFilter,
    query: &str,
    reveal_hidden: bool,
) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let now = time::now();
    let visible: Vec<MetadataRow> = core.store.read(|c| {
        let snap = Snapshot::load(c, &images_dir, now, None)?;
        Ok(snap
            .series
            .iter()
            .filter(|r| reveal_hidden || !r.hidden)
            .filter_map(|r| snap.metadata_row(r.id))
            .collect())
    })?;

    let counts = FilterCounts {
        all: visible.len() as u64,
        series: visible.iter().filter(|r| !is_movie(&r.series)).count() as u64,
        movies: visible.iter().filter(|r| is_movie(&r.series)).count() as u64,
        missing_files: visible.iter().filter(|r| r.series.missing).count() as u64,
    };

    let mut rows: Vec<MetadataRow> = visible
        .into_iter()
        .filter(|r| match filter {
            MetadataFilter::All => true,
            MetadataFilter::Series => !is_movie(&r.series),
            MetadataFilter::Movies => is_movie(&r.series),
            MetadataFilter::MissingFiles => r.series.missing,
        })
        .filter(|r| sort::matches_query(&r.series, query))
        .collect();
    rows.sort_by_key(|r| r.series.title.to_lowercase());
    Ok(Reply::Metadata { rows, counts })
}

/// A path back to what the library calls it: a series first, since a film's
/// path is both, then a file.
pub fn lookup(core: &Core, path: &str) -> Result<Reply, CoreError> {
    core.store.read(|c| {
        if let Some(series) = series_at(c, path)? {
            return Ok(Reply::Lookup {
                series: Some(series),
                file: None,
            });
        }
        match file_at(c, path)? {
            Some((file, series)) => Ok(Reply::Lookup {
                series: Some(series),
                file: Some(file),
            }),
            None => Ok(Reply::Lookup {
                series: None,
                file: None,
            }),
        }
    })
}

fn series_at(conn: &Connection, path: &str) -> Result<Option<u64>, CoreError> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM series WHERE path = ?1 ORDER BY id LIMIT 1",
            params![path],
            |r| r.get(0),
        )
        .optional()?;
    Ok(id.map(|v| v as u64))
}

fn file_at(conn: &Connection, path: &str) -> Result<Option<(u64, u64)>, CoreError> {
    let row: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, series_id FROM files WHERE path = ?1",
            params![path],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.map(|(file, series)| (file as u64, series as u64)))
}
