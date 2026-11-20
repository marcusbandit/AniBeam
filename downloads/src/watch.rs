//! A followed show's next episode, looked for when it airs (#215).
//!
//! Following a show is a subscription: the torrent client's RSS
//! auto-downloader holds the rule, with the group and quality the person
//! picked, and keeps reading the feed on its own, even while Anibeam is
//! closed. What it does not know is when an episode is due. This module
//! does: for each followed show it sets one timer for the episode that airs
//! next, and one for each episode that aired in the last three days and is
//! not here yet. A little after it airs it asks the client to read the
//! show's feed now; it looks again a few hours on, then a day on, each time
//! reading the library first and asking the feed again only when the episode
//! is still missing. Three days after it aired it stops and says so in one
//! activity line, once; the rule itself goes on looking.
//!
//! Which shows are followed is what the last read of the client's rules
//! linked to the library, kept in `follows`, so the timers are set at launch
//! without reaching the client. A timer only asks the feed through a client
//! that is already running: nothing here starts one, and nothing polls the
//! library.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};

use rusqlite::{Connection, params};

use crate::Downloads;
use anibeam_contract::*;
use anibeam_foundation::events::Signal;
use anibeam_foundation::services::Services;
use anibeam_foundation::sync::recover;

/// When a followed episode is looked for, counted from the moment it aired:
/// a little after it airs, since a release takes a while to be posted, then
/// a few hours on and a day on. Each look is done if the episode is here and
/// otherwise asks the feed again; the last, three days on, says so once and
/// stops instead.
pub const STEPS: [i64; 4] = [20 * 60, 6 * 3600, 24 * 3600, WINDOW];

/// How far back an aired episode is still watched for after a restart: the
/// last step.
const WINDOW: i64 = 72 * 3600;

/// The steps still ahead of `now` for an episode that aired at `aired_at`,
/// as absolute times. The last step already past is done now rather than
/// dropped, since this process has not done it: a restart half an hour
/// after an episode aired still asks the feed straight away, and one past
/// three days gives up straight away. The give-up is always the last time.
pub fn steps_ahead(aired_at: i64, now: i64) -> Vec<i64> {
    let all = STEPS.map(|d| aired_at + d);
    // The steps are in order: past the first, look now
    let mut out = if all[0] <= now { vec![now] } else { Vec::new() };
    out.extend(all.into_iter().filter(|at| *at > now));
    out
}

/// One followed episode to watch for.
#[derive(Clone, Debug, PartialEq)]
pub struct Due {
    pub series: u64,
    pub title: String,
    pub episode: u32,
    pub aired_at: i64,
}

/// Every followed show's episodes worth a timer: the next one to air, and
/// any aired in the last three days that the library does not hold and has
/// not already been given up on. A hidden or missing show is left out.
pub fn due(conn: &Connection, now: i64) -> Result<Vec<Due>, CoreError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT f.series_id, s.folder_name, e.number, e.aired_at
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
                episode: r.get::<_, i64>(2)? as u32,
                aired_at: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// What a step finds before it acts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// The episode is in the library: the watch is done.
    Arrived,
    /// The show is no longer followed, is hidden or gone, or the episode
    /// moved to another date: this watch is done, and a new one is set for
    /// the new date by the arming that read it.
    Stale,
    /// Still due and still missing.
    Missing,
}

/// Where a watched episode stands now, read from the library alone.
pub fn standing(conn: &Connection, d: &Due) -> Result<Standing, CoreError> {
    let (followed, aired_at, here): (i64, Option<i64>, i64) = conn.query_row(
        "SELECT
             EXISTS (SELECT 1 FROM follows f JOIN series s ON s.id = f.series_id
                     WHERE f.series_id = ?1 AND s.hidden = 0 AND s.missing_since IS NULL),
             (SELECT e.aired_at FROM series s JOIN anilist_episodes e ON e.anilist_id = s.anilist_id
              WHERE s.id = ?1 AND e.number = ?2),
             EXISTS (SELECT 1 FROM files WHERE series_id = ?1 AND kind = 'episode'
                     AND number >= ?2 AND number < ?2 + 1)",
        params![d.series as i64, i64::from(d.episode)],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(if here == 1 {
        Standing::Arrived
    } else if followed == 0 || aired_at != Some(d.aired_at) {
        Standing::Stale
    } else {
        Standing::Missing
    })
}

/// The followed shows as the last read of the client's rules linked them:
/// every enabled rule whose folder is a library show, each feed of a show
/// kept. Replaces what was there, since the rules are the truth about what
/// is followed.
pub async fn remember(downloads: &Arc<Downloads>, rules: &[SubscriptionRule]) {
    let follows: Vec<(u64, String)> = rules
        .iter()
        .filter(|r| r.rule_enabled)
        .filter_map(|r| r.show.as_ref().map(|s| (s.series_id, r.feed_url.clone())))
        .collect();
    let now = anibeam_foundation::time::now_secs();
    let saved = downloads
        .services
        .store
        .tx_async(move |tx| {
            tx.execute("DELETE FROM follows", [])?;
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO follows (series_id, feed_url, read_at) VALUES (?1, ?2, ?3)",
            )?;
            for (series, feed) in &follows {
                stmt.execute(params![*series as i64, feed, now])?;
            }
            Ok(())
        })
        .await;
    match saved {
        Ok(()) => arm(downloads),
        Err(e) => tracing::warn!("followed shows not kept: {e}"),
    }
}

/// The episodes a timer is set for, at the date it was set for. A new date
/// is a new key, so a rescheduled episode gets a timer of its own, and the
/// old one finds itself stale at its next step.
static ARMED: LazyLock<Mutex<HashSet<(u64, u32, i64)>>> = LazyLock::new(Default::default);

fn armed() -> std::sync::MutexGuard<'static, HashSet<(u64, u32, i64)>> {
    recover(&ARMED)
}

/// A watch's place in `ARMED`, given up however the task ends: done,
/// stale, or dropped with the runtime when the core shuts down, so a core
/// opened again in the same process arms it afresh.
struct Slot((u64, u32, i64));

impl Drop for Slot {
    fn drop(&mut self) {
        armed().remove(&self.0);
    }
}

/// Arms the timers again when the metadata below says an airing refresh
/// moved a schedule (#854), which may be a new time for a followed show's
/// next episode. Set once, by `Core::open`.
pub fn listen(downloads: &Arc<Downloads>) {
    let weak = Arc::downgrade(downloads);
    downloads.services.bus.on_signal(Arc::new(move |signal| {
        if signal == Signal::SchedulesMoved
            && let Some(downloads) = weak.upgrade()
        {
            arm(&downloads);
        }
    }));
}

/// Sets a timer for every followed episode that has none yet. Called at
/// launch, after the rules are read, after a schedule changes, and when a
/// watch ends, so the episode after it gets one.
pub fn arm(downloads: &Arc<Downloads>) {
    let now = anibeam_foundation::time::now_secs();
    let found = match downloads.services.store.read(|c| due(c, now)) {
        Ok(found) => found,
        Err(e) => {
            tracing::debug!("followed episodes not read: {e}");
            return;
        }
    };
    let owner = downloads.clone();
    for d in found {
        let key = (d.series, d.episode, d.aired_at);
        if !armed().insert(key) {
            continue;
        }
        let downloads = owner.clone();
        owner.services.handle.spawn(async move {
            let _slot = Slot(key);
            let ended = watch(&downloads, &d).await;
            if ended {
                arm(&downloads);
            }
        });
    }
}

/// The longest one sleep goes before the wall clock is read again, so a
/// machine that slept through a step does it on waking rather than hours
/// late: the runtime's clock does not count a suspend.
const NAP: u64 = 10 * 60;

/// One episode's watch, step by step. True when it ended on its own (the
/// episode arrived, or the watch gave up), so the next one can be armed.
async fn watch(downloads: &Downloads, d: &Due) -> bool {
    let steps = steps_ahead(d.aired_at, anibeam_foundation::time::now_secs());
    for (i, &at) in steps.iter().enumerate() {
        // The runtime ends this sleep with the core, so a shutdown needs
        // nothing of its own here.
        loop {
            let left = at - anibeam_foundation::time::now_secs();
            if left <= 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs((left as u64).min(NAP))).await;
        }
        let read = d.clone();
        let now = match downloads
            .services
            .store
            .write_async(move |c| standing(c, &read))
            .await
        {
            Ok(s) => s,
            Err(e) => {
                tracing::debug!("followed episode not read: {e}");
                continue;
            }
        };
        match now {
            Standing::Arrived => return true,
            Standing::Stale => return false,
            Standing::Missing if i + 1 == steps.len() => {
                give_up(&downloads.services, d).await;
                return true;
            }
            Standing::Missing => refresh(downloads, d.series).await,
        }
    }
    true
}

/// A client to ask the feeds through, only one that runs without the core
/// starting it: the one this session already reached, or the person's own
/// (Existing). Anibeam's own managed client is never started for this.
async fn running_client(downloads: &Downloads) -> Option<Arc<super::qbt::QbtClient>> {
    if let Some(client) = downloads.state().qbt.clone() {
        return Some(client);
    }
    let settings = super::load_settings(&downloads.services).await.ok()?;
    if settings.torrent_mode != TorrentMode::Existing {
        return None;
    }
    super::qbt_client(downloads).await.ok()
}

/// Asks the client to read every feed that follows the show, now.
async fn refresh(downloads: &Downloads, series: u64) {
    let Some(client) = running_client(downloads).await else {
        return;
    };
    let feeds: Vec<String> = match downloads
        .services
        .store
        .write_async(move |c| {
            let mut stmt = c.prepare("SELECT feed_url FROM follows WHERE series_id = ?1")?;
            let rows = stmt
                .query_map([series as i64], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .await
    {
        Ok(f) => f,
        Err(e) => {
            tracing::debug!("followed feeds not read: {e}");
            return;
        }
    };
    let paths = match client.feeds().await {
        Ok(paths) => paths,
        Err(e) => {
            tracing::debug!("feeds not read for a followed episode: {e}");
            return;
        }
    };
    for (path, url) in paths {
        if feeds.contains(&url)
            && let Err(e) = client.refresh_feed(&path).await
        {
            tracing::debug!("feed {path} not refreshed: {e}");
        }
    }
}

/// The one line for an episode that never came, said once ever.
async fn give_up(services: &Services, d: &Due) {
    let (series, episode, now) = (d.series, d.episode, anibeam_foundation::time::now_secs());
    let first = services
        .store
        .write_async(move |c| {
            Ok(c.execute(
                "INSERT OR IGNORE INTO follow_misses (series_id, episode, said_at) VALUES (?1, ?2, ?3)",
                params![series as i64, i64::from(episode), now],
            )? == 1)
        })
        .await;
    if !matches!(first, Ok(true)) {
        return;
    }
    services.bus.say(
        Level::Warn,
        Stage::Downloads,
        anibeam_contract::words!(
            "core-downloads-episode-missed",
            episode = d.episode,
            title = &d.title
        ),
        format!(
            "followed episode {} of series {} not here three days after it aired",
            d.episode, d.series
        ),
        EventBody::Notice,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Teams;

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
                aired + 20 * 60,
                aired + 6 * H,
                aired + 24 * H,
                aired + 72 * H
            ]
        );
        // Past the first look, which this process never did: look now.
        let now = aired + H;
        assert_eq!(steps_ahead(aired, now)[..2], [now, aired + 6 * H]);
        // After the second: look again at once, then the rest.
        let now = aired + 10 * H;
        assert_eq!(
            steps_ahead(aired, now),
            vec![now, aired + 24 * H, aired + 72 * H]
        );
        let late = aired + 80 * H;
        assert_eq!(steps_ahead(aired, late), vec![late]);
    }

    fn teams() -> (tempfile::TempDir, Teams) {
        let dir = tempfile::tempdir().unwrap();
        let teams = Teams::open(dir.path(), anibeam_net::FakeHttp::new());
        (dir, teams)
    }

    fn exec(teams: &Teams, sql: &'static str) {
        teams
            .services
            .store
            .write(move |c| Ok(c.execute_batch(sql)?))
            .unwrap();
    }

    /// The episodes `due` sets a timer for.
    fn episodes(teams: &Teams, now: i64) -> Vec<u32> {
        let found = teams.services.store.read(|c| due(c, now)).unwrap();
        found.iter().map(|d| d.episode).collect()
    }

    /// A followed show with episodes 1 to 3 out (1 here, 2 a day ago and
    /// missing, 3 a week ago and missing) and 4 due tomorrow.
    fn followed(teams: &Teams, now: i64) {
        exec(
            teams,
            "INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0);
             INSERT INTO anilist_media (id, media_type, title_romaji, status, format, fetched_at)
               VALUES (9, 'ANIME', 'Show', 'RELEASING', 'TV', 0);
             INSERT INTO series (source_id, kind, path, folder_name, added_at, provider, anilist_id)
               VALUES (1, 'show', '/lib/Show', 'Show', 0, 'anilist', 9);
             INSERT INTO files (series_id, path, size, mtime, kind, number, label, episode_key, sidecars, seen_at)
               VALUES (1, '/lib/Show/01.mkv', 0, 0, 'episode', 1, 'EP1', '1', '[]', 0);
             INSERT INTO follows (series_id, feed_url, read_at) VALUES (1, 'https://feed.invalid/rss', 0);",
        );
        teams
            .services
            .store
            .write(move |c| {
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
        let (_dir, teams) = teams();
        let now = 2_000_000_000;
        followed(&teams, now);
        assert_eq!(episodes(&teams, now), vec![2, 4]);

        // Given up on once, the recent miss is not watched again after a
        // restart; and an episode that arrives is not missing.
        exec(
            &teams,
            "INSERT INTO follow_misses (series_id, episode, said_at) VALUES (1, 2, 0)",
        );
        assert_eq!(episodes(&teams, now), vec![4]);
        // A step reads where the episode stands: here, gone stale with a new
        // date, or still missing.
        let due4 = Due {
            series: 1,
            title: "Show".into(),
            episode: 4,
            aired_at: now + 86_400,
        };
        let standing_of = |d: &Due| teams.services.store.read(|c| standing(c, d)).unwrap();
        assert_eq!(standing_of(&due4), Standing::Missing);
        let due1 = Due {
            episode: 1,
            aired_at: now - 14 * 86_400,
            ..due4.clone()
        };
        assert_eq!(standing_of(&due1), Standing::Arrived);
        let moved = Due {
            aired_at: now + 2 * 86_400,
            ..due4.clone()
        };
        assert_eq!(standing_of(&moved), Standing::Stale);
        teams.shutdown();
    }

    /// An airing refresh that moved a schedule says so on the bus, and the
    /// timers are armed before `signal` returns, where the refresh used to
    /// call `arm` itself (#854).
    #[test]
    fn a_moved_schedule_arms_the_followed_episodes() {
        let (_dir, teams) = teams();
        let now = anibeam_foundation::time::now_secs();
        followed(&teams, now);
        let next = (1, 4, now + 86_400);
        let missed = (1, 2, now - 86_400);
        assert!(!armed().contains(&next));
        teams.services.bus.signal(Signal::SchedulesMoved);
        {
            let armed = armed();
            assert!(armed.contains(&next) && armed.contains(&missed));
        }
        teams.shutdown();
    }

    /// A show the rules no longer follow, or that the person hid, has no
    /// timer.
    #[test]
    fn an_unfollowed_or_hidden_show_is_not_watched() {
        let (_dir, teams) = teams();
        let now = 2_000_000_000;
        followed(&teams, now);
        exec(&teams, "UPDATE series SET hidden = 1");
        assert!(episodes(&teams, now).is_empty());
        exec(&teams, "UPDATE series SET hidden = 0");
        teams
            .services
            .handle
            .block_on(remember(&teams.downloads, &[]));
        assert!(episodes(&teams, now).is_empty());
        teams.shutdown();
    }
}
