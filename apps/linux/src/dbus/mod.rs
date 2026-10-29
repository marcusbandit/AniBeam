//! The session bus: the single-instance hand-off on `com.marcusrosado.AniBeam` and MPRIS
//! on the same connection. Both start from the Shell singleton, which exists exactly once
//! and exists as soon as the engine loads. The Mac has no session bus, so MPRIS never
//! starts there (bridge/shell.rs) and its media keys and Now Playing come from `macnp`.

#[cfg(not(any(target_os = "macos", windows)))]
pub mod inhibit;
pub mod instance;
// Still compiled on the Mac, where nothing starts it, so the bus code only it reaches
// (the hand-off's serving half in `instance`, the Shell's Raise and Quit) needs no gates
// of its own.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub mod mpris;
