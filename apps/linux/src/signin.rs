//! The Mac's tracker sign-in window (#1176): the authorize page in an
//! ASWebAuthenticationSession window rather than a browser tab. Every AppKit object lives in
//! cpp/signin.mm; this side hands it the url. Compiled on macOS only, and called from the
//! Shell invokables only, so always on the Qt thread, which on the Mac is the main thread.

use std::ffi::{CString, c_char};

#[link(name = "AuthenticationServices", kind = "framework")]
unsafe extern "C" {}

// cpp/signin.mm. The string is copied before the call returns.
unsafe extern "C" {
    fn anibeam_signin_start(url: *const c_char) -> bool;
    fn anibeam_signin_close();
}

/// Opens the sign-in window on `url`, replacing one already open. False when it could not
/// start, and the caller opens the browser instead.
pub fn start(url: &str) -> bool {
    let Ok(url) = CString::new(url) else {
        return false;
    };
    // SAFETY: a NUL-terminated string that outlives the call; the .mm copies it.
    unsafe { anibeam_signin_start(url.as_ptr()) }
}

/// Closes the sign-in window if one is open.
pub fn close() {
    // SAFETY: takes nothing; a call with no window open does nothing.
    unsafe { anibeam_signin_close() }
}
