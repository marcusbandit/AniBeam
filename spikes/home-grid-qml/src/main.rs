mod bridge;
mod kitty;
mod library;
mod palettes;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    // Headless check of the data layer: print both JSON blocks and leave before Qt exists.
    if std::env::args().any(|a| a == "--dump") {
        println!("== library");
        println!("{}", serde_json::to_string_pretty(&library::load()).unwrap());
        println!("== palettes");
        println!("{}", serde_json::to_string_pretty(&palettes::load()).unwrap());
        return;
    }

    bridge::qobject::use_opengl_scene_graph();
    let mut app = QGuiApplication::new();
    bridge::qobject::set_desktop_file_name(&QString::from("anibeam-proto"));

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/dev/anibeam/proto/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}
