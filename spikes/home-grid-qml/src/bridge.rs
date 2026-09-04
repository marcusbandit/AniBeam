//! The bridge: one Rust QObject the QML engine owns as a singleton. It hands the QML
//! side two JSON strings, the library records and the palettes, and can re-read them.

use core::pin::Pin;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "C++" {
        include!("helpers.h");
        /// QQuickWindow::setGraphicsApi(OpenGL); must run before QGuiApplication exists.
        fn use_opengl_scene_graph();
        /// QGuiApplication::setDesktopFileName so the compositor pairs the window with the .desktop entry.
        fn set_desktop_file_name(name: &QString);
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, library_json)]
        #[qproperty(QString, palettes_json)]
        type Proto = super::ProtoRust;

        /// Re-reads metadata, progress cache, view history, kitty config and the portal.
        #[qinvokable]
        fn reload(self: Pin<&mut Self>);
    }
}

pub struct ProtoRust {
    library_json: QString,
    palettes_json: QString,
}

impl Default for ProtoRust {
    fn default() -> Self {
        Self {
            library_json: QString::from(&crate::library::load().to_string()),
            palettes_json: QString::from(&crate::palettes::load().to_string()),
        }
    }
}

impl qobject::Proto {
    pub fn reload(mut self: Pin<&mut Self>) {
        let library = QString::from(&crate::library::load().to_string());
        let palettes = QString::from(&crate::palettes::load().to_string());
        self.as_mut().set_library_json(library);
        self.as_mut().set_palettes_json(palettes);
    }
}
