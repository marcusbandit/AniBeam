//! A card, and the pure rules a card is drawn from. The rules at the top
//! are ports of Electron's `airingUtils.ts` and `SeriesDetailPage.tsx`; the
//! assembly below turns a `Snapshot`'s rows into the contract's records,
//! and every job builds its `SeriesChanged` batch through `cards_for`.

use std::path::Path;

use rusqlite::Connection;

use crate::contract::*;
use crate::library::snapshot::{AiringRow, FileRow, SeriesRow, Snapshot};
use crate::library::{labels, titles};
use crate::time;

// ---------------------------------------------------------------------------
// The pure rules
// ---------------------------------------------------------------------------

/// The fraction of the strip reserved for the dark "total unknown" cap, so a
/// bar without a published episode count can never read as complete.
/// Electron's `UNKNOWN_TAIL_PCT = 15`.
pub const UNKNOWN_TAIL: f64 = 0.15;

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

/// `episodes` is (file id, number) sorted by number. `last_completed` is the
/// most recently completed episode's number. With something watched, the
/// episode after it; with nothing watched, the first one on disk, above zero
/// when a tracker entry says the series has been started at all.
pub fn next_up(
    episodes: &[(u64, f64)],
    last_completed: Option<f64>,
    tracker_progress: Option<u32>,
) -> Option<u64> {
    let last = last_completed
        .unwrap_or(0.0)
        .max(f64::from(tracker_progress.unwrap_or(0)));
    if last > 0.0 {
        return episodes
            .iter()
            .find(|(_, n)| *n == last + 1.0)
            .or_else(|| episodes.iter().find(|(_, n)| *n > last))
            .map(|(f, _)| *f);
    }
    if tracker_progress.is_some() {
        return episodes.iter().find(|(_, n)| *n > 0.0).map(|(f, _)| *f);
    }
    episodes.first().map(|(f, _)| *f)
}
/// The highest episode that has aired, and the earliest one still to come.
pub(super) fn aired_and_scheduled(airing: &[AiringRow], now: i64) -> (u32, Option<(u32, i64)>) {
    let mut latest_aired = 0;
    let mut next_scheduled: Option<(u32, i64)> = None;
    for a in airing {
        let Some(at) = a.aired_at else { continue };
        if at <= now {
            latest_aired = latest_aired.max(a.number);
        } else if next_scheduled.is_none_or(|(_, t)| at < t) {
            next_scheduled = Some((a.number, at));
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

        let latest_downloaded = episodes.last().map_or(0.0, |f| f.number).max(0.0) as u32;
        let (latest_aired, next_scheduled) = aired_and_scheduled(airing, now);
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
            status,
            hidden: row.hidden,
            missing: row.missing,
            match_info: row.provider.map(|provider| MatchInfo {
                provider,
                anilist_id: row.anilist_id,
                mal_id: row.mal_id,
                tmdb_id: row.tmdb_id,
                tmdb_kind: row.tmdb_kind,
                confirmed: row.confirmed,
            }),
            episodes_on_disk: episodes.len() as u64,
            extras_on_disk: (files.len() - episodes.len()) as u64,
            total_episodes,
            total_is_estimate,
            code,
            watched,
            watched_state: watched_state(watched, total, latest_aired, latest_downloaded),
            strip: strip(
                watched,
                total,
                latest_aired,
                next_scheduled.map(|(n, _)| n),
                latest_downloaded,
                status,
            ),
            community_score: media
                .and_then(|m| m.average_score)
                .map(|s| f64::from(s) / 10.0),
            my_score: entry.and_then(|e| e.score),
            list_status: entry.and_then(|e| e.status),
            next_airing: next_scheduled.map(|(episode, at)| Airing {
                episode,
                at: time::from_secs(at),
            }),
            last_viewed_at: self.views.get(&row.id).map(|(_, at)| time::from_secs(*at)),
            latest_activity_at: time::from_secs(
                self.latest_activity(row, &episodes, airing, files, now),
            ),
        }
    }

    fn title_of(&self, row: &SeriesRow) -> String {
        let media = row.media.as_ref();
        titles::resolve(
            self.lang,
            media.and_then(|m| m.title_romaji.as_deref()),
            media.and_then(|m| m.title_english.as_deref()),
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
            .or_else(|| match self.airing_of(row).len() as u64 {
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
    fn next_up_rules() {
        let disk = vec![(10u64, 1.0), (11, 2.0), (12, 3.0), (13, 5.0)];
        assert_eq!(next_up(&disk, Some(2.0), Some(1)), Some(12));
        assert_eq!(next_up(&disk, Some(3.0), None), Some(13));
        assert_eq!(next_up(&disk, None, Some(0)), Some(10));
        assert_eq!(next_up(&disk, None, None), Some(10));
        assert_eq!(next_up(&disk, Some(5.0), Some(5)), None);
        assert_eq!(next_up(&[], Some(1.0), None), None);
    }
}
