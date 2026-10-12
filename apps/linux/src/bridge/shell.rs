//! Shell: what QML needs to know about this run before anything else exists. The version,
//! the --shoot arguments and, from Task 7 on, the page a shoot opens, and from Task 9 on
//! the JSON props that page opens with. A --drive run also carries its script's text and
//! the directory its shots land in, and the input invokables the QML driver sends through.
//! The url of shell.conf, in the config directory, rides along for the few things QML keeps
//! for itself (the library's tracker nudge, once dismissed), so a --root run keeps them
//! inside its sandbox too.
//!
//! It is also where the session bus starts, because this object exists exactly once and
//! exists as soon as the engine loads Main.qml. Its thread handle is what the
//! org.freedesktop.Application interface and MPRIS's Raise and Quit queue their work on.

use core::pin::Pin;

use cxx_qt::Threading;
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
        #[qproperty(QString, version)]
        #[qproperty(QString, shoot)]
        #[qproperty(QString, page)]
        #[qproperty(QString, props)]
        #[qproperty(i32, shoot_width)]
        #[qproperty(i32, shoot_height)]
        #[qproperty(QString, drive)]
        #[qproperty(QString, drive_out)]
        #[qproperty(QString, settings_url)]
        type Shell = super::ShellRust;

        /// A second launch, or MPRIS Raise, asked for the window; the token is the launcher's.
        #[qsignal]
        fn activate_requested(self: Pin<&mut Self>, token: QString);
        /// MPRIS Quit. The window closes itself; nothing here ends the process.
        #[qsignal]
        fn quit_requested(self: Pin<&mut Self>);
        /// Raises `window` (the QML Window) with the xdg-activation token.
        #[qinvokable]
        unsafe fn raise_window(self: &Self, window: *mut QObject, token: &QString);
        /// --drive: a mouse event at window coordinates (see cpp/helpers.h for `kind`).
        #[qinvokable]
        unsafe fn drive_mouse(self: &Self, window: *mut QObject, kind: i32, x: f64, y: f64, button: i32, modifiers: i32);
        /// --drive: a wheel event; 120 is one notch.
        #[qinvokable]
        unsafe fn drive_wheel(self: &Self, window: *mut QObject, x: f64, y: f64, dx: i32, dy: i32, modifiers: i32);
        /// --drive: a key press or release with the text it types.
        #[qinvokable]
        unsafe fn drive_key(self: &Self, window: *mut QObject, press: bool, key: i32, modifiers: i32, text: &QString);
        /// --drive: one line to stderr, bare, with none of the QML logger's prefixes.
        #[qinvokable]
        fn drive_log(self: &Self, line: &QString);
    }

    impl cxx_qt::Threading for Shell {}
    impl cxx_qt::Initialize for Shell {}
}

pub struct ShellRust {
    version: QString,
    shoot: QString,
    page: QString,
    props: QString,
    shoot_width: i32,
    shoot_height: i32,
    drive: QString,
    drive_out: QString,
    settings_url: QString,
}

impl Default for ShellRust {
    fn default() -> Self {
        let a = crate::runtime::args();
        let shooting = a.shoot.is_some() || a.drive.is_some();
        // main has already read and checked the script and left on a bad one, so a failure
        // here is the file changing underfoot; the driver reports an empty script as one.
        let drive = a
            .drive
            .as_deref()
            .map(|p| crate::args::load_drive(std::path::Path::new(p)).unwrap_or_default())
            .unwrap_or_default();
        let drive_out = if a.drive.is_some() {
            crate::args::drive_out(a.out.as_deref(), std::env::var("ANIBEAM_CAPTURES").ok())
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };
        ShellRust {
            version: QString::from(anibeam_core::VERSION),
            shoot: QString::from(a.shoot.as_deref().unwrap_or("")),
            page: QString::from(a.page.as_deref().unwrap_or("library")),
            props: QString::from(a.props.as_deref().unwrap_or("")),
            shoot_width: if shooting { a.width as i32 } else { 0 },
            shoot_height: if shooting { a.height as i32 } else { 0 },
            drive: QString::from(drive.as_str()),
            drive_out: QString::from(drive_out.as_str()),
            settings_url: settings_url(&crate::runtime::paths().config_dir()),
        }
    }
}

/// shell.conf as a file url QtCore's Settings can take as its `location`. Made absolute
/// first, since a relative --root is relative to where the shell was started, and encoded
/// by Qt rather than by string concatenation, so a `#`, `?` or `%` in the path is part of the
/// path and not the start of a fragment, a query or an escape.
fn settings_url(config_dir: &std::path::Path) -> QString {
    let file = config_dir.join("shell.conf");
    let file = std::path::absolute(&file).unwrap_or(file);
    let url = cxx_qt_lib::QUrl::from_local_file(&QString::from(file.to_string_lossy().as_ref()));
    QString::from(String::from_utf8_lossy(url.to_encoded().as_slice()).as_ref())
}

impl cxx_qt::Initialize for qobject::Shell {
    fn initialize(self: Pin<&mut Self>) {
        // The bus name and MPRIS start once the Shell exists, which is once the engine
        // loads. A run whose --root lets it start beside a real one, a --shoot, finds both
        // names taken and says so; it draws its page either way.
        let shell = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            if let Some(h) = crate::dbus::mpris::start(shell).await {
                crate::dbus::mpris::install(h)
            }
        });
    }
}

impl qobject::Shell {
    /// # Safety
    /// `window` is a live QObject: QML hands its own Window here and the call returns
    /// before that Window can be destroyed.
    pub unsafe fn raise_window(&self, window: *mut cxx_qt::QObject, token: &QString) {
        // A panic crossing the FFI aborts the process, so it stops here instead.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            crate::bridge::helpers::ffi::raise_window(window, token);
        }));
        if result.is_err() {
            eprintln!("anibeam: shell: raiseWindow panicked");
        }
    }
}

impl qobject::Shell {
    /// # Safety
    /// `window` is the live QML Window, as for `raise_window`.
    pub unsafe fn drive_mouse(&self, window: *mut cxx_qt::QObject, kind: i32, x: f64, y: f64, button: i32, modifiers: i32) {
        unsafe { crate::bridge::helpers::ffi::drive_mouse(window, kind, x, y, button, modifiers) }
    }

    /// # Safety
    /// `window` is the live QML Window, as for `raise_window`.
    pub unsafe fn drive_wheel(&self, window: *mut cxx_qt::QObject, x: f64, y: f64, dx: i32, dy: i32, modifiers: i32) {
        unsafe { crate::bridge::helpers::ffi::drive_wheel(window, x, y, dx, dy, modifiers) }
    }

    /// # Safety
    /// `window` is the live QML Window, as for `raise_window`.
    pub unsafe fn drive_key(&self, window: *mut cxx_qt::QObject, press: bool, key: i32, modifiers: i32, text: &QString) {
        unsafe { crate::bridge::helpers::ffi::drive_key(window, press, key, modifiers, text) }
    }

    pub fn drive_log(&self, line: &QString) {
        eprintln!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::settings_url;

    /// The url is absolute whatever the root was given as, and the characters a url
    /// would read as delimiters are escaped into the path.
    #[test]
    fn the_settings_url_is_absolute_and_escaped() {
        let url = settings_url(std::path::Path::new("/tmp/a #b?c%d/anibeam")).to_string();
        assert_eq!(url, "file:///tmp/a%20%23b%3Fc%25d/anibeam/shell.conf");
        let relative = settings_url(std::path::Path::new("rel/anibeam")).to_string();
        let cwd = std::env::current_dir().unwrap();
        assert!(relative.starts_with("file:///"), "{relative}");
        assert!(relative.ends_with("/rel/anibeam/shell.conf"), "{relative}");
        assert!(
            relative.contains(cwd.file_name().unwrap().to_str().unwrap()),
            "{relative}"
        );
    }
}
