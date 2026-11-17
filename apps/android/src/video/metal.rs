//! The iPhone's renderer. mpv has no Metal render API, only OpenGL ES, and Slint draws with
//! Metal through wgpu, so the two meet in IOSurfaces: mpv draws each frame on an EAGL
//! context of its own into a texture backed by an IOSurface, and the same IOSurface is a
//! Metal texture that Slint imports as a wgpu texture. No pixel is copied.
//!
//! The ring of three surfaces is built once per size; each slot's `slint::Image` is built
//! once on the UI thread and kept in `Images`. A Metal texture holds its IOSurface, so an
//! image the page still shows stays valid after the renderer lets go of the slot.

use std::ffi::{CStr, c_char, c_int, c_uint, c_void};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use objc2::msg_send;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyClass, AnyObject, ProtocolObject};
use objc2_core_foundation::{CFDictionary, CFNumber, CFRetained, CFString};
use objc2_io_surface::{
    IOSurfaceRef, kIOSurfaceBytesPerElement, kIOSurfaceHeight, kIOSurfacePixelFormat,
    kIOSurfaceWidth,
};
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage,
};
use slint::wgpu_30::wgpu;

use super::{OnFrame, OnStarted};
use crate::mpv::Mpv;
use crate::mpv::ffi::*;

unsafe extern "C" {
    fn glGenTextures(n: c_int, out: *mut c_uint);
    fn glDeleteTextures(n: c_int, ids: *const c_uint);
    fn glBindTexture(target: c_uint, id: c_uint);
    fn glTexParameteri(target: c_uint, name: c_uint, value: c_int);
    fn glGenFramebuffers(n: c_int, out: *mut c_uint);
    fn glDeleteFramebuffers(n: c_int, ids: *const c_uint);
    fn glBindFramebuffer(target: c_uint, id: c_uint);
    fn glFramebufferTexture2D(
        target: c_uint,
        attachment: c_uint,
        textarget: c_uint,
        texture: c_uint,
        level: c_int,
    );
    fn glCheckFramebufferStatus(target: c_uint) -> c_uint;
    fn glFinish();

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
/// dlsym's search of every loaded image, `RTLD_DEFAULT` on Apple's platforms.
const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;
/// `kCVPixelFormatType_32RGBA`: bytes in R, G, B, A order, what Slint's Metal import reads.
const PIXEL_RGBA: i32 = 0x5247_4241;

/// One drawn frame: the slot of the ring mpv drew it into.
#[derive(Clone)]
pub struct Frame {
    ring: Arc<Ring>,
    slot: usize,
}

impl Frame {
    /// The frame as an image for the page, from `images` once the slot has one. Called on
    /// the UI thread.
    pub fn image(&self, images: &mut Images) -> Option<slint::Image> {
        if images.ring != self.ring.id {
            *images = Images {
                ring: self.ring.id,
                slots: Default::default(),
            };
        }
        let slot = images.slots.get_mut(self.slot)?;
        if slot.is_none() {
            match slint::Image::try_from(self.ring.textures[self.slot].clone()) {
                Ok(image) => *slot = Some(image),
                Err(e) => {
                    crate::local::log(&format!("video: Slint refused the frame: {e:?}"));
                    return None;
                }
            }
        }
        self.ring.shown.store(self.slot, Ordering::Release);
        slot.clone()
    }
}

/// The page's images of the ring's slots, made on the UI thread, since `slint::Image` stays
/// there.
#[derive(Default)]
pub struct Images {
    ring: u64,
    slots: [Option<slint::Image>; 3],
}

/// Slint's wgpu device: a new one means a new renderer, whose textures the old ring's are
/// not.
#[derive(Clone, Debug, PartialEq)]
pub struct Share {
    device: wgpu::Device,
}

/// Slint's device, from the rendering notifier.
pub fn current_share(api: &slint::GraphicsAPI<'_>) -> Option<Share> {
    match api {
        slint::GraphicsAPI::WGPU30 { device, .. } => Some(Share {
            device: device.clone(),
        }),
        _ => None,
    }
}

/// The three surfaces' wgpu textures, as the UI thread sees them.
struct Ring {
    id: u64,
    textures: [wgpu::Texture; 3],
    /// The slot the page shows, which mpv must not draw over: a UI thread running behind
    /// would show a frame half drawn.
    shown: AtomicUsize,
}

static RINGS: AtomicU64 = AtomicU64::new(1);

/// The longest side the ring is drawn at in the iOS Simulator, whose OpenGL ES is a software
/// rasterizer that cannot fill a phone's screen at a film's rate. Slint scales the frame up.
/// A phone draws at the picture's own size.
const SIMULATOR_SIDE: u32 = 720;

/// The size the ring is drawn at for a picture of `size`.
fn fit(size: (u32, u32)) -> (u32, u32) {
    let long = size.0.max(size.1);
    if !crate::platform::emulator() || long <= SIMULATOR_SIDE {
        return size;
    }
    let scale = |n: u32| ((n as u64 * SIMULATOR_SIDE as u64 / long as u64) as u32).max(1);
    (scale(size.0), scale(size.1))
}

unsafe extern "C" fn proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    // OpenGLES is linked into the app, so every GL entry point is a symbol in it.
    unsafe { dlsym(RTLD_DEFAULT, name) }
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
    /// Starts drawing `mpv`'s picture at `size` for Slint's device in `share`. None when the
    /// thread would not start; whether EAGL and mpv took it comes later, through
    /// `on_started`.
    pub fn start(
        mpv: &Arc<Mpv>,
        share: Share,
        size: (u32, u32),
        on_frame: OnFrame,
        on_started: OnStarted,
    ) -> Option<Video> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                dirty: true,
                size: fit(size),
                stop: false,
            }),
            wake: Condvar::new(),
        });
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let thread = {
            let shared = shared.clone();
            let mpv = mpv.clone();
            let device = share.device.clone();
            std::thread::Builder::new()
                .name("mpv-render".into())
                .spawn(move || render(mpv, device, shared, on_frame, ready_tx))
                .ok()?
        };
        // The UI thread never waits on the renderer's start.
        std::thread::spawn(move || {
            let r = ready_rx
                .recv()
                .unwrap_or_else(|_| Err("the renderer stopped".into()));
            if let Err(e) = &r {
                crate::local::log(&format!("video: {e}"));
            }
            on_started(r);
        });
        Some(Video {
            shared,
            share,
            thread: Some(thread),
        })
    }

    pub fn resize(&self, size: (u32, u32)) {
        let size = fit(size);
        let mut s = self.shared.state.lock().unwrap();
        if s.size != size {
            s.size = size;
            s.dirty = true;
            self.shared.wake.notify_one();
        }
    }

    pub fn share(&self) -> Share {
        self.share.clone()
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

/// One ring at one size: the GL side's textures and framebuffers over the surfaces, and the
/// wgpu textures over the same surfaces.
struct Target {
    textures: [c_uint; 3],
    fbos: [c_uint; 3],
    size: (u32, u32),
    ring: Arc<Ring>,
}

impl Target {
    /// Builds the ring on the current EAGL context, `eagl`. None, with the reason logged,
    /// when a surface, the GL binding or the Metal texture is refused.
    unsafe fn new(eagl: &AnyObject, device: &wgpu::Device, size: (u32, u32)) -> Option<Target> {
        let mtl: Retained<ProtocolObject<dyn MTLDevice>> = {
            // SAFETY: the hal device is only read for its MTLDevice, which is thread safe.
            let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }?;
            hal.raw_device().clone()
        };
        let mut textures = [0; 3];
        let mut fbos = [0; 3];
        unsafe {
            glGenTextures(3, textures.as_mut_ptr());
            glGenFramebuffers(3, fbos.as_mut_ptr());
        }
        let free = |textures: &[c_uint; 3], fbos: &[c_uint; 3]| unsafe {
            glDeleteFramebuffers(3, fbos.as_ptr());
            glDeleteTextures(3, textures.as_ptr());
        };
        let mut wrapped = Vec::with_capacity(3);
        for i in 0..3 {
            let Some(texture) = (unsafe { slot(eagl, &mtl, device, size, textures[i], fbos[i]) })
            else {
                free(&textures, &fbos);
                return None;
            };
            wrapped.push(texture);
        }
        unsafe {
            glBindTexture(GL_TEXTURE_2D, 0);
            glBindFramebuffer(GL_FRAMEBUFFER, 0);
        }
        let textures_wgpu: [wgpu::Texture; 3] = wrapped.try_into().ok()?;
        Some(Target {
            textures,
            fbos,
            size,
            ring: Arc::new(Ring {
                id: RINGS.fetch_add(1, Ordering::Relaxed),
                textures: textures_wgpu,
                shown: AtomicUsize::new(usize::MAX),
            }),
        })
    }

    unsafe fn free(&self) {
        unsafe {
            glDeleteFramebuffers(3, self.fbos.as_ptr());
            glDeleteTextures(3, self.textures.as_ptr());
        }
    }
}

/// One slot: an IOSurface, bound to GL texture `texture` with framebuffer `fbo` on it, and
/// wrapped as a Metal, then wgpu, texture on Slint's device.
unsafe fn slot(
    eagl: &AnyObject,
    mtl: &ProtocolObject<dyn MTLDevice>,
    device: &wgpu::Device,
    size: (u32, u32),
    texture: c_uint,
    fbo: c_uint,
) -> Option<wgpu::Texture> {
    let (w, h) = size;
    let surface = surface(w, h)?;
    unsafe {
        glBindTexture(GL_TEXTURE_2D, texture);
        let bound: bool = msg_send![
            eagl,
            texImageIOSurface: &*surface,
            target: GL_TEXTURE_2D as usize,
            internalFormat: GL_RGBA as usize,
            width: w,
            height: h,
            format: GL_RGBA as usize,
            type: GL_UNSIGNED_BYTE as usize,
            plane: 0u32
        ];
        if !bound {
            crate::local::log("video: GL would not take the IOSurface");
            return None;
        }
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        glBindFramebuffer(GL_FRAMEBUFFER, fbo);
        glFramebufferTexture2D(
            GL_FRAMEBUFFER,
            GL_COLOR_ATTACHMENT0,
            GL_TEXTURE_2D,
            texture,
            0,
        );
        let status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
        if status != GL_FRAMEBUFFER_COMPLETE {
            crate::local::log(&format!(
                "video: the frame buffer is incomplete (0x{status:x})"
            ));
            return None;
        }
    }
    // SAFETY: a plain 2D descriptor, then a texture over the surface it describes.
    let raw = unsafe {
        let desc = MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
            MTLPixelFormat::RGBA8Unorm,
            w as usize,
            h as usize,
            false,
        );
        desc.setUsage(MTLTextureUsage::ShaderRead | MTLTextureUsage::RenderTarget);
        mtl.newTextureWithDescriptor_iosurface_plane(&desc, &surface, 0)
    };
    let Some(raw) = raw else {
        crate::local::log("video: Metal would not take the IOSurface");
        return None;
    };
    let extent = wgpu::Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };
    // SAFETY: the texture was made on Slint's own MTLDevice, as the descriptor below says,
    // and holds its IOSurface for as long as it lives.
    let texture = unsafe {
        let hal = wgpu::hal::metal::Device::texture_from_raw(
            raw,
            wgpu::TextureFormat::Rgba8Unorm,
            MTLTextureType::Type2D,
            1,
            1,
            wgpu::hal::CopyExtent {
                width: w,
                height: h,
                depth: 1,
            },
            None,
        );
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(
            hal,
            &wgpu::TextureDescriptor {
                label: Some("mpv frame"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            wgpu::wgt::TextureUses::RESOURCE,
        )
    };
    Some(texture)
}

/// A `w` by `h` IOSurface of four bytes a pixel, in RGBA order.
fn surface(w: u32, h: u32) -> Option<CFRetained<IOSurfaceRef>> {
    let numbers = [
        CFNumber::new_i32(w as i32),
        CFNumber::new_i32(h as i32),
        CFNumber::new_i32(4),
        CFNumber::new_i32(PIXEL_RGBA),
    ];
    // SAFETY: IOSurface's own key constants, read once the framework is linked.
    let keys: [&CFString; 4] = unsafe {
        [
            kIOSurfaceWidth,
            kIOSurfaceHeight,
            kIOSurfaceBytesPerElement,
            kIOSurfacePixelFormat,
        ]
    };
    let values: Vec<&CFNumber> = numbers.iter().map(|n| &**n).collect();
    let properties = CFDictionary::from_slices(&keys, &values);
    // SAFETY: the dictionary holds the keys IOSurfaceCreate documents, with CFNumbers.
    let surface = unsafe { IOSurfaceRef::new(properties.as_opaque()) };
    if surface.is_none() {
        crate::local::log(&format!("video: no {w}x{h} IOSurface"));
    }
    surface
}

/// An OpenGL ES 3 context, or 2 where 3 is refused, made current on this thread.
fn eagl_context() -> Option<Retained<AnyObject>> {
    let class = AnyClass::get(c"EAGLContext")?;
    for api in [3usize, 2] {
        // SAFETY: `initWithAPI:` takes an EAGLRenderingAPI and returns a context or nil.
        let context: Option<Retained<AnyObject>> =
            unsafe { msg_send![msg_send![class, alloc], initWithAPI: api] };
        if let Some(context) = context {
            // SAFETY: the current context is per thread; this thread keeps it until it ends.
            let current: bool = unsafe { msg_send![class, setCurrentContext: &*context] };
            if current {
                return Some(context);
            }
        }
    }
    None
}

fn clear_context() {
    if let Some(class) = AnyClass::get(c"EAGLContext") {
        // SAFETY: nil clears this thread's current context.
        let _: bool = unsafe { msg_send![class, setCurrentContext: std::ptr::null::<AnyObject>()] };
    }
}

fn render(
    mpv: Arc<Mpv>,
    device: wgpu::Device,
    shared: Arc<Shared>,
    on_frame: OnFrame,
    ready: std::sync::mpsc::Sender<Result<(), String>>,
) {
    autoreleasepool(|_| {
        let Some(eagl) = eagl_context() else {
            let _ = ready.send(Err("no OpenGL ES context".into()));
            return;
        };
        let mut init = mpv_opengl_init_params {
            get_proc_address: Some(proc_address),
            get_proc_address_ctx: std::ptr::null_mut(),
        };
        let mut params = [
            mpv_render_param {
                kind: RENDER_PARAM_API_TYPE,
                data: c"opengl".as_ptr() as *mut c_void,
            },
            mpv_render_param {
                kind: RENDER_PARAM_OPENGL_INIT_PARAMS,
                data: (&mut init as *mut mpv_opengl_init_params).cast(),
            },
            mpv_render_param {
                kind: RENDER_PARAM_INVALID,
                data: std::ptr::null_mut(),
            },
        ];
        let mut ctx: *mut mpv_render_context = std::ptr::null_mut();
        let code = unsafe { mpv_render_context_create(&mut ctx, mpv.handle, params.as_mut_ptr()) };
        if code < 0 {
            let word = unsafe { CStr::from_ptr(mpv_error_string(code)) }
                .to_string_lossy()
                .into_owned();
            let _ = ready.send(Err(format!("mpv's render context: {word}")));
            clear_context();
            return;
        }
        unsafe {
            mpv_render_context_set_update_callback(
                ctx,
                Some(on_update),
                Arc::as_ptr(&shared) as *mut c_void,
            )
        };
        let _ = ready.send(Ok(()));
        frames(ctx, &eagl, &device, &shared, &on_frame);
        unsafe { mpv_render_context_free(ctx) };
        clear_context();
        drop(eagl);
    });
}

/// The frame loop, until the renderer stops.
fn frames(
    ctx: *mut mpv_render_context,
    eagl: &AnyObject,
    device: &wgpu::Device,
    shared: &Shared,
    on_frame: &OnFrame,
) {
    let mut target: Option<Target> = None;
    // The rings of the size before, kept until two frames of the new size have gone out,
    // since Slint may still be showing one of them.
    let mut retired: Vec<(Target, u8)> = Vec::new();
    let mut next = 0usize;
    // A frame mpv has for the screen and that has not been drawn yet.
    let mut owed = false;
    // The next slot was still on the page: look again shortly, without waiting for mpv. A
    // page that has not moved on in 100 ms (a frame it dropped) is drawn over all the same.
    let mut retry = false;
    let mut retries = 0u32;
    loop {
        if retry {
            std::thread::sleep(Duration::from_millis(2));
        }
        let size = {
            let mut s = shared.state.lock().unwrap();
            // A held renderer keeps its frame waiting, and looks again shortly: the hold
            // ends on the UI thread, which does not wake this one.
            while (!(s.dirty || retry) || super::held()) && !s.stop {
                s = if super::held() {
                    shared
                        .wake
                        .wait_timeout(s, Duration::from_millis(100))
                        .unwrap()
                        .0
                } else {
                    shared.wake.wait(s).unwrap()
                };
            }
            if s.stop {
                break;
            }
            s.dirty = false;
            s.size
        };
        retry = false;
        autoreleasepool(|_| {
            let flags = unsafe { mpv_render_context_update(ctx) };
            owed |= flags & RENDER_UPDATE_FRAME != 0;
            if size.0 == 0 || size.1 == 0 {
                return;
            }
            let resized = target.as_ref().is_none_or(|t| t.size != size);
            if resized {
                if let Some(t) = target.take() {
                    retired.push((t, 0));
                }
                target = unsafe { Target::new(eagl, device, size) };
                if target.is_none() {
                    crate::local::log("video: the frame surfaces would not build");
                    return;
                }
            }
            // A new size draws the frame on screen again, even when paused.
            if !owed && !resized {
                return;
            }
            let t = target.as_ref().unwrap();
            if t.ring.shown.load(Ordering::Acquire) == next && retries < 50 {
                retry = true;
                retries += 1;
                return;
            }
            retries = 0;
            owed = false;
            let mut fbo = mpv_opengl_fbo {
                fbo: t.fbos[next] as c_int,
                w: size.0 as c_int,
                h: size.1 as c_int,
                internal_format: GL_RGBA8,
            };
            // Unflipped: mpv then writes the picture's top into the surface's first row,
            // which is the top row Metal reads.
            let mut flip: c_int = 0;
            let mut params = [
                mpv_render_param {
                    kind: RENDER_PARAM_OPENGL_FBO,
                    data: (&mut fbo as *mut mpv_opengl_fbo).cast(),
                },
                mpv_render_param {
                    kind: RENDER_PARAM_FLIP_Y,
                    data: (&mut flip as *mut c_int).cast(),
                },
                mpv_render_param {
                    kind: RENDER_PARAM_INVALID,
                    data: std::ptr::null_mut(),
                },
            ];
            unsafe {
                mpv_render_context_render(ctx, params.as_mut_ptr());
                // The frame is whole in the surface before Metal samples it.
                glFinish();
                mpv_render_context_report_swap(ctx);
            }
            on_frame(Frame {
                ring: t.ring.clone(),
                slot: next,
            });
            next = (next + 1) % 3;
            for r in retired.iter_mut() {
                r.1 += 1;
            }
            retired.retain(|(t, frames)| {
                if *frames >= 2 {
                    unsafe { t.free() };
                    false
                } else {
                    true
                }
            });
        });
    }
    if let Some(t) = target.take() {
        unsafe { t.free() };
    }
    for (t, _) in retired.drain(..) {
        unsafe { t.free() };
    }
}
