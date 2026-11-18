//! Stats (#216): what a year of watching adds up to, read off the watch log
//! (#593: every finish of an episode, with when) and the records behind each
//! series (length, genres, studio, season). A rewatch adds to the year it
//! happened in; progress set by hand adds nothing.
//!
//! Pure over one snapshot and the log: no request, no write. An episode's minutes are its
//! record's length, or a plain default when the record names none, so a
//! number is always there; the page says hours, rounded.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::contract::*;
use crate::foundation::titles;
use crate::library::history::How;
use crate::library::snapshot::{SeriesRow, Snapshot};

const SHOW_MINUTES: u32 = 24;
const FILM_MINUTES: u32 = 100;
const TOP: usize = 8;

/// The minutes one episode of `row` counts for.
fn minutes_of(row: &SeriesRow) -> u32 {
    row.media
        .as_ref()
        .and_then(|m| m.duration)
        .filter(|d| *d > 0)
        .unwrap_or(match row.kind {
            SeriesKind::Movie => FILM_MINUTES,
            _ => SHOW_MINUTES,
        })
}

/// Whether a series is finished: the library's own rule (the provider says
/// Finished and every episode of its own count is watched, never a guessed
/// or hand-set count), or the same rule over the history alone, so a show
/// whose files are gone still counts. Only plain episode keys count there:
/// an episode 0, a half episode, a special or an extra finishes nothing.
fn finished(snap: &Snapshot, row: &SeriesRow, done: &[(String, i64)]) -> bool {
    if snap.finished_watched(row, &snap.watched_of(row)) {
        return true;
    }
    let Some(media) = row.media.as_ref() else {
        return false;
    };
    if media
        .status
        .as_deref()
        .and_then(AiringStatus::from_provider)
        != Some(AiringStatus::Finished)
    {
        return false;
    }
    if row.kind == SeriesKind::Movie {
        return !done.is_empty();
    }
    let Some(total) = media.episodes.filter(|n| *n > 0) else {
        return false;
    };
    let keys: HashSet<u32> = done
        .iter()
        .filter_map(|(key, _)| key.parse::<u32>().ok())
        .collect();
    (1..=total).all(|n| keys.contains(&n))
}

/// Every finish Stats counts, by series: what the watch log holds but the
/// progress set by hand.
pub(crate) type Finishes = HashMap<u64, Vec<(String, i64)>>;

/// Reads the counted finishes off the watch log.
pub(crate) fn finishes(conn: &Connection) -> Result<Finishes, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT series_id, episode_key, at FROM watch_log WHERE profile_id = 1 AND how <> ?1",
    )?;
    let rows = stmt.query_map([How::Marked.as_str()], |r| {
        Ok((
            r.get::<_, i64>(0)? as u64,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    let mut out = Finishes::new();
    for row in rows {
        let (series, key, at) = row?;
        out.entry(series).or_default().push((key, at));
    }
    Ok(out)
}

/// When a finished series was finished: the moment the last of its watched
/// episodes was first finished, so watching one again never moves it. None
/// when any of them has no counted finish: a series finished partly by hand
/// was finished by hand, and Stats counts no finish for it.
fn finished_at(done: &[(String, i64)], log: &[(String, i64)]) -> Option<i64> {
    let mut first: HashMap<&str, i64> = HashMap::new();
    for (key, at) in log {
        first
            .entry(key.as_str())
            .and_modify(|t| *t = (*t).min(*at))
            .or_insert(*at);
    }
    done.iter()
        .map(|(key, _)| first.get(key.as_str()).copied())
        .collect::<Option<Vec<i64>>>()?
        .into_iter()
        .max()
}

/// The calendar year and month (1 to 12) of a moment, `offset` seconds east of UTC.
fn year_month(at: i64, offset: i64) -> (i32, u32) {
    ::time::OffsetDateTime::from_unix_timestamp(at + offset)
        .map_or((1970, 1), |t| (t.year(), u32::from(u8::from(t.month()))))
}

#[derive(Default)]
struct Tally {
    episodes: u32,
    series: HashSet<u64>,
}

/// Adds a series' episodes under `name`.
fn tally(map: &mut HashMap<String, Tally>, name: String, series: u64, episodes: u32) {
    let t = map.entry(name).or_default();
    t.episodes += episodes;
    t.series.insert(series);
}

fn top(map: HashMap<String, Tally>) -> Vec<StatCount> {
    let mut out: Vec<StatCount> = map
        .into_iter()
        .map(|(name, t)| StatCount {
            name,
            episodes: t.episodes,
            series: t.series.len() as u32,
        })
        .collect();
    out.sort_by(|a, b| {
        b.episodes
            .cmp(&a.episodes)
            .then_with(|| b.series.cmp(&a.series))
            .then_with(|| a.name.cmp(&b.name))
    });
    out.truncate(TOP);
    out
}

/// The year's stats. `year` None is the year `now` falls in. `month_starts`,
/// when the shell sends the thirteen local midnights that open each month of
/// `year` and the next, places the year's moments in their months across a
/// daylight saving change; anything else is placed by `offset`.
pub fn compute(
    snap: &Snapshot,
    log: &Finishes,
    year: Option<i32>,
    offset: i64,
    month_starts: &[i64],
    now: i64,
) -> Stats {
    let year = year.unwrap_or_else(|| year_month(now, offset).0);
    let place = |at: i64| -> (i32, u32) {
        if month_starts.len() == 13 && at >= month_starts[0] && at < month_starts[12] {
            let m = month_starts.partition_point(|start| *start <= at) as u32;
            return (year, m);
        }
        year_month(at, offset)
    };
    let mut years: Vec<i32> = Vec::new();
    let mut months: Vec<StatMonth> = (1..=12)
        .map(|month| StatMonth {
            month,
            episodes: 0,
            minutes: 0,
            series_completed: 0,
        })
        .collect();
    let (mut genres, mut studios, mut seasons) = (HashMap::new(), HashMap::new(), HashMap::new());
    let mut per_series: Vec<StatSeries> = Vec::new();
    for row in &snap.series {
        let done = snap.completed.get(&row.id).map_or(&[][..], Vec::as_slice);
        let finishes = log.get(&row.id).map_or(&[][..], Vec::as_slice);
        if finishes.is_empty() {
            continue;
        }
        let minutes = minutes_of(row);
        let mut episodes = 0u32;
        for (_, at) in finishes {
            let (y, m) = place(*at);
            if !years.contains(&y) {
                years.push(y);
            }
            if y != year {
                continue;
            }
            episodes += 1;
            let slot = &mut months[(m - 1) as usize];
            slot.episodes += 1;
            slot.minutes += u64::from(minutes);
        }
        // Finished by the library's own rule (the provider says Finished and
        // every episode of its count is watched), in the month the last of
        // its episodes was first watched.
        if finished(snap, row, done)
            && let Some(last) = finished_at(done, finishes)
        {
            let (y, m) = place(last);
            if y == year {
                months[(m - 1) as usize].series_completed += 1;
            }
        }
        if episodes == 0 {
            continue;
        }
        let media = row.media.as_ref();
        for g in media.iter().flat_map(|m| &m.genres) {
            tally(&mut genres, g.clone(), row.id, episodes);
        }
        if let Some(studio) = media.and_then(|m| m.studio.as_ref()).filter(|s| !s.is_empty()) {
            tally(&mut studios, studio.clone(), row.id, episodes);
        }
        if let Some((season, y)) = media.and_then(|m| m.season.as_deref().zip(m.year)) {
            let name = format!("{} {y}", title_case(season));
            tally(&mut seasons, name, row.id, episodes);
        }
        per_series.push(StatSeries {
            series: row.id,
            title: titles::resolve(
                snap.lang,
                media.and_then(|m| m.title_romaji.as_deref()),
                media.and_then(|m| m.title_english.as_deref()),
                media.and_then(|m| m.title_native.as_deref()),
                &row.folder_name,
            ),
            poster: row.poster_path.clone(),
            episodes,
            minutes: u64::from(episodes) * u64::from(minutes),
        });
    }
    years.sort_unstable_by(|a, b| b.cmp(a));
    per_series.sort_by(|a, b| {
        b.minutes
            .cmp(&a.minutes)
            .then_with(|| a.title.cmp(&b.title))
    });
    per_series.truncate(5);
    Stats {
        year,
        years,
        minutes: months.iter().map(|m| m.minutes).sum(),
        episodes: months.iter().map(|m| m.episodes).sum(),
        series_completed: months.iter().map(|m| m.series_completed).sum(),
        months,
        genres: top(genres),
        studios: top(studios),
        seasons: top(seasons),
        top_series: per_series,
    }
}

fn title_case(season: &str) -> String {
    let lower = season.to_lowercase();
    let mut c = lower.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// `GetStats`: the year read off one snapshot and the watch log.
pub(crate) fn get_call(
    services: &crate::foundation::services::Services,
    year: Option<i32>,
    utc_offset: i32,
    month_starts: &[i64],
) -> Result<Reply, CoreError> {
    let images_dir = services.paths.images_dir();
    let now = crate::time::now();
    let stats = services.store.read(|c| {
        let snap = Snapshot::load(c, &images_dir, now, None)?;
        let log = finishes(c)?;
        Ok(compute(
            &snap,
            &log,
            year,
            i64::from(utc_offset),
            month_starts,
            crate::time::to_secs(now),
        ))
    })?;
    Ok(Reply::Stats { stats })
}

#[cfg(test)]
mod tests {
    use super::year_month;

    #[test]
    fn a_moment_falls_in_its_local_month() {
        // 2026-01-01 00:30 UTC: an hour west of UTC it is still December
        let at = 1_767_227_400;
        assert_eq!(year_month(at, 0), (2026, 1));
        assert_eq!(year_month(at, -3600), (2025, 12));
        assert_eq!(year_month(at, 9 * 3600), (2026, 1));
        // A leap day, and a moment before 1970
        assert_eq!(year_month(1_709_164_800, 0), (2024, 2));
        assert_eq!(year_month(-1, 0), (1969, 12));
    }
}
