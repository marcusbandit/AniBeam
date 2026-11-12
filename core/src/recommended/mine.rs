//! The user's own list as the recommender reads it, and what the library
//! holds. The same read serves the fit, which learns from it, and the row,
//! which leaves out what the user has watched.

use std::collections::{HashMap, HashSet};

use anibeam_recs::{Entry, Status};
use rusqlite::Connection;

use crate::contract::*;
use crate::trackers::accounts;

/// One show on the user's list, as the row reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Listed {
    pub status: ListStatus,
    pub progress: u32,
    /// Out of 10.
    pub score: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct Mine {
    /// What the models learn from, every status included.
    pub entries: Vec<Entry>,
    /// The same, by AniList id, for the row and the page.
    pub listed: HashMap<u64, Listed>,
    /// The user's AniList id, which is left out of their own neighbours.
    pub user: Option<u64>,
    /// Every series the library has files for, by AniList id. A hidden one
    /// is in `hidden` instead, and nothing recommends it.
    pub owned: HashMap<u64, u64>,
    pub hidden: HashSet<u64>,
}

fn status_of(s: ListStatus) -> Status {
    match s {
        ListStatus::Watching => Status::Watching,
        ListStatus::Repeating => Status::Repeating,
        ListStatus::Completed => Status::Completed,
        ListStatus::Paused => Status::Paused,
        ListStatus::Dropped => Status::Dropped,
        ListStatus::Planning => Status::Planning,
    }
}

impl Mine {
    /// The list from the AniList account when it is connected, which is what
    /// the recommender's ids are. Without it, a connected MyAnimeList list
    /// for every show the library's metadata maps to an AniList id. The list
    /// kept on this device fills in what neither carries.
    pub fn read(conn: &Connection) -> Result<Mine, CoreError> {
        let mut mine = Mine::default();
        let anilist =
            accounts::load_row(conn, Tracker::Anilist)?.filter(|row| row.connected_at.is_some());
        let mal = accounts::load_row(conn, Tracker::Mal)?.filter(|row| row.connected_at.is_some());
        mine.user = anilist.as_ref().and_then(|row| row.user_id);
        let sql = if anilist.is_some() {
            Some(
                "SELECT media_id, status, score, repeat, progress FROM tracker_entries
                  WHERE tracker = 'anilist' AND profile_id = 1",
            )
        } else if mal.is_some() {
            // A MAL id maps to AniList through the library's metadata, else
            // through the recommender's own copy of the show, so a show
            // written to MAL from its show page reads back here too.
            Some(
                "SELECT COALESCE(
                          (SELECT m.id FROM anilist_media m WHERE m.mal_id = t.media_id LIMIT 1),
                          (SELECT r.anilist_id FROM rec_shows r
                            WHERE r.media IS NOT NULL
                              AND json_extract(r.media, '$.idMal') = t.media_id LIMIT 1)),
                        t.status, t.score, t.repeat, t.progress
                   FROM tracker_entries t
                  WHERE t.tracker = 'mal' AND t.profile_id = 1",
            )
        } else {
            None
        };
        let mut push =
            |id: i64, status: Option<String>, score: Option<f64>, repeat: i64, progress: i64| {
                let Some(status) = status.as_deref().and_then(ListStatus::from_column) else {
                    return;
                };
                let id = id as u64;
                if mine.listed.contains_key(&id) {
                    return;
                }
                let score = score.filter(|s| *s > 0.0);
                mine.listed.insert(
                    id,
                    Listed {
                        status,
                        progress: u32::try_from(progress).unwrap_or(0),
                        score,
                    },
                );
                mine.entries.push(Entry {
                    show: id,
                    status: status_of(status),
                    // The list keeps tenths out of 10; the models read AniList's 100.
                    score: score.map(|s| (s * 10.0) as f32),
                    repeat: u32::try_from(repeat).unwrap_or(0),
                });
            };
        if let Some(sql) = sql {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, Option<i64>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<f64>>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })?;
            for row in rows {
                // A MAL entry nothing maps to AniList stays out.
                if let (Some(id), status, score, repeat, progress) = row? {
                    push(id, status, score, repeat, progress);
                }
            }
        }
        // The list kept on this device has no progress of its own; the
        // episodes finished here stand in for it.
        let mut stmt = conn.prepare(
            "SELECT s.anilist_id, l.status, l.score,
                    (SELECT COUNT(*) FROM completed c WHERE c.series_id = s.id AND c.profile_id = 1)
               FROM local_entries l
               JOIN series s ON s.id = l.series_id
              WHERE s.anilist_id IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<f64>>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (id, status, score, finished) = row?;
            push(id, status, score, 0, finished);
        }
        let mut stmt = conn.prepare(
            "SELECT anilist_id, MIN(id), MAX(hidden) FROM series
              WHERE anilist_id IS NOT NULL AND missing_since IS NULL
              GROUP BY anilist_id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)? as u64,
                r.get::<_, i64>(1)? as u64,
                r.get::<_, i64>(2)? == 1,
            ))
        })?;
        for row in rows {
            let (anilist, series, hidden) = row?;
            if hidden {
                mine.hidden.insert(anilist);
            } else {
                mine.owned.insert(anilist, series);
            }
        }
        Ok(mine)
    }

    /// Whether the user has yet to watch the show: it is on no list, it is
    /// on Planning, or it is on Watching or On hold with no more than a
    /// first episode or two seen.
    pub fn unwatched(&self, id: u64, episodes: Option<u32>) -> bool {
        match self.listed.get(&id) {
            None => true,
            Some(l) => match l.status {
                ListStatus::Planning => true,
                ListStatus::Watching | ListStatus::Paused => barely_started(l.progress, episodes),
                _ => false,
            },
        }
    }

    /// Whether the show sits on one of the user's lists and is still to
    /// watch: the On list scope.
    pub fn on_list(&self, id: u64, episodes: Option<u32>) -> bool {
        self.listed.contains_key(&id) && self.unwatched(id, episodes)
    }

    /// The list as the ranking filters it: a show barely started reads as
    /// Planning, so it is ranked like one still to watch. What the models
    /// learn from is `entries` as it is.
    pub fn for_ranking(&self, episodes: impl Fn(u64) -> Option<u32>) -> Vec<Entry> {
        self.entries
            .iter()
            .map(|e| {
                let barely = matches!(e.status, Status::Watching | Status::Paused)
                    && self.unwatched(e.show, episodes(e.show));
                Entry {
                    status: if barely { Status::Planning } else { e.status },
                    ..e.clone()
                }
            })
            .collect()
    }
}

/// A first episode or two, and never more than a quarter of the show.
pub fn barely_started(progress: u32, episodes: Option<u32>) -> bool {
    progress == 0 || (progress <= 2 && episodes.is_none_or(|total| progress * 4 <= total))
}

/// The AniList ids the library has, hidden ones left out, as the job reads
/// them for the catalogue.
pub fn owned_ids(mine: &Mine) -> Vec<u64> {
    let mut ids: Vec<u64> = mine.owned.keys().copied().collect();
    ids.sort_unstable();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_first_episode_or_two_is_barely_started_and_a_quarter_is_not() {
        assert!(barely_started(0, Some(3)));
        assert!(barely_started(2, Some(12)));
        assert!(barely_started(2, None));
        assert!(!barely_started(1, Some(3)), "a third of a short show");
        assert!(!barely_started(3, Some(24)));
    }
}
