//! The bottom of Anibeam's core (#854): the plumbing every team shares and the
//! small helpers more than one team reads. The store and its migrations, the
//! event bus and the activity log, the jobs registry, the paths, the clock,
//! the preferences, the profiles, the secrets, the redaction of what the log
//! keeps, starting a helper program, and `Services`, which holds them for the
//! teams. Nothing here names a team. It depends on the contract crate below
//! it, the host API's records and the room name rules, and on no part of the
//! core.
//!
//! The core hands each module on under the path it had there:
//! `anibeam_core::store`, `anibeam_core::events` and the rest at its root,
//! `anibeam_core::foundation::services` and the others that sat under
//! `core/src/foundation/` in its `foundation` module.

pub mod above;
pub mod atomic_file;
pub mod blocking;
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
pub mod process;
pub mod profiles;
pub mod range;
pub mod redact;
pub mod relations;
pub mod secrets;
pub mod services;
pub mod size;
pub mod store;
pub mod sync;
pub mod testing;
pub mod time;
pub mod titles;
pub mod trust;
pub mod watch;
pub mod words;
