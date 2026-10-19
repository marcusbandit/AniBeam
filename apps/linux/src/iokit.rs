//! The macOS display-sleep hold: an IOKit power assertion of the kind mpv's own Cocoa
//! window takes, which the render API never reaches (#78). Compiled on macOS only; the
//! whole file is two IOKit calls and the CFStrings they take.

use std::ffi::{c_char, c_void};

type CFStringRef = *const c_void;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: CFStringRef,
        assertion_level: u32,
        assertion_name: CFStringRef,
        assertion_id: *mut u32,
    ) -> i32;
    fn IOPMAssertionRelease(assertion_id: u32) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFRelease(cf: *const c_void);
}

/// kIOPMAssertionLevelOn and kCFStringEncodingUTF8, from IOPMLib.h and CFString.h.
const LEVEL_ON: u32 = 255;
const UTF8: u32 = 0x0800_0100;
const IO_SUCCESS: i32 = 0;

/// The assertion id IOPMAssertionRelease takes back.
pub struct Hold(u32);

pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Backend
    }

    pub async fn acquire(&mut self) -> Option<Hold> {
        // kIOPMAssertionTypePreventUserIdleDisplaySleep is this string.
        let kind = c"PreventUserIdleDisplaySleep";
        let name = c"AniBeam: playing an episode";
        let mut id = 0u32;
        // SAFETY: both strings are NUL terminated and outlive the calls, the CFStrings are
        // released once IOKit has taken its own references (CFRelease crashes on null, so a
        // failed create is never released), and `id` is a valid out pointer.
        let status = unsafe {
            let kind = CFStringCreateWithCString(std::ptr::null(), kind.as_ptr(), UTF8);
            let name = CFStringCreateWithCString(std::ptr::null(), name.as_ptr(), UTF8);
            let status = if kind.is_null() || name.is_null() {
                -1
            } else {
                IOPMAssertionCreateWithName(kind, LEVEL_ON, name, &mut id)
            };
            for s in [kind, name] {
                if !s.is_null() {
                    CFRelease(s);
                }
            }
            status
        };
        if status == IO_SUCCESS {
            Some(Hold(id))
        } else {
            eprintln!("anibeam: idle inhibit: IOPMAssertionCreateWithName returned {status}");
            None
        }
    }

    pub async fn release(&mut self, hold: Hold) {
        // SAFETY: the id came from a successful create and is released once, since the
        // hold is consumed.
        unsafe { IOPMAssertionRelease(hold.0) };
    }
}
