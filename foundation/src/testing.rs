//! The shared services over a test's own directory, for a team's tests that
//! need a store and a runtime but not the whole core (#854). Not part of the
//! contract and not for the app: tests are all that call it, the teams' in
//! the crates above as well as this one's, which is why it is compiled in
//! rather than kept behind `#[cfg(test)]`, as `store::testing` is.

use std::path::Path;
use std::sync::Arc;

use crate::http::Http;
use crate::paths::CorePaths;
use crate::secrets::Secrets;
use crate::services::Services;

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
    /// are cancelled, then `Services::close`.
    pub fn shutdown(mut self) {
        self.services.jobs.cancel_all();
        self.services.close(self.runtime.take());
    }
}

impl Drop for Running {
    /// A test that never called `shutdown`: the jobs are cancelled, and the
    /// runtime goes the background way, since one dropped from inside its
    /// own workers panics.
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            self.services.jobs.cancel_all();
            runtime.shutdown_background();
        }
    }
}
