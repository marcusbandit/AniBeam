//! The C++ helpers, declared once here and reached as `bridge::helpers::ffi::*`. A cxx-qt
//! bridge with only `extern "C++"` blocks is what cxx-qt-build expects in `.files()`.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qobject.h");
        type QObject = cxx_qt::QObject;
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
    }

    unsafe extern "C++" {
        include!("helpers.h");
        /// QQuickWindow::setGraphicsApi(OpenGL); must run before the first window exists.
        fn use_opengl_scene_graph();
        /// QGuiApplication::setDesktopFileName, so the Wayland app id is the desktop entry's.
        fn set_desktop_file_name(name: &QString);
        /// The two environment variables the spikes settled, before QGuiApplication reads them.
        fn set_render_loop_env();
        /// Shows, raises and activates a window, with the launcher's xdg-activation token
        /// in the environment first. The pointer is a QML Window, so it outlives the call.
        unsafe fn raise_window(window: *mut QObject, token: &QString);
        /// The resolved tokens into QGuiApplication's palette, so a stock control matches.
        fn set_app_palette(
            window: &QColor,
            text: &QColor,
            base: &QColor,
            highlight: &QColor,
            highlighted_text: &QColor,
            button: &QColor,
            button_text: &QColor,
        );
        /// The --drive input: through QtGui's window-system entry points, as QtTest does.
        /// kind 0 move, 1 press, 2 release, 3 release ending a click. `window` is a QML Window.
        unsafe fn drive_mouse(
            window: *mut QObject,
            kind: i32,
            x: f64,
            y: f64,
            button: i32,
            modifiers: i32,
        );
        unsafe fn drive_wheel(
            window: *mut QObject,
            x: f64,
            y: f64,
            dx: i32,
            dy: i32,
            modifiers: i32,
        );
        unsafe fn drive_key(
            window: *mut QObject,
            press: bool,
            key: i32,
            modifiers: i32,
            text: &QString,
        );
        /// A poster's average colour, saturation-weighted, decoded at 8x12 and cached per
        /// path; invalid for a file that cannot be read.
        fn poster_tone(path: &QString) -> QColor;
        /// The pointer hidden (or shown) over every window: an application override cursor.
        fn set_cursor_hidden(hidden: bool);
        /// Dev only: loads the shell's QML files from `dir` instead of the compiled resources.
        fn install_qml_dir(engine: Pin<&mut QQmlApplicationEngine>, dir: &QString);
        /// A URL opened with the desktop's handler through xdg-open, detached.
        fn open_url(url: &QString) -> bool;
    }

    unsafe extern "C++" {
        include!("devtools.h");
        /// Developer options (#287): the image interceptor and the `dev` image provider.
        fn install_dev_tools(engine: Pin<&mut QQmlApplicationEngine>);
        /// Measure decodes through the provider; with `images` false nothing loads.
        fn dev_set_image_flags(measure: bool, images: bool);
        /// Milliseconds since the process started, the clock every record shares.
        fn dev_now_ms() -> f64;
        /// The decodes since `since` and the running totals, as JSON text.
        fn dev_image_stats(since: f64) -> QString;
        /// The x-ray of an item tree, as JSON text. Both pointers are QML objects.
        unsafe fn dev_scene(item: *mut QObject, window: *mut QObject) -> QString;
        /// Drops what the scene graph can rebuild, collects JS garbage, trims the heap.
        unsafe fn dev_release(window: *mut QObject);
    }
}
