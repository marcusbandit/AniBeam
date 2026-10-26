//! The Windows display-sleep hold: a power request of the kind mpv's own window takes, which
//! the render API never reaches (#78). Compiled on Windows only; the whole file is four
//! kernel32 calls. A power request belongs to its handle, not to a thread, so the worker
//! task can take it on one tokio thread and drop it on another.

use std::ffi::c_void;

type Handle = *mut c_void;

/// REASON_CONTEXT with the simple string arm of its union; the padding is the larger
/// detailed arm, so the struct has the size the kernel expects.
#[repr(C)]
struct ReasonContext {
    version: u32,
    flags: u32,
    simple: *const u16,
    _detailed: [usize; 2],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn PowerCreateRequest(context: *const ReasonContext) -> Handle;
    fn PowerSetRequest(request: Handle, kind: i32) -> i32;
    fn PowerClearRequest(request: Handle, kind: i32) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
}

/// POWER_REQUEST_CONTEXT_VERSION, POWER_REQUEST_CONTEXT_SIMPLE_STRING and
/// PowerRequestDisplayRequired, from minwinbase.h and winnt.h.
const CONTEXT_VERSION: u32 = 0;
const SIMPLE_STRING: u32 = 1;
const DISPLAY_REQUIRED: i32 = 0;
const INVALID_HANDLE: isize = -1;

/// The request handle, as an address so the hold can cross threads.
pub struct Hold(isize);

pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Backend
    }

    pub async fn acquire(&mut self) -> Option<Hold> {
        let reason: Vec<u16> = "AniBeam: playing an episode"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let context = ReasonContext {
            version: CONTEXT_VERSION,
            flags: SIMPLE_STRING,
            simple: reason.as_ptr(),
            _detailed: [0; 2],
        };
        // SAFETY: the context and its NUL-terminated string outlive the create, which copies
        // the string; a failed create is never set or closed.
        unsafe {
            let request = PowerCreateRequest(&context);
            if request.is_null() || request as isize == INVALID_HANDLE {
                eprintln!("anibeam: idle inhibit: PowerCreateRequest failed");
                return None;
            }
            if PowerSetRequest(request, DISPLAY_REQUIRED) == 0 {
                eprintln!("anibeam: idle inhibit: PowerSetRequest failed");
                CloseHandle(request);
                return None;
            }
            Some(Hold(request as isize))
        }
    }

    pub async fn release(&mut self, hold: Hold) {
        // SAFETY: the handle came from a successful create and set, and is cleared and
        // closed once, since the hold is consumed.
        unsafe {
            let request = hold.0 as Handle;
            PowerClearRequest(request, DISPLAY_REQUIRED);
            CloseHandle(request);
        }
    }
}
