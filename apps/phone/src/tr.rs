//! The catalog lookup is the core's: `i18n::tr` and the language it speaks. `lang.rs` answers
//! Slint's `Tr.lookup` with it and keeps the language; Rust code calls it through `tr!`.
pub use anibeam_core::i18n::*;
