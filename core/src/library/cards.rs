//! A card, and the pure rules a card is drawn from. The rules at the top
//! are ports of Electron's `airingUtils.ts` and `SeriesDetailPage.tsx`; the
//! assembly below turns a `Snapshot`'s rows into the contract's records,
//! and every job builds its `SeriesChanged` batch through `cards_for`.

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use rusqlite::Connection;

use crate::contract::*;
use crate::library::snapshot::{AiringRow, FileRow, NextEpisode, SeriesRow, Snapshot};
use crate::library::{labels, titles};
use crate::time;

// ---------------------------------------------------------------------------
// The pure rules
// ---------------------------------------------------------------------------

/// The fraction of the strip reserved for the dark "total unknown" cap, so a
/// bar without a published episode count can never read as complete.
/// Electron's `UNKNOWN_TAIL_PCT = 15`.
pub const UNKNOWN_TAIL: f64 = 0.15;

/// The most cubes a row draws: past this a count says more than a row.
pub const CUBES_MAX: u32 = 400;

/// The inputs of one series' cube row, gathered in `card_of`.
pub struct CubeFacts<'a> {
    pub total: Option<u32>,
    pub latest_aired: u32,
    pub next_scheduled: Option<u32>,
    pub status: Option<AiringStatus>,
    /// Tracker progress: episodes 1 to this are watched and synced.
    pub tracked: Option<u32>,
    /// Distinct whole episode numbers on disk.
    pub on_disk: &'a BTreeSet<u32>,
    /// Episode numbers finished here, in the `completed` table.
    pub watched_here: &'a BTreeSet<u32>,
}

/// One state code per episode from 1 to the total, the cube row every
/// surface draws (the shell's `Tokens.cubeStates` holds the colours). A new
/// state is a new code here and a new entry there. Precedence, high first:
///   w  watched and tracked online (tracker progress covers it)
///   l  watched here, not tracked
///   d  on disk, untouched
///   m  missing: aired, not on disk, and a later episode is
///   g  not downloaded: aired, not on disk, nothing later is
///   f  not aired yet
/// Downloading (t) and error (e) are client states; the Downloads page lays
/// them over this row, the core does not know them per card.
pub fn cubes(f: &CubeFacts) -> String {
    let last_disk = f.on_disk.iter().next_back().copied().unwrap_or(0);
    let tracked = f.tracked.unwrap_or(0);
    let total = match f.total.filter(|t| *t > 0) {
        Some(t) => t,
        None => f.latest_aired.max(last_disk).max(tracked),
    }
    .min(CUBES_MAX);
    if total == 0 {
        return String::new();
    }
    let still_airing = f.next_scheduled.is_some()
        || matches!(
            f.status,
            Some(AiringStatus::Releasing | AiringStatus::NotYetReleased | AiringStatus::Hiatus)
        );
    let aired = if still_airing {
        f.latest_aired
            .max(f.next_scheduled.map_or(0, |n| n.saturating_sub(1)))
            .max(last_disk)
            .max(tracked)
    } else {
        total
    };
    (1..=total)
        .map(|n| {
            if n <= tracked {
                'w'
            } else if f.watched_here.contains(&n) {
                'l'
            } else if f.on_disk.contains(&n) {
                'd'
            } else if n <= aired && n < last_disk {
                'm'
            } else if n <= aired {
                'g'
            } else {
                'f'
            }
        })
        .collect()
}

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;

/// The next episode and when it should air, guessed from the episodes
/// already out, for an airing series whose record names no date ahead (a
/// real schedule always wins; `card_of` asks only without one). Needs one
/// dated episode out. The cadence is the gap between the first two
/// consecutive dated episodes in whole weeks, at least one, else one week;
/// a later break never changes it. The guess runs on from the newest dated
/// episode, one cadence per episode, starting past `released` (the furthest
/// episode known out, a file on disk counts). A guess a full day in the
/// past with nothing released moves a week later, as many weeks as it takes.
pub fn predict_next(
    airing: &[AiringRow],
    released: u32,
    total: Option<u32>,
    status: Option<AiringStatus>,
    now: i64,
) -> Option<(u32, i64)> {
    if status != Some(AiringStatus::Releasing) {
        return None;
    }
    let mut aired: Vec<(u32, i64)> = airing
        .iter()
        .filter_map(|a| a.aired_at.filter(|at| *at <= now).map(|at| (a.number, at)))
        .collect();
    aired.sort_unstable();
    let &(last, last_at) = aired.last()?;
    let next = released.max(last) + 1;
    if total.is_some_and(|t| t > 0 && next > t) {
        return None;
    }
    let cadence = aired
        .windows(2)
        .find(|w| w[1].0 == w[0].0 + 1)
        .map_or(WEEK, |w| {
            let weeks = ((w[1].1 - w[0].1) as f64 / WEEK as f64).round() as i64;
            weeks.max(1) * WEEK
        });
    let mut at = last_at + i64::from(next - last) * cadence;
    if at + DAY <= now {
        at += ((now - at - DAY) / WEEK + 1) * WEEK;
    }
    Some((next, at))
}

/// The three segments of the strip on a card, as fractions of 0 to 1. The
/// port of `computeCardProgress`: watched, aired but unwatched (only while
/// the show is still releasing), and the unknown-total cap.
pub fn strip(
    watched: Option<u32>,
    total: Option<u32>,
    latest_aired: u32,
    next_scheduled: Option<u32>,
    latest_downloaded: u32,
    status: Option<AiringStatus>,
) -> Strip {
    let mut released = latest_aired
        .max(next_scheduled.map_or(0, |n| n.saturating_sub(1)))
        .max(latest_downloaded);
    let known_total = total.filter(|t| *t > 0);
    let finished = status == Some(AiringStatus::Finished);
    if let (true, Some(t)) = (finished, known_total) {
        released = t;
    }
    let fully_released = finished || known_total.is_some_and(|t| released >= t);
    // No tracker entry means there is no watch progress to be behind on, so
    // an untracked card never paints the aired-unwatched segment.
    let show_behind = watched.is_some() && !fully_released;

    if let Some(t) = known_total {
        let t = f64::from(t);
        let pct = |n: u32| (f64::from(n) / t).clamp(0.0, 1.0);
        return Strip {
            watched: pct(watched.unwrap_or(0)),
            aired_unwatched: if show_behind { pct(released) } else { 0.0 },
            unknown: 0.0,
        };
    }

    let extent = released.max(watched.unwrap_or(0));
    if extent == 0 {
        return Strip {
            watched: 0.0,
            aired_unwatched: 0.0,
            unknown: 0.0,
        };
    }
    let usable = 1.0 - UNKNOWN_TAIL;
    let pct = |n: u32| (f64::from(n) / f64::from(extent) * usable).clamp(0.0, usable);
    Strip {
        watched: pct(watched.unwrap_or(0)),
        aired_unwatched: if show_behind { pct(released) } else { 0.0 },
        unknown: UNKNOWN_TAIL,
    }
}

/// The port of `classifyWatchProgress`. Reachable is the highest episode
/// that could be played today: the later of the latest aired one and the
/// latest one on disk, because plenty of shows carry no air dates at all.
pub fn watched_state(
    watched: Option<u32>,
    total: Option<u32>,
    latest_aired: u32,
    latest_downloaded: u32,
) -> WatchedState {
    let Some(w) = watched else {
        return WatchedState::Unknown;
    };
    let known_total = total.filter(|t| *t > 0);
    if known_total.is_some_and(|t| w >= t) {
        return WatchedState::CaughtUp;
    }
    let reachable = latest_aired.max(latest_downloaded);
    if reachable > 0 && w < reachable {
        return WatchedState::Behind;
    }
    if known_total.is_none() {
        return WatchedState::Unknown;
    }
    WatchedState::CaughtUp
}

/// The known total, else the aired estimate (the later of aired and
/// watched) marked as one. The port of `formatWatchedLabel`'s denominator.
pub fn total_with_estimate(
    total: Option<u32>,
    latest_aired: u32,
    watched: Option<u32>,
) -> (Option<u32>, bool) {
    if let Some(t) = total.filter(|t| *t > 0) {
        return (Some(t), false);
    }
    if latest_aired > 0 {
        return (Some(latest_aired.max(watched.unwrap_or(0))), true);
    }
    (None, false)
}

/// Done with it: nothing left to see and nothing more coming, the state a
/// non-rewatcher files a series away under. The tracker's own COMPLETED
/// outranks everything: the user said done, and a stale stored status that
/// still says releasing must not second-guess them (the airing refresh's
/// status write may not have landed yet), so a completed series leaves the
/// grid and the airing rail the same day they mark it. Without the
/// tracker's word, a still-releasing series is never done with: caught up
/// is current, not finished, however little there is left to wait for.
/// Otherwise the watched count has to reach a published total. An
/// estimated total never counts: it is the later of aired and watched, so
/// it only says how far the user has got, and reaching the newest episode
/// on hand is not the same as the show being over. A film is the one
/// exception that needs no total, since seeing it once is all there is.
pub fn watched_out(c: &SeriesCard) -> bool {
    if c.list_status == Some(ListStatus::Completed) {
        return true;
    }
    if c.status == Some(AiringStatus::Releasing) {
        return false;
    }
    let Some(w) = c.watched.filter(|w| *w > 0) else {
        return false;
    };
    if is_film(c) {
        return true;
    }
    if c.total_is_estimate {
        return false;
    }
    c.total_episodes.is_some_and(|t| w >= t)
}

/// A film, by our own kind or the provider's format.
pub fn is_film(c: &SeriesCard) -> bool {
    c.kind == SeriesKind::Movie || c.format.as_deref() == Some("MOVIE")
}

/// The season label the library groups by: "Fall 2025". `word` is the
/// provider's own season (AniList's WINTER, SPRING, SUMMER, FALL, any
/// case), title-cased and joined with `year`, else with the start date's
/// year. Without a word the season comes from the start date's month
/// (Jan to Mar Winter, Apr to Jun Spring, Jul to Sep Summer, Oct to Dec
/// Fall) and the year from the same date. A year alone is not a season,
/// and a word with no year anywhere is not one either.
pub fn season_label(
    word: Option<&str>,
    start_date: Option<&str>,
    year: Option<u32>,
) -> Option<String> {
    let date_year = start_date
        .and_then(|d| d.get(..4))
        .and_then(|y| y.parse::<u32>().ok())
        .filter(|y| *y > 0);
    let date_month = start_date
        .and_then(|d| d.get(5..7))
        .and_then(|m| m.parse::<u32>().ok())
        .filter(|m| (1..=12).contains(m));
    let from_word = word.map(str::trim).filter(|w| !w.is_empty()).and_then(|w| {
        let mut chars = w.chars();
        let first = chars.next()?.to_uppercase().collect::<String>();
        let rest = chars.as_str().to_lowercase();
        Some(format!("{first}{rest}"))
    });
    let (name, label_year) = match from_word {
        Some(name) => (name, year.or(date_year)?),
        None => {
            let name = match date_month? {
                1..=3 => "Winter",
                4..=6 => "Spring",
                7..=9 => "Summer",
                _ => "Fall",
            };
            (name.to_string(), date_year?)
        }
    };
    Some(format!("{name} {label_year}"))
}

/// `episodes` is (file id, season, number) in season then number order.
/// `last_completed` is the most recently completed episode's season and
/// number. A tracker's progress counts episodes of the first season on
/// disk, since AniList numbers one season per entry and a folder matched to
/// a second season's entry holds that season's files: progress 2 against
/// S02E01 to S02E03 is S02E02, not S01E02. With something watched, the episode after it in
/// (season, number) order: the next number of the same season, else the
/// first episode past it, which is how finishing S01E10 lands on S02E01.
/// With nothing watched, the first one on disk, above zero when a tracker
/// entry says the series has been started at all.
pub fn next_up(
    episodes: &[(u64, u32, f64)],
    last_completed: Option<(u32, f64)>,
    tracker_progress: Option<u32>,
) -> Option<u64> {
    let first_season = episodes
        .iter()
        .find(|(_, s, n)| *s > 0 && *n > 0.0)
        .map_or(1, |(_, s, _)| *s);
    let progress = tracker_progress
        .filter(|p| *p > 0)
        .map(|p| (first_season, f64::from(p)));
    let last = match (last_completed.filter(|(_, n)| *n > 0.0), progress) {
        (Some(a), Some(b)) => Some(if after(a, b) { a } else { b }),
        (a, b) => a.or(b),
    };
    if let Some((season, number)) = last {
        return episodes
            .iter()
            .find(|(_, s, n)| *s == season && *n == number + 1.0)
            .or_else(|| {
                episodes
                    .iter()
                    .find(|(_, s, n)| after((*s, *n), (season, number)))
            })
            .map(|(f, _, _)| *f);
    }
    if tracker_progress.is_some() {
        return episodes
            .iter()
            .find(|(_, _, n)| *n > 0.0)
            .map(|(f, _, _)| *f);
    }
    episodes.first().map(|(f, _, _)| *f)
}

/// Whether `a` comes after `b` in (season, number) order.
fn after(a: (u32, f64), b: (u32, f64)) -> bool {
    a.0 > b.0 || (a.0 == b.0 && a.1 > b.1)
}
/// The highest episode that has aired, and the earliest one still to come.
/// An AniList schedule carries no season.
pub(super) fn aired_and_scheduled(airing: &[AiringRow], now: i64) -> (u32, Option<NextEpisode>) {
    let mut latest_aired = 0;
    let mut next_scheduled: Option<NextEpisode> = None;
    for a in airing {
        let Some(at) = a.aired_at else { continue };
        if at <= now {
            latest_aired = latest_aired.max(a.number);
        } else if next_scheduled.is_none_or(|n| at < n.at) {
            next_scheduled = Some(NextEpisode {
                season: None,
                number: a.number,
                at,
            });
        }
    }
    (latest_aired, next_scheduled)
}

impl Snapshot {
    pub fn card(&self, id: u64) -> Option<SeriesCard> {
        self.row(id).map(|row| self.card_of(row))
    }

    pub fn cards(&self) -> Vec<SeriesCard> {
        self.series.iter().map(|row| self.card_of(row)).collect()
    }

    pub(super) fn card_of(&self, row: &SeriesRow) -> SeriesCard {
        let now = time::to_secs(self.now);
        let files = self.files_of(row.id);
        let episodes = self.episodes_on_disk(row.id);
        let airing = self.airing_of(row);
        let media = row.media.as_ref();
        let imdb = self.imdb.get(&row.id);

        let latest_downloaded = episodes.last().map_or(0.0, |f| f.number).max(0.0) as u32;
        let (latest_aired, next_scheduled) = self.aired_and_next(row, now);
        let status = media
            .and_then(|m| m.status.as_deref())
            .and_then(AiringStatus::from_provider);
        let total = media.and_then(|m| m.episodes);
        let entry = self.tracker(row);
        let watched = entry.map(|e| e.progress);
        let (total_episodes, total_is_estimate) = total_with_estimate(total, latest_aired, watched);

        let code = match row.kind {
            SeriesKind::Movie => Some("Movie".to_string()),
            SeriesKind::Show => episodes
                .last()
                .map(|f| labels::episode_code(f.season, f.number)),
        };

        SeriesCard {
            id: row.id,
            kind: row.kind,
            path: row.path.clone(),
            title: self.title_of(row),
            titles: Titles {
                romaji: media.and_then(|m| m.title_romaji.clone()),
                english: media.and_then(|m| m.title_english.clone()),
                native: media.and_then(|m| m.title_native.clone()),
                folder: row.folder_name.clone(),
            },
            poster: row.poster_path.clone(),
            format: media.and_then(|m| m.format.clone()),
            genres: media.map(|m| m.genres.clone()).unwrap_or_default(),
            status,
            hidden: row.hidden,
            missing: row.missing,
            match_info: row.provider.map(|provider| MatchInfo {
                provider,
                anilist_id: row.anilist_id,
                mal_id: row.mal_id,
                tmdb_id: row.tmdb_id,
                tmdb_kind: row.tmdb_kind,
                tvmaze_id: row.tvmaze_id,
                confirmed: row.confirmed,
            }),
            episodes_on_disk: episodes.len() as u64,
            extras_on_disk: (files.len() - episodes.len()) as u64,
            total_episodes,
            total_is_estimate,
            code,
            watched,
            watched_state: watched_state(watched, total, latest_aired, latest_downloaded),
            cubes: if row.kind == SeriesKind::Movie {
                String::new()
            } else {
                let on_disk: BTreeSet<u32> = episodes
                    .iter()
                    .filter(|f| f.number >= 1.0 && f.number.fract() == 0.0)
                    .map(|f| f.number as u32)
                    .collect();
                let done: HashSet<&str> = self
                    .completed
                    .get(&row.id)
                    .map(|rows| rows.iter().map(|(k, _)| k.as_str()).collect())
                    .unwrap_or_default();
                let watched_here: BTreeSet<u32> = episodes
                    .iter()
                    .filter(|f| f.number >= 1.0 && done.contains(f.episode_key.as_str()))
                    .map(|f| f.number as u32)
                    .collect();
                cubes(&CubeFacts {
                    total,
                    latest_aired,
                    next_scheduled: next_scheduled.map(|n| n.number),
                    status,
                    tracked: watched,
                    on_disk: &on_disk,
                    watched_here: &watched_here,
                })
            },
            strip: strip(
                watched,
                total,
                latest_aired,
                next_scheduled.map(|n| n.number),
                latest_downloaded,
                status,
            ),
            community_score: media
                .and_then(|m| m.average_score)
                .map(|s| f64::from(s) / 10.0),
            my_score: entry.and_then(|e| e.score),
            list_status: entry.and_then(|e| e.status),
            next_airing: match next_scheduled {
                Some(n) => Some(Airing {
                    episode: n.number,
                    at: time::from_secs(n.at),
                    season: n.season,
                    predicted: false,
                }),
                None => predict_next(
                    airing,
                    latest_aired.max(latest_downloaded),
                    total,
                    status,
                    now,
                )
                .map(|(episode, at)| Airing {
                    episode,
                    at: time::from_secs(at),
                    season: self.card_season(row),
                    predicted: true,
                }),
            },
            last_viewed_at: self.views.get(&row.id).map(|(_, at)| time::from_secs(*at)),
            added_at: time::from_secs(row.added_at),
            downloaded_at: files.iter().map(|f| f.mtime).max().map(time::from_secs),
            downloading: Vec::new(),
            latest_activity_at: time::from_secs(self.latest_activity(
                row,
                &self.card_episodes(row, &episodes),
                airing,
                files,
                now,
            )),
            franchise: self.franchises.get(&row.id).cloned(),
            year: media.and_then(|m| m.year),
            season: season_label(
                media.and_then(|m| m.season.as_deref()),
                media.and_then(|m| m.start_date.as_deref()),
                media.and_then(|m| m.year),
            ),
            imdb_id: imdb.map(|p| format!("tt{:07}", p.id)),
            imdb_score: imdb.and_then(|p| p.rating).map(|t| f64::from(t) / 10.0),
            imdb_votes: imdb.and_then(|p| p.votes),
        }
    }

    fn title_of(&self, row: &SeriesRow) -> String {
        let media = row.media.as_ref();
        titles::resolve(
            self.lang,
            media.and_then(|m| m.title_romaji.as_deref()),
            media.and_then(|m| m.title_english.as_deref()),
            media.and_then(|m| m.title_native.as_deref()),
            &row.folder_name,
        )
    }

    /// The port of `getAiringSortInfo`: the shown episode's own past air
    /// date, else the latest past-aired on-disk episode's, else the newest
    /// file's mtime, else the day the series was added.
    fn latest_activity(
        &self,
        row: &SeriesRow,
        episodes: &[&FileRow],
        airing: &[AiringRow],
        files: &[FileRow],
        now: i64,
    ) -> i64 {
        let mut best_aired: Option<(i64, u32)> = None;
        for a in airing {
            let Some(at) = a.aired_at else { continue };
            if at > now || !episodes.iter().any(|f| f.number == f64::from(a.number)) {
                continue;
            }
            if best_aired.is_none_or(|(t, _)| at > t) {
                best_aired = Some((at, a.number));
            }
        }
        let highest_on_disk = episodes.last().map_or(0.0, |f| f.number).max(0.0) as u32;
        let shown = highest_on_disk.max(best_aired.map_or(0, |(_, e)| e));
        let shown_air = airing
            .iter()
            .find(|a| a.number == shown)
            .and_then(|a| a.aired_at);
        match shown_air.filter(|t| *t <= now) {
            Some(t) => t,
            None => match best_aired {
                Some((t, _)) => t,
                None => files.iter().map(|f| f.mtime).max().unwrap_or(row.added_at),
            },
        }
    }

    /// The metadata table's row: what is on disk against what is expected.
    pub fn metadata_row(&self, id: u64) -> Option<MetadataRow> {
        let row = self.row(id)?;
        let card = self.card_of(row);
        let media = row.media.as_ref();
        let have = card.episodes_on_disk;
        let expected = media
            .and_then(|m| m.episodes)
            .filter(|t| *t > 0)
            .map(u64::from)
            .or_else(|| match self.episodes_listed(row) as u64 {
                0 => None,
                n => Some(n),
            })
            .or(match row.kind {
                SeriesKind::Movie => Some(1),
                SeriesKind::Show => None,
            });
        let alt_title = media
            .and_then(|m| m.title_romaji.clone())
            .filter(|r| !r.trim().is_empty() && *r != card.title);
        Some(MetadataRow {
            series: card,
            alt_title,
            provider: row.provider,
            have,
            expected,
            extra_on_disk: expected.map_or(0, |e| have.saturating_sub(e)),
        })
    }
}

/// What every job uses to build a `SeriesChanged` batch: the cards for
/// exactly these series, in the order asked for.
pub fn cards_for(
    conn: &Connection,
    images_dir: &Path,
    ids: &[u64],
) -> Result<Vec<SeriesCard>, CoreError> {
    let snap = Snapshot::load(conn, images_dir, time::now(), Some(ids))?;
    Ok(ids.iter().filter_map(|id| snap.card(*id)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 0.0001, "{a} vs {b}");
    }

    #[test]
    fn strip_matches_electrons_cases() {
        let s = strip(
            Some(5),
            Some(12),
            8,
            Some(9),
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.watched, 5.0 / 12.0);
        near(s.aired_unwatched, 8.0 / 12.0);
        near(s.unknown, 0.0);
        let s = strip(Some(5), Some(12), 12, None, 0, Some(AiringStatus::Finished));
        near(s.aired_unwatched, 0.0);
        let s = strip(
            Some(3),
            Some(12),
            12,
            None,
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.aired_unwatched, 0.0);
        let s = strip(Some(5), None, 8, None, 0, Some(AiringStatus::Releasing));
        near(s.watched, 5.0 / 8.0 * 0.85);
        near(s.aired_unwatched, 0.85);
        near(s.unknown, 0.15);
        let s = strip(Some(5), None, 0, None, 0, None);
        near(s.watched, 0.85);
        near(s.aired_unwatched, 0.0);
        near(s.unknown, 0.15);
        let s = strip(Some(2), None, 10, None, 0, Some(AiringStatus::Finished));
        near(s.aired_unwatched, 0.0);
        near(s.unknown, 0.15);
        let s = strip(None, Some(12), 8, None, 0, Some(AiringStatus::Releasing));
        near(s.watched, 0.0);
        near(s.aired_unwatched, 0.0);
        let s = strip(
            Some(6),
            Some(12),
            0,
            Some(9),
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.aired_unwatched, 8.0 / 12.0);
        let s = strip(
            Some(0),
            Some(12),
            0,
            Some(5),
            0,
            Some(AiringStatus::Releasing),
        );
        near(s.aired_unwatched, 4.0 / 12.0);
        let s = strip(Some(2), Some(12), 0, None, 7, Some(AiringStatus::Releasing));
        near(s.aired_unwatched, 7.0 / 12.0);
        let s = strip(Some(15), Some(12), 0, None, 0, Some(AiringStatus::Finished));
        near(s.watched, 1.0);
        let s = strip(None, None, 0, None, 0, None);
        near(s.watched, 0.0);
        near(s.aired_unwatched, 0.0);
        near(s.unknown, 0.0);
        let s = strip(
            Some(0),
            Some(12),
            0,
            Some(1),
            0,
            Some(AiringStatus::NotYetReleased),
        );
        near(s.watched, 0.0);
        near(s.aired_unwatched, 0.0);
    }

    #[test]
    fn watched_state_rules() {
        assert_eq!(watched_state(None, Some(12), 8, 0), WatchedState::Unknown);
        assert_eq!(watched_state(Some(4), Some(12), 8, 0), WatchedState::Behind);
        assert_eq!(
            watched_state(Some(8), Some(12), 8, 0),
            WatchedState::CaughtUp
        );
        assert_eq!(watched_state(Some(4), Some(12), 0, 8), WatchedState::Behind);
        assert_eq!(
            watched_state(Some(12), Some(12), 8, 0),
            WatchedState::CaughtUp
        );
        assert_eq!(watched_state(Some(3), None, 0, 0), WatchedState::Unknown);
        assert_eq!(watched_state(Some(3), None, 5, 0), WatchedState::Behind);
    }

    #[test]
    fn total_estimate_rule() {
        assert_eq!(total_with_estimate(Some(12), 8, Some(5)), (Some(12), false));
        assert_eq!(total_with_estimate(None, 8, Some(5)), (Some(8), true));
        assert_eq!(total_with_estimate(None, 8, Some(9)), (Some(9), true));
        assert_eq!(total_with_estimate(None, 0, Some(5)), (None, false));
        assert_eq!(total_with_estimate(Some(0), 0, None), (None, false));
    }

    #[test]
    fn season_label_rules() {
        // AniList's word wins, title-cased, with its seasonYear.
        assert_eq!(
            season_label(Some("FALL"), Some("2012-10-09"), Some(2012)),
            Some("Fall 2012".into())
        );
        // A word without seasonYear takes the year off the start date.
        assert_eq!(
            season_label(Some("WINTER"), Some("2018-01-11"), None),
            Some("Winter 2018".into())
        );
        // seasonYear outranks the date's year: a Winter season that starts
        // in December belongs to the next year.
        assert_eq!(
            season_label(Some("winter"), Some("2023-12-28"), Some(2024)),
            Some("Winter 2024".into())
        );
        // No word: the month decides, the date supplies the year.
        assert_eq!(
            season_label(None, Some("2016-01-06"), None),
            Some("Winter 2016".into())
        );
        assert_eq!(
            season_label(None, Some("2016-03-31"), None),
            Some("Winter 2016".into())
        );
        assert_eq!(
            season_label(None, Some("2016-04-01"), None),
            Some("Spring 2016".into())
        );
        assert_eq!(
            season_label(None, Some("2016-07-12"), None),
            Some("Summer 2016".into())
        );
        assert_eq!(
            season_label(None, Some("2016-09-30"), None),
            Some("Summer 2016".into())
        );
        assert_eq!(
            season_label(None, Some("2016-10-01"), None),
            Some("Fall 2016".into())
        );
        assert_eq!(
            season_label(None, Some("2016-12-31"), None),
            Some("Fall 2016".into())
        );
        // Without a word the date's year is the label's year even when
        // `year` disagrees: the date is the only thing that named the season.
        assert_eq!(
            season_label(None, Some("2019-07-08"), Some(2020)),
            Some("Summer 2019".into())
        );
        // Nothing to derive from: a year alone, a word alone, or no date.
        assert_eq!(season_label(None, None, Some(2016)), None);
        assert_eq!(season_label(Some("FALL"), None, None), None);
        assert_eq!(season_label(None, None, None), None);
        assert_eq!(season_label(Some(""), None, Some(2016)), None);
        // Garbage dates derive nothing.
        assert_eq!(season_label(None, Some("2016"), None), None);
        assert_eq!(season_label(None, Some("0000-00-00"), None), None);
        assert_eq!(season_label(None, Some("2016-13-01"), None), None);
    }

    #[test]
    fn next_up_rules() {
        let disk = vec![(10u64, 1, 1.0), (11, 1, 2.0), (12, 1, 3.0), (13, 1, 5.0)];
        assert_eq!(next_up(&disk, Some((1, 2.0)), Some(1)), Some(12));
        assert_eq!(next_up(&disk, Some((1, 3.0)), None), Some(13));
        assert_eq!(next_up(&disk, None, Some(0)), Some(10));
        assert_eq!(next_up(&disk, None, None), Some(10));
        assert_eq!(next_up(&disk, Some((1, 5.0)), Some(5)), None);
        assert_eq!(next_up(&[], Some((1, 1.0)), None), None);
    }

    /// Two seasons on disk: the episode after S02E03 is S02E04, never
    /// S01E04; finishing season 1 lands on season 2's first; a tracker's
    /// progress is season 1's and never outranks a later season's history.
    #[test]
    fn next_up_walks_within_the_season_then_into_the_next() {
        let mut disk: Vec<(u64, u32, f64)> = (1..=10u32)
            .map(|n| (u64::from(n), 1, f64::from(n)))
            .collect();
        disk.extend((1..=8u32).map(|n| (20 + u64::from(n), 2, f64::from(n))));
        assert_eq!(next_up(&disk, Some((2, 3.0)), None), Some(24));
        assert_eq!(next_up(&disk, Some((1, 10.0)), None), Some(21));
        assert_eq!(next_up(&disk, Some((1, 3.0)), None), Some(4));
        assert_eq!(next_up(&disk, Some((2, 3.0)), Some(10)), Some(24));
        assert_eq!(next_up(&disk, Some((2, 8.0)), None), None);
        assert_eq!(next_up(&disk, None, None), Some(1));
    }

    /// A folder matched to a second season's AniList entry: the tracker's
    /// progress counts that season's episodes, so progress 2 resumes at
    /// S02E03, and a finished season has nothing next rather than S02E01.
    #[test]
    fn next_up_reads_tracker_progress_against_the_season_on_disk() {
        let disk: Vec<(u64, u32, f64)> = (1..=3u32)
            .map(|n| (578 + u64::from(n), 2, f64::from(n)))
            .collect();
        assert_eq!(next_up(&disk, None, Some(2)), Some(581));
        assert_eq!(next_up(&disk, None, Some(3)), None);
        assert_eq!(next_up(&disk, None, Some(0)), Some(579));
        // Local history in the same season still wins when it is further on.
        assert_eq!(next_up(&disk, Some((2, 2.0)), Some(1)), Some(581));
        // Season 0 specials never decide which season the progress counts.
        let mut with_specials = vec![(1u64, 0, 1.0)];
        with_specials.extend(disk.iter().copied());
        assert_eq!(next_up(&with_specials, None, Some(2)), Some(581));
    }

    #[cfg(test)]
    fn card() -> SeriesCard {
        use std::time::UNIX_EPOCH;
        SeriesCard {
            id: 1,
            kind: SeriesKind::Show,
            path: String::new(),
            title: String::new(),
            titles: Titles {
                romaji: None,
                english: None,
                native: None,
                folder: String::new(),
            },
            poster: None,
            format: None,
            genres: Vec::new(),
            status: Some(AiringStatus::Finished),
            hidden: false,
            missing: false,
            match_info: None,
            episodes_on_disk: 12,
            extras_on_disk: 0,
            total_episodes: Some(12),
            total_is_estimate: false,
            code: None,
            watched: Some(12),
            watched_state: WatchedState::CaughtUp,
            strip: Strip {
                watched: 1.0,
                aired_unwatched: 0.0,
                unknown: 0.0,
            },
            cubes: String::new(),
            community_score: None,
            my_score: None,
            list_status: None,
            next_airing: None,
            last_viewed_at: None,
            latest_activity_at: UNIX_EPOCH,
            added_at: UNIX_EPOCH,
            downloaded_at: None,
            downloading: Vec::new(),
            franchise: None,
            year: None,
            season: None,
            imdb_id: None,
            imdb_score: None,
            imdb_votes: None,
        }
    }

    #[test]
    fn watched_out_rules() {
        let mut c = card();
        assert!(watched_out(&c));

        // Mid-way or untracked is not done, whatever the status says.
        c.watched = Some(5);
        assert!(!watched_out(&c));
        c.watched = None;
        assert!(!watched_out(&c));

        // Caught up on a releasing show without the completed mark is
        // current, not finished with.
        c.watched = Some(12);
        c.status = Some(AiringStatus::Releasing);
        assert!(!watched_out(&c));
        // But the tracker's word outranks a stale status: the user said
        // done, and the airing refresh's status write may not have
        // landed yet.
        c.list_status = Some(ListStatus::Completed);
        assert!(watched_out(&c));

        // No published total: the estimate (the later of aired and
        // watched) only says how far the user has got, so reaching it is
        // never done with, whatever the status says. This is the case of
        // having watched the newest episode on hand of a show with no
        // published total.
        for status in [
            Some(AiringStatus::Finished),
            Some(AiringStatus::Hiatus),
            Some(AiringStatus::NotYetReleased),
            None,
        ] {
            let mut c = card();
            c.status = status;
            c.total_episodes = Some(8);
            c.total_is_estimate = true;
            c.watched = Some(5);
            assert!(!watched_out(&c));
            c.watched = Some(8);
            assert!(!watched_out(&c));
            c.watched = Some(9);
            assert!(!watched_out(&c));
            // Only the tracker's word files it away.
            c.list_status = Some(ListStatus::Completed);
            assert!(watched_out(&c));
        }

        // A published total reached on a show with no status (unmatched
        // status, or a provider without one) is done with.
        let mut c = card();
        c.status = None;
        assert!(watched_out(&c));

        // Nothing knowable at all is not done, short of the tracker's word.
        let mut c = card();
        c.total_episodes = None;
        c.watched = Some(3);
        assert!(!watched_out(&c));
        c.list_status = Some(ListStatus::Completed);
        assert!(watched_out(&c));

        // A film seen once is done with.
        let mut c = card();
        c.kind = SeriesKind::Movie;
        c.format = Some("MOVIE".to_string());
        c.total_episodes = Some(1);
        c.watched = Some(1);
        assert!(watched_out(&c));
        // Also when the total is only an estimate or unknown, as with a
        // film matched somewhere that publishes no episode count.
        c.total_is_estimate = true;
        assert!(watched_out(&c));
        c.total_episodes = None;
        c.total_is_estimate = false;
        c.format = None;
        assert!(watched_out(&c));
        c.watched = Some(0);
        assert!(!watched_out(&c));

        // Hidden is a separate drawer; watched does not decide for it.
        let mut c = card();
        c.hidden = true;
        assert!(watched_out(&c));
    }

    fn set(ns: &[u32]) -> BTreeSet<u32> {
        ns.iter().copied().collect()
    }

    #[test]
    fn cubes_mark_every_state_in_precedence() {
        // 12 episodes, 7 aired, tracker at 2, 3 watched here, 1 to 5 minus 4
        // on disk: 4 is a hole before 5, 6 and 7 aired with nothing after
        let disk = set(&[1, 2, 3, 5]);
        let here = set(&[3]);
        let row = cubes(&CubeFacts {
            total: Some(12),
            latest_aired: 7,
            next_scheduled: Some(8),
            status: Some(AiringStatus::Releasing),
            tracked: Some(2),
            on_disk: &disk,
            watched_here: &here,
        });
        assert_eq!(row, "wwlmdggfffff");
    }

    #[test]
    fn cubes_of_a_finished_show_have_nothing_unaired() {
        let disk = set(&[1, 2]);
        let none = set(&[]);
        let row = cubes(&CubeFacts {
            total: Some(4),
            latest_aired: 0,
            next_scheduled: None,
            status: Some(AiringStatus::Finished),
            tracked: None,
            on_disk: &disk,
            watched_here: &none,
        });
        assert_eq!(row, "ddgg");
    }

    #[test]
    fn cubes_without_a_total_run_to_the_furthest_known_episode() {
        let disk = set(&[3]);
        let none = set(&[]);
        let row = cubes(&CubeFacts {
            total: None,
            latest_aired: 0,
            next_scheduled: None,
            status: None,
            tracked: None,
            on_disk: &disk,
            watched_here: &none,
        });
        assert_eq!(row, "mmd");
        let empty = cubes(&CubeFacts {
            total: None,
            latest_aired: 0,
            next_scheduled: None,
            status: None,
            tracked: None,
            on_disk: &none,
            watched_here: &none,
        });
        assert_eq!(empty, "");
    }

    fn dated(rows: &[(u32, i64)]) -> Vec<AiringRow> {
        rows.iter()
            .map(|(number, at)| AiringRow {
                number: *number,
                title: None,
                aired_at: Some(*at),
            })
            .collect()
    }

    const T0: i64 = 1_790_000_000;

    #[test]
    fn one_episode_out_predicts_a_week_on() {
        let rows = dated(&[(1, T0)]);
        let now = T0 + 2 * DAY;
        let got = predict_next(&rows, 1, Some(12), Some(AiringStatus::Releasing), now);
        assert_eq!(got, Some((2, T0 + WEEK)));
    }

    #[test]
    fn two_weeks_between_the_first_two_means_every_other_week() {
        // The 1st and the 14th: thirteen days rounds to two weeks
        let rows = dated(&[(1, T0), (2, T0 + 13 * DAY)]);
        let now = T0 + 14 * DAY;
        let got = predict_next(&rows, 2, None, Some(AiringStatus::Releasing), now);
        assert_eq!(got, Some((3, T0 + 13 * DAY + 2 * WEEK)));
    }

    #[test]
    fn a_later_break_keeps_the_weekly_cadence() {
        // Weekly to episode 4, then a two week break before 5
        let rows = dated(&[
            (1, T0),
            (2, T0 + WEEK),
            (3, T0 + 2 * WEEK),
            (4, T0 + 3 * WEEK),
            (5, T0 + 5 * WEEK),
        ]);
        let now = T0 + 5 * WEEK + DAY;
        let got = predict_next(&rows, 5, None, Some(AiringStatus::Releasing), now);
        assert_eq!(got, Some((6, T0 + 6 * WEEK)));
    }

    #[test]
    fn a_missed_guess_moves_a_week_at_a_time() {
        let rows = dated(&[(1, T0), (2, T0 + WEEK)]);
        // Episode 3 was due at T0 + 2 weeks; it is now a day and a bit past
        let late = T0 + 2 * WEEK + DAY + 1;
        let got = predict_next(&rows, 2, None, Some(AiringStatus::Releasing), late);
        assert_eq!(got, Some((3, T0 + 3 * WEEK)));
        // Within its own day it stands
        let same_day = T0 + 2 * WEEK + 3600;
        let got = predict_next(&rows, 2, None, Some(AiringStatus::Releasing), same_day);
        assert_eq!(got, Some((3, T0 + 2 * WEEK)));
        // Three weeks overdue: three weeks later
        let far = T0 + 4 * WEEK + 2 * DAY;
        let got = predict_next(&rows, 2, None, Some(AiringStatus::Releasing), far);
        assert_eq!(got, Some((3, T0 + 5 * WEEK)));
    }

    #[test]
    fn a_file_on_disk_past_the_dates_counts_as_out() {
        let rows = dated(&[(1, T0), (2, T0 + WEEK)]);
        let now = T0 + 2 * WEEK + 2 * DAY;
        // Episode 3 is on disk with no date: 4 is next, two cadences on from 2
        let got = predict_next(&rows, 3, None, Some(AiringStatus::Releasing), now);
        assert_eq!(got, Some((4, T0 + 3 * WEEK)));
    }

    #[test]
    fn no_guess_without_a_date_past_the_end_or_off_air() {
        let none: Vec<AiringRow> = Vec::new();
        assert_eq!(
            predict_next(&none, 3, None, Some(AiringStatus::Releasing), T0),
            None
        );
        let rows = dated(&[(11, T0), (12, T0 + WEEK)]);
        let now = T0 + WEEK + DAY;
        assert_eq!(
            predict_next(&rows, 12, Some(12), Some(AiringStatus::Releasing), now),
            None
        );
        assert_eq!(
            predict_next(&rows, 12, None, Some(AiringStatus::Finished), now),
            None
        );
        assert_eq!(
            predict_next(&rows, 12, None, Some(AiringStatus::Hiatus), now),
            None
        );
    }
}
