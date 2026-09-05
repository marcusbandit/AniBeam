// One build script, no CMake. cxx-qt-build finds Qt through qmake6, runs moc, rcc,
// qmlcachegen and qmltyperegistrar, compiles the C++ beside the generated bridge and
// links the pacman Qt. MpvQt has no .pc file, so its two libraries are named here.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("com.marcusrosado.AniBeam")
            .version(1, 0)
            .qml_file("qml/Main.qml"),
    )
    .qt_module("Quick")
    .files(["src/bridge/helpers.rs", "src/bridge/shell.rs"])
    .include_dir("cpp")
    // mpvqt_export.h includes mpvqt_version.h bare; CMake's target used to supply this.
    .include_dir("/usr/include/MpvQt")
    .cpp_files(["cpp/helpers.cpp"])
    // qrc:/qt/qml/com/marcusrosado/AniBeam/assets/icon.png
    .qrc_resources(["assets/icon.png"])
    .build();

    println!("cargo:rustc-link-lib=MpvQt");
    println!("cargo:rustc-link-lib=mpv");
}
