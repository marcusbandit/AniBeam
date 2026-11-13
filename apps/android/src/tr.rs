//! The Slint side's word lookup: answers `Tr.t` and `Tr.n` from the core's catalog, the
//! same `i18n::tr` the Rust side calls.

use anibeam_core::i18n::tr;
use slint::ComponentHandle;

use crate::{AppWindow, Tr};

pub fn install(window: &AppWindow) {
    let t = window.global::<Tr>();
    t.on_t(|id| tr(&id, &[]).into());
    t.on_n(|id, name, n| tr(&id, &[(name.as_str(), n.into())]).into());
}
