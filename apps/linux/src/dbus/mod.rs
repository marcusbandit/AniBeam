//! The session bus: the single-instance hand-off on `com.marcusrosado.AniBeam` and MPRIS
//! on the same connection. Both start from the Shell singleton, which exists exactly once
//! and exists as soon as the engine loads.

#[cfg(not(any(target_os = "macos", windows)))]
pub mod inhibit;
pub mod instance;
pub mod mpris;
