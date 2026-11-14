//! Anibeam's core: every rule, used by every shell through the contract in
//! `contract`, which hands on the `anibeam-contract` crate below it. The
//! plumbing every team shares is the `anibeam-foundation` crate, handed on
//! here module by module under the names it had in the core.

pub(crate) mod act;
pub mod agent;
pub mod aired;
pub mod airing_now;
pub(crate) mod app;
pub mod calendar;
pub mod cast;
pub mod changelog;
pub mod contract;
pub mod core;
pub mod downloads;
pub mod events;
pub mod feed;
pub mod first_run;
pub mod franchise;
pub mod host;
pub mod images;
pub mod launch;
pub mod library;
pub mod metadata;
/// The network layer. Not part of the contract and not exported to any
/// shell; it is public only so the integration tests can reach `FakeHttp`
/// and `Core::open_with_http`.
#[doc(hidden)]
pub mod net;
pub mod paths;
pub mod phone;
pub mod playback;
pub mod presence;
pub(crate) mod process;
pub mod recommended;
pub mod releases;
pub mod repos;
pub mod room;
pub mod stats;
pub mod trackers;
pub mod transfer;
pub mod update;

/// The foundation crate as a whole, so `crate::foundation::services::Services`
/// and the other helpers that sat under `core/src/foundation/` keep their paths.
pub use anibeam_foundation as foundation;
pub use anibeam_foundation::{
    RELEASE, VERSION, atomic_file, html, jobs, prefs, profiles, redact, store, time, words,
};
pub(crate) use anibeam_foundation::{percent, sync};

pub use anibeam_contract::i18n;
pub use contract::*;
pub use core::Core;
pub use paths::CorePaths;

uniffi::setup_scaffolding!();
