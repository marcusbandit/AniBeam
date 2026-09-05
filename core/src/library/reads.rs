//! The library's read side: the six calls a shell makes to draw the home
//! grid, the airing rail, a series page, the metadata table, and to turn a
//! path back into an id. Every one of them runs on the reader connection,
//! so a list never waits behind a long write.

use rusqlite::{params, Connection, OptionalExtension};

use crate::contract::*;
use crate::core::Core;
use crate::library::cards::{self, Snapshot};
use crate::library::sort;
use crate::time;

/// A film to the tabs: the folder was classified as one, or the match says
/// the title is one. Carried from `MetadataTab.tsx`'s `isMovie`.
fn is_movie(c: &SeriesCard) -> bool {
    c.kind == SeriesKind::Movie || c.format.as_deref() == Some("MOVIE")
}

/// Hidden series live in their own tab and are never mixed into the others;
/// a missing series appears under no tab at all.
fn in_tab(c: &SeriesCard, tab: Tab) -> bool {
    if c.missing {
        return false;
    }
    match tab {
        Tab::Hidden => c.hidden,
        Tab::All => !c.hidden,
        Tab::Series => !c.hidden && !is_movie(c),
        Tab::Movies => !c.hidden && is_movie(c),
    }
}

fn load_cards(core: &Core) -> Result<Vec<SeriesCard>, CoreError> {
    let images = core.paths.images_dir();
    let now = time::now();
    core.store.read(|c| Ok(Snapshot::load(c, &images, now, None)?.cards()))
}

/// The home grid. `reveal_hidden` is the shell's own tab visibility: the
/// Hidden tab always lists what it holds.
pub fn list_series(core: &Core, tab: Tab, query: &str, sort_key: Sort, direction: Direction) -> Result<Reply, CoreError> {
    let mut series = load_cards(core)?;
    series.retain(|c| in_tab(c, tab) && sort::matches_query(c, query));
    sort::sort_cards(&mut series, sort_key, direction);
    Ok(Reply::Series { series })
}

/// The airing rail: what is still releasing and has something on disk,
/// freshest first.
pub fn list_airing(core: &Core, offset: u64, limit: u64) -> Result<Reply, CoreError> {
    let mut cards = load_cards(core)?;
    cards.retain(|c| c.status == Some(AiringStatus::Releasing) && c.episodes_on_disk > 0 && !c.hidden && !c.missing);
    cards.sort_by_key(|c| std::cmp::Reverse(c.latest_activity_at));
    let series = cards.into_iter().skip(offset as usize).take(limit as usize).collect();
    Ok(Reply::Series { series })
}

/// One series page, loaded through a snapshot scoped to that series alone.
pub fn get_series(core: &Core, series: u64) -> Result<Reply, CoreError> {
    let images = core.paths.images_dir();
    let now = time::now();
    let detail = core.store.read(|c| {
        let snap = Snapshot::load(c, &images, now, Some(&[series]))?;
        snap.detail(c, series)
    })?;
    match detail {
        Some(detail) => Ok(Reply::SeriesDetail { detail: Box::new(detail) }),
        None => Err(CoreError::NotFound { what: Entity::Series, id: series }),
    }
}

pub fn set_hidden(core: &Core, series: u64, hidden: bool) -> Result<Reply, CoreError> {
    let changed = core.store.write(move |c| {
        Ok(c.execute("UPDATE series SET hidden = ?1 WHERE id = ?2", params![i64::from(hidden), series as i64])?)
    })?;
    if changed == 0 {
        return Err(CoreError::NotFound { what: Entity::Series, id: series });
    }
    let images = core.paths.images_dir();
    let cards = core.store.read(|c| cards::cards_for(c, &images, &[series]))?;
    let title = cards.first().map_or_else(String::new, |c| c.title.clone());
    let what = if hidden { "hidden" } else { "shown" };
    core.bus.debug(Stage::Library, format!("{title} {what}"), EventBody::SeriesChanged { series: cards });
    Ok(Reply::Ok)
}

/// The metadata table. The counts are over the visible set, so they answer
/// "how many of what you can see", and only this call reveals hidden rows.
pub fn list_metadata(core: &Core, filter: MetadataFilter, query: &str, reveal_hidden: bool) -> Result<Reply, CoreError> {
    let images = core.paths.images_dir();
    let now = time::now();
    let visible: Vec<MetadataRow> = core.store.read(|c| {
        let snap = Snapshot::load(c, &images, now, None)?;
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
            return Ok(Reply::Lookup { series: Some(series), file: None });
        }
        match file_at(c, path)? {
            Some((file, series)) => Ok(Reply::Lookup { series: Some(series), file: Some(file) }),
            None => Ok(Reply::Lookup { series: None, file: None }),
        }
    })
}

fn series_at(conn: &Connection, path: &str) -> Result<Option<u64>, CoreError> {
    let id: Option<i64> = conn
        .query_row("SELECT id FROM series WHERE path = ?1 ORDER BY id LIMIT 1", params![path], |r| r.get(0))
        .optional()?;
    Ok(id.map(|v| v as u64))
}

fn file_at(conn: &Connection, path: &str) -> Result<Option<(u64, u64)>, CoreError> {
    let row: Option<(i64, i64)> = conn
        .query_row("SELECT id, series_id FROM files WHERE path = ?1", params![path], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    Ok(row.map(|(file, series)| (file as u64, series as u64)))
}
