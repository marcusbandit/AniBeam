//! Pure helpers and constants that more than one team reads. Nothing here
//! imports the rest of the core, only the contract crate below it, so a
//! team that needs one of these no longer imports another team for it, and
//! the folder can become the foundation crate as it is (#854).

pub mod machine;
pub mod magnet;
pub mod names;
pub mod size;
pub mod titles;
pub mod trust;
pub mod watch;
