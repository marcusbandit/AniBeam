//! The downloads team over the teams below it, for the team's own tests
//! (#854): what `Core::open` builds and links for it, and nothing above it, so
//! the tests can go with the team when it leaves the core.

use std::path::Path;
use std::sync::{Arc, Weak};

use anibeam_foundation::testing::Running;

use super::Downloads;
use crate::foundation::services::Services;
use crate::net::{Http, Net};
use crate::releases::{Releases, Searcher};

pub(crate) struct Teams {
    pub(crate) services: Arc<Services>,
    pub(crate) releases: Arc<Releases>,
    pub(crate) downloads: Arc<Downloads>,
    running: Running,
}

impl Teams {
    /// Over `dir`, with `http` for the network. The releases ask the
    /// downloads about endpoints, and the downloads hear a moved schedule on
    /// the bus, as in `Core::open`.
    pub(crate) fn open(dir: &Path, http: Arc<dyn Http>) -> Teams {
        let running = Running::open(dir, http);
        let services = running.services.clone();
        let net = Arc::new(Net::new(services.clone(), crate::library::cards::cards_for));
        let releases = Arc::new(Releases::new(services.clone()));
        let downloads = Arc::new(Downloads::new(services.clone(), net));
        releases
            .searcher
            .set(Arc::downgrade(&downloads) as Weak<dyn Searcher>);
        super::watch::listen(&downloads);
        Teams {
            services,
            releases,
            downloads,
            running,
        }
    }

    /// What `Core::shutdown` does for these teams: the sidecar stops, then
    /// the services.
    pub(crate) fn shutdown(self) {
        self.downloads.stop();
        self.running.shutdown();
    }
}
