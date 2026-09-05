//! The trackers: AniList and MAL, where their credentials live and what
//! the core does with them.

pub mod accounts;
pub mod cache;
pub mod oauth;
pub mod secrets;

pub use secrets::{Secrets, StoreKind};
