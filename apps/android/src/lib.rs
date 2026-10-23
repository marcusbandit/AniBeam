//! AniBeam for Android: the Slint screens and the thread that syncs behind them.
//!
//! `android_main` is the whole entry point. The stock NativeActivity loads this library
//! and calls it; Java only holds the download service's notification and turns the screen
//! for the player.

slint::include_modules!();

mod android;
mod app;
mod find;
mod local;
mod mpv;
mod phone;
mod player;
mod posters;
mod prefs;
mod remote;
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
    })
    .expect("the Slint Android backend starts");

    if let Err(e) = run(data_dir, commands, inbox) {
        eprintln!("AniBeam stopped: {e}");
    }
}
