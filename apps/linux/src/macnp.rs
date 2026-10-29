//! The Mac's Now Playing centre and media keys (#187), behind the same `Player.mpris*`
//! seam MPRIS fills on Linux. Every MediaPlayer and AppKit object lives in
//! cpp/nowplaying.mm; this side keeps the state the page publishes and hands it over as
//! plain C values. Compiled on macOS only.
//!
//! Every call into the .mm comes from a Player invokable, so it runs on the Qt thread,
//! which on the Mac is the main thread MediaPlayer wants. A remote command arrives on a
//! thread Apple does not name and only queues the page's signal, the same hop MPRIS makes
//! from the runtime.

use std::ffi::{CStr, CString, c_char};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use mpris_server::PlaybackStatus;

use crate::media::{State, carry_position, clock_rate, drifted};

// The frameworks cpp/nowplaying.mm calls, one block each as src/iokit.rs links its own.
#[link(name = "MediaPlayer", kind = "framework")]
unsafe extern "C" {}
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}

// cpp/nowplaying.mm. A status is 0 stopped, 1 playing, 2 paused; a stop releases the keys
// and empties Now Playing. The rate is the clock's, 0 while paused. The strings are copied
// before the call returns.
unsafe extern "C" {
    fn anibeam_np_set_command(command: extern "C" fn(name: *const c_char, value: f64));
    fn anibeam_np_publish(
        status: i32,
        title: *const c_char,
        artist: *const c_char,
        art_url: *const c_char,
        duration: f64,
        elapsed: f64,
        rate: f64,
        can_next: bool,
        can_prev: bool,
    );
    fn anibeam_np_elapsed(elapsed: f64);
}

/// A remote command from the .mm, on whatever thread Apple runs it. A panic must not unwind
/// into Objective-C, so it stops here.
extern "C" fn command(name: *const c_char, value: f64) {
    let r = std::panic::catch_unwind(|| {
        if name.is_null() {
            return;
        }
        // SAFETY: the .mm passes one of its own NUL-terminated string literals.
        let name = unsafe { CStr::from_ptr(name) };
        crate::media::send(&name.to_string_lossy(), value);
    });
    if r.is_err() {
        eprintln!("anibeam: now playing: a command panicked");
    }
}

/// Now Playing's clock as the system keeps it: the elapsed time last written, when, and
/// the rate written with it.
#[derive(Clone, Copy)]
struct Clock {
    elapsed: f64,
    at: Instant,
    rate: f64,
}

impl Clock {
    fn set(elapsed: f64, rate: f64) -> Self {
        Clock {
            elapsed,
            at: Instant::now(),
            rate,
        }
    }
}

struct Inner {
    state: State,
    clock: Clock,
}

pub struct Handle {
    inner: Mutex<Inner>,
}

impl Handle {
    /// Everything the page knows, with the position carried forward, as the whole
    /// dictionary at once. The elapsed time it carries is the last observed frame's.
    pub fn update(&self, next: State) {
        let mut next = next;
        let rate = clock_rate(&next);
        if let Ok(mut i) = self.inner.lock() {
            carry_position(&i.state, &mut next);
            i.state = next.clone();
            i.clock = Clock::set(next.position_secs, rate);
        }
        let status = match next.status {
            PlaybackStatus::Stopped => 0,
            PlaybackStatus::Playing => 1,
            PlaybackStatus::Paused => 2,
        };
        let text = |s: &str| CString::new(s).unwrap_or_default();
        let (title, artist) = (text(&next.title), text(&next.artist));
        let art = text(next.art_url.as_deref().unwrap_or_default());
        // SAFETY: on the Qt thread (see the module comment), and the three strings are NUL
        // terminated and outlive the call.
        unsafe {
            anibeam_np_publish(
                status,
                title.as_ptr(),
                artist.as_ptr(),
                art.as_ptr(),
                next.length_secs,
                next.position_secs,
                rate,
                next.can_next,
                next.can_prev,
            )
        }
    }
    /// Kept for the next publish, and written only when it leaves the clock the system
    /// runs by itself: a frame where that clock says is not news to it.
    pub fn position(&self, secs: f64) {
        let write = self.inner.lock().is_ok_and(|mut i| {
            i.state.position_secs = secs;
            let c = i.clock;
            let off = drifted(c.elapsed, c.rate, c.at.elapsed().as_secs_f64(), secs);
            if off {
                i.clock = Clock::set(secs, c.rate);
            }
            off
        });
        if write {
            // SAFETY: on the Qt thread, as for update.
            unsafe { anibeam_np_elapsed(secs) }
        }
    }
    /// A seek is the jump the clock cannot see, so it is written at once.
    pub fn seeked(&self, secs: f64) {
        if let Ok(mut i) = self.inner.lock() {
            i.state.position_secs = secs;
            let rate = clock_rate(&i.state);
            i.clock = Clock::set(secs, rate);
        }
        // SAFETY: on the Qt thread, as for update.
        unsafe { anibeam_np_elapsed(secs) }
    }
}

static HANDLE: OnceLock<Handle> = OnceLock::new();

/// Made by the first publish, on the Qt thread. Nothing here needs the runtime, and
/// registering the callback creates no MediaPlayer object: the keys are only taken once a
/// page has something playing.
pub fn handle() -> Option<&'static Handle> {
    Some(HANDLE.get_or_init(|| {
        // SAFETY: `command` is a plain function that lives for the whole process.
        unsafe { anibeam_np_set_command(command) };
        Handle {
            inner: Mutex::new(Inner {
                state: State::default(),
                clock: Clock::set(0.0, 0.0),
            }),
        }
    }))
}
