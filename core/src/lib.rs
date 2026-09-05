//! AniBeam's core. One crate, every rule; every shell uses it through the
//! contract in `contract`.

pub mod contract;
pub mod core;
pub mod events;
pub mod feed;
pub mod franchise;
pub mod images;
pub mod jobs;
pub mod library;
pub mod metadata;
/// The network layer. Not part of the contract and not exported to any
/// shell; it is public only so the integration tests can reach `FakeHttp`
/// and `Core::open_with_http`.
#[doc(hidden)]
pub mod net;
pub mod paths;
pub mod prefs;
pub mod store;
pub mod subscriptions;
pub mod time;
pub mod trackers;

pub use contract::*;
pub use core::Core;
pub use paths::CorePaths;

/// The describe string computed by build.rs: `2.0.0.r14.g1a2b3c4` at a commit,
/// `CARGO_PKG_VERSION` when git is absent. The shell and the CLI print it.
pub const VERSION: &str = env!("ANIBEAM_VERSION");

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_not_empty() {
        assert!(!super::VERSION.is_empty());
    }
}
