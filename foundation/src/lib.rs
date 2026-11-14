//! The bottom of Anibeam's core (#854): the plumbing every team shares and the
//! small helpers more than one team reads. The store and its migrations, the
//! event bus and the activity log, the jobs registry, the paths, the clock,
//! the preferences, the profiles, the secrets, the redaction of what the log
//! keeps, and `Services`, which holds them for the teams. Nothing here names a
//! team; it depends on the contract crate below it and on no part of the core.
//!
//! The core hands each module on under the name it had there, so
//! `anibeam_core::store`, `anibeam_core::events` and the rest resolve as
//! before, and `anibeam_core::foundation::...` reaches this crate as a whole.

pub mod above;
pub mod atomic_file;
pub mod events;
pub mod html;
pub mod http;
pub mod jobs;
pub mod machine;
pub mod magnet;
pub mod names;
pub mod paths;
pub mod percent;
pub mod prefs;
pub mod profiles;
pub mod range;
pub mod redact;
pub mod relations;
pub mod secrets;
pub mod services;
pub mod size;
pub mod store;
pub mod sync;
pub mod time;
pub mod titles;
pub mod trust;
pub mod watch;
pub mod words;

/// The describe string computed by build.rs: `2.0.0.r14.g1a2b3c4` at a commit,
/// `CARGO_PKG_VERSION` when git is absent. The shell and the CLI print it, and the
/// provider clients send it in their user agent.
pub const VERSION: &str = env!("ANIBEAM_VERSION");

/// The version this build ships as: `2.4.0` on a stable release, `2.4.0-nightly.20261030.12`
/// on a nightly, `2.4.0-pr.280.3` on a pull request's test build, and the workspace version
/// on a build CI did not make. The update check reads its track and its age from this.
pub const RELEASE: &str = env!("ANIBEAM_RELEASE");
