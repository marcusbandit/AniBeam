//! The picture: mpv's render API draws each frame into a texture on a GL context of its own,
//! which shares textures with the one Slint draws with, and Slint shows the texture as an
//! image under the controls.
//!
//! The render thread never touches Slint's context, so mpv cannot leave GL state behind for
//! Skia to trip over, and decoding a frame never holds up a touch. Three textures turn in a
//! ring: mpv draws the next while Slint shows the last.
//!
//! Slint's context goes when the app leaves the screen and a new one comes back with it, so
//! the renderer is tied to the context it was started against; `app.rs` starts a fresh one
//! when the context changes.

// Off Android the types stand in for a player that never opens.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use std::sync::Arc;

use crate::mpv::Mpv;

/// One drawn frame: the texture, and its size in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub texture: u32,
    pub width: u32,
    pub height: u32,
}

pub type OnFrame = Box<dyn Fn(Frame) + Send + 'static>;
/// Called once from the renderer's thread: Ok when mpv has its render context, so a file
/// loaded from then on gets a picture; Err with the reason when EGL or mpv refused.
pub type OnStarted = Box<dyn FnOnce(Result<(), String>) + Send + 'static>;

/// Slint's EGL context, read while it is current inside the rendering notifier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Share {
    display: usize,
    context: usize,
    config: usize,
}

#[cfg(target_os = "android")]
pub use imp::{Video, current_share};

#[cfg(not(target_os = "android"))]
pub struct Video;

#[cfg(not(target_os = "android"))]
impl Video {
    pub fn start(_mpv: &Arc<Mpv>, _share: Share, _size: (u32, u32), _on_frame: OnFrame, _on_started: OnStarted) -> Option<Video> {
        None
    }
    pub fn resize(&self, _size: (u32, u32)) {}
    pub fn share(&self) -> Share {
        Share { display: 0, context: 0, config: 0 }
    }
}

#[cfg(not(target_os = "android"))]
pub fn current_share() -> Option<Share> {
    None
}

#[cfg(target_os = "android")]
mod imp {
    #![allow(non_snake_case)]
    use super::*;
    use std::ffi::{CStr, c_char, c_int, c_uint, c_void};
    use std::sync::{Condvar, Mutex};

    use crate::mpv::ffi::*;

    type EGLDisplay = *mut c_void;
    type EGLContext = *mut c_void;
    type EGLConfig = *mut c_void;
    type EGLSurface = *mut c_void;

    const EGL_NONE: i32 = 0x3038;
    const EGL_CONFIG_ID: i32 = 0x3028;
    const EGL_CONTEXT_CLIENT_VERSION: i32 = 0x3098;
    const EGL_WIDTH: i32 = 0x3057;
    const EGL_HEIGHT: i32 = 0x3056;

    unsafe extern "C" {
        fn eglGetCurrentDisplay() -> EGLDisplay;
        fn eglGetCurrentContext() -> EGLContext;
        fn eglQueryContext(d: EGLDisplay, c: EGLContext, attr: i32, value: *mut i32) -> c_uint;
        fn eglChooseConfig(d: EGLDisplay, attrs: *const i32, configs: *mut EGLConfig, size: i32, n: *mut i32) -> c_uint;
        fn eglCreateContext(d: EGLDisplay, config: EGLConfig, share: EGLContext, attrs: *const i32) -> EGLContext;
        fn eglDestroyContext(d: EGLDisplay, c: EGLContext) -> c_uint;
        fn eglCreatePbufferSurface(d: EGLDisplay, config: EGLConfig, attrs: *const i32) -> EGLSurface;
        fn eglDestroySurface(d: EGLDisplay, s: EGLSurface) -> c_uint;
        fn eglMakeCurrent(d: EGLDisplay, draw: EGLSurface, read: EGLSurface, c: EGLContext) -> c_uint;
        fn eglReleaseThread() -> c_uint;
        fn eglGetError() -> i32;
        fn eglGetProcAddress(name: *const c_char) -> *mut c_void;

        fn glGenTextures(n: c_int, out: *mut c_uint);
        fn glDeleteTextures(n: c_int, ids: *const c_uint);
        fn glBindTexture(target: c_uint, id: c_uint);
        fn glTexImage2D(
            target: c_uint,
            level: c_int,
            internal: c_int,
            w: c_int,
            h: c_int,
            border: c_int,
            format: c_uint,
            ty: c_uint,
            data: *const c_void,
        );
        fn glTexParameteri(target: c_uint, name: c_uint, value: c_int);
        fn glGenFramebuffers(n: c_int, out: *mut c_uint);
        fn glDeleteFramebuffers(n: c_int, ids: *const c_uint);
        fn glBindFramebuffer(target: c_uint, id: c_uint);
        fn glFramebufferTexture2D(target: c_uint, attachment: c_uint, textarget: c_uint, texture: c_uint, level: c_int);
        fn glCheckFramebufferStatus(target: c_uint) -> c_uint;
        fn glFinish();

        fn dlopen(name: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    }

    const GL_TEXTURE_2D: c_uint = 0x0DE1;
    const GL_RGBA: c_uint = 0x1908;
    const GL_RGBA8: c_int = 0x8058;
    const GL_UNSIGNED_BYTE: c_uint = 0x1401;
    const GL_TEXTURE_MIN_FILTER: c_uint = 0x2801;
    const GL_TEXTURE_MAG_FILTER: c_uint = 0x2800;
    const GL_TEXTURE_WRAP_S: c_uint = 0x2802;
    const GL_TEXTURE_WRAP_T: c_uint = 0x2803;
    const GL_LINEAR: c_int = 0x2601;
    const GL_CLAMP_TO_EDGE: c_int = 0x812F;
    const GL_FRAMEBUFFER: c_uint = 0x8D40;
    const GL_COLOR_ATTACHMENT0: c_uint = 0x8CE0;
    const GL_FRAMEBUFFER_COMPLETE: c_uint = 0x8CD5;

    /// Slint's context, while it is current: only inside the rendering notifier. The config
    /// lookup runs once per context, not once per frame.
    pub fn current_share() -> Option<Share> {
        thread_local! {
            static LAST: std::cell::Cell<Option<Share>> = const { std::cell::Cell::new(None) };
            /// A context whose config could not be found, so it is not asked for every frame.
            static MISSED: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
        }
        unsafe {
            let display = eglGetCurrentDisplay();
            let context = eglGetCurrentContext();
            if display.is_null() || context.is_null() {
                return None;
            }
            if let Some(last) = LAST.get()
                && last.display == display as usize
                && last.context == context as usize
            {
                return Some(last);
            }
            if MISSED.get() == (display as usize, context as usize) {
                return None;
            }
            let mut id = 0;
            eglQueryContext(display, context, EGL_CONFIG_ID, &mut id);
            let attrs = [EGL_CONFIG_ID, id, EGL_NONE];
            let mut config: EGLConfig = std::ptr::null_mut();
            let mut n = 0;
            if eglChooseConfig(display, attrs.as_ptr(), &mut config, 1, &mut n) == 0 || n < 1 {
                crate::local::log(&format!("video: config {id} not found (0x{:x})", eglGetError()));
                MISSED.set((display as usize, context as usize));
                return None;
            }
            let share = Share { display: display as usize, context: context as usize, config: config as usize };
            LAST.set(Some(share));
            Some(share)
        }
    }

    unsafe extern "C" fn proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
        // eglGetProcAddress answers for extensions everywhere and for core calls on most
        // drivers; the GLES library itself answers for the rest.
        static GLES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let p = unsafe { eglGetProcAddress(name) };
        if !p.is_null() {
            return p;
        }
        let lib = *GLES.get_or_init(|| unsafe { dlopen(c"libGLESv3.so".as_ptr(), 2) as usize });
        if lib == 0 { std::ptr::null_mut() } else { unsafe { dlsym(lib as *mut c_void, name) } }
    }

    #[derive(Default)]
    struct State {
        dirty: bool,
        size: (u32, u32),
        stop: bool,
    }

    struct Shared {
        state: Mutex<State>,
        wake: Condvar,
    }

    unsafe extern "C" fn on_update(ctx: *mut c_void) {
        let shared = unsafe { &*(ctx as *const Shared) };
        shared.state.lock().unwrap().dirty = true;
        shared.wake.notify_one();
    }

    pub struct Video {
        shared: Arc<Shared>,
        share: Share,
        thread: Option<std::thread::JoinHandle<()>>,
    }

    impl Video {
        /// Starts drawing `mpv`'s picture at `size` on a context shared with `share`. None
        /// when the thread would not start; whether EGL and mpv took it comes later, through
        /// `on_started`.
        pub fn start(mpv: &Arc<Mpv>, share: Share, size: (u32, u32), on_frame: OnFrame, on_started: OnStarted) -> Option<Video> {
            let shared = Arc::new(Shared {
                state: Mutex::new(State { dirty: true, size, stop: false }),
                wake: Condvar::new(),
            });
            let (ready_tx, ready_rx) = std::sync::mpsc::channel();
            let thread = {
                let shared = shared.clone();
                let mpv = mpv.clone();
                std::thread::Builder::new()
                    .name("mpv-render".into())
                    .spawn(move || render(mpv, share, shared, on_frame, ready_tx))
                    .ok()?
            };
            // The UI thread never waits on the renderer's start: EGL and mpv can take their
            // time there, and a touch must not.
            std::thread::spawn(move || {
                let r = ready_rx.recv().unwrap_or_else(|_| Err("the renderer stopped".into()));
                if let Err(e) = &r {
                    crate::local::log(&format!("video: {e}"));
                }
                on_started(r);
            });
            Some(Video { shared, share, thread: Some(thread) })
        }

        pub fn resize(&self, size: (u32, u32)) {
            let mut s = self.shared.state.lock().unwrap();
            if s.size != size {
                s.size = size;
                s.dirty = true;
                self.shared.wake.notify_one();
            }
        }

        pub fn share(&self) -> Share {
            self.share
        }
    }

    impl Drop for Video {
        fn drop(&mut self) {
            self.shared.state.lock().unwrap().stop = true;
            self.shared.wake.notify_one();
            if let Some(t) = self.thread.take() {
                let _ = t.join();
            }
        }
    }

    struct Target {
        textures: [c_uint; 3],
        fbos: [c_uint; 3],
        size: (u32, u32),
    }

    impl Target {
        unsafe fn new(size: (u32, u32)) -> Option<Target> {
            let mut t = Target { textures: [0; 3], fbos: [0; 3], size };
            unsafe {
                glGenTextures(3, t.textures.as_mut_ptr());
                glGenFramebuffers(3, t.fbos.as_mut_ptr());
                for i in 0..3 {
                    glBindTexture(GL_TEXTURE_2D, t.textures[i]);
                    glTexImage2D(
                        GL_TEXTURE_2D,
                        0,
                        GL_RGBA8,
                        size.0 as c_int,
                        size.1 as c_int,
                        0,
                        GL_RGBA,
                        GL_UNSIGNED_BYTE,
                        std::ptr::null(),
                    );
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
                    glBindFramebuffer(GL_FRAMEBUFFER, t.fbos[i]);
                    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, t.textures[i], 0);
                    if glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                        t.free();
                        return None;
                    }
                }
                glBindTexture(GL_TEXTURE_2D, 0);
                glBindFramebuffer(GL_FRAMEBUFFER, 0);
            }
            Some(t)
        }

        unsafe fn free(&self) {
            unsafe {
                glDeleteFramebuffers(3, self.fbos.as_ptr());
                glDeleteTextures(3, self.textures.as_ptr());
            }
        }
    }

    fn render(
        mpv: Arc<Mpv>,
        share: Share,
        shared: Arc<Shared>,
        on_frame: OnFrame,
        ready: std::sync::mpsc::Sender<Result<(), String>>,
    ) {
        let display = share.display as EGLDisplay;
        let config = share.config as EGLConfig;
        unsafe {
            let attrs = [EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE];
            let mut context = eglCreateContext(display, config, share.context as EGLContext, attrs.as_ptr());
            if context.is_null() {
                let attrs = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
                context = eglCreateContext(display, config, share.context as EGLContext, attrs.as_ptr());
            }
            if context.is_null() {
                let _ = ready.send(Err(format!("no shared EGL context (0x{:x})", eglGetError())));
                return;
            }
            // Surfaceless where the driver allows it, else a one pixel pbuffer: the frames
            // go to textures, never to a surface of this context's own.
            let mut surface: EGLSurface = std::ptr::null_mut();
            if eglMakeCurrent(display, surface, surface, context) == 0 {
                let attrs = [EGL_WIDTH, 1, EGL_HEIGHT, 1, EGL_NONE];
                surface = eglCreatePbufferSurface(display, config, attrs.as_ptr());
                if surface.is_null() || eglMakeCurrent(display, surface, surface, context) == 0 {
                    let _ = ready.send(Err(format!("the EGL context would not bind (0x{:x})", eglGetError())));
                    if !surface.is_null() {
                        eglDestroySurface(display, surface);
                    }
                    eglDestroyContext(display, context);
                    return;
                }
            }

            let mut init = mpv_opengl_init_params {
                get_proc_address: Some(proc_address),
                get_proc_address_ctx: std::ptr::null_mut(),
            };
            let mut params = [
                mpv_render_param { kind: RENDER_PARAM_API_TYPE, data: c"opengl".as_ptr() as *mut c_void },
                mpv_render_param { kind: RENDER_PARAM_OPENGL_INIT_PARAMS, data: (&mut init as *mut mpv_opengl_init_params).cast() },
                mpv_render_param { kind: RENDER_PARAM_INVALID, data: std::ptr::null_mut() },
            ];
            let mut ctx: *mut mpv_render_context = std::ptr::null_mut();
            let code = mpv_render_context_create(&mut ctx, mpv.handle, params.as_mut_ptr());
            if code < 0 {
                let word = CStr::from_ptr(mpv_error_string(code)).to_string_lossy().into_owned();
                let _ = ready.send(Err(format!("mpv's render context: {word}")));
                eglMakeCurrent(display, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
                if !surface.is_null() {
                    eglDestroySurface(display, surface);
                }
                eglDestroyContext(display, context);
                return;
            }
            mpv_render_context_set_update_callback(ctx, Some(on_update), Arc::as_ptr(&shared) as *mut c_void);
            let _ = ready.send(Ok(()));

            let mut target: Option<Target> = None;
            // The textures of the size before, kept until two frames of the new size have
            // gone out, since Slint may still be showing one of them.
            let mut retired: Vec<(Target, u8)> = Vec::new();
            let mut next = 0usize;
            loop {
                let size = {
                    let mut s = shared.state.lock().unwrap();
                    while !s.dirty && !s.stop {
                        s = shared.wake.wait(s).unwrap();
                    }
                    if s.stop {
                        break;
                    }
                    s.dirty = false;
                    s.size
                };
                let flags = mpv_render_context_update(ctx);
                if size.0 == 0 || size.1 == 0 {
                    continue;
                }
                let resized = target.as_ref().is_none_or(|t| t.size != size);
                if resized {
                    if let Some(t) = target.take() {
                        retired.push((t, 0));
                    }
                    target = Target::new(size);
                    if target.is_none() {
                        crate::local::log("video: the frame textures would not build");
                        continue;
                    }
                }
                // A new size draws the frame on screen again, even when paused.
                if flags & RENDER_UPDATE_FRAME == 0 && !resized {
                    continue;
                }
                let t = target.as_ref().unwrap();
                let mut fbo = mpv_opengl_fbo {
                    fbo: t.fbos[next] as c_int,
                    w: size.0 as c_int,
                    h: size.1 as c_int,
                    internal_format: GL_RGBA8,
                };
                // Unflipped: mpv then writes the picture's top into the texture's first row,
                // which is what Slint reads as TopLeft.
                let mut flip: c_int = 0;
                let mut params = [
                    mpv_render_param { kind: RENDER_PARAM_OPENGL_FBO, data: (&mut fbo as *mut mpv_opengl_fbo).cast() },
                    mpv_render_param { kind: RENDER_PARAM_FLIP_Y, data: (&mut flip as *mut c_int).cast() },
                    mpv_render_param { kind: RENDER_PARAM_INVALID, data: std::ptr::null_mut() },
                ];
                mpv_render_context_render(ctx, params.as_mut_ptr());
                // The frame is whole before Slint's context samples it.
                glFinish();
                mpv_render_context_report_swap(ctx);
                on_frame(Frame { texture: t.textures[next], width: size.0, height: size.1 });
                next = (next + 1) % 3;
                for r in retired.iter_mut() {
                    r.1 += 1;
                }
                retired.retain(|(t, frames)| {
                    if *frames >= 2 {
                        t.free();
                        false
                    } else {
                        true
                    }
                });
            }

            mpv_render_context_free(ctx);
            if let Some(t) = target.take() {
                t.free();
            }
            for (t, _) in retired.drain(..) {
                t.free();
            }
            eglMakeCurrent(display, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut());
            if !surface.is_null() {
                eglDestroySurface(display, surface);
            }
            eglDestroyContext(display, context);
            eglReleaseThread();
        }
    }
}
