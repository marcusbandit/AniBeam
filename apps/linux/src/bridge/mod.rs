//! Every `#[cxx_qt::bridge]` lives in this one directory: cxx-qt panics on bridges spread
//! across directories of one QML module (QTBUG-93443). Nothing outside `bridge/` mentions
//! a Qt type except `main.rs`, which constructs the application.

pub mod door;
pub mod fmt;
pub mod helpers;
pub mod model;
pub mod shell;
pub mod theme;
