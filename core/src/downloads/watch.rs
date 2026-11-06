//! A followed show's next episode, looked for when it airs (#215).
//!
//! Following a show is a subscription: the torrent client's RSS
//! auto-downloader holds the rule, with the group and quality the person
//! picked, and keeps reading the feed on its own, even while AniBeam is
//! closed. What it does not know is when an episode is due. This module
//! does: for each followed show it sets one timer for the episode that airs
//! next, and one for an episode that aired in the last three days and is not
//! here yet. At the moment it airs (and a little after) it asks the client
//! to read the show's feed now; it looks again a few hours on, then a day
//! on, each time only reading the library, and asking the feed again when
//! the episode is still missing. Three days after it aired it stops and
//! says so in one activity line, once; the rule itself goes on looking.
//!
//! Which shows are followed is what the last read of the client's rules
//! linked to the library, kept in `follows`, so the timers are set at launch
//! without reaching the client. A timer only asks the feed through a client
//! that is already running: nothing here starts one, and nothing polls the
//! library.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};

use rusqlite::{Connection, params};

use crate::contract::*;
use crate::core::Core;

/// When a followed episode is looked for, counted from the moment it aired:
/// the feed is asked a little after it airs, since a release takes a while
/// to be posted, then the library is checked (and the feed asked again) a
/// few hours on and a day on, and three days on the watch gives up.
pub const STEPS: [(i64, Step); 4] = [
    (20 * 60, Step::Refresh),
    (6 * 3600, Step::Check),
    (24 * 3600, Step::Check),
    (72 * 3600, Step::GiveUp),
];

/// How far back an aired episode is still watched for after a restart.
const WINDOW: i64 = 72 * 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Ask the feed now.
    Refresh,
    /// Done if the episode is here; otherwise ask the feed again.
    Check,
    /// Done if the episode is here; otherwise say so once and stop.
    GiveUp,
}

/// The steps still ahead of `now` for an episode that aired at `aired_at`,
/// as absolute times. A step already past is dropped, except the last one
/// before now when it is a check or the give-up: a restart then looks once
/// straight away rather than waiting for the next step.
pub fn steps_ahead(aired_at: i64, now: i64) -> Vec<(i64, Step)> {
    let all: Vec<(i64, Step)> = STEPS.iter().map(|(d, s)| (aired_at + d, *s)).collect();
    let mut out: Vec<(i64, Step)> = Vec::new();
    if let Some(&(_, step)) = all.iter().rev().find(|(at, _)| *at <= now)
        && step != Step::Refresh
    {
        out.push((now, step));
    }
    out.extend(all.into_iter().filter(|(at, _)| *at > now));
    out
}

/// One followed episode to watch for.
#[derive(Clone, Debug, PartialEq)]
pub struct Due {
    pub series: u64,
    pub title: String,
    pub feed_url: String,
    pub episode: u32,
    pub aired_at: i64,
}

/// Every followed show's episodes worth a timer: the next one to air, and
/// any aired in the last three days that the library does not hold and has
/// not already been given up on. A hidden or missing show is left out.
pub fn due(conn: &Connection, now: i64) -> Result<Vec<Due>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT f.series_id, s.folder_name, f.feed_url, e.number, e.aired_at
         FROM follows f
         JOIN series s ON s.id = f.series_id
         JOIN anilist_episodes e ON e.anilist_id = s.anilist_id
         WHERE s.hidden = 0 AND s.missing_since IS NULL AND e.aired_at IS NOT NULL
           AND (
             (e.aired_at > ?1 AND e.aired_at = (SELECT MIN(n.aired_at) FROM anilist_episodes n
                                                 WHERE n.anilist_id = s.anilist_id AND n.aired_at > ?1))
             OR (e.aired_at BETWEEN ?1 - ?2 AND ?1
                 AND NOT EXISTS (SELECT 1 FROM files fi WHERE fi.series_id = f.series_id
                                 AND fi.kind = 'episode' AND fi.number >= e.number AND fi.number < e.number + 1)
                 AND NOT EXISTS (SELECT 1 FROM follow_misses m WHERE m.series_id = f.series_id
                                 AND m.episode = e.number)))
         ORDER BY e.aired_at",
    )?;
    let rows = stmt
        .query_map(params![now, WINDOW], |r| {
            Ok(Due {
                series: r.get::<_, i64>(0)? as u64,
                title: r.get(1)?,
                feed_url: r.get(2)?,
                episode: r.get::<_, i64>(3)? as u32,
                aired_at: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Whether the library holds the episode now.
pub fn arrived(conn: &Connection, series: u64, episode: u32) -> Result<bool, CoreError> {
    let n: i64 = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM files WHERE series_id = ?1 AND kind = 'episode'
                        AND number >= ?2 AND number < ?2 + 1)",
        params![series as i64, f64::from(episode)],
        |r| r.get(0),
    )?;
    Ok(n == 1)
}

/// The followed shows as the last read of the client's rules linked them:
/// every enabled rule whose folder is a library show. Replaces what was
/// there, since the rules are the truth about what is followed.
pub fn remember(core: &Core, rules: &[SubscriptionRule]) {
    let follows: Vec<(u64, String)> = rules
        .iter()
        .filter(|r| r.rule_enabled)
        .filter_map(|r| r.show.as_ref().map(|s| (s.series_id, r.feed_url.clone())))
        .collect();
    let now = crate::time::now_secs();
    let saved = core.store.tx(move |tx| {
        tx.execute("DELETE FROM follows", [])?;
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO follows (series_id, feed_url, read_at) VALUES (?1, ?2, ?3)",
        )?;
        for (series, feed) in &follows {
            stmt.execute(params![*series as i64, feed, now])?;
        }
        Ok(())
    });
    match saved {
        Ok(()) => arm(core),
        Err(e) => tracing::warn!("followed shows not kept: {e}"),
    }
}

/// The episodes a timer is already set for, so arming again sets none twice.
static ARMED: LazyLock<Mutex<HashSet<(u64, u32)>>> = LazyLock::new(Default::default);

fn armed() -> std::sync::MutexGuard<'static, HashSet<(u64, u32)>> {
    ARMED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Sets a timer for every followed episode that has none yet. Called at
/// launch, after the rules are read, and after a schedule changes.
pub fn arm(core: &Core) {
    let now = crate::time::now_secs();
    let found = match core.store.read(|c| due(c, now)) {
        Ok(found) => found,
        Err(e) => {
            tracing::debug!("followed episodes not read: {e}");
            return;
        }
    };
    let Ok(owner) = core.owner() else { return };
    for d in found {
        if !armed().insert((d.series, d.episode)) {
            continue;
        }
        let core = owner.clone();
        owner.handle.spawn(async move {
            watch(&core, &d).await;
            armed().remove(&(d.series, d.episode));
        });
    }
}

/// One episode's watch, step by step, until it arrives or the watch gives
/// up.
async fn watch(core: &Arc<Core>, d: &Due) {
    for (at, step) in steps_ahead(d.aired_at, crate::time::now_secs()) {
        let wait = (at - crate::time::now_secs()).max(0) as u64;
        // The runtime ends this sleep with the core, so a shutdown needs
        // nothing of its own here.
        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
        let (series, episode) = (d.series, d.episode);
        let here = core
            .store
            .read(|c| arrived(c, series, episode))
            .unwrap_or(false);
        if here {
            return;
        }
        match step {
            Step::Refresh | Step::Check => refresh(core, &d.feed_url).await,
            Step::GiveUp => {
                give_up(core, d);
                return;
            }
        }
    }
}

/// Asks the client to read the show's feed now, only through a client that
/// is already running.
async fn refresh(core: &Arc<Core>, feed_url: &str) {
    let Some(client) = core.downloads.state().qbt.clone() else {
        return;
    };
    let path = match client.feeds().await {
        Ok(feeds) => feeds
            .into_iter()
            .find(|(_, url)| url == feed_url)
            .map(|(p, _)| p),
        Err(e) => {
            tracing::debug!("feeds not read for a followed episode: {e}");
            return;
        }
    };
    if let Some(path) = path
        && let Err(e) = client.refresh_feed(&path).await
    {
        tracing::debug!("feed {path} not refreshed: {e}");
    }
}

/// The one line for an episode that never came, said once ever.
fn give_up(core: &Arc<Core>, d: &Due) {
    let (series, episode, now) = (d.series, d.episode, crate::time::now_secs());
    let first = core.store.write(move |c| {
        Ok(c.execute(
            "INSERT OR IGNORE INTO follow_misses (series_id, episode, said_at) VALUES (?1, ?2, ?3)",
            params![series as i64, episode as i64, now],
        )? == 1)
    });
    if !matches!(first, Ok(true)) {
        return;
    }
    core.bus.say(
        Level::Warn,
        Stage::Downloads,
        missed_line(&d.title, d.episode),
        format!(
            "followed episode {} of series {} not here three days after it aired",
            d.episode, d.series
        ),
        EventBody::Notice,
    );
}

/// What the activity log says when a followed episode never came.
pub fn missed_line(title: &str, episode: u32) -> String {
    format!(
        "Episode {episode} of {title} has not turned up three days after it aired. Your feed keeps looking for it."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: i64 = 3600;

    /// Before it airs every step is ahead; a restart in the middle looks
    /// once at once and keeps the rest; past three days only the give-up
    /// is left, straight away.
    #[test]
    fn the_steps_follow_the_moment_it_aired() {
        let aired = 1_000_000;
        assert_eq!(
            steps_ahead(aired, aired - H),
            vec![
                (aired + 20 * 60, Step::Refresh),
                (aired + 6 * H, Step::Check),
                (aired + 24 * H, Step::Check),
                (aired + 72 * H, Step::GiveUp),
            ]
        );
        // Between the refresh and the first check: nothing to redo now.
        assert_eq!(
            steps_ahead(aired, aired + H)[0],
            (aired + 6 * H, Step::Check)
        );
        // After the first check: look again at once, then the rest.
        let now = aired + 10 * H;
        assert_eq!(
            steps_ahead(aired, now),
            vec![
                (now, Step::Check),
                (aired + 24 * H, Step::Check),
                (aired + 72 * H, Step::GiveUp),
            ]
        );
        let late = aired + 80 * H;
        assert_eq!(steps_ahead(aired, late), vec![(late, Step::GiveUp)]);
    }

    fn core() -> (tempfile::TempDir, Arc<Core>) {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::CorePaths::under(dir.path());
        let core = Core::open_with_http_and_secrets(
            paths.clone(),
            crate::net::FakeHttp::new(),
            crate::trackers::Secrets::file_only(paths.secrets_path()),
        )
        .unwrap();
        (dir, core)
    }

    /// A followed show with episodes 1 to 3 out (1 here, 2 a day ago and
    /// missing, 3 a week ago and missing) and 4 due tomorrow.
    fn followed(core: &Core, now: i64) {
        core.store
            .write(move |c| {
                c.execute_batch(
                    "INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0);
                     INSERT INTO anilist_media (id, media_type, title_romaji, status, format, fetched_at)
                       VALUES (9, 'ANIME', 'Show', 'RELEASING', 'TV', 0);
                     INSERT INTO series (source_id, kind, path, folder_name, added_at, provider, anilist_id)
                       VALUES (1, 'show', '/lib/Show', 'Show', 0, 'anilist', 9);
                     INSERT INTO files (series_id, path, size, mtime, kind, number, label, episode_key, sidecars, seen_at)
                       VALUES (1, '/lib/Show/01.mkv', 0, 0, 'episode', 1, 'EP1', '1', '[]', 0);
                     INSERT INTO follows (series_id, feed_url, read_at) VALUES (1, 'https://feed.invalid/rss', 0);",
                )?;
                for (n, at) in [
                    (1, now - 14 * 86_400),
                    (2, now - 86_400),
                    (3, now - 7 * 86_400),
                    (4, now + 86_400),
                    (5, now + 8 * 86_400),
                ] {
                    c.execute(
                        "INSERT INTO anilist_episodes (anilist_id, number, aired_at) VALUES (9, ?1, ?2)",
                        params![n, at],
                    )?;
                }
                Ok(())
            })
            .unwrap();
    }

    /// The watch takes the episode due next and the one that aired a day ago
    /// and is missing; not the one already here, not one aired a week ago,
    /// not one past the next.
    #[test]
    fn the_watch_takes_the_next_episode_and_a_recent_miss() {
        let (_dir, core) = core();
        let now = 2_000_000_000;
        followed(&core, now);
        let found: Vec<u32> = core
            .store
            .read(|c| due(c, now))
            .unwrap()
            .iter()
            .map(|d| d.episode)
            .collect();
        assert_eq!(found, vec![2, 4]);

        // Given up on once, the recent miss is not watched again after a
        // restart; and an episode that arrives is not missing.
        core.store
            .write(move |c| {
                c.execute(
                    "INSERT INTO follow_misses (series_id, episode, said_at) VALUES (1, 2, ?1)",
                    [now],
                )?;
                Ok(())
            })
            .unwrap();
        let found: Vec<u32> = core
            .store
            .read(|c| due(c, now))
            .unwrap()
            .iter()
            .map(|d| d.episode)
            .collect();
        assert_eq!(found, vec![4]);
        assert!(core.store.read(|c| arrived(c, 1, 1)).unwrap());
        assert!(!core.store.read(|c| arrived(c, 1, 4)).unwrap());
        core.shutdown();
    }

    /// A show the rules no longer follow, or that the person hid, has no
    /// timer.
    #[test]
    fn an_unfollowed_or_hidden_show_is_not_watched() {
        let (_dir, core) = core();
        let now = 2_000_000_000;
        followed(&core, now);
        core.store
            .write(|c| {
                c.execute("UPDATE series SET hidden = 1", [])?;
                Ok(())
            })
            .unwrap();
        assert!(core.store.read(|c| due(c, now)).unwrap().is_empty());
        core.store
            .write(|c| {
                c.execute("UPDATE series SET hidden = 0", [])?;
                Ok(())
            })
            .unwrap();
        remember(&core, &[]);
        assert!(core.store.read(|c| due(c, now)).unwrap().is_empty());
        core.shutdown();
    }

    #[test]
    fn the_miss_is_one_plain_line() {
        let line = missed_line("Frieren", 5);
        assert!(line.starts_with("Episode 5 of Frieren"));
        assert!(!line.contains(['\u{2013}', '\u{2014}']));
    }
}
