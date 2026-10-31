//! Dev: what the Developer tab and the page load overlay read (#287). QML owns the switches
//! (kept in shell.conf) and hands them over through `configure`; everything here answers in
//! JSON text, which QML parses with JSON.parse, so a large answer never becomes a deep
//! QJsonObject on the way.
//!
//! `forced` is true under ANIBEAM_DEV=1 only: developer options are on and the switch that
//! turns them off is not shown. A debug build starts with them off like a release build, so
//! its captures look like what ships; the switch under About turns them on.

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, forced)]
        type Dev = super::DevRust;

        /// Developer options on or off, whether images load, whether the Door keeps whole
        /// library reads. Off undoes all three.
        #[qinvokable]
        fn configure(self: &Dev, enabled: bool, images: bool, read_cache: bool);
        /// Milliseconds on the clock every record shares.
        #[qinvokable]
        fn now(self: &Dev) -> f64;
        /// A start-up step, printed under ANIBEAM_TIMING=1 with its time since launch.
        #[qinvokable]
        fn mark(self: &Dev, step: &QString);
        /// The Door calls that started at or after `since`.
        #[qinvokable]
        fn calls(self: &Dev, since: f64) -> QString;
        /// The newest `limit` events, every level, with the size each one crossed at.
        #[qinvokable]
        fn events(self: &Dev, limit: i32) -> QString;
        /// The decodes since `since` and the totals since the start.
        #[qinvokable]
        fn images(self: &Dev, since: f64) -> QString;
        /// The x-ray of `item`'s tree inside `window`.
        #[qinvokable]
        unsafe fn scene(self: &Dev, item: *mut QObject, window: *mut QObject) -> QString;
        /// The process's memory in kilobytes, as the kernel counts it.
        #[qinvokable]
        fn memory(self: &Dev) -> QString;
        /// Gives back what can be rebuilt: scene graph caches, JS garbage, free heap.
        #[qinvokable]
        unsafe fn release(self: &Dev, window: *mut QObject);
    }
}

pub struct DevRust {
    forced: bool,
}

impl Default for DevRust {
    fn default() -> Self {
        DevRust {
            forced: std::env::var("ANIBEAM_DEV").as_deref() == Ok("1"),
        }
    }
}

fn text(v: serde_json::Value) -> QString {
    QString::from(&*v.to_string())
}

impl qobject::Dev {
    pub fn configure(&self, enabled: bool, images: bool, read_cache: bool) {
        crate::devstats::set_on(enabled);
        crate::bridge::helpers::ffi::dev_set_image_flags(enabled, !enabled || images);
        crate::read_cache::set_paused(enabled && !read_cache);
    }

    pub fn now(&self) -> f64 {
        crate::devstats::now()
    }

    pub fn mark(&self, step: &QString) {
        crate::devstats::startup_mark(&step.to_string());
    }

    pub fn calls(&self, since: f64) -> QString {
        text(crate::devstats::calls_since(since))
    }

    pub fn events(&self, limit: i32) -> QString {
        text(crate::devstats::events(limit.max(0) as usize))
    }

    pub fn images(&self, since: f64) -> QString {
        crate::bridge::helpers::ffi::dev_image_stats(since)
    }

    /// # Safety
    /// Both pointers come from QML and name live objects for the length of the call.
    pub unsafe fn scene(
        &self,
        item: *mut cxx_qt::QObject,
        window: *mut cxx_qt::QObject,
    ) -> QString {
        unsafe { crate::bridge::helpers::ffi::dev_scene(item, window) }
    }

    pub fn memory(&self) -> QString {
        text(crate::devstats::memory())
    }

    /// # Safety
    /// The pointer comes from QML and names the live window.
    pub unsafe fn release(&self, window: *mut cxx_qt::QObject) {
        unsafe { crate::bridge::helpers::ffi::dev_release(window) }
    }
}
