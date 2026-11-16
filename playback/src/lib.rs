//! The playback rules: what a session is, what a tick means, and the four
//! things the core does with one, plus what every player picks the same way:
//! the mpv options Anibeam owns and the track pick. The seek bar, the keys and
//! the rest of what a player draws are the shell's.
//!
//! The playback team (#854) is this crate: the session rules here, Discord
//! presence in `presence`, casting in `cast` behind the `cast` feature, and
//! the watch together room in `room`, the top of the team, which nothing
//! below it names. It sits above the downloads and names nothing of the app.
//! The core hands it on as `anibeam_core::playback`, `presence`, `cast` and
//! `room`.

pub mod cast;
pub mod options;
pub mod presence;
pub mod room;
pub mod session;
pub mod skip;
mod team;
pub mod tracks;

pub use team::{Follower, Playback};
