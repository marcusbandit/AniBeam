// One build script, no CMake. cxx-qt-build finds Qt through qmake6, runs moc, rcc,
// qmlcachegen and qmltyperegistrar, compiles the C++ beside the generated bridge
// and links the pacman Qt. MpvQt has no .pc file, so its two libraries are named here.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("dev.anibeam.spike")
            .version(1, 0)
            .qml_file("qml/Main.qml"),
    )
    .qt_module("Quick")
    .files(["src/spike.rs"])
    .include_dir("cpp")
    // MpvQt has no .pc file and its export header includes mpvqt_version.h bare, so the
    // include dir CMake's MpvQt::MpvQt target would have supplied is named by hand.
    .include_dir("/usr/include/MpvQt")
    // Headers get moc (and QML_ELEMENT registration into the module above), .cpp files get compiled.
    .cpp_files(["cpp/spikevideo.h", "cpp/spikevideo.cpp", "cpp/helpers.cpp"])
    // Lands at qrc:/qt/qml/dev/anibeam/spike/assets/icon.png
    .qrc_resources(["assets/icon.png"])
    .build();

    println!("cargo:rustc-link-lib=MpvQt");
    println!("cargo:rustc-link-lib=mpv");
}
