//! The Door's read cache: the answers to whole-library reads, kept while the core's state
//! generation stands still. Every event but a job's progress moves the generation before
//! any listener hears it (`EventBus::emit`), so a page that rereads on an event always
//! misses, and the four or five pages and timers that reread on one event share one answer.
//!
//! Two things move a card without a write, and the cache answers both with an age limit:
//! the time of day (an episode airs, the airing rail and the tab a series sits in follow
//! `now`), and the rare write that lands after the event that announced it. Thirty seconds
//! is the same beat the library page's own clock refreshes on.
//!
//! A read put with `keep_stale` (the library page's, Door::library_view) also outlives its
//! generation as the key's last known answer, which the Door shows at once while it reads
//! again on a thread of its own. Every other answer goes when the generation moves.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anibeam_core::Call;

/// How long a kept answer lives even when nothing changed.
pub const MAX_AGE: Duration = Duration::from_secs(30);

/// Developer options can turn the cache off for the session, to see what it saves.
static PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_paused(paused: bool) {
    PAUSED.store(paused, std::sync::atomic::Ordering::Relaxed);
}

fn paused() -> bool {
    PAUSED.load(std::sync::atomic::Ordering::Relaxed)
}

/// The key a call is kept under, or `None` for a call that is not kept: only the reads
/// that load the whole library, which are the ones worth keeping and the ones with no
/// side effect a repeat would owe.
pub fn key_of(call: &Call) -> Option<String> {
    match call {
        Call::ListSeries { .. }
        | Call::ListAiring { .. }
        | Call::ListLibrary { .. }
        | Call::ListFeed { .. }
        | Call::ListMetadata { .. } => serde_json::to_string(call).ok(),
        _ => None,
    }
}

/// How many answers are kept: every distinct read is a key (a search per word typed), and
/// an old one is only good as a stale first paint.
const KEEP: usize = 48;

pub struct ReadCache<V> {
    generation: u64,
    /// key -> (the generation it was read under, when, the answer, whether it outlives its
    /// generation). Only an answer put with `keep_stale` stays, no longer fresh, as the
    /// key's last known answer (`stale`); every other one goes when the generation moves.
    entries: HashMap<String, (u64, Instant, V, bool)>,
    enabled: bool,
}

impl<V: Clone> ReadCache<V> {
    pub fn new(enabled: bool) -> Self {
        ReadCache {
            generation: 0,
            entries: HashMap::new(),
            enabled,
        }
    }

    fn advance(&mut self, generation: u64) {
        if generation > self.generation {
            self.entries
                .retain(|_, (g, _, _, keep)| *keep || *g >= generation);
            self.generation = generation;
        }
    }

    pub fn get(&mut self, key: &str, generation: u64, now: Instant) -> Option<V> {
        if !self.enabled || paused() {
            return None;
        }
        self.advance(generation);
        match self.entries.get(key) {
            Some((g, at, v, _)) if *g == generation && now.duration_since(*at) < MAX_AGE => {
                Some(v.clone())
            }
            _ => None,
        }
    }

    /// The last answer kept under `key` with `keep_stale`, fresh or not: what a page shows
    /// at once while a fresh read runs elsewhere (Door::library_view).
    pub fn stale(&self, key: &str) -> Option<V> {
        if !self.enabled || paused() {
            return None;
        }
        self.entries
            .get(key)
            .filter(|(_, _, _, keep)| *keep)
            .map(|(_, _, v, _)| v.clone())
    }

    /// `generation` is the one read *before* the answer was computed: a change that landed
    /// during the read moved the number past it, so the answer is never kept as current.
    /// With `keep_stale` it is still kept as the last known answer when it is newer than
    /// what the key holds: the Door's background re-read lands after the generation moved.
    pub fn put(&mut self, key: String, generation: u64, now: Instant, v: V, keep_stale: bool) {
        if !self.enabled || paused() {
            return;
        }
        if generation < self.generation {
            let newer = self
                .entries
                .get(&key)
                .is_none_or(|(g, _, _, _)| *g <= generation);
            if keep_stale && newer {
                self.entries.insert(key, (generation, now, v, true));
            }
            return;
        }
        self.advance(generation);
        if self.entries.len() >= KEEP && !self.entries.contains_key(&key) {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, (_, at, _, _))| *at)
                .map(|(k, _)| k.clone());
            if let Some(k) = oldest {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(key, (generation, now, v, keep_stale));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::*;
    use std::sync::Arc;

    fn series_call() -> Call {
        Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        }
    }

    #[test]
    fn only_whole_library_reads_are_kept_and_arguments_tell_them_apart() {
        assert!(key_of(&series_call()).is_some());
        assert!(
            key_of(&Call::ListAiring {
                offset: 0,
                limit: 9
            })
            .is_some()
        );
        assert!(key_of(&Call::GetSeries { series: 1 }).is_none());
        assert!(key_of(&Call::ListWatching).is_none());
        assert!(
            key_of(&Call::SetHidden {
                series: 1,
                hidden: true
            })
            .is_none()
        );
        assert_ne!(
            key_of(&Call::ListAiring {
                offset: 0,
                limit: 9
            }),
            key_of(&Call::ListAiring {
                offset: 0,
                limit: 10
            })
        );
    }

    #[test]
    fn a_new_generation_or_old_age_drops_an_answer() {
        let t = Instant::now();
        let mut c = ReadCache::new(true);
        c.put("k".into(), 3, t, 1, false);
        assert_eq!(c.get("k", 3, t + Duration::from_secs(1)), Some(1));
        assert_eq!(c.get("k", 3, t + MAX_AGE), None, "too old");
        c.put("k".into(), 3, t, 1, false);
        assert_eq!(c.get("k", 4, t), None, "the generation moved");
        // An answer computed under a generation that has since moved is never kept.
        c.put("k".into(), 3, t, 2, false);
        assert_eq!(c.get("k", 4, t), None);
        // Not fresh, but still the last known answer
        c.put("k".into(), 4, t, 5, true);
        assert_eq!(c.get("k", 6, t), None);
        assert_eq!(c.stale("k"), Some(5));
        // A plain answer goes with its generation
        c.put("p".into(), 6, t, 7, false);
        assert_eq!(c.get("p", 8, t), None);
        assert_eq!(c.stale("p"), None);
        // A re-read from an older generation still replaces an older kept answer
        c.put("k".into(), 5, t, 9, true);
        assert_eq!(c.stale("k"), Some(9));
        let mut off = ReadCache::new(false);
        off.put("k".into(), 1, t, 1, false);
        assert_eq!(off.get("k", 1, t), None, "a disabled cache keeps nothing");
    }

    /// The read path the Door takes, against a real core: the same read twice in one
    /// generation is one load, and a write's change event makes the next read fresh.
    #[test]
    fn a_change_event_invalidates_a_kept_read() {
        let dir = tempfile::tempdir().unwrap();
        let paths = CorePaths::under(dir.path());
        let secrets = anibeam_core::trackers::Secrets::file_only(paths.secrets_path());
        let core: Arc<Core> = Core::open_with_secrets(paths, secrets).unwrap();
        let mut cache: ReadCache<serde_json::Value> = ReadCache::new(true);
        let read = |cache: &mut ReadCache<serde_json::Value>, call: Call| {
            let key = key_of(&call).unwrap();
            let generation = core.generation();
            if let Some(v) = cache.get(&key, generation, Instant::now()) {
                return (v, true);
            }
            let v = crate::json::dispatch(&core, call);
            cache.put(key, generation, Instant::now(), v.clone(), false);
            (v, false)
        };
        let all = |v: &serde_json::Value| v["reply"]["series"].as_array().unwrap().len();
        let hidden_call = || Call::ListSeries {
            tab: Tab::Hidden,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: true,
        };

        // A series in the store, as a scan would leave it.
        let series = core
            .store()
            .write(|c| {
                c.execute("INSERT INTO sources (path, available, added_at) VALUES ('/lib', 1, 0)", [])?;
                c.execute(
                    "INSERT INTO series (source_id, kind, path, folder_name, added_at) VALUES (1, 'show', '/lib/A', 'A', 0)",
                    [],
                )?;
                Ok(c.last_insert_rowid() as u64)
            })
            .unwrap();
        // The direct insert emitted nothing, so this is the first read of its generation.
        let (first, hit) = read(&mut cache, hidden_call());
        assert!(!hit);
        assert_eq!(all(&first), 0);
        let (again, hit) = read(&mut cache, hidden_call());
        assert!(hit, "the same read in the same generation is kept");
        assert_eq!(again, first);

        core.call(Call::SetHidden {
            series,
            hidden: true,
        })
        .unwrap();
        let (after, hit) = read(&mut cache, hidden_call());
        assert!(!hit, "SetHidden's SeriesChanged moved the generation");
        assert_eq!(all(&after), 1, "and the read sees the hidden series");
        core.shutdown();
    }
}
