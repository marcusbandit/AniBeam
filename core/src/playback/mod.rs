//! The playback rules: what a session is, what a tick means, and the four
//! things the core does with one, plus what every player picks the same way:
//! the mpv options AniBeam owns and the track pick. The seek bar, the keys and
//! the rest of what a player draws are the shell's.

pub mod options;
pub mod session;
pub mod skip;
pub mod tracks;
