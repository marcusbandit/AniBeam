//! One sync: hello, then the snapshot, then the posters whose tag changed. When the host
//! does not answer, AniList fills in airing times instead, at most once an hour.
//!
//! Everything here blocks; the app runs it on its own thread.

use std::collections::HashSet;

use crate::address::HostAddress;
use crate::anilist;
use crate::api::Hello;
use crate::client::{HostClient, HostError};
use crate::schedule;
use crate::store::{AniListCache, Cache, Store};

/// AniList is asked at most this often, and only while the host is away.
pub const ANILIST_EVERY: i64 = 3600;

pub struct Syncer {
    agent: ureq::Agent,
    store: Store,
}

impl Syncer {
    pub fn new(store: Store) -> Self {
        Syncer {
            agent: crate::client::agent(),
            store,
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    fn client(&self, address: &HostAddress) -> HostClient {
        HostClient::new(self.agent.clone(), address.clone())
    }

    /// Settings' "Test connection": only the hello.
    pub fn test(&self, address: &HostAddress) -> Result<Hello, HostError> {
        self.client(address).hello()
    }

    /// Hello and snapshot, stored before anything else, so the screens can show it while
    /// the posters are still coming. The list of other AniBeams is carried over from the
    /// last sync with this host; [`Syncer::fetch_peers`] brings it up to date afterwards.
    pub fn fetch_snapshot(&self, address: &HostAddress, now: i64) -> Result<Cache, HostError> {
        let client = self.client(address);
        let hello = client.hello()?;
        let snapshot = client.snapshot()?;
        let offers_peers = hello
            .features
            .iter()
            .any(|f| f == crate::api::FEATURE_PEERS);
        let cache = Cache {
            address: address.display(),
            peers: carried(self.store.cache(), &address.display(), offers_peers),
            host: hello.host,
            host_version: hello.version,
            synced_at: now,
            snapshot,
        };
        if let Err(e) = self.store.save_cache(&cache) {
            return Err(HostError::Bad(tr!("sync-save-failed", why = e.to_string())));
        }
        Ok(cache)
    }

    /// Asks the host for the other AniBeams it sees and stores the answer. Returns the
    /// cache when the list changed. The list is a bonus: a host that cannot answer it
    /// leaves the last list in place.
    pub fn fetch_peers(&self, address: &HostAddress, cache: &Cache) -> Option<Cache> {
        let peers = updated(&cache.peers, self.client(address).peers())?;
        let cache = Cache {
            peers,
            ..cache.clone()
        };
        self.store.save_cache(&cache).ok()?;
        Some(cache)
    }

    /// Fetches every poster whose tag differs from the one on disk, calling `fetched` with
    /// each series id as its image lands, and deletes posters of series that are gone.
    /// A poster that fails is left for the next sync.
    pub fn fetch_posters(
        &self,
        address: &HostAddress,
        cache: &Cache,
        mut fetched: impl FnMut(u64),
    ) {
        let client = self.client(address);
        let mut keep = HashSet::new();
        for series in &cache.snapshot.series {
            let Some(tag) = &series.poster else { continue };
            keep.insert(series.id);
            if self.store.poster_tag(series.id).as_deref() == Some(tag.as_str()) {
                continue;
            }
            match client.poster(series.id) {
                Ok((bytes, _)) => {
                    if self.store.save_poster(series.id, tag, &bytes).is_ok() {
                        fetched(series.id);
                    }
                }
                // The host went away mid-sync: the rest would only time out one by one.
                Err(HostError::Unreachable(_)) => break,
                Err(_) => {}
            }
        }
        self.store.prune_posters(&keep);
    }

    /// Asks AniList for the cached shows' airing times when the last ask was an hour or
    /// more ago. Returns the new AniList cache when it asked, answered or not.
    pub fn refresh_anilist_if_due(&self, cache: &Cache, now: i64) -> Option<AniListCache> {
        let mut anilist = self.store.anilist();
        if now - anilist.attempted_at < ANILIST_EVERY {
            return None;
        }
        let ids = schedule::anilist_ids(cache);
        if ids.is_empty() {
            return None;
        }
        anilist.attempted_at = now;
        match anilist::fetch(&self.agent, &ids) {
            Ok(shows) => {
                anilist.fetched_at = now;
                anilist.error = None;
                anilist.shows = shows;
            }
            Err(why) => anilist.error = Some(why),
        }
        let _ = self.store.save_anilist(&anilist);
        Some(anilist)
    }
}

/// The peers a new snapshot starts from: the last sync's, when it was this host and the
/// host still offers the list.
fn carried(last: Option<Cache>, address: &str, offers: bool) -> Vec<crate::api::PeerHost> {
    match last {
        Some(last) if offers && last.address == address => last.peers,
        _ => Vec::new(),
    }
}

/// The new list when the host answered one that differs from `now`.
fn updated(
    now: &[crate::api::PeerHost],
    answer: Result<Vec<crate::api::PeerHost>, HostError>,
) -> Option<Vec<crate::api::PeerHost>> {
    answer.ok().filter(|list| list != now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::PeerHost;

    fn peer(name: &str) -> PeerHost {
        PeerHost {
            name: name.into(),
            url: format!("http://{name}:8790"),
            version: "2.4.0".into(),
        }
    }

    fn cache_at(address: &str, peers: Vec<PeerHost>) -> Cache {
        Cache {
            address: address.into(),
            peers,
            ..Cache::default()
        }
    }

    #[test]
    fn a_failed_list_keeps_the_last_one() {
        let now = vec![peer("laptop")];
        let failed = Err(HostError::Bad("500".into()));
        assert_eq!(updated(&now, failed), None);
        assert_eq!(updated(&now, Ok(now.clone())), None, "nothing changed");
        assert_eq!(
            updated(&now, Ok(vec![])),
            Some(vec![]),
            "an answered empty list is real"
        );
    }

    #[test]
    fn a_new_snapshot_keeps_the_list_only_from_the_same_host() {
        let last = || Some(cache_at("desktop:8790", vec![peer("laptop")]));
        assert_eq!(carried(last(), "desktop:8790", true), vec![peer("laptop")]);
        assert!(carried(last(), "study:8790", true).is_empty());
        assert!(
            carried(last(), "desktop:8790", false).is_empty(),
            "a host that stopped offering the list has none"
        );
        assert!(carried(None, "desktop:8790", true).is_empty());
    }
}
