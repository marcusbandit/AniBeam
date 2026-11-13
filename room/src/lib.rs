//! Watch together rooms (#846): everyone in a room plays their own copy of the
//! episode, and the room keeps the play state, who is in it and what they watch.
//!
//! This crate has no I/O. [`proto`] is the wire format, [`Room`] the state
//! machine a relay drives, [`code`] the room codes and keys, and [`page`] the
//! join page a shared link opens. The clock and the randomness come from the
//! caller, so the same code runs in the native relay, the Cloudflare Worker and
//! the core. The design is `docs/superpowers/specs/2026-11-13-watch-together-rooms.md`.

pub mod code;
pub mod page;
pub mod proto;
pub mod room;
mod siphash;

pub use room::{ConnId, Out, Room, To};
