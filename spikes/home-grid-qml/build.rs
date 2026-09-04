// One build script, no CMake. cxx-qt-build finds Qt through qmake6, runs moc, rcc,
// qmlcachegen and qmltyperegistrar, compiles the C++ beside the generated bridge
// and links the pacman Qt. No mpv in this prototype: the question is the look.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("dev.anibeam.proto")
            .version(1, 0)
            .qml_file("qml/Main.qml")
            .qml_file("qml/Theme.qml")
            .qml_file("qml/Corner.qml")
            .qml_file("qml/Chip.qml")
            .qml_file("qml/Card.qml")
            .qml_file("qml/Rail.qml")
            .qml_file("qml/Seg.qml")
            .qml_file("qml/Knob.qml")
            .qml_file("qml/KnobBar.qml"),
    )
    .qt_module("Quick")
    .files(["src/bridge.rs"])
    .include_dir("cpp")
    .cpp_files(["cpp/helpers.cpp"])
    // Lands at qrc:/qt/qml/dev/anibeam/proto/assets/icon.png
    .qrc_resources(["assets/icon.png"])
    .build();
}
