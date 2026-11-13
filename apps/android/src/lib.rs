//! AniBeam for Android: the Slint screens and the thread that syncs behind them.
//!
//! `android_main` is the whole entry point. The stock NativeActivity loads this library
//! and calls it; Java only holds the download service's notification and turns the screen
//! for the player.

// Clippy asks for `const` on thread_local initialisers that already have it on targets
// without `#[thread_local]`, Android among them (rust-lang/rust-clippy#17566).
#![cfg_attr(target_os = "android", allow(clippy::missing_const_for_thread_local))]

slint::include_modules!();

// First, so its `tr!` reaches every module after it.
#[macro_use]
mod lang;
mod activity;
mod airing;
mod android;
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

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android: slint::android::AndroidApp) {
    use slint::android::android_activity::{MainEvent, PollEvent};

    let data_dir = android
        .internal_data_path()
        .unwrap_or_else(|| "/data/data/com.marcusrosado.anibeam/files".into());
    // The activity itself, for the calls only an Activity answers; ndk_context holds the
    // Application. The reference lives as long as `android`, which outlives `run`.
    android::set_activity(android.activity_as_ptr());
    let (commands, inbox) = std::sync::mpsc::channel();

    // Coming back to the app is a good moment to look at the host again.
    let on_resume = commands.clone();
    slint::android::init_with_event_listener(android, move |event| {
        if let PollEvent::Main(MainEvent::Resume { .. }) = event {
            let _ = on_resume.send(worker::Command::Sync);
        }
        // The phone's light or dark setting or its language changed: Appearance's System mode
        // and a language left to follow the phone follow it.
        if let PollEvent::Main(MainEvent::ConfigChanged { .. }) = event {
            let _ = slint::invoke_from_event_loop(|| {
                look::system_changed();
                lang::system_changed();
            });
        }
    })
    .expect("the Slint Android backend starts");

    if let Err(e) = run(data_dir, commands, inbox) {
        eprintln!("AniBeam stopped: {e}");
    }
}
