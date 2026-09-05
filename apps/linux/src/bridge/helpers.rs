//! The C++ helpers, declared once here and reached as `bridge::helpers::ffi::*`. A cxx-qt
//! bridge with only `extern "C++"` blocks is what cxx-qt-build expects in `.files()`.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
    }

    unsafe extern "C++" {
        include!("helpers.h");
        /// QQuickWindow::setGraphicsApi(OpenGL); must run before the first window exists.
        fn use_opengl_scene_graph();
        /// QGuiApplication::setDesktopFileName, so the Wayland app id is the desktop entry's.
        fn set_desktop_file_name(name: &QString);
        /// The two environment variables the spikes settled, before QGuiApplication reads them.
        fn set_render_loop_env();
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
    }
}
