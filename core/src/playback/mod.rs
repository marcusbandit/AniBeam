//! The playback rules: what a session is, what a tick means, and the four
//! things the core does with one. Everything the player decides for itself,
//! the mpv configuration, the tracks, the seek bar, is the shell's; what is
//! here is the history the library reads back.

pub mod session;
