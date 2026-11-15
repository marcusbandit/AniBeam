//! The picture: mpv's render API draws each frame on a GL context of its own, on a thread of
//! its own, and Slint shows the frame as an image under the controls.
//!
//! The render thread never touches Slint's context, so mpv cannot leave GL state behind for
//! the renderer to trip over, and decoding a frame never holds up a touch. Three frames turn
//! in a ring: mpv draws the next while Slint shows the last.
//!
//! Android (`video/gl.rs`) draws into textures of an EGL context that shares them with
//! Slint's. The iPhone (`video/metal.rs`) draws with OpenGL ES into IOSurfaces, which Slint's
//! Metal renderer reads as wgpu textures without a copy.
//!
//! Slint's context goes when the app leaves the screen and a new one comes back with it, so
//! the renderer is tied to the context it was started against; the player starts a fresh one
//! when the context changes.

// Without libmpv (and the GL that build.rs links beside it) the types stand in for a player
// that never opens.
#![cfg_attr(not(libmpv), allow(dead_code))]

use std::sync::atomic::{AtomicBool, Ordering};

pub type OnFrame = Box<dyn Fn(Frame) + Send + 'static>;
/// Called once from the renderer's thread: Ok when mpv has its render context, so a file
/// loaded from then on gets a picture; Err with the reason when GL or mpv refused.
pub type OnStarted = Box<dyn FnOnce(Result<(), String>) + Send + 'static>;

#[cfg(all(libmpv, target_os = "android"))]
mod gl;
#[cfg(all(libmpv, target_os = "android"))]
pub use gl::{Frame, Images, Share, Video, current_share};

#[cfg(all(libmpv, target_os = "ios"))]
mod metal;
#[cfg(all(libmpv, target_os = "ios"))]
pub use metal::{Frame, Images, Share, Video, current_share};

#[cfg(not(libmpv))]
pub use host::{Frame, Images, Share, Video, current_share};

/// The app is off the screen, or about to be: no GL work may start. iOS ends an app that
/// draws with OpenGL ES in the background, so the iPhone sets this when the app resigns
/// active, and the renderer holds its frames until it is cleared.
static HELD: AtomicBool = AtomicBool::new(false);

/// Holds or frees the renderers' GL work.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub fn hold(on: bool) {
    HELD.store(on, Ordering::SeqCst);
}

/// Whether GL work is held.
pub fn held() -> bool {
    HELD.load(Ordering::SeqCst)
}

#[cfg(not(libmpv))]
mod host {
    use std::sync::Arc;

    use super::{OnFrame, OnStarted};
    use crate::mpv::Mpv;

    /// No frame ever comes.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Frame;

    impl Frame {
        pub fn image(&self, _images: &mut Images) -> Option<slint::Image> {
            None
        }
    }

    #[derive(Default)]
    pub struct Images {}

    #[derive(Clone, Debug, PartialEq)]
    pub struct Share;

    pub struct Video;

    impl Video {
        pub fn start(
            _mpv: &Arc<Mpv>,
            _share: Share,
            _size: (u32, u32),
            _on_frame: OnFrame,
            _on_started: OnStarted,
        ) -> Option<Video> {
            None
        }
        pub fn resize(&self, _size: (u32, u32)) {}
        pub fn share(&self) -> Share {
            Share
        }
    }

    pub fn current_share(_api: &slint::GraphicsAPI<'_>) -> Option<Share> {
        None
    }
}
