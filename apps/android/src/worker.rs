//! The sync thread. It owns the network and every write to the store; the UI thread only
//! sends it commands and gets events back through the Slint event loop.

use std::sync::mpsc::{Receiver, Sender};

use anibeam_companion::address::HostAddress;
use anibeam_companion::client::HostError;
use anibeam_companion::store::{AniListCache, Cache, Settings, Store};
use anibeam_companion::sync::Syncer;

pub enum Command {
    /// Sync with the saved address.
    Sync,
    /// Save this address, then sync with it.
    SaveAndSync(HostAddress),
    /// Only ask the host who it is.
    Test(HostAddress),
}

pub enum Event {
    SyncStarted,
    Snapshot(Cache),
    Poster(u64, crate::posters::Decoded),
    AniList(AniListCache),
    /// The sync ended; `None` when the host answered.
    SyncDone { at: i64, error: Option<HostError> },
    TestStarted,
    TestDone(Result<String, String>),
}

/// Starts the thread. `deliver` hands each event to the UI thread.
pub fn start(
    store: Store,
    inbox: Receiver<Command>,
    deliver: impl Fn(Event) + Send + 'static,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("anibeam-sync".into())
        .spawn(move || Worker { syncer: Syncer::new(store), deliver }.run(inbox))
        .expect("the sync thread starts")
}

struct Worker<F: Fn(Event)> {
    syncer: Syncer,
    deliver: F,
}

impl<F: Fn(Event)> Worker<F> {
    fn run(self, inbox: Receiver<Command>) {
        // Posters already on disk first, so a cached library has its pictures at once.
        if let Some(cache) = self.syncer.store().cache() {
            for series in &cache.snapshot.series {
                self.send_poster(series.id);
            }
        }
        while let Ok(first) = inbox.recv() {
            // Several sync requests in a row (launch, resume, the timer) make one sync.
            let mut sync = false;
            for command in std::iter::once(first).chain(inbox.try_iter()) {
                match command {
                    Command::Sync => sync = true,
                    Command::SaveAndSync(address) => {
                        let _ = self.syncer.store().save_settings(&Settings { address: address.display() });
                        sync = true;
                    }
                    Command::Test(address) => self.test(&address),
                }
            }
            if sync {
                self.sync();
            }
        }
    }

    fn send_poster(&self, series_id: u64) {
        if let Some(decoded) = self.syncer.store().poster(series_id).and_then(|b| crate::posters::decode(&b)) {
            (self.deliver)(Event::Poster(series_id, decoded));
        }
    }

    fn test(&self, address: &HostAddress) {
        (self.deliver)(Event::TestStarted);
        let result = self
            .syncer
            .test(address)
            .map(|hello| format!("Reached {}, AniBeam {}.", hello.host, hello.version))
            .map_err(|e| e.to_string());
        (self.deliver)(Event::TestDone(result));
    }

    fn sync(&self) {
        let store = self.syncer.store();
        let address = HostAddress::parse(&store.settings().address)
            .unwrap_or_else(|_| HostAddress::parse(anibeam_companion::address::DEFAULT_ADDRESS).unwrap());
        (self.deliver)(Event::SyncStarted);
        let now = anibeam_companion::now();
        match self.syncer.fetch_snapshot(&address, now) {
            Ok(cache) => {
                eprintln!(
                    "AniBeam sync: {} answered, {} series, {} subscriptions",
                    address.display(),
                    cache.snapshot.series.len(),
                    cache.snapshot.subscriptions.len()
                );
                (self.deliver)(Event::Snapshot(cache.clone()));
                self.syncer.fetch_posters(&address, &cache, |id| self.send_poster(id));
                crate::airing::sync(&address);
                (self.deliver)(Event::SyncDone { at: now, error: None });
            }
            Err(error) => {
                eprintln!("AniBeam sync: {} did not: {error:?}", address.display());
                if let Some(cache) = store.cache()
                    && let Some(anilist) = self.syncer.refresh_anilist_if_due(&cache, now)
                {
                    eprintln!("AniBeam sync: asked AniList, {} shows, error {:?}", anilist.shows.len(), anilist.error);
                    (self.deliver)(Event::AniList(anilist));
                }
                (self.deliver)(Event::SyncDone { at: now, error: Some(error) });
            }
        }
    }
}

/// Commands go in through this; cloning it is cheap.
pub type Commands = Sender<Command>;
