//! The contract every shell talks through: calls, replies, records, events and errors. It
//! lives in its own crate, `anibeam-contract`, below the core (#855). This module hands
//! its modules on, so `anibeam_core::contract::...` and, through the glob in `lib.rs`,
//! `anibeam_core::...` name the same items they always have.
//!
//! The modules are named one by one rather than as `anibeam_contract::*`, so the contract's
//! uniffi plumbing (its `UniFfiTag`, `uniffi_reexport_scaffolding!`) does not sit here beside
//! the core's own, and `i18n` stays at the crate root. A module added to the contract is
//! added here too; a shell that names one of its types fails to build until it is.

pub use anibeam_contract::{
    calls, column_strings, downloads, enums, error, events, records, words,
};
pub use anibeam_contract::{
    calls::*, downloads::*, enums::*, error::*, events::*, records::*, words::*,
};
