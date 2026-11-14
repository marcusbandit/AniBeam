//! Pure helpers and constants that more than one team reads. Nothing here
//! imports the rest of the core, only the contract crate below it, so a
//! team that needs one of these no longer imports another team for it, and
//! the folder can become the foundation crate as it is (#854).
//!
//! `secrets` and `services` are the exceptions so far. They reach modules
//! that are foundation too but still sit at the crate root: `sync` and
//! `atomic_file` for the one, the store, the bus, the jobs registry and the
//! paths for the other. `services` also names net's `Http` trait, which
//! comes down here when the crates are pulled out.

pub mod above;
pub mod machine;
pub mod magnet;
pub mod names;
pub mod secrets;
pub mod services;
pub mod size;
pub mod titles;
pub mod trust;
pub mod watch;
