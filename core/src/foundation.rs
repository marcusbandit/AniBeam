//! The helpers that sat under `core/src/foundation/` live in the foundation
//! crate now (#854). This hands each on under the path it had, so
//! `crate::foundation::services::Services` and the rest keep resolving. The
//! foundation modules that sat at the core's root are handed on in `lib.rs`.

pub use anibeam_foundation::{
    above, blocking, http, machine, magnet, names, range, relations, secrets, services, size,
    titles, trust, watch,
};
