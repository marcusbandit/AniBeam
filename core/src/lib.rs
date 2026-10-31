//! AniBeam's core. One crate, every rule; every shell uses it through the
//! contract in `contract`.

pub mod airing_now;
pub mod cast;
pub mod changelog;
pub mod contract;
pub mod core;
pub mod downloads;
pub mod events;
pub mod feed;
pub mod franchise;
pub mod host;
pub mod images;
pub mod jobs;
pub mod launch;
pub mod library;
pub mod metadata;
/// The network layer. Not part of the contract and not exported to any
/// shell; it is public only so the integration tests can reach `FakeHttp`
/// and `Core::open_with_http`.
#[doc(hidden)]
pub mod net;
pub mod paths;
pub(crate) mod percent;
pub mod phone;
pub mod playback;
pub mod prefs;
pub mod presence;
pub(crate) mod process;
pub mod releases;
pub mod repos;
pub mod store;
pub mod subscriptions;
pub(crate) mod sync;
pub mod time;
pub mod trackers;
pub mod transfer;
pub mod update;
pub mod words;

pub use contract::*;
pub use core::Core;
pub use paths::CorePaths;

/// The describe string computed by build.rs: `2.0.0.r14.g1a2b3c4` at a commit,
/// `CARGO_PKG_VERSION` when git is absent. The shell and the CLI print it.
pub const VERSION: &str = env!("ANIBEAM_VERSION");

/// The version this build ships as: `2.4.0` on a stable release, `2.4.0-nightly.20261030.12`
/// on a nightly, `2.4.0-pr.280.3` on a pull request's test build, and the workspace version
/// on a build CI did not make. The update check reads its track and its age from this.
pub const RELEASE: &str = env!("ANIBEAM_RELEASE");

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_not_empty() {
        assert!(!super::VERSION.is_empty());
    }
}
