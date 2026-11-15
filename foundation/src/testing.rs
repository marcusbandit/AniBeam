//! The shared services over a test's own directory, for a team's tests that
//! need a store and a runtime but not the whole core (#854). Not part of the
//! contract and not for the app: a crate's own `#[cfg(test)]` code is all
//! that calls it, which is why it is compiled into the crate at all.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::http::Http;
use crate::paths::CorePaths;
use crate::secrets::Secrets;
use crate::services::Services;

/// How long `shutdown` lets the runtime's tasks see the cancellation, as
/// the core's own shutdown does.
const GRACE: Duration = Duration::from_secs(2);

/// The services, opened the way the core opens them, and the runtime they
/// run on.
pub struct Running {
    pub services: Arc<Services>,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Running {
    /// Over `dir`, with `http` for the network and the secrets in a file
    /// there, never the machine's keyring.
    pub fn open(dir: &Path, http: Arc<dyn Http>) -> Running {
        let paths = CorePaths::under(dir);
        let secrets = Secrets::file_only(paths.secrets_path());
        let (services, runtime) = Services::open(paths, http, secrets).unwrap();
        Running {
            services,
            runtime: Some(runtime),
        }
    }

    /// As the core's shutdown ends, once its teams have stopped: the jobs
    /// are cancelled, the runtime gets its grace, and the store is closed.
    pub fn shutdown(mut self) {
        self.services.jobs.cancel_all();
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(GRACE);
        }
        self.services.store.checkpoint();
        self.services.store.close();
    }
}

impl Drop for Running {
    /// A runtime dropped from inside its own workers panics, so one still
    /// here goes the background way, as the core's does.
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
