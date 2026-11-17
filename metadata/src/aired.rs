//! Episode out (#212): the moment an episode of a show on the Watching list
//! airs, the core says so once, as an `EpisodeOut` event a shell can turn into
//! a desktop notification.
//!
//! Nothing here polls. The schedule the core already keeps (`anilist_episodes`)
//! says when the next followed episode airs, and one alarm is set for that
//! moment; it is set again after any change that can move the schedule or the
//! list, the way the calendar file is rewritten. A suspend stops the monotonic
//! clock the alarm runs on, so a shell that hears the machine wake calls
//! `Resumed` and the alarm is set again at once (#594); the Linux desktop does,
//! through logind. Until every shell does, an alarm also never sleeps longer
//! than `LONGEST_SLEEP`, so a wake no shell reports misses nothing for long:
//! waking early only reads the schedule again, it asks no provider anything.
//!
//! What has been said is kept as a mark in the settings table, so a restart
//! never says it twice. An episode that aired while the app was closed is said
//! at the next launch when it aired within `CATCH_UP_SECS`; older ones stay
//! quiet rather than arriving as a pile.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use rusqlite::{Connection, params};

use crate::Metadata;
use anibeam_contract::*;
use anibeam_foundation::events::Listening;
use anibeam_foundation::prefs;
use anibeam_foundation::services::Services;
use anibeam_foundation::store::settings;
use anibeam_foundation::sync::recover;
use anibeam_foundation::time;
use anibeam_foundation::titles;

pub const CATCH_UP_SECS: i64 = 24 * 3600;
pub const LONGEST_SLEEP: Duration = Duration::from_secs(30 * 60);

/// The AniList ids on the Watching list, on either tracker: AniList's own
/// entries, and a MAL entry through the library series that carries both ids.
/// A show the library hides is not followed.
const FOLLOWED: &str = "SELECT media_id FROM tracker_entries
        WHERE tracker = 'anilist' AND profile_id = 1 AND status IN ('watching', 'repeating')
     UNION
     SELECT s.anilist_id FROM tracker_entries t JOIN series s ON s.mal_id = t.media_id
        WHERE t.tracker = 'mal' AND t.profile_id = 1 AND t.status IN ('watching', 'repeating') AND s.anilist_id IS NOT NULL";

fn followed_filter() -> String {
    format!(
        "e.anilist_id IN ({FOLLOWED})
         AND NOT EXISTS (SELECT 1 FROM series h WHERE h.anilist_id = e.anilist_id AND h.hidden = 1)"
    )
}

/// The next followed broadcast after `after`, if the schedule knows one.
pub(crate) fn next_at(conn: &Connection, after: i64) -> Result<Option<i64>, CoreError> {
    let sql = format!(
        "SELECT min(e.aired_at) FROM anilist_episodes e WHERE e.aired_at > ?1 AND {}",
        followed_filter()
    );
    Ok(conn.query_row(&sql, params![after], |r| r.get::<_, Option<i64>>(0))?)
}

/// Every followed episode that aired in `(from, to]`, oldest first.
pub(crate) fn due(conn: &Connection, from: i64, to: i64) -> Result<Vec<EpisodeOut>, CoreError> {
    let lang = prefs::load_preferences(conn)?.title_language;
    let sql = format!(
        "SELECT e.anilist_id, e.number, e.aired_at, m.title_romaji, m.title_english, m.title_native,
                (SELECT s.id FROM series s WHERE s.anilist_id = e.anilist_id AND s.missing_since IS NULL
                   AND EXISTS (SELECT 1 FROM files f WHERE f.series_id = s.id) ORDER BY s.id LIMIT 1)
         FROM anilist_episodes e LEFT JOIN anilist_media m ON m.id = e.anilist_id
         WHERE e.aired_at > ?1 AND e.aired_at <= ?2 AND {}
         ORDER BY e.aired_at, e.anilist_id, e.number",
        followed_filter()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![from, to], |r| {
        let anilist_id = r.get::<_, i64>(0)? as u64;
        let romaji: Option<String> = r.get(3)?;
        let english: Option<String> = r.get(4)?;
        let native: Option<String> = r.get(5)?;
        Ok(EpisodeOut {
            anilist_id,
            episode: r.get::<_, i64>(1)? as u32,
            at: time::from_secs(r.get(2)?),
            title: titles::resolve(
                lang,
                romaji.as_deref(),
                english.as_deref(),
                native.as_deref(),
                &format!("AniList {anilist_id}"),
            ),
            romaji,
            english,
            native,
            series: r.get::<_, Option<i64>>(6)?.map(|id| id as u64),
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// The state on the core: the listener that re-arms after a change, the one
/// alarm, and a re-arm waiting out `SETTLE`.
#[derive(Default)]
pub(crate) struct Aired {
    listener: Mutex<Option<Listening>>,
    alarm: Mutex<Option<tokio::task::JoinHandle<()>>>,
    queued: AtomicBool,
    /// One announce at a time, so an alarm and a re-arm never say one episode twice.
    saying: Mutex<()>,
}

use crate::calendar::SETTLE;

/// How far the mark may lag before a quiet re-arm moves it. Re-arms come
/// after every finished job, so a write each time would be a transaction per
/// settle window for nothing; a mark a few minutes old says the same.
const MARK_STEP: i64 = 10 * 60;

/// Says every followed episode out since the mark, moves the mark to now, and
/// answers the next broadcast to wake for. The caller holds `saying`.
fn announce(services: &Services) -> Result<Option<i64>, CoreError> {
    let now = time::now_secs();
    let (out, next) = services.store.tx(move |tx| {
        let mark = settings::get::<i64>(tx, settings::EPISODES_OUT_AT)?;
        let from = match mark {
            Some(mark) => mark.max(now - CATCH_UP_SECS),
            None => now,
        };
        let out = if from < now {
            due(tx, from, now)?
        } else {
            Vec::new()
        };
        // Never backwards: a clock stepped back would say again what the
        // mark already covers.
        let moved = mark.map_or(now, |m| m.max(now));
        if !out.is_empty() || mark.is_none_or(|m| moved - m >= MARK_STEP) {
            settings::set(tx, settings::EPISODES_OUT_AT, &moved)?;
        }
        Ok((out, next_at(tx, now)?))
    })?;
    for ep in out {
        let plain = anibeam_contract::words!(
            "core-aired-episode-out",
            number = ep.episode.to_string(),
            title = &ep.title,
        );
        let message = format!(
            "episode out: {} {} (anilist {})",
            ep.title, ep.episode, ep.anilist_id
        );
        services.bus.say(
            Level::Info,
            Stage::Metadata,
            plain,
            message,
            EventBody::EpisodeOut { episode: ep },
        );
    }
    Ok(next)
}

/// Announces what is due, then sets the one alarm for the next broadcast,
/// replacing any alarm set before.
pub(crate) fn arm(metadata: &Arc<Metadata>) {
    // Held until the alarm is set, so a second arm with an older answer for
    // the next broadcast cannot replace a newer alarm.
    let _one = recover(&metadata.aired.saying);
    // A core that stopped, or never started, says nothing and sets no alarm.
    if !listening(metadata) {
        return;
    }
    let next = match announce(&metadata.services) {
        Ok(next) => next,
        Err(e) => {
            tracing::warn!("episode out: {e}");
            None
        }
    };
    let mut alarm = recover(&metadata.aired.alarm);
    if let Some(old) = alarm.take() {
        old.abort();
    }
    // Checked again under the alarm's lock: a `stop` since the first check has aborted
    // the alarm already, so a new one would outlive it.
    if !listening(metadata) {
        return;
    }
    let Some(at) = next else { return };
    let wait = Duration::from_secs((at - time::now_secs()).max(0) as u64 + 1).min(LONGEST_SLEEP);
    let weak = Arc::downgrade(metadata);
    *alarm = Some(metadata.services.handle.spawn(async move {
        tokio::time::sleep(wait).await;
        let Some(metadata) = weak.upgrade() else {
            return;
        };
        let handle = metadata.services.handle.clone();
        let _ = handle.spawn_blocking(move || arm(&metadata)).await;
    }));
}

/// Whether an event can move the schedule or the list: the calendar's net
/// (any job that ends, a show hidden or gone, a source removed, a tracker
/// change), less the alarm's own announcements.
fn may_change(event: &Event) -> bool {
    !matches!(event.body, EventBody::EpisodeOut { .. }) && crate::calendar::may_change(event)
}

/// Subscribes the re-arming listener and sets the first alarm. Called once
/// the core has started, so the launch says what aired while it was closed.
pub fn listen(metadata: &Arc<Metadata>) {
    let listener = Arc::new(AfterChange {
        metadata: Arc::downgrade(metadata),
    });
    *recover(&metadata.aired.listener) =
        Some(Listening::new(metadata.services.bus.clone(), listener));
    arm_soon(metadata);
}

/// After a suspend: the alarm slept on a clock that stopped, so it is set again now, which
/// says whatever aired meanwhile. Nothing on a core that is not started (`arm` checks).
pub fn resumed(metadata: &Arc<Metadata>) {
    arm_soon(metadata);
}

/// `arm` off the caller's thread: it reads and writes the store.
fn arm_soon(metadata: &Arc<Metadata>) {
    let owner = metadata.clone();
    metadata.services.handle.spawn_blocking(move || arm(&owner));
}

/// Whether `listen` ran and `stop` has not.
fn listening(metadata: &Metadata) -> bool {
    recover(&metadata.aired.listener).is_some()
}

pub fn stop(metadata: &Metadata) {
    let sub = recover(&metadata.aired.listener).take();
    drop(sub);
    if let Some(alarm) = recover(&metadata.aired.alarm).take() {
        alarm.abort();
    }
}

struct AfterChange {
    metadata: Weak<Metadata>,
}

impl EventListener for AfterChange {
    fn on_event(&self, event: Event) {
        if !may_change(&event) {
            return;
        }
        let Some(metadata) = self.metadata.upgrade() else {
            return;
        };
        if metadata.aired.queued.swap(true, Ordering::SeqCst) {
            return;
        }
        let weak = self.metadata.clone();
        metadata.services.handle.spawn(async move {
            tokio::time::sleep(SETTLE).await;
            let Some(metadata) = weak.upgrade() else {
                return;
            };
            metadata.aired.queued.store(false, Ordering::SeqCst);
            let handle = metadata.services.handle.clone();
            let _ = handle.spawn_blocking(move || arm(&metadata)).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the owner's Watching list is followed: a friend's, on either
    /// tracker, never wakes the owner's computer for a broadcast (#863).
    #[test]
    fn a_friends_watching_list_is_not_followed() {
        let c = anibeam_foundation::store::testing::conn();
        c.execute_batch(
            "INSERT INTO profiles (id, created_at) VALUES (2, 0);
             INSERT INTO anilist_media (id, mal_id, media_type, title_romaji) VALUES (10, 110, 'ANIME', 'Owner show'), (20, 120, 'ANIME', 'Friend show');
             INSERT INTO sources (id, path, added_at) VALUES (1, '/tv', 0);
             INSERT INTO series (id, source_id, kind, path, folder_name, added_at, anilist_id, mal_id)
                 VALUES (1, 1, 'show', '/tv/Friend show', 'Friend show', 0, 20, 120);
             INSERT INTO tracker_entries (profile_id, tracker, media_id, status, fetched_at) VALUES
                 (1, 'anilist', 10, 'watching', 0),
                 (2, 'anilist', 20, 'watching', 0), (2, 'mal', 120, 'watching', 0);
             INSERT INTO anilist_episodes (anilist_id, number, aired_at) VALUES (10, 1, 2000), (20, 1, 1500);",
        )
        .unwrap();
        assert_eq!(next_at(&c, 1000).unwrap(), Some(2000));
    }
}
