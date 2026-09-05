//! Shell: what QML needs to know about this run before anything else exists. The version,
//! the --shoot arguments and, from Task 7 on, the page a shoot opens.

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
        #[qproperty(i32, shoot_width)]
        #[qproperty(i32, shoot_height)]
        type Shell = super::ShellRust;
    }
}

pub struct ShellRust {
    version: QString,
    shoot: QString,
    page: QString,
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
            shoot_width: if shooting { a.width as i32 } else { 0 },
            shoot_height: if shooting { a.height as i32 } else { 0 },
        }
    }
}
