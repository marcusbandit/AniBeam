//! The releases team (#854): endpoint repos, the endpoints installed from
//! them, and the feed.

use std::sync::Arc;

use crate::repos::Pending;
use anibeam_contract::{CoreError, DownloadsEngine};
use anibeam_foundation::above::Above;
use anibeam_foundation::services::Services;

pub struct Releases {
    pub services: Arc<Services>,
    /// Repo indexes fetched for a preview and not trusted yet.
    pub(crate) pending: Pending,
    /// The engine an endpoint change goes to, and what anirss lists: the
    /// downloads, through the releases' own trait.
    pub searcher: Above<dyn Searcher>,
}

impl Releases {
    pub fn new(services: Arc<Services>) -> Releases {
        Releases {
            services,
            pending: Pending::default(),
            searcher: Above::default(),
        }
    }
}

/// What the releases ask of the team that searches their endpoints, the
/// downloads above them (#854). The downloads implement it and `Core::open`
/// links them to `Releases::searcher`, so the releases name no team above
/// them.
pub trait Searcher: Send + Sync {
    /// The engine the downloads settings pick. A store that cannot be read
    /// is an error, not a quiet fall back to one or the other.
    fn engine(&self) -> Result<DownloadsEngine, CoreError>;
    /// The [`AnirssView`] of the anirss last read. None until one has been
    /// read, and always on the phone, which has none.
    fn anirss_view(&self) -> Option<AnirssView>;
    /// What is installed from repos changed (#143): a running anirss gets
    /// it, and the page reads the endpoint list again.
    fn endpoints_changed(self: Arc<Self>);
}

/// The anirss the downloads last read: its version, whether it lists
/// `endpoint_writes`, the names of its endpoints, whether the desktop puts
/// the installed endpoints into it at all, and the oldest version that takes
/// them.
pub struct AnirssView {
    pub version: String,
    pub endpoint_writes: bool,
    pub endpoints: Vec<String>,
    pub sync: EndpointSync,
    pub min_version: &'static str,
}

/// Whether the endpoints installed from repos go into the anirss the desktop reads, and
/// why not when they do not (#499).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EndpointSync {
    /// Anibeam's own sidecar, in a run that may change the owner's setup.
    #[default]
    Syncs,
    /// A server the person runs themselves (Remote mode), which keeps its own config.
    Theirs,
    /// A sandbox root or a drive, which never writes into the owner's real anirss, or a
    /// run whose settings could not be read, which writes nothing it cannot be sure of.
    Sandbox,
}
