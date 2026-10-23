//! libmpv, the desktop's player, over its C API: one core per open episode, its events on a
//! thread of their own, and the picture drawn through the render API by `video.rs`.
//!
//! The libraries come from mpv-android's releases (`scripts/fetch-mpv.sh`). Off Android
//! there is no libmpv to link, so the same type opens nothing and every call is a no-op;
//! the rest of the app, and its tests, build on the desktop all the same.

// Off Android the types stand in for a player that never opens.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use std::sync::Arc;

/// A property the player watches, with its value as mpv sent it.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Flag(bool),
    Double(f64),
    Text(String),
    /// The property has no value right now, such as `time-pos` before a file plays.
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MpvEvent {
    FileLoaded,
    /// The file ended or failed to open; `error` carries mpv's word when it failed.
    EndFile { error: Option<String> },
    Property(&'static str, Value),
}

/// What the player observes, with the format each is read in. `track-list` comes as JSON
/// text, the same list the desktop parses.
pub const OBSERVED: [(&str, Format); 9] = [
    ("time-pos", Format::Double),
    ("duration", Format::Double),
    ("pause", Format::Flag),
    ("eof-reached", Format::Flag),
    ("seeking", Format::Flag),
    ("paused-for-cache", Format::Flag),
    ("track-list", Format::Text),
    ("aid", Format::Text),
    ("sid", Format::Text),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Flag,
    Double,
    Text,
}

/// The options set before mpv starts, the phone's own: the picture goes through the render
/// API, decoding uses MediaCodec with a copy back, which works with any GL context, and
/// sound goes to AudioTrack. No GL interop for hardware frames: the copy does not need it,
/// and the AImageReader one wants a Surface the render API never gives it. AniBeam's owned
/// options and the subtitle table follow at load.
pub const BOOT_OPTIONS: [(&str, &str); 7] = [
    ("gpu-hwdec-interop", "no"),
    ("vo", "libmpv"),
    ("opengl-es", "yes"),
    ("hwdec", "mediacodec-copy"),
    ("ao", "audiotrack,opensles"),
    ("idle", "yes"),
    ("terminal", "no"),
];

#[cfg(target_os = "android")]
pub use imp::Mpv;
#[cfg(not(target_os = "android"))]
pub use host::Mpv;

pub type Listener = Box<dyn Fn(MpvEvent) + Send + 'static>;

#[cfg(target_os = "android")]
pub(crate) mod ffi {
    #![allow(non_camel_case_types, dead_code)]
    use std::ffi::{c_char, c_int, c_void};

    #[repr(C)]
    pub struct mpv_handle {
        _p: [u8; 0],
    }
    #[repr(C)]
    pub struct mpv_render_context {
        _p: [u8; 0],
    }
    #[repr(C)]
    pub struct mpv_event {
        pub event_id: c_int,
        pub error: c_int,
        pub reply_userdata: u64,
        pub data: *mut c_void,
    }
    #[repr(C)]
    pub struct mpv_event_property {
        pub name: *const c_char,
        pub format: c_int,
        pub data: *mut c_void,
    }
    #[repr(C)]
    pub struct mpv_event_log_message {
        pub prefix: *const c_char,
        pub level: *const c_char,
        pub text: *const c_char,
        pub log_level: c_int,
    }
    #[repr(C)]
    pub struct mpv_event_end_file {
        pub reason: c_int,
        pub error: c_int,
        pub playlist_entry_id: i64,
        pub playlist_insert_id: i64,
        pub playlist_insert_num_entries: c_int,
    }
    #[repr(C)]
    pub struct mpv_render_param {
        pub kind: c_int,
        pub data: *mut c_void,
    }
    #[repr(C)]
    pub struct mpv_opengl_init_params {
        pub get_proc_address: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void>,
        pub get_proc_address_ctx: *mut c_void,
    }
    #[repr(C)]
    pub struct mpv_opengl_fbo {
        pub fbo: c_int,
        pub w: c_int,
        pub h: c_int,
        pub internal_format: c_int,
    }

    pub const FORMAT_NONE: c_int = 0;
    pub const FORMAT_STRING: c_int = 1;
    pub const FORMAT_FLAG: c_int = 3;
    pub const FORMAT_DOUBLE: c_int = 5;

    pub const EVENT_NONE: c_int = 0;
    pub const EVENT_SHUTDOWN: c_int = 1;
    pub const EVENT_LOG_MESSAGE: c_int = 2;
    pub const EVENT_SET_PROPERTY_REPLY: c_int = 4;
    pub const EVENT_COMMAND_REPLY: c_int = 5;
    pub const EVENT_END_FILE: c_int = 7;
    pub const EVENT_FILE_LOADED: c_int = 8;
    pub const EVENT_PROPERTY_CHANGE: c_int = 22;

    pub const END_FILE_REASON_ERROR: c_int = 4;

    pub const RENDER_PARAM_INVALID: c_int = 0;
    pub const RENDER_PARAM_API_TYPE: c_int = 1;
    pub const RENDER_PARAM_OPENGL_INIT_PARAMS: c_int = 2;
    pub const RENDER_PARAM_OPENGL_FBO: c_int = 3;
    pub const RENDER_PARAM_FLIP_Y: c_int = 4;
    pub const RENDER_UPDATE_FRAME: u64 = 1;

    unsafe extern "C" {
        pub fn mpv_create() -> *mut mpv_handle;
        pub fn mpv_initialize(ctx: *mut mpv_handle) -> c_int;
        pub fn mpv_terminate_destroy(ctx: *mut mpv_handle);
        pub fn mpv_set_option_string(ctx: *mut mpv_handle, name: *const c_char, data: *const c_char) -> c_int;
        pub fn mpv_set_property_string(ctx: *mut mpv_handle, name: *const c_char, data: *const c_char) -> c_int;
        pub fn mpv_set_property_async(
            ctx: *mut mpv_handle,
            reply_userdata: u64,
            name: *const c_char,
            format: c_int,
            data: *mut c_void,
        ) -> c_int;
        pub fn mpv_get_property_string(ctx: *mut mpv_handle, name: *const c_char) -> *mut c_char;
        pub fn mpv_free(data: *mut c_void);
        pub fn mpv_command(ctx: *mut mpv_handle, args: *mut *const c_char) -> c_int;
        pub fn mpv_command_async(ctx: *mut mpv_handle, reply_userdata: u64, args: *mut *const c_char) -> c_int;
        pub fn mpv_observe_property(ctx: *mut mpv_handle, reply_userdata: u64, name: *const c_char, format: c_int) -> c_int;
        pub fn mpv_wait_event(ctx: *mut mpv_handle, timeout: f64) -> *mut mpv_event;
        pub fn mpv_wakeup(ctx: *mut mpv_handle);
        pub fn mpv_request_log_messages(ctx: *mut mpv_handle, min_level: *const c_char) -> c_int;
        pub fn mpv_error_string(error: c_int) -> *const c_char;

        pub fn mpv_render_context_create(
            res: *mut *mut mpv_render_context,
            mpv: *mut mpv_handle,
            params: *mut mpv_render_param,
        ) -> c_int;
        pub fn mpv_render_context_set_update_callback(
            ctx: *mut mpv_render_context,
            callback: Option<unsafe extern "C" fn(*mut c_void)>,
            callback_ctx: *mut c_void,
        );
        pub fn mpv_render_context_update(ctx: *mut mpv_render_context) -> u64;
        pub fn mpv_render_context_render(ctx: *mut mpv_render_context, params: *mut mpv_render_param) -> c_int;
        pub fn mpv_render_context_report_swap(ctx: *mut mpv_render_context);
        pub fn mpv_render_context_free(ctx: *mut mpv_render_context);

        /// libavcodec's: MediaCodec and AudioTrack reach Java through the VM it is given.
        pub fn av_jni_set_java_vm(vm: *mut c_void, log_ctx: *mut c_void) -> c_int;
    }
}

#[cfg(target_os = "android")]
mod imp {
    use super::*;
    use std::ffi::{CStr, CString, c_char};
    use std::sync::Once;

    use ffi::*;

    pub struct Mpv {
        pub(crate) handle: *mut mpv_handle,
        events: std::sync::Mutex<Option<std::thread::JoinHandle<()>>>,
    }

    // libmpv's client API is thread safe: every call takes the core's own lock.
    unsafe impl Send for Mpv {}
    unsafe impl Sync for Mpv {}

    fn c(s: &str) -> CString {
        CString::new(s.replace('\0', "")).unwrap()
    }

    /// Whether this is the Android emulator rather than a phone.
    fn emulator() -> bool {
        unsafe extern "C" {
            fn __system_property_get(name: *const c_char, value: *mut c_char) -> i32;
        }
        let mut value = [0 as c_char; 92];
        let n = unsafe { __system_property_get(c"ro.hardware".as_ptr(), value.as_mut_ptr()) };
        let hw = unsafe { CStr::from_ptr(value.as_ptr()) }.to_string_lossy();
        n > 0 && (hw == "ranchu" || hw == "goldfish")
    }

    fn error(code: i32) -> String {
        unsafe { CStr::from_ptr(mpv_error_string(code)) }.to_string_lossy().into_owned()
    }

    impl Mpv {
        pub fn new(listener: Listener) -> Result<Arc<Mpv>, String> {
            static VM: Once = Once::new();
            VM.call_once(|| unsafe {
                av_jni_set_java_vm(ndk_context::android_context().vm(), std::ptr::null_mut());
            });
            let handle = unsafe { mpv_create() };
            if handle.is_null() {
                return Err("mpv did not start".into());
            }
            for (k, v) in BOOT_OPTIONS {
                // The emulator's MediaCodec H.264 decoder (c2.goldfish) streaks the picture
                // with black lines through the copy; software decoding there is clean.
                let v = if k == "hwdec" && emulator() { "no" } else { v };
                let (k, v) = (c(k), c(v));
                unsafe { mpv_set_option_string(handle, k.as_ptr(), v.as_ptr()) };
            }
            // mpv's own warnings and errors go to logcat beside the app's.
            unsafe { mpv_request_log_messages(handle, c"warn".as_ptr()) };
            let code = unsafe { mpv_initialize(handle) };
            if code < 0 {
                unsafe { mpv_terminate_destroy(handle) };
                return Err(format!("mpv did not start: {}", error(code)));
            }
            for (i, (name, format)) in OBSERVED.iter().enumerate() {
                let format = match format {
                    Format::Flag => FORMAT_FLAG,
                    Format::Double => FORMAT_DOUBLE,
                    Format::Text => FORMAT_STRING,
                };
                let name = c(name);
                unsafe { mpv_observe_property(handle, i as u64, name.as_ptr(), format) };
            }
            let mpv = Arc::new(Mpv { handle, events: std::sync::Mutex::new(None) });
            let raw = handle as usize;
            let thread = std::thread::Builder::new()
                .name("mpv-events".into())
                .spawn(move || pump(raw as *mut mpv_handle, listener))
                .map_err(|e| e.to_string())?;
            *mpv.events.lock().unwrap() = Some(thread);
            Ok(mpv)
        }

        /// Sets a property without waiting on the core, so a tap never holds the UI thread
        /// while a slow file opens. mpv copies the value and keeps the order of the calls.
        pub fn set(&self, name: &str, value: &str) {
            let (k, v) = (c(name), c(value));
            let mut p = v.as_ptr();
            let code = unsafe {
                mpv_set_property_async(self.handle, 0, k.as_ptr(), FORMAT_STRING, (&mut p as *mut *const c_char).cast())
            };
            if code < 0 {
                crate::local::log(&format!("mpv: {name}={value}: {}", error(code)));
            }
        }

        pub fn command(&self, args: &[&str]) {
            let owned: Vec<CString> = args.iter().map(|a| c(a)).collect();
            let mut ptrs: Vec<*const c_char> = owned.iter().map(|a| a.as_ptr()).collect();
            ptrs.push(std::ptr::null());
            // Async, so a seek on a slow file never holds the UI thread; mpv keeps the order.
            let code = unsafe { mpv_command_async(self.handle, 0, ptrs.as_mut_ptr()) };
            if code < 0 {
                crate::local::log(&format!("mpv: {args:?}: {}", error(code)));
            }
        }

        /// A blocking command, for the few whose effect the next line reads (sub-add).
        pub fn command_now(&self, args: &[&str]) -> bool {
            let owned: Vec<CString> = args.iter().map(|a| c(a)).collect();
            let mut ptrs: Vec<*const c_char> = owned.iter().map(|a| a.as_ptr()).collect();
            ptrs.push(std::ptr::null());
            unsafe { mpv_command(self.handle, ptrs.as_mut_ptr()) >= 0 }
        }

        pub fn get(&self, name: &str) -> Option<String> {
            let k = c(name);
            let p = unsafe { mpv_get_property_string(self.handle, k.as_ptr()) };
            if p.is_null() {
                return None;
            }
            let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
            unsafe { mpv_free(p.cast()) };
            Some(s)
        }
    }

    impl Drop for Mpv {
        fn drop(&mut self) {
            // The event thread ends on the shutdown event that terminate sends; it must be
            // gone before the handle is, so it is woken and joined first.
            self.command(&["quit"]);
            unsafe { mpv_wakeup(self.handle) };
            if let Some(t) = self.events.lock().unwrap().take() {
                let _ = t.join();
            }
            unsafe { mpv_terminate_destroy(self.handle) };
        }
    }

    fn pump(handle: *mut mpv_handle, listener: Listener) {
        loop {
            let e = unsafe { &*mpv_wait_event(handle, -1.0) };
            match e.event_id {
                EVENT_SHUTDOWN => return,
                EVENT_FILE_LOADED => listener(MpvEvent::FileLoaded),
                // The async sets and commands answer here; only a refusal is worth a line.
                EVENT_SET_PROPERTY_REPLY | EVENT_COMMAND_REPLY if e.error < 0 => {
                    crate::local::log(&format!("mpv: an async request failed: {}", super::imp::error(e.error)));
                }
                EVENT_LOG_MESSAGE => {
                    let m = unsafe { &*(e.data as *const mpv_event_log_message) };
                    let (prefix, text) = unsafe { (CStr::from_ptr(m.prefix), CStr::from_ptr(m.text)) };
                    crate::local::log(&format!(
                        "mpv[{}] {}",
                        prefix.to_string_lossy(),
                        text.to_string_lossy().trim_end()
                    ));
                }
                EVENT_END_FILE => {
                    let end = unsafe { &*(e.data as *const mpv_event_end_file) };
                    let error = (end.reason == END_FILE_REASON_ERROR).then(|| super::imp::error(end.error));
                    listener(MpvEvent::EndFile { error });
                }
                EVENT_PROPERTY_CHANGE => {
                    let p = unsafe { &*(e.data as *const mpv_event_property) };
                    let Some((name, _)) = OBSERVED.get(e.reply_userdata as usize) else { continue };
                    let value = unsafe {
                        match p.format {
                            FORMAT_FLAG => Value::Flag(*(p.data as *const i32) != 0),
                            FORMAT_DOUBLE => Value::Double(*(p.data as *const f64)),
                            FORMAT_STRING => {
                                Value::Text(CStr::from_ptr(*(p.data as *const *const c_char)).to_string_lossy().into_owned())
                            }
                            _ => Value::None,
                        }
                    };
                    listener(MpvEvent::Property(name, value));
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
mod host {
    use super::*;

    /// No libmpv off Android: nothing opens, so nothing plays.
    pub struct Mpv;

    impl Mpv {
        pub fn new(_listener: Listener) -> Result<Arc<Mpv>, String> {
            Err("the player runs on Android only".into())
        }
        pub fn set(&self, _name: &str, _value: &str) {}
        pub fn command(&self, _args: &[&str]) {}
        pub fn command_now(&self, _args: &[&str]) -> bool {
            false
        }
        pub fn get(&self, _name: &str) -> Option<String> {
            None
        }
    }
}
