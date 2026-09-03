mod spike;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use std::sync::OnceLock;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// The tokio runtime lives for the whole process; the QML engine constructs the
/// singleton itself, so the bridge reaches the runtime through this rather than a field.
pub fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("anibeam-core")
            .enable_time()
            .build()
            .expect("tokio runtime")
    })
}

fn main() {
    spike::qobject::use_opengl_scene_graph();
    let mut app = QGuiApplication::new();
    spike::qobject::set_desktop_file_name(&QString::from("anibeam-spike"));

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/dev/anibeam/spike/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}
