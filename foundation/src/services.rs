//! The plumbing every team shares (#854): the store, the event bus, the jobs
//! registry, the paths, the runtime handle, the one transport and the
//! secrets. `Core::open` builds it once through [`Services::open`] and keeps
//! it as an `Arc`, and a function that needs nothing more takes this instead
//! of the whole core. A team's tests open it the same way (`testing`).

use std::sync::Arc;
use std::time::Duration;

use crate::events::EventBus;
use crate::http::Http;
use crate::jobs::Jobs;
use crate::paths::CorePaths;
use crate::secrets::{KEYRING_UNAVAILABLE, Secrets};
use crate::store::Store;
use anibeam_contract::{CoreError, EventBody, Stage};

/// The runtime's worker threads.
const WORKERS: usize = 4;

/// How long `close` waits for the runtime's tasks to see the jobs cancelled
/// before it stops waiting and closes the store anyway.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

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
    /// Opens what every team shares over `paths`, in this order: the store,
    /// the bus, the image cache's directory, the runtime and the jobs over
    /// it (#997). The runtime comes back beside the services: its owner keeps
    /// it, and shuts it down once the jobs are cancelled.
    pub fn open(
        paths: CorePaths,
        http: Arc<dyn Http>,
        secrets: Arc<Secrets>,
    ) -> Result<(Arc<Services>, tokio::runtime::Runtime), CoreError> {
        let store = Store::open(&paths.db_path())?;
        let bus = EventBus::new(store.clone())?;
        // Creating the cache directory is opening, the same as creating the
        // data directory the database file lives in. Before the runtime, so
        // that nothing after it can fail and drop a live runtime here.
        let images_dir = paths.images_dir();
        std::fs::create_dir_all(&images_dir)
            .map_err(|e| CoreError::io_at(images_dir.to_string_lossy(), e))?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(WORKERS)
            .thread_name("anibeam-core")
            .enable_all()
            .build()
            .map_err(|e| CoreError::internal(format!("runtime: {e}")))?;
        let handle = runtime.handle().clone();
        let jobs = Jobs::new(handle.clone(), bus.clone());
        let services = Arc::new(Services {
            paths,
            store,
            bus,
            jobs,
            handle,
            http,
            secrets,
        });
        Ok((services, runtime))
    }

    /// Closes what `open` opened, once the caller has cancelled the jobs: the
    /// runtime, if it is still there, gets [`SHUTDOWN_GRACE`] for its tasks
    /// to see that, then the store is checkpointed and closed.
    pub fn close(&self, runtime: Option<tokio::runtime::Runtime>) {
        if let Some(runtime) = runtime {
            // Two seconds is the whole budget for the tasks to observe the
            // cancellation. A shell is on its way out by now, and an ffprobe
            // or a keyring call that will not stop must not be the reason the
            // window hangs around.
            runtime.shutdown_timeout(SHUTDOWN_GRACE);
        }
        self.store.checkpoint();
        self.store.close();
    }

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
