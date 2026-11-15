//! The plumbing every team shares (#854): the store, the event bus, the jobs
//! registry, the paths, the runtime handle, the one transport and the
//! secrets. `Core::open` builds it once and keeps it as an `Arc`, and a
//! function that needs nothing more takes this instead of the whole core.

use std::sync::Arc;

use crate::events::EventBus;
use crate::http::Http;
use crate::jobs::Jobs;
use crate::paths::CorePaths;
use crate::secrets::{KEYRING_UNAVAILABLE, Secrets};
use crate::store::Store;
use anibeam_contract::{EventBody, Stage};

pub struct Services {
    pub paths: CorePaths,
    pub store: Arc<Store>,
    pub bus: Arc<EventBus>,
    pub jobs: Arc<Jobs>,
    /// Cloned from the core's runtime at `open`, so any task can spawn work
    /// through it while the runtime itself stays with the core.
    pub handle: tokio::runtime::Handle,
    /// One transport for every provider, so a test swaps the whole network
    /// out with one `FakeHttp`. The provider clients each hold their own
    /// limiter over it; the IMDb refresh, which has nothing to pace, sends
    /// through it directly.
    pub http: Arc<dyn Http>,
    /// Where the tracker tokens live: the desktop keyring when there is
    /// one, `secrets.json` when there is not. Built by `open` without any
    /// I/O; which store it uses is settled on first use, warmed by `start`.
    /// Read through [`Services::secrets`] on a calling thread.
    pub secrets: Arc<Secrets>,
}

impl Services {
    /// The secrets facade, with the store choice settled. `start` warms
    /// that choice on a blocking thread, so a call arriving after launch
    /// finds it made; a call that beats the warm-up makes it here, on the
    /// shell's calling thread, which is where keyring work belongs. Either
    /// way the "no keyring" line is written once, by whoever finished the
    /// probe.
    pub fn secrets(&self) -> &Arc<Secrets> {
        if self.secrets.warm() {
            self.bus
                .info(Stage::System, KEYRING_UNAVAILABLE, EventBody::Notice);
        }
        &self.secrets
    }
}
