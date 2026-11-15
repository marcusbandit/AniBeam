//! Starting a helper program lives in the foundation crate now (#854), since
//! the downloads and the host both start them. This hands it on under the
//! path it had, so `crate::process::command` and the rest keep resolving.

pub(crate) use anibeam_foundation::process::*;
