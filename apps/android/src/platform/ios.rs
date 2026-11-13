//! The iPhone: its entry point and the hinge. Every other call is `other.rs`'s stand-in
//! until the iPhone brings its own.
//!
//! The hinge comes from UIKit's `UIHingeInteraction` (iOS 27.1, the iPhone Duo), added to
//! the app's window the first time the layout asks. UIKit says how far the hinge is open,
//! not where the crease runs, so the crease is never reported and the layout takes it for a
//! book fold's, down the middle of the window: `fold` says which way the middle runs.

use std::cell::{Cell, RefCell};

use block2::RcBlock;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_core_foundation::CGRect;

pub use super::other::*;

/// Called by `main.rs`: runs the app over its data directory in the app's container.
pub fn ios_main() {
    // HOME is the app's own container on iOS. Application Support is the app's, and not
    // shown in Files.
    let data_dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("Library/Application Support/AniBeam");
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("AniBeam: no data directory at {}: {e}", data_dir.display());
    }
    let (commands, inbox) = std::sync::mpsc::channel();
    if let Err(e) = crate::run(data_dir, commands, inbox) {
        eprintln!("AniBeam stopped: {e}");
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
    /// No hinge here: an iPhone that does not fold, or an iOS older than 27.1.
    Absent,
    /// The hinge's angle in degrees, 0 shut and 180 flat.
    Angle(f32),
    /// A hinge answered before, but the window has left the hierarchy that reports it for
    /// now, as it may while the phone moves between its screens.
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
            match HINGE.get() {
                Hinge::Angle(_) | Hinge::Lost => Hinge::Lost,
                _ => Hinge::Absent,
            }
        } else {
            let status: isize = msg_send![hinge, status];
            let radians: f64 = msg_send![hinge, angle];
            Hinge::Angle(degrees(status, radians))
        }
    };
    HINGE.set(next);
}

/// The angle the layout takes: UIKit's status where it says shut, half or fully open,
/// since Apple has the angle come as often and as finely as the system likes, and the
/// angle itself otherwise.
fn degrees(status: isize, radians: f64) -> f32 {
    let angle = radians.to_degrees() as f32;
    match status {
        // UIHingeStatusClosed
        1 => 0.0,
        // UIHingeStatusPartiallyOpen: bent, whatever the angle reads.
        2 => angle.clamp(31.0, 169.0),
        // UIHingeStatusFullyOpen
        3 => 180.0,
        _ => angle,
    }
}

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
