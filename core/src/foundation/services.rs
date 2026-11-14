//! The plumbing every team shares (#854): the store, the event bus, the jobs
//! registry, the paths, the runtime handle, the one transport and the
//! secrets. `Core::open` builds it once and keeps it as an `Arc`, and a
//! function that needs nothing more takes this instead of the whole core.

use std::sync::Arc;

use crate::contract::{EventBody, Stage};
use crate::events::EventBus;
use crate::foundation::secrets::{KEYRING_UNAVAILABLE, Secrets};
use crate::jobs::Jobs;
use crate::net::Http;
use crate::paths::CorePaths;
use crate::store::Store;

pub struct Services {
    pub(crate) paths: CorePaths,
    pub(crate) store: Arc<Store>,
    pub(crate) bus: Arc<EventBus>,
    pub(crate) jobs: Arc<Jobs>,
    /// Cloned from the core's runtime at `open`, so any task can spawn work
    /// through it while the runtime itself stays with the core.
    pub(crate) handle: tokio::runtime::Handle,
    /// One transport for every provider, so a test swaps the whole network
    /// out with one `FakeHttp`. The provider clients each hold their own
    /// limiter over it; the IMDb refresh, which has nothing to pace, sends
    /// through it directly.
    pub(crate) http: Arc<dyn Http>,
    /// Where the tracker tokens live: the desktop keyring when there is
    /// one, `secrets.json` when there is not. Built by `open` without any
    /// I/O; which store it uses is settled on first use, warmed by `start`.
    /// Read through [`Services::secrets`] on a calling thread.
    pub(crate) secrets: Arc<Secrets>,
}

impl Services {
    /// The secrets facade, with the store choice settled. `start` warms
    /// that choice on a blocking thread, so a call arriving after launch
    /// finds it made; a call that beats the warm-up makes it here, on the
    /// shell's calling thread, which is where keyring work belongs. Either
    /// way the "no keyring" line is written once, by whoever finished the
    /// probe.
    pub(crate) fn secrets(&self) -> &Arc<Secrets> {
        if self.secrets.warm() {
            self.bus
                .info(Stage::System, KEYRING_UNAVAILABLE, EventBody::Notice);
        }
        &self.secrets
    }
}
