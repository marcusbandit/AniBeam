//! Stats (#216): what a year of watching adds up to, read off the history the
//! core already keeps (`completed`, one row per episode finished, with when)
//! and the records behind each series (length, genres, studio, season).
//!
//! Pure over one snapshot: no request, no write. An episode's minutes are its
//! record's length, or a plain default when the record names none, so a
//! number is always there; the page says hours, rounded.

use std::collections::{HashMap, HashSet};

use crate::contract::*;
use crate::library::snapshot::{SeriesRow, Snapshot};
use crate::library::titles;

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

/// The calendar year and month (1 to 12) of a moment, `offset` seconds east of UTC.
pub fn year_month(at: i64, offset: i64) -> (i32, u32) {
    let days = (at + offset).div_euclid(86_400);
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
    (year, month)
}

#[derive(Default)]
struct Tally {
    episodes: u32,
    series: std::collections::HashSet<u64>,
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
        let Some(done) = snap.completed.get(&row.id) else {
            continue;
        };
        let minutes = minutes_of(row);
        let mut episodes = 0u32;
        for (_, at) in done {
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
        // every episode of its count is watched), in the month the last
        // finished episode was watched.
        if finished(snap, row, done)
            && let Some(last) = done.iter().map(|(_, at)| *at).max()
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
        for g in media.map(|m| m.genres.clone()).unwrap_or_default() {
            let t: &mut Tally = genres.entry(g).or_default();
            t.episodes += episodes;
            t.series.insert(row.id);
        }
        if let Some(studio) = media
            .and_then(|m| m.studio.clone())
            .filter(|s| !s.is_empty())
        {
            let t: &mut Tally = studios.entry(studio).or_default();
            t.episodes += episodes;
            t.series.insert(row.id);
        }
        if let (Some(season), Some(y)) = (
            media.and_then(|m| m.season.clone()),
            media.and_then(|m| m.year),
        ) {
            let t: &mut Tally = seasons
                .entry(format!("{} {y}", title_case(&season)))
                .or_default();
            t.episodes += episodes;
            t.series.insert(row.id);
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
