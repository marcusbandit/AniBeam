//! The seek preview: a second libmpv core on the same file, with no picture and no sound,
//! that seeks to the keyframe under the finger and hands back a small frame for the bubble
//! over the seek bar. It lives on a thread of its own and only ever works on the newest
//! spot, so a fast drag never queues a backlog.

#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use std::sync::mpsc::{Receiver, Sender, channel};

/// One preview frame, RGBA, and the second it shows.
pub struct Shot {
    pub at: f64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// The preview core for one open episode. Dropping it ends the thread, which closes the core.
pub struct Thumbs {
    tx: Sender<f64>,
}

impl Thumbs {
    pub fn open(path: &str, on_shot: impl Fn(Shot) + Send + 'static) -> Option<Thumbs> {
        let (tx, rx) = channel();
        let path = path.to_string();
        std::thread::Builder::new()
            .name("mpv-thumbs".into())
            .spawn(move || imp::run(&path, rx, on_shot))
            .ok()?;
        Some(Thumbs { tx })
    }

    /// Ask for the frame at `t` seconds; a newer ask replaces one not yet started.
    pub fn at(&self, t: f64) {
        let _ = self.tx.send(t);
    }
}

/// Waits for an ask, then takes every ask already queued behind it and keeps the newest.
fn newest(rx: &Receiver<f64>) -> Option<f64> {
    let mut t = rx.recv().ok()?;
    while let Ok(n) = rx.try_recv() {
        t = n;
    }
    Some(t)
}

/// The width the preview frames are scaled to before they leave mpv.
const WIDTH: u32 = 320;

#[cfg(target_os = "android")]
mod imp {
    use super::*;
    use crate::mpv::ffi::*;
    use std::ffi::{CStr, CString, c_char, c_int, c_void};

    #[repr(C)]
    struct Node {
        u: NodeUnion,
        format: c_int,
    }
    #[repr(C)]
    union NodeUnion {
        string: *mut c_char,
        flag: c_int,
        int64: i64,
        double: f64,
        list: *mut NodeList,
        ba: *mut ByteArray,
    }
    #[repr(C)]
    struct NodeList {
        num: c_int,
        values: *mut Node,
        keys: *mut *mut c_char,
    }
    #[repr(C)]
    struct ByteArray {
        data: *mut c_void,
        size: usize,
    }

    const FORMAT_INT64: c_int = 4;
    const FORMAT_NODE_MAP: c_int = 8;
    const FORMAT_BYTE_ARRAY: c_int = 9;
    const EVENT_PLAYBACK_RESTART: c_int = 21;

    unsafe extern "C" {
        fn mpv_command_ret(ctx: *mut mpv_handle, args: *mut *const c_char, result: *mut Node) -> c_int;
        fn mpv_free_node_contents(node: *mut Node);
    }

    fn c(s: &str) -> CString {
        CString::new(s.replace('\0', "")).unwrap()
    }

    fn command(h: *mut mpv_handle, args: &[&str]) -> bool {
        let owned: Vec<CString> = args.iter().map(|a| c(a)).collect();
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|a| a.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        unsafe { mpv_command(h, ptrs.as_mut_ptr()) >= 0 }
    }

    /// Waits up to `secs` for an event of `id`; false on a timeout, an end or a shutdown.
    fn wait_for(h: *mut mpv_handle, id: c_int, secs: f64) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs_f64(secs);
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now()).as_secs_f64();
            if left <= 0.0 {
                return false;
            }
            let e = unsafe { &*mpv_wait_event(h, left) };
            match e.event_id {
                x if x == id => return true,
                EVENT_LOG_MESSAGE => {
                    let m = unsafe { &*(e.data as *const mpv_event_log_message) };
                    let (prefix, text) = unsafe { (CStr::from_ptr(m.prefix), CStr::from_ptr(m.text)) };
                    crate::local::log(&format!("thumbs[{}] {}", prefix.to_string_lossy(), text.to_string_lossy().trim_end()));
                }
                EVENT_SHUTDOWN | EVENT_END_FILE => return false,
                _ => {}
            }
        }
    }

    pub fn run(path: &str, rx: Receiver<f64>, on_shot: impl Fn(Shot)) {
        let h = unsafe { mpv_create() };
        if h.is_null() {
            return;
        }
        let scale = format!("scale=w={WIDTH}:h=-2");
        let options = [
            ("vo", "null"),
            ("ao", "null"),
            ("aid", "no"),
            ("sid", "no"),
            // Software decoding: a thumbnail needs the frame in memory, and MediaCodec is
            // busy with the episode itself.
            ("hwdec", "no"),
            ("pause", "yes"),
            ("keep-open", "yes"),
            ("idle", "yes"),
            ("terminal", "no"),
            ("hr-seek", "no"),
            ("vd-lavc-skiploopfilter", "all"),
            ("vd-lavc-fast", "yes"),
            ("vd-lavc-threads", "2"),
            ("cache", "no"),
            ("vf", scale.as_str()),
        ];
        for (k, v) in options {
            let (k, v) = (c(k), c(v));
            unsafe { mpv_set_option_string(h, k.as_ptr(), v.as_ptr()) };
        }
        unsafe { mpv_request_log_messages(h, c"warn".as_ptr()) };
        if unsafe { mpv_initialize(h) } < 0 {
            unsafe { mpv_terminate_destroy(h) };
            return;
        }
        let loaded = command(h, &["loadfile", path]) && wait_for(h, EVENT_FILE_LOADED, 10.0);
        if loaded {
            while let Some(t) = newest(&rx) {
                if !command(h, &["seek", &t.to_string(), "absolute+keyframes"]) {
                    continue;
                }
                if !wait_for(h, EVENT_PLAYBACK_RESTART, 3.0) {
                    crate::local::log("thumbs: no restart after seek");
                    continue;
                }
                match grab(h) {
                    Some(mut shot) => {
                        shot.at = t;
                        on_shot(shot);
                    }
                    None => crate::local::log("thumbs: no frame"),
                }
            }
        }
        unsafe { mpv_terminate_destroy(h) };
    }

    /// `screenshot-raw video`: the decoded frame after the scale, bgr0, as a node map.
    fn grab(h: *mut mpv_handle) -> Option<Shot> {
        let owned = [c("screenshot-raw"), c("video")];
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|a| a.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        let mut node = Node { u: NodeUnion { int64: 0 }, format: 0 };
        let code = unsafe { mpv_command_ret(h, ptrs.as_mut_ptr(), &mut node) };
        if code < 0 {
            crate::local::log(&format!("thumbs: screenshot-raw failed {code}"));
            return None;
        }
        let shot = unsafe { read(&node) };
        unsafe { mpv_free_node_contents(&mut node) };
        shot
    }

    unsafe fn read(node: &Node) -> Option<Shot> {
        if node.format != FORMAT_NODE_MAP {
            return None;
        }
        let list = unsafe { &*node.u.list };
        let (mut w, mut hgt, mut stride) = (0i64, 0i64, 0i64);
        let mut bytes: &[u8] = &[];
        let mut format = String::new();
        for i in 0..list.num as usize {
            let key = unsafe { CStr::from_ptr(*list.keys.add(i)) }.to_string_lossy();
            let v = unsafe { &*list.values.add(i) };
            match (key.as_ref(), v.format) {
                ("w", FORMAT_INT64) => w = unsafe { v.u.int64 },
                ("h", FORMAT_INT64) => hgt = unsafe { v.u.int64 },
                ("stride", FORMAT_INT64) => stride = unsafe { v.u.int64 },
                ("format", FORMAT_STRING) => format = unsafe { CStr::from_ptr(v.u.string) }.to_string_lossy().into_owned(),
                ("data", FORMAT_BYTE_ARRAY) => {
                    let ba = unsafe { &*v.u.ba };
                    bytes = unsafe { std::slice::from_raw_parts(ba.data as *const u8, ba.size) };
                }
                _ => {}
            }
        }
        if w <= 0 || hgt <= 0 || stride < w * 4 || bytes.len() < (stride * hgt) as usize || format != "bgr0" {
            crate::local::log(&format!("thumbs: odd frame {w}x{hgt} stride {stride} {format} {} bytes", bytes.len()));
            return None;
        }
        let (w, hgt, stride) = (w as usize, hgt as usize, stride as usize);
        let mut rgba = Vec::with_capacity(w * hgt * 4);
        for row in bytes.chunks(stride).take(hgt) {
            for px in row[..w * 4].chunks_exact(4) {
                rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
            }
        }
        Some(Shot { at: 0.0, width: w as u32, height: hgt as u32, rgba })
    }
}

#[cfg(not(target_os = "android"))]
mod imp {
    use super::*;

    /// No libmpv off Android: the asks are drained and nothing comes back.
    pub fn run(_path: &str, rx: Receiver<f64>, _on_shot: impl Fn(Shot)) {
        while newest(&rx).is_some() {}
    }
}
