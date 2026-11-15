//! The downloads team over the teams below it, for the team's own tests
//! (#854): each built as `Core::open` builds it, with the two links of
//! `Core::open`'s that these tests lean on, and nothing above the downloads,
//! so the tests can go with the team when it leaves the core.

use std::path::Path;
use std::sync::{Arc, Weak};

use anibeam_foundation::testing::Running;

use super::Downloads;
use anibeam_foundation::services::Services;
use anibeam_net::{Http, Net};
use anibeam_releases::{Releases, Searcher};

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
        let net = Arc::new(Net::new(
            services.clone(),
            anibeam_library::cards::cards_for,
        ));
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
