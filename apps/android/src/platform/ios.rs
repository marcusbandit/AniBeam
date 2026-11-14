//! The iPhone: its entry point, the hinge, and the phone's language and light or dark
//! setting. Every other call is `other.rs`'s stand-in until the iPhone brings its own.
//!
//! The hinge comes from UIKit's `UIHingeInteraction` (iOS 27.1, the iPhone Duo), added to
//! the app's window the first time the layout asks. UIKit says how far the hinge is open,
//! not where the crease runs, so the crease is never reported and the layout takes it for a
//! book fold's, down the middle of the window: `fold` says which way the middle runs.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, NSObjectProtocol, ProtocolObject};
use objc2_core_foundation::CGRect;
use objc2_foundation::{
    NSCurrentLocaleDidChangeNotification, NSError, NSLocale, NSNotification, NSNotificationCenter,
    NSNotificationName, NSNumber, NSOperationQueue, NSString, NSURL, NSURLIsExcludedFromBackupKey,
};

use crate::worker::Command;

pub use super::other::*;

/// Called by `main.rs`: runs the app over its data directory in the app's container.
pub fn ios_main() {
    // HOME is the app's own container on iOS. Application Support is the app's, and not
    // shown in Files.
    let data_dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("Library/Application Support/AniBeam");
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
fn observe(commands: Sender<Command>) -> Vec<Retained<ProtocolObject<dyn NSObjectProtocol>>> {
    let center = NSNotificationCenter::defaultCenter();
    let queue = NSOperationQueue::mainQueue();
    let foreground = RcBlock::new(move |_: NonNull<NSNotification>| {
        let _ = commands.send(Command::Sync);
    });
    let changed = RcBlock::new(|_: NonNull<NSNotification>| {
        let _ = slint::invoke_from_event_loop(crate::system_changed);
    });
    // UIKit's notification names are strings equal to their constants' names.
    let will_enter_foreground = NSString::from_str("UIApplicationWillEnterForegroundNotification");
    let did_become_active = NSString::from_str("UIApplicationDidBecomeActiveNotification");
    let watch = |name: &NSNotificationName, block: &RcBlock<dyn Fn(NonNull<NSNotification>)>| {
        // SAFETY: the blocks touch only a Sender and Slint's thread-safe queue, and run on the
        // main queue, the UI thread.
        unsafe {
            center.addObserverForName_object_queue_usingBlock(Some(name), None, Some(&queue), block)
        }
    };
    vec![
        watch(&will_enter_foreground, &foreground),
        watch(&did_become_active, &changed),
        // SAFETY: a Foundation constant, read once it is linked.
        watch(unsafe { NSCurrentLocaleDidChangeNotification }, &changed),
    ]
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
        Hinge::Absent => crate::crease::Reading::Silent,
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
