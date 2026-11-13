//! The contract every shell talks through: calls, replies, records, events and errors. It
//! lives in its own crate, `anibeam-contract`, below the core (#855). This module hands all
//! of it on, so `anibeam_core::contract::...` and, through the glob in `lib.rs`,
//! `anibeam_core::...` name the same items they always have.

pub use anibeam_contract::{
    calls, column_strings, downloads, enums, error, events, records, words,
};
pub use anibeam_contract::{
    calls::*, downloads::*, enums::*, error::*, events::*, records::*, words::*,
};
