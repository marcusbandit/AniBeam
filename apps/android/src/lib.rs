//! AniBeam for Android: the Slint screens and the thread that syncs behind them.
//!
//! The entry point is the platform's (`platform/android.rs`'s `android_main` on the phone),
//! which hands `run` the data directory and calls back in through the functions below.
//! Everything that differs by platform goes through `platform`.

// Clippy asks for `const` on thread_local initialisers that already have it on targets
// without `#[thread_local]`, Android among them (rust-lang/rust-clippy#17566). Allowed
// there only, so the host clippy run still holds the lint.
#![cfg_attr(target_os = "android", allow(clippy::missing_const_for_thread_local))]

slint::include_modules!();

// First, so its `tr!` reaches every module after it.
#[macro_use]
mod lang;
mod activity;
mod airing;
mod app;
mod backup;
mod crease;
mod desk;
mod disk;
mod feed;
mod files;
mod find;
mod focus;
mod inspect;
mod listmove;
mod local;
mod look;
mod mpv;
mod notes;
mod notify;
mod phone;
mod platform;
mod player;
mod posters;
mod prefs;
mod remote;
mod repos;
mod schedule;
mod seasons;
mod show;
mod show_match;
mod stats;
mod stream;
mod subtitles;
mod thumbs;
mod tr;
mod versions;
mod video;
mod worker;

pub use app::run;

// The calls a platform makes into the app. A target whose platform never makes them
// still keeps them.

/// The device's light or dark setting or its language changed: Appearance's System mode
/// and a language left to follow the device follow it. Called on the UI loop.
#[allow(dead_code)]
pub(crate) fn system_changed() {
    look::system_changed();
    lang::system_changed();
}

/// The platform moved the audio focus; `code` is Android's `AudioManager` change. Called on
/// the UI loop.
#[allow(dead_code)]
pub(crate) fn audio_focus(code: i32) {
    player::audio_focus(code);
}
