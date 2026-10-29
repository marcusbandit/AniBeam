//! Every `#[cxx_qt::bridge]` lives in this one directory: cxx-qt panics on bridges spread
//! across directories of one QML module (QTBUG-93443). Nothing outside `bridge/` mentions
//! a Qt type except `main.rs`, which constructs the application.

pub mod door;
pub mod fmt;
pub mod helpers;
pub mod model;
// mprisUpdate carries the eight fields a media widget shows, two past clippy's argument
// limit, and a bag would only move the names somewhere QML cannot see them. The lint lands
// on the bridge's generated function as well, and cxx-qt allows no attribute of our own on
// a bridge module, so the allow sits on the module declaration here.
#[allow(clippy::too_many_arguments)]
pub mod player;
// quitRequested's one sender is MPRIS's Quit, and the Mac has no MPRIS, so there nothing
// emits it; Main.qml still listens. The allow sits here for the same reason as player's.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub mod shell;
pub mod theme;
