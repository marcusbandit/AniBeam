//! AniBeam for Android: the Slint screens and the thread that syncs behind them.
//!
//! `android_main` is the whole entry point. The stock NativeActivity loads this library
//! and calls it; there is no Java or Kotlin in the app.

slint::include_modules!();

mod app;
mod posters;
mod rows;
mod worker;

pub use app::run;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android: slint::android::AndroidApp) {
    use slint::android::android_activity::{MainEvent, PollEvent};

    let data_dir = android
        .internal_data_path()
        .unwrap_or_else(|| "/data/data/com.marcusrosado.anibeam/files".into());
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
