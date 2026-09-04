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
            .qml_file("qml/Icon.qml")
            .qml_file("qml/Seg.qml")
            .qml_file("qml/Knob.qml")
            .qml_file("qml/KnobBar.qml")
            .qml_file("qml/Switch.qml")
            .qml_file("qml/Button.qml")
            .qml_file("qml/Field.qml")
            .qml_file("qml/Dropdown.qml")
            .qml_file("qml/Swatches.qml")
            .qml_file("qml/SliderRow.qml")
            .qml_file("qml/SettingRow.qml")
            .qml_file("qml/Panel.qml")
            .qml_file("qml/LookPane.qml")
            .qml_file("qml/LookPreview.qml")
            .qml_file("qml/SubtitlePreview.qml")
            .qml_file("qml/SettingsPage.qml")
            .qml_file("qml/StatusStrip.qml")
            .qml_file("qml/ActivityDrawer.qml"),
    )
    .qt_module("Quick")
    .files(["src/bridge.rs"])
    .include_dir("cpp")
    .cpp_files(["cpp/helpers.cpp"])
    // Land at qrc:/qt/qml/dev/anibeam/proto/assets/icon.png and assets/icons/<name>.svg; the
    // icons are the Lucide set, only the ones the QML names, ISC licensed (assets/icons/LICENSE)
    .qrc_resources([
        "assets/icon.png",
        "assets/icons/house.svg",
        "assets/icons/rss.svg",
        "assets/icons/eye.svg",
        "assets/icons/database.svg",
        "assets/icons/settings.svg",
        "assets/icons/folder.svg",
        "assets/icons/palette.svg",
        "assets/icons/play.svg",
        "assets/icons/hard-drive.svg",
        "assets/icons/folder-open.svg",
        "assets/icons/user-check.svg",
        "assets/icons/shapes.svg",
        "assets/icons/captions.svg",
        "assets/icons/archive.svg",
        "assets/icons/folder-plus.svg",
        "assets/icons/refresh-cw.svg",
        "assets/icons/trash-2.svg",
        "assets/icons/arrow-up-right.svg",
        "assets/icons/log-in.svg",
        "assets/icons/log-out.svg",
        "assets/icons/upload.svg",
        "assets/icons/download.svg",
        "assets/icons/copy.svg",
        "assets/icons/x.svg",
        "assets/icons/moon.svg",
        "assets/icons/sun.svg",
        "assets/icons/monitor.svg",
        "assets/icons/circle-alert.svg",
        "assets/icons/activity.svg",
    ])
    .build();
}
