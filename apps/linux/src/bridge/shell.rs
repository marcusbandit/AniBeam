//! Shell: what QML needs to know about this run before anything else exists. The version,
//! the --shoot arguments and, from Task 7 on, the page a shoot opens, and from Task 9 on
//! the JSON props that page opens with.
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
}

impl Default for ShellRust {
    fn default() -> Self {
        let a = crate::runtime::args();
        let shooting = a.shoot.is_some();
        ShellRust {
            version: QString::from(anibeam_core::VERSION),
            shoot: QString::from(a.shoot.as_deref().unwrap_or("")),
            page: QString::from(a.page.as_deref().unwrap_or("library")),
            props: QString::from(a.props.as_deref().unwrap_or("")),
            shoot_width: if shooting { a.width as i32 } else { 0 },
            shoot_height: if shooting { a.height as i32 } else { 0 },
        }
    }
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
