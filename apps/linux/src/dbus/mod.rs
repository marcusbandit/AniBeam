//! The session bus: the single-instance hand-off on `com.marcusrosado.AniBeam` and MPRIS
//! on the same connection. Both start from the Shell singleton, which exists exactly once
//! and exists as soon as the engine loads. The Mac has no MPRIS: its media keys and Now
//! Playing come from `macnp`, and the hand-off gets a connection of its own there.

#[cfg(not(any(target_os = "macos", windows)))]
pub mod inhibit;
pub mod instance;
#[cfg(not(target_os = "macos"))]
pub mod mpris;
