//! Every target but Android: the same calls as `android.rs`, each doing the harmless thing,
//! so the app and its tests build and run on the desktop. Nothing here reaches a phone.

use super::FocusAsk;

/// Writes `line` to stderr, under AniBeamCore.
pub fn log(line: &str) {
    eprintln!("AniBeamCore: {line}");
}

/// No browser to open: the link is logged.
pub fn open_url(url: &str) {
    crate::local::log(&format!("open {url}"));
}

/// Nothing can be picked here.
pub fn pick_file(
    _kind: crate::files::Pick,
    _path: &std::path::Path,
    _name: &str,
    _result: &std::path::Path,
) -> Result<(), String> {
    Err(tr!("files-none-here"))
}

/// The language the environment names, as `LANG` holds it.
pub fn system_locale() -> String {
    std::env::var("LANG").unwrap_or_default()
}

/// Dark, the app's own default.
pub fn system_dark() -> bool {
    true
}

/// Nothing to ready: a target that links libmpv here hands it nothing first.
#[cfg_attr(not(libmpv), allow(dead_code))]
pub fn prepare_mpv() {}

/// Never the Android emulator.
#[cfg_attr(not(libmpv), allow(dead_code))]
pub fn emulator() -> bool {
    false
}

/// No notification channels to rename.
pub fn relocalize() {}

/// No download service: it counts as started.
pub fn start_service(shown: &crate::notify::Ongoing) -> bool {
    let _ = shown;
    true
}

/// No download service: its notification counts as changed.
pub fn update_service(shown: &crate::notify::Ongoing) -> bool {
    let _ = shown;
    true
}

/// No notification to post.
pub fn episode_ready(id: i32, title: &str, text: &str) {
    let _ = (id, title, text);
}

/// No download service to stop.
pub fn stop_service() {}

/// The network counts as free.
pub fn unmetered() -> Option<bool> {
    Some(true)
}

/// No permission to ask for.
pub fn ask_notifications() -> bool {
    true
}

/// No notification settings to open.
pub fn notification_settings() {}

/// Notifications count as allowed.
pub fn notifications_allowed() -> bool {
    true
}

/// Every request for the audio focus is granted.
pub fn take_audio_focus() -> FocusAsk {
    FocusAsk::Granted
}

/// No audio focus to give back.
pub fn give_audio_focus() {}

/// No clipboard: nothing is copied.
pub fn copy_text(label: &str, text: &str) -> bool {
    let _ = (label, text);
    false
}

/// No hinge.
pub fn fold() -> (Option<f32>, bool) {
    (None, false)
}

/// The crease is never reported: the answer `Crease.at` gives on a phone without the window
/// extensions.
pub fn crease() -> crate::crease::Reading {
    crate::crease::decode(crate::crease::SILENT)
}

/// The window's brightness, halfway.
pub fn brightness() -> f32 {
    0.5
}

/// No brightness to set.
pub fn set_brightness(level: f32) {
    let _ = level;
}

/// The media volume, halfway.
pub fn volume() -> f32 {
    0.5
}

/// No volume to set.
pub fn set_volume(level: f32) {
    let _ = level;
}

/// No screen to turn.
pub fn player_screen(on: bool) {
    let _ = on;
}

/// No Wi-Fi multicast to hold.
pub fn multicast(on: bool) {
    let _ = on;
}

/// No display to keep on.
pub fn player_awake(on: bool) {
    let _ = on;
}

/// The Default launcher icon.
pub fn app_icon() -> i32 {
    0
}

/// No launcher icon to change.
pub fn set_app_icon(which: i32) {
    let _ = which;
}

/// No Material You colours.
pub fn line_colours() -> Option<(u32, u32)> {
    None
}

/// No system bars to paint.
pub fn bars(argb: u32) {
    let _ = argb;
}
