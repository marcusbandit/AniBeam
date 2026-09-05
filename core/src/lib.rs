//! AniBeam's core. One crate, every rule; every shell uses it through the
//! contract in `contract`.

pub mod contract;
pub mod events;
pub mod paths;
pub mod store;
pub mod time;

pub use contract::*;
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
