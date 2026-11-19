//! The iPhone: its entry point, the hinge, the phone's language and light or dark setting,
//! and what the player needs of it: the audio session, the screen kept on, and the renderer
//! stopped while the app is off the screen. Every other call is `other.rs`'s stand-in until
//! the iPhone brings its own.
//!
//! The hinge comes from UIKit's `UIHingeInteraction` (iOS 27.1, the iPhone Duo), added to
//! the app's window the first time the layout asks. UIKit says how far the hinge is open,
//! not where the crease runs, so the crease is never reported and the layout takes it for a
//! book fold's, down the middle of the window: `fold` says which way the middle runs.
//!
//! The status bar and the home indicator are winit's view controller's to answer UIKit for,
//! so the player asks for them through winit's window. So are the orientations: an iPhone
//! that does not fold stays upright outside the player, and the player turns to landscape
//! (#937).
//!
//! Links open in the app, in a Safari sheet, and a tracker's sign-in page in the system's
//! sign-in sheet, so the app stays active and the core's loopback listener hears the
//! provider's redirect (#1175).

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{
    AnyClass, AnyObject, AnyProtocol, ClassBuilder, NSObject, NSObjectProtocol, ProtocolObject, Sel,
};
use objc2::{msg_send, sel};
use objc2_core_foundation::CGRect;
use objc2_foundation::{
    NSCurrentLocaleDidChangeNotification, NSError, NSLocale, NSNotification, NSNotificationCenter,
    NSNotificationName, NSNumber, NSOperationQueue, NSString, NSURL, NSURLIsExcludedFromBackupKey,
};
use slint::ComponentHandle;
use slint::winit_030::WinitWindowAccessor;
use slint::winit_030::winit::platform::ios::{ValidOrientations, WindowExtIOS};
use slint::winit_030::winit::window::Window as WinitWindow;

use super::FocusAsk;
use crate::worker::Command;

// `other.rs`'s stand-ins for the calls the iPhone does not answer yet.
pub use super::other::{
    ask_notifications, bars, brightness, copy_text, episode_ready, line_colours, log, multicast,
    notification_settings, notifications_allowed, pick_file, relocalize, set_brightness,
    set_volume, start_service, stop_service, unmetered, update_service, volume,
};

/// Called by `main.rs`: runs the app over its data directory in the app's container.
pub fn ios_main() {
    // HOME is the app's own container on iOS. Application Support is the app's, and not
    // shown in Files.
    let mut data_dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("Library/Application Support/Anibeam");
    // Builds before 2.5 spelled the folder with a capital B, and the iPhone's disk tells the
    // two apart. The data moves once; the library's source follows it in `local`. A move
    // that fails runs this launch from the old folder, so the next launch tries again
    // instead of finding an empty new one.
    let before = data_dir.with_file_name(concat!("Ani", "Beam"));
    if before.is_dir()
        && !data_dir.exists()
        && let Err(e) = std::fs::rename(&before, &data_dir)
    {
        eprintln!("Anibeam: could not move {}: {e}", before.display());
        data_dir = before;
    }
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("Anibeam: no data directory at {}: {e}", data_dir.display());
    }
    // Application Support goes into iCloud and computer backups, and downloaded episodes are
    // large and can be downloaded again, so the library folder stays out of them.
    let library = data_dir.join(crate::local::LIBRARY_DIR);
    match std::fs::create_dir_all(&library) {
        Ok(()) => keep_out_of_backups(&library),
        Err(e) => eprintln!("Anibeam: no library folder at {}: {e}", library.display()),
    }
    let (commands, inbox) = std::sync::mpsc::channel();
    // Held for the whole run: the notification centre keeps the observers while they live.
    let _observers = observe(commands.clone());
    if let Err(e) = crate::run(data_dir, commands, inbox) {
        eprintln!("Anibeam stopped: {e}");
    }
}

/// Sets `NSURLIsExcludedFromBackupKey` on `dir`, which covers everything in it. Set at every
/// start, since a restore from a backup can leave the folder without it.
fn keep_out_of_backups(dir: &Path) {
    let Some(url) = NSURL::from_directory_path(dir) else {
        eprintln!("Anibeam: no file URL for {}", dir.display());
        return;
    };
    let yes = NSNumber::new_bool(true);
    // SAFETY: the key takes an NSNumber holding a BOOL.
    let set =
        unsafe { url.setResourceValue_forKey_error(Some(&yes), NSURLIsExcludedFromBackupKey) };
    if let Err(e) = set {
        eprintln!("Anibeam: {} stays in backups: {e:?}", dir.display());
    }
}

/// What `android_main` hears from the activity, from UIKit's notifications: coming back to
/// the foreground looks at the host again, and the phone's light or dark setting or its
/// language changing has Appearance's System mode and a language left to follow the phone
/// follow it. The light or dark setting is read again each time the app becomes active, which
/// covers a change made in Settings or Control Center.
///
/// The player hears the rest. iOS ends an app that draws with OpenGL ES in the background, so
/// the renderer is held from the moment the app resigns active and let go of, on the main
/// thread before the handler returns, when it enters the background; it comes back when the
/// app is active again. The audio session's interruptions (a call, an alarm) are Android's
/// passing focus loss and its gain back, and headphones coming out pause the episode.
fn observe(commands: Sender<Command>) -> Vec<Retained<ProtocolObject<dyn NSObjectProtocol>>> {
    let center = NSNotificationCenter::defaultCenter();
    let queue = NSOperationQueue::mainQueue();
    let foreground = RcBlock::new(move |_: NonNull<NSNotification>| {
        let _ = commands.send(Command::Sync);
    });
    let changed = RcBlock::new(|_: NonNull<NSNotification>| {
        let _ = slint::invoke_from_event_loop(crate::system_changed);
    });
    let resigning = RcBlock::new(|_: NonNull<NSNotification>| crate::video::hold(true));
    // Called straight away, not through Slint's queue: the GL must be gone when it returns.
    let background = RcBlock::new(|_: NonNull<NSNotification>| crate::player::background());
    let active = RcBlock::new(|_: NonNull<NSNotification>| {
        turnable();
        crate::video::hold(false);
        crate::player::foreground();
    });
    let interrupted = RcBlock::new(|note: NonNull<NSNotification>| {
        // AVAudioSessionInterruptionType 1 began, 0 ended; option 1 is ShouldResume.
        let code = match (
            info_number(note, "AVAudioSessionInterruptionTypeKey"),
            info_number(note, "AVAudioSessionInterruptionOptionKey"),
        ) {
            (Some(1), _) => -2,
            (Some(0), Some(options)) if options & 1 != 0 => 1,
            _ => return,
        };
        let _ = slint::invoke_from_event_loop(move || crate::audio_focus(code));
    });
    let route = RcBlock::new(|note: NonNull<NSNotification>| {
        // AVAudioSessionRouteChangeReasonOldDeviceUnavailable: the headphones came out.
        if info_number(note, "AVAudioSessionRouteChangeReasonKey") == Some(2) {
            let _ = slint::invoke_from_event_loop(|| crate::audio_focus(-1));
        }
    });
    // UIKit's and AVFoundation's notification names are strings equal to their constants'
    // names.
    let will_enter_foreground = NSString::from_str("UIApplicationWillEnterForegroundNotification");
    let did_become_active = NSString::from_str("UIApplicationDidBecomeActiveNotification");
    let will_resign_active = NSString::from_str("UIApplicationWillResignActiveNotification");
    let did_enter_background = NSString::from_str("UIApplicationDidEnterBackgroundNotification");
    let interruption = NSString::from_str("AVAudioSessionInterruptionNotification");
    let route_change = NSString::from_str("AVAudioSessionRouteChangeNotification");
    let watch = |name: &NSNotificationName, block: &RcBlock<dyn Fn(NonNull<NSNotification>)>| {
        // SAFETY: the blocks touch a Sender, Slint's thread-safe queue and the player's
        // UI-thread state, and run on the main queue, the UI thread.
        unsafe {
            center.addObserverForName_object_queue_usingBlock(Some(name), None, Some(&queue), block)
        }
    };
    vec![
        watch(&will_enter_foreground, &foreground),
        watch(&did_become_active, &changed),
        watch(&will_resign_active, &resigning),
        watch(&did_enter_background, &background),
        watch(&did_become_active, &active),
        watch(&interruption, &interrupted),
        watch(&route_change, &route),
        // SAFETY: a Foundation constant, read once it is linked.
        watch(unsafe { NSCurrentLocaleDidChangeNotification }, &changed),
    ]
}

/// A number in a notification's user info, by key.
fn info_number(note: NonNull<NSNotification>, key: &str) -> Option<usize> {
    let key = NSString::from_str(key);
    // SAFETY: the notification is alive for the block's call; its user info is a dictionary
    // or nil, and the values read here are NSNumbers.
    unsafe {
        let info: Option<Retained<AnyObject>> = msg_send![note.as_ref(), userInfo];
        let value: Option<Retained<AnyObject>> = msg_send![&*info?, objectForKey: &*key];
        let value = value?;
        Some(msg_send![&*value, unsignedIntegerValue])
    }
}

/// The app's AVAudioSession.
fn audio_session() -> Option<Retained<AnyObject>> {
    let class = AnyClass::get(c"AVAudioSession")?;
    // SAFETY: `sharedInstance` returns the app's session.
    unsafe { msg_send![class, sharedInstance] }
}

/// Sets the audio session's category once, before mpv's first core: playback, which plays
/// with the ringer switched to silent and in the background (Info.plist's audio mode), for
/// a film. mpv's AudioUnit output sets the same when it opens.
pub fn prepare_mpv() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let Some(session) = audio_session() else {
            return;
        };
        let category = NSString::from_str("AVAudioSessionCategoryPlayback");
        let mode = NSString::from_str("AVAudioSessionModeMoviePlayback");
        // SAFETY: the category and mode are AVFoundation's own strings; no options.
        let r: Result<(), Retained<NSError>> = unsafe {
            msg_send![&*session, setCategory: &*category, mode: &*mode, options: 0usize, error: _]
        };
        if let Err(e) = r {
            log(&format!(
                "audio session category: {}",
                e.localizedDescription()
            ));
        }
    });
}

/// The iOS Simulator, whose VideoToolbox does not reach GL dependably: mpv decodes in
/// software there.
pub fn emulator() -> bool {
    cfg!(target_abi = "sim")
}

/// `AVAudioSessionErrorCodeInsufficientPriority`: another session, a call, holds the audio.
const INSUFFICIENT_PRIORITY: isize = 0x2170_7269;

/// Activates the audio session, which other apps' sound gives way to.
pub fn take_audio_focus() -> FocusAsk {
    prepare_mpv();
    let Some(session) = audio_session() else {
        return FocusAsk::Unavailable;
    };
    // SAFETY: `setActive:error:` takes a BOOL.
    let r: Result<(), Retained<NSError>> =
        unsafe { msg_send![&*session, setActive: true, error: _] };
    match r {
        Ok(()) => FocusAsk::Granted,
        Err(e) if e.code() == INSUFFICIENT_PRIORITY => FocusAsk::Refused,
        Err(e) => {
            log(&format!("audio session: {}", e.localizedDescription()));
            FocusAsk::Unavailable
        }
    }
}

/// Deactivates the audio session, telling the app that gave way it can play again.
pub fn give_audio_focus() {
    let Some(session) = audio_session() else {
        return;
    };
    // SAFETY: option 1 is AVAudioSessionSetActiveOptionNotifyOthersOnDeactivation.
    let r: Result<(), Retained<NSError>> =
        unsafe { msg_send![&*session, setActive: false, withOptions: 1usize, error: _] };
    if let Err(e) = r {
        log(&format!("audio session off: {}", e.localizedDescription()));
    }
}

/// Keeps the screen on while an episode plays: UIKit's idle timer is off.
pub fn player_awake(on: bool) {
    let Some(app_class) = AnyClass::get(c"UIApplication") else {
        return;
    };
    // SAFETY: called on the UI thread, the main thread, which UIApplication wants.
    unsafe {
        let app: *mut AnyObject = msg_send![app_class, sharedApplication];
        if !app.is_null() {
            let _: () = msg_send![app, setIdleTimerDisabled: on];
        }
    }
}

thread_local! {
    /// The app's window, made on the UI thread, whose winit window the player turns to.
    static APP_WINDOW: RefCell<Option<slint::Weak<crate::AppWindow>>> = const { RefCell::new(None) };
}

/// The app's window is made: kept, so the player can reach winit's window under it.
pub fn window_up(window: &crate::AppWindow) {
    APP_WINDOW.set(Some(window.as_weak()));
}

/// Runs `f` on winit's window, once the app's window has one; false before.
fn with_winit(f: impl FnOnce(&WinitWindow)) -> bool {
    let Some(window) = APP_WINDOW.with_borrow(|w| w.as_ref().and_then(|w| w.upgrade())) else {
        return false;
    };
    window.window().with_winit_window(f).is_some()
}

/// The player's screen: while it is up the status bar is hidden and the home indicator fades
/// after a moment, as in Apple's own players; when it closes both come back. UIKit lays the
/// safe area out again, and winit tells Slint, so the pages keep clear of the bar.
///
/// The player also turns the window to landscape, and back to what the window takes outside
/// it when it closes.
pub fn player_screen(on: bool) {
    PLAYING.set(on);
    with_winit(|w| {
        w.set_prefers_status_bar_hidden(on);
        w.set_prefers_home_indicator_hidden(on);
    });
    turn();
}

/// `UIInterfaceOrientationMaskPortrait`.
const PORTRAIT: usize = 1 << 1;
/// `UIInterfaceOrientationMaskLandscape`: left and right.
const LANDSCAPE: usize = (1 << 3) | (1 << 4);

thread_local! {
    /// The player has the screen.
    static PLAYING: Cell<bool> = const { Cell::new(false) };
    /// UIKit has reported a hinge angle: this iPhone folds.
    static FOLDS: Cell<bool> = const { Cell::new(false) };
    /// winit's window has been told the orientations once.
    static TURNED: Cell<bool> = const { Cell::new(false) };
}

/// The orientations the window may take, as winit names them and as UIKit's mask: landscape
/// while the player is open, either on a foldable, and upright on an iPhone that does not
/// fold, whose short screen has no room for the wide layout.
fn orientations() -> (ValidOrientations, usize) {
    if PLAYING.get() {
        (ValidOrientations::Landscape, LANDSCAPE)
    } else if FOLDS.get() {
        (
            ValidOrientations::LandscapeAndPortrait,
            PORTRAIT | LANDSCAPE,
        )
    } else {
        (ValidOrientations::Portrait, PORTRAIT)
    }
}

/// Tells winit's window the orientations the first time it is there, which turns an iPhone
/// held sideways at launch upright.
fn turnable() {
    if !TURNED.get() {
        turn();
    }
}

/// Tells winit's view controller the orientations, and asks the window's scene to turn to
/// one of them if it is not in one already: winit only asks UIKit to try, which iOS 16 and
/// later no longer act on by themselves.
fn turn() {
    let (valid, mask) = orientations();
    if !with_winit(|w| w.set_valid_orientations(valid)) {
        return;
    }
    TURNED.set(true);
    let Some(window) = app_window() else {
        return;
    };
    let Some(prefs_class) = AnyClass::get(c"UIWindowSceneGeometryPreferencesIOS") else {
        return;
    };
    let refused = RcBlock::new(|error: *mut NSError| {
        // SAFETY: UIKit passes an NSError it keeps alive for the call.
        if let Some(error) = unsafe { error.as_ref() } {
            log(&format!(
                "iOS kept the orientation: {}",
                error.localizedDescription()
            ));
        }
    });
    // SAFETY: plain UIKit calls on the main thread. `rootViewController` and `windowScene`
    // return objects or nil; the preferences take a UIInterfaceOrientationMask, and the
    // scene copies the error block.
    unsafe {
        let controller: Option<Retained<AnyObject>> = msg_send![&*window, rootViewController];
        if let Some(controller) = controller {
            let _: () = msg_send![&*controller, setNeedsUpdateOfSupportedInterfaceOrientations];
        }
        let scene: Option<Retained<AnyObject>> = msg_send![&*window, windowScene];
        let Some(scene) = scene else {
            return;
        };
        let prefs: Retained<AnyObject> = msg_send![
            msg_send![prefs_class, alloc],
            initWithInterfaceOrientations: mask
        ];
        let _: () = msg_send![
            &*scene,
            requestGeometryUpdateWithPreferences: &*prefs,
            errorHandler: &*refused
        ];
    }
}

/// The phone's language as iOS lists it first: `da-DK`, `pt-BR`, `zh-Hans-CN`, the same
/// shape Android gives.
pub fn system_locale() -> String {
    NSLocale::preferredLanguages()
        .firstObject()
        .map(|l| l.to_string())
        .unwrap_or_default()
}

/// The alternate icon sets in the asset catalog, in the picker's order after Default: 1
/// Dark, 2 Line art. Their names are listed in project.yml.
const ICONS: [&str; 2] = ["AppIconDark", "AppIconLine"];

/// The home screen icon showing now: 0 Default, 1 Dark, 2 Line art.
pub fn app_icon() -> i32 {
    let Some(app_class) = AnyClass::get(c"UIApplication") else {
        return 0;
    };
    // SAFETY: plain UIKit getters on the UI thread; `alternateIconName` is an NSString or nil.
    let name = unsafe {
        let app: *mut AnyObject = msg_send![app_class, sharedApplication];
        if app.is_null() {
            return 0;
        }
        let name: Option<Retained<NSString>> = msg_send![app, alternateIconName];
        name.map(|n| n.to_string())
    };
    name.and_then(|n| ICONS.iter().position(|i| *i == n))
        .map_or(0, |i| i as i32 + 1)
}

/// Shows home screen icon `which`; iOS tells the person with an alert. A refusal
/// is logged.
pub fn set_app_icon(which: i32) {
    let Some(app_class) = AnyClass::get(c"UIApplication") else {
        return;
    };
    let name: Option<Retained<NSString>> = usize::try_from(which - 1)
        .ok()
        .and_then(|i| ICONS.get(i))
        .map(|n| NSString::from_str(n));
    let done = RcBlock::new(|error: *mut NSError| {
        // SAFETY: UIKit passes nil or an NSError it keeps alive for the call.
        if let Some(error) = unsafe { error.as_ref() } {
            log(&format!(
                "iOS refused the app icon: {}",
                error.localizedDescription()
            ));
        }
    });
    // SAFETY: `setAlternateIconName:completionHandler:` is called on the main thread, which
    // is where Settings' callbacks run, with an NSString or nil and a block UIKit copies.
    unsafe {
        let app: *mut AnyObject = msg_send![app_class, sharedApplication];
        if app.is_null() {
            return;
        }
        let _: () = msg_send![
            app,
            setAlternateIconName: name.as_deref(),
            completionHandler: &*done
        ];
    }
}

/// The phone is in dark mode: the window's style once there is one, the screen's before.
pub fn system_dark() -> bool {
    const DARK: isize = 2; // UIUserInterfaceStyleDark
    // SAFETY: plain UIKit getters on the UI thread; each returns an object or nil, and
    // `userInterfaceStyle` an NSInteger.
    unsafe {
        let traits: *mut AnyObject = match app_window() {
            Some(window) => msg_send![&*window, traitCollection],
            None => {
                let Some(screen) = AnyClass::get(c"UIScreen") else {
                    return true;
                };
                let main: *mut AnyObject = msg_send![screen, mainScreen];
                if main.is_null() {
                    return true;
                }
                msg_send![main, traitCollection]
            }
        };
        if traits.is_null() {
            return true;
        }
        let style: isize = msg_send![traits, userInterfaceStyle];
        style == DARK
    }
}

/// What UIKit has said about the hinge. Kept on the UI thread, where both the layout's
/// timer and UIKit's handler run.
#[derive(Clone, Copy, PartialEq)]
enum Hinge {
    /// Not asked yet: the window was not up.
    Unasked,
    /// Asked, and UIKit has not answered yet.
    Asked,
    /// No hinge here: an iOS older than 27.1, with no `UIHingeInteraction` class. The only
    /// proof there is no hinge; a nil hinge from UIKit is `Lost`.
    Absent,
    /// The hinge's angle in degrees, 0 shut and 180 flat.
    Angle(f32),
    /// UIKit reported no hinge for now: the window has left the hierarchy that reports it,
    /// as it may while the phone moves between its screens, or the app opened while it was
    /// out (launched folded, on the cover screen). The layout keeps asking, so a hinge that
    /// turns up later is followed; an iPhone that does not fold stays here.
    Lost,
}

thread_local! {
    static HINGE: Cell<Hinge> = const { Cell::new(Hinge::Unasked) };
    /// The window the hinge interaction sits on.
    static WINDOW: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

/// The hinge's angle in degrees, None without one, and whether the crease runs across the
/// window: the iPhone Duo's inner screen folds across its long side, so it runs across
/// when the window is taller than wide.
pub fn fold() -> (Option<f32>, bool) {
    turnable();
    let angle = match ask() {
        Hinge::Angle(a) => Some(a),
        _ => None,
    };
    (angle, upright())
}

/// UIKit never says where the crease is. An iPhone with no hinge never will say, so the
/// layout stops asking; one with a hinge is laid out by the guess.
pub fn crease() -> crate::crease::Reading {
    match ask() {
        // Silent stops the layout's timer, whose fold() call turns the window: wait for it.
        Hinge::Absent if TURNED.get() => crate::crease::Reading::Silent,
        _ => crate::crease::Reading::None,
    }
}

/// The hinge as last heard, adding the interaction to the window the first time there is
/// one to add it to.
fn ask() -> Hinge {
    let state = HINGE.get();
    if state != Hinge::Unasked {
        return state;
    }
    let Some(class) = AnyClass::get(c"UIHingeInteraction") else {
        HINGE.set(Hinge::Absent);
        return Hinge::Absent;
    };
    let Some(window) = app_window() else {
        return Hinge::Unasked;
    };
    let handler = RcBlock::new(|_: *mut AnyObject, update: *mut AnyObject| heard(update));
    // SAFETY: UIHingeInteraction's designated initialiser takes a block of
    // (UIHingeInteraction *, UIHingeInteractionUpdate *), and any UIView takes an
    // interaction. UIKit keeps the block and the interaction alive with the window.
    unsafe {
        let interaction: Retained<AnyObject> =
            msg_send![msg_send![class, alloc], initWithUpdateHandler: &*handler];
        let _: () = msg_send![&*window, addInteraction: &*interaction];
    }
    WINDOW.set(Some(window));
    HINGE.set(Hinge::Asked);
    Hinge::Asked
}

/// UIKit's handler: the hinge on the first call and on every change.
fn heard(update: *mut AnyObject) {
    // SAFETY: `update` is the UIHingeInteractionUpdate UIKit hands the handler, whose
    // `hinge` is a UIHinge or nil; `status` is an NSInteger and `angle` a CGFloat.
    let next = unsafe {
        let hinge: *mut AnyObject = if update.is_null() {
            std::ptr::null_mut()
        } else {
            msg_send![update, hinge]
        };
        if hinge.is_null() {
            Hinge::Lost
        } else {
            let status: isize = msg_send![hinge, status];
            let radians: f64 = msg_send![hinge, angle];
            Hinge::Angle(degrees(status, radians))
        }
    };
    HINGE.set(next);
    // A hinge that reports an angle is a foldable's: from then on the window turns freely.
    if matches!(next, Hinge::Angle(_)) && !FOLDS.replace(true) {
        turn();
    }
}

/// The angle the layout takes: UIKit's status where it says shut or half open, since Apple
/// has the angle come as often and as finely as the system likes, and the angle itself
/// otherwise. Fully open can still be a bend the layout calls partly folded (under 170
/// degrees, as `fold.slint` reads it, and as Android reads its hinge sensor), so the angle
/// wins there when it reads one.
fn degrees(status: isize, radians: f64) -> f32 {
    let angle = radians.to_degrees() as f32;
    match status {
        // UIHingeStatusClosed
        1 => 0.0,
        // UIHingeStatusPartiallyOpen: bent, whatever the angle reads.
        2 => angle.clamp(31.0, 169.0),
        // UIHingeStatusFullyOpen: flat unless the angle reads a bend.
        3 if angle > 30.0 && angle < BENT_UNDER => angle,
        3 => 180.0,
        _ => angle,
    }
}

/// The angle under which the layout takes the phone for bent, `fold.slint`'s `bent`.
const BENT_UNDER: f32 = 170.0;

/// The window is taller than wide.
fn upright() -> bool {
    WINDOW.with_borrow(|window| {
        window.as_ref().is_some_and(|w| {
            // SAFETY: a UIWindow is a UIView, whose `bounds` is a CGRect.
            let b: CGRect = unsafe { msg_send![&**w, bounds] };
            b.size.height > b.size.width
        })
    })
}

/// The app's window: the first window of its first window scene, once winit has made it.
fn app_window() -> Option<Retained<AnyObject>> {
    let app_class = AnyClass::get(c"UIApplication")?;
    let scene_class = AnyClass::get(c"UIWindowScene")?;
    // SAFETY: plain UIKit getters on the UI thread; each returns an object or nil.
    unsafe {
        let app: *mut AnyObject = msg_send![app_class, sharedApplication];
        if app.is_null() {
            return None;
        }
        let scenes: *mut AnyObject = msg_send![app, connectedScenes];
        let scenes: *mut AnyObject = msg_send![scenes, allObjects];
        let count: usize = msg_send![scenes, count];
        for i in 0..count {
            let scene: *mut AnyObject = msg_send![scenes, objectAtIndex: i];
            let is_window_scene: bool = msg_send![scene, isKindOfClass: scene_class];
            if !is_window_scene {
                continue;
            }
            let windows: *mut AnyObject = msg_send![scene, windows];
            let window: *mut AnyObject = msg_send![windows, firstObject];
            if let Some(w) = Retained::retain(window) {
                return Some(w);
            }
        }
        None
    }
}

/// What a link sheet holds while it is up: the sign-in session and the object that tells it
/// which window to sit on, or the Safari sheet showing the page.
enum Sheet {
    /// `ASWebAuthenticationSession` and its presentation context provider, which the session
    /// holds weakly.
    Auth {
        session: Retained<AnyObject>,
        _anchor: Retained<AnyObject>,
    },
    /// An `SFSafariViewController` presented as a page sheet.
    Safari(Retained<AnyObject>),
}

thread_local! {
    /// The sign-in sheet that is up, closed when the sign-in ends.
    static SIGN_IN: RefCell<Option<Sheet>> = const { RefCell::new(None) };
}

/// Opens `url` in a Safari sheet over the app: the page shows in the app, which stays
/// active. A link Safari cannot show (not http or https) goes to the app that takes it.
pub fn open_url(url: &str) {
    let url = url.to_string();
    let _ = slint::invoke_from_event_loop(move || {
        let Some(link) = ns_url(&url) else {
            log(&format!("not a link: {url}"));
            return;
        };
        let web = url.starts_with("https://") || url.starts_with("http://");
        if !(web && safari_sheet(&link).is_some()) {
            hand_off(&link);
        }
    });
}

/// Shows a tracker's sign-in page in the system's sign-in sheet. Its redirect is the core's
/// loopback listener, which answers while the app stays active under the sheet; the sheet
/// closes when the sign-in ends (`close_sign_in`). Where the sheet cannot start, the page
/// shows in a Safari sheet instead, which closes the same way.
pub fn open_sign_in(url: &str) {
    let url = url.to_string();
    let _ = slint::invoke_from_event_loop(move || {
        dismiss();
        let Some(link) = ns_url(&url) else {
            log(&format!("not a sign-in link: {url}"));
            return;
        };
        let sheet = auth_session(&link).or_else(|| {
            log("the sign-in sheet did not start: the page opens in Safari's sheet");
            safari_sheet(&link).map(Sheet::Safari)
        });
        match sheet {
            Some(sheet) => SIGN_IN.set(Some(sheet)),
            None => hand_off(&link),
        }
    });
}

/// Closes the sign-in sheet, once the sign-in has ended, connected or not.
pub fn close_sign_in() {
    let _ = slint::invoke_from_event_loop(dismiss);
}

/// Takes the sign-in sheet down, if one is up.
fn dismiss() {
    let Some(sheet) = SIGN_IN.take() else {
        return;
    };
    // SAFETY: plain UIKit and AuthenticationServices calls on the main thread. `cancel`
    // takes an ASWebAuthenticationSession down; a presented controller dismisses itself.
    unsafe {
        match sheet {
            Sheet::Auth { session, .. } => {
                let _: () = msg_send![&*session, cancel];
            }
            Sheet::Safari(controller) => {
                let _: () = msg_send![
                    &*controller,
                    dismissViewControllerAnimated: true,
                    completion: None::<&block2::Block<dyn Fn()>>
                ];
            }
        }
    }
}

/// `url` as an NSURL, None if it does not parse.
fn ns_url(url: &str) -> Option<Retained<NSURL>> {
    NSURL::URLWithString(&NSString::from_str(url))
}

/// Starts an `ASWebAuthenticationSession` on `url`, sharing Safari's cookies so a person
/// already signed in to the provider is asked only to allow. No callback scheme: the
/// redirect is plain http to the loopback, which the session cannot catch, so it ends when
/// `close_sign_in` cancels it, or when the person cancels it.
fn auth_session(url: &NSURL) -> Option<Sheet> {
    let class = AnyClass::get(c"ASWebAuthenticationSession")?;
    let anchor = anchor()?;
    let done = RcBlock::new(|_: *mut NSURL, error: *mut NSError| {
        // SAFETY: the session passes nil or an NSError it keeps alive for the call.
        if let Some(error) = unsafe { error.as_ref() } {
            // ASWebAuthenticationSessionErrorCodeCanceledLogin: closed by the person or by
            // close_sign_in; the core's job ends on its own.
            if error.code() != 1 {
                log(&format!("sign-in sheet: {}", error.localizedDescription()));
            }
        }
        let _ = slint::invoke_from_event_loop(|| {
            SIGN_IN.with_borrow_mut(|s| {
                if matches!(s, Some(Sheet::Auth { .. })) {
                    *s = None;
                }
            })
        });
    });
    // SAFETY: the designated initialiser takes an NSURL, a nil scheme and a block the session
    // copies; the provider is held weakly, so `anchor` is kept beside the session. `start`
    // returns a BOOL and is called on the main thread.
    unsafe {
        let session: Option<Retained<AnyObject>> = msg_send![
            msg_send![class, alloc],
            initWithURL: url,
            callbackURLScheme: None::<&NSString>,
            completionHandler: &*done
        ];
        let session = session?;
        let _: () = msg_send![&*session, setPresentationContextProvider: &*anchor];
        let _: () = msg_send![&*session, setPrefersEphemeralWebBrowserSession: false];
        let started: bool = msg_send![&*session, start];
        started.then_some(Sheet::Auth {
            session,
            _anchor: anchor,
        })
    }
}

/// A new presentation context provider for the sign-in session: its one method answers the
/// app's window. The class is made once, at run time, since the app has no Objective-C.
fn anchor() -> Option<Retained<AnyObject>> {
    static CLASS: std::sync::OnceLock<Option<&'static AnyClass>> = std::sync::OnceLock::new();
    let class = (*CLASS.get_or_init(|| {
        let mut builder = ClassBuilder::new(
            c"AnibeamSignInAnchor",
            <NSObject as objc2::ClassType>::class(),
        )?;
        if let Some(protocol) = AnyProtocol::get(c"ASWebAuthenticationPresentationContextProviding")
        {
            builder.add_protocol(protocol);
        }
        extern "C-unwind" fn window(_: &AnyObject, _: Sel, _: *mut AnyObject) -> *mut AnyObject {
            app_window().map_or(std::ptr::null_mut(), Retained::autorelease_return)
        }
        // SAFETY: the method's types are the protocol's: it takes the session and returns
        // a UIWindow (ASPresentationAnchor), autoreleased as a getter's return is.
        unsafe {
            builder.add_method(
                sel!(presentationAnchorForWebAuthenticationSession:),
                window as extern "C-unwind" fn(_, _, _) -> _,
            );
        }
        Some(builder.register())
    }))?;
    // SAFETY: an NSObject subclass with no ivars: `new` makes one.
    unsafe { msg_send![class, new] }
}

/// Presents `url` in an `SFSafariViewController` as a page sheet over whatever is showing.
fn safari_sheet(url: &NSURL) -> Option<Retained<AnyObject>> {
    let class = AnyClass::get(c"SFSafariViewController")?;
    let host = top_controller()?;
    const PAGE_SHEET: isize = 1; // UIModalPresentationPageSheet
    // SAFETY: plain SafariServices and UIKit calls on the main thread; the controller takes
    // an http or https NSURL, and presenting retains it.
    unsafe {
        let controller: Option<Retained<AnyObject>> =
            msg_send![msg_send![class, alloc], initWithURL: url];
        let controller = controller?;
        let _: () = msg_send![&*controller, setModalPresentationStyle: PAGE_SHEET];
        let _: () = msg_send![
            &*host,
            presentViewController: &*controller,
            animated: true,
            completion: None::<&block2::Block<dyn Fn()>>
        ];
        Some(controller)
    }
}

/// The view controller showing on top: the window's root, or what it presents.
fn top_controller() -> Option<Retained<AnyObject>> {
    let window = app_window()?;
    // SAFETY: plain UIKit getters on the main thread; each returns a controller or nil.
    unsafe {
        let root: Option<Retained<AnyObject>> = msg_send![&*window, rootViewController];
        let mut top = root?;
        loop {
            let next: Option<Retained<AnyObject>> = msg_send![&*top, presentedViewController];
            match next {
                Some(next) => top = next,
                None => return Some(top),
            }
        }
    }
}

/// Hands `url` to the app that takes it, leaving Anibeam.
fn hand_off(url: &NSURL) {
    let (Some(app_class), Some(dictionary)) = (
        AnyClass::get(c"UIApplication"),
        AnyClass::get(c"NSDictionary"),
    ) else {
        return;
    };
    // SAFETY: `openURL:options:completionHandler:` on the main thread, with an empty options
    // dictionary and no handler.
    unsafe {
        let app: *mut AnyObject = msg_send![app_class, sharedApplication];
        if app.is_null() {
            return;
        }
        let options: Retained<AnyObject> = msg_send![dictionary, dictionary];
        let _: () = msg_send![
            app,
            openURL: url,
            options: &*options,
            completionHandler: None::<&block2::Block<dyn Fn(objc2::runtime::Bool)>>
        ];
    }
}
