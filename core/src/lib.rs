//! Anibeam's core: every rule, used by every shell through the contract in
//! `contract`, which hands on the `anibeam-contract` crate below it. The
//! plumbing every team shares is the `anibeam-foundation` crate, the network
//! team is `anibeam-net`, the library team `anibeam-library`, and the releases,
//! metadata, trackers, downloads, playback and host teams `anibeam-releases`,
//! `anibeam-metadata`, `anibeam-trackers`, `anibeam-downloads`,
//! `anibeam-playback` and `anibeam-host`, each handed on here under the name it
//! had in the core.
//!
//! What is left in this crate is the app team: `Core` and its call router,
//! launch, update, the changelog, the agent, stats, the uniffi export and
//! `Subscription`.

pub(crate) mod act;
pub mod agent;
pub(crate) mod app;
pub mod changelog;
pub mod contract;
pub mod core;
pub mod events;
pub mod first_run;
pub mod foundation;
pub mod launch;
pub mod paths;
pub mod stats;
pub mod update;

pub(crate) use anibeam_foundation::sync;
pub use anibeam_foundation::{
    atomic_file, html, jobs, prefs, profiles, redact, store, time, words,
};

/// The network layer, in its own crate. Not part of the contract and not
/// exported to any shell; it is public only so the integration tests can
/// reach `FakeHttp` and `Core::open_with_http`.
#[doc(hidden)]
pub use anibeam_net as net;
pub use anibeam_net::{VERSION, images};

/// The library team, in its own crate above net.
pub use anibeam_library as library;

/// The releases team, in its own crate above library: the release logic, the
/// endpoint repos and the Feed page's reads.
pub use anibeam_releases as releases;
pub use anibeam_releases::{feed, repos};

/// The metadata team, in its own crate above library: matching, the
/// franchise graph, Recommended, Airing now, the aired episodes and the
/// calendar feed.
pub use anibeam_metadata as metadata;
pub use anibeam_metadata::{aired, airing_now, calendar, franchise, recommended};

/// The trackers team, in its own crate above metadata: AniList and MAL, the
/// list cache and the writes, Watching, On hold and Plan to watch.
pub use anibeam_trackers as trackers;

/// The downloads team, in its own crate above trackers: the backends, search,
/// rules, rolling downloads and the watch for new episodes, with the phone's
/// torrent engine behind its `phone` feature.
pub use anibeam_downloads as downloads;
pub use anibeam_downloads::phone;

/// The playback team, in its own crate above downloads: the session rules,
/// Discord presence, casting behind its `cast` feature, and the watch
/// together room.
pub use anibeam_playback as playback;
pub use anibeam_playback::{cast, presence, room};

/// The host team, in its own crate above downloads: the tailnet host the
/// phone reads and the library's export and import.
pub use anibeam_host as host;
pub use anibeam_host::transfer;

pub use anibeam_contract::i18n;
pub use contract::*;
pub use core::Core;
pub use paths::CorePaths;

/// The version this build ships as: `2.4.0` on a stable release, `2.4.0-nightly.20261030.12`
/// on a nightly, `2.4.0-pr.280.3` on a pull request's test build, and the workspace version
/// on a build CI did not make. The update check reads its track and its age from this.
pub const RELEASE: &str = env!("ANIBEAM_RELEASE");

uniffi::setup_scaffolding!();
