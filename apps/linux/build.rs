// One build script, no CMake. cxx-qt-build finds Qt through qmake6, runs moc, rcc,
// qmlcachegen and qmltyperegistrar, compiles the C++ beside the generated bridge and
// links the pacman Qt. MpvQt has no .pc file, so its two libraries are named here.
//
// On Arch both live under /usr. Elsewhere (the macOS workflow builds MpvQt from source
// into its own prefix and takes mpv from Homebrew) MPVQT_PREFIX and MPV_PREFIX say where.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    println!("cargo:rerun-if-env-changed=MPVQT_PREFIX");
    println!("cargo:rerun-if-env-changed=MPV_PREFIX");
    let mpvqt_prefix = std::env::var("MPVQT_PREFIX").unwrap_or_else(|_| "/usr".to_string());
    // The prefixes the system does not already search. /usr is never named: -I/usr/include
    // breaks libstdc++'s #include_next, and /usr/lib is on the linker's path anyway.
    let mut extra: Vec<String> = [Some(mpvqt_prefix.clone()), std::env::var("MPV_PREFIX").ok()]
        .into_iter()
        .flatten()
        .filter(|p| p != "/usr")
        .collect();
    extra.dedup();
    let mut builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("com.marcusrosado.AniBeam")
            .version(1, 0)
            .qml_file("qml/Main.qml")
            .qml_file("qml/Driver.qml")
            .qml_file("qml/Tokens.qml")
            .qml_file("qml/TokensPage.qml")
            .qml_file("qml/Corner.qml")
            .qml_file("qml/Frost.qml")
            .qml_file("qml/Blocker.qml")
            .qml_file("qml/WheelStep.qml")
            .qml_file("qml/Label.qml")
            .qml_file("qml/Icon.qml")
            .qml_file("qml/Chip.qml")
            .qml_file("qml/Seg.qml")
            .qml_file("qml/Switch.qml")
            .qml_file("qml/Button.qml")
            .qml_file("qml/Field.qml")
            .qml_file("qml/Dropdown.qml")
            .qml_file("qml/Swatches.qml")
            .qml_file("qml/SliderRow.qml")
            .qml_file("qml/SettingRow.qml")
            .qml_file("qml/Panel.qml")
            .qml_file("qml/FillRows.qml")
            .qml_file("qml/Frame.qml")
            .qml_file("qml/Nav.qml")
            .qml_file("qml/Menu.qml")
            .qml_file("qml/Tooltip.qml")
            .qml_file("qml/InlineConfirm.qml")
            .qml_file("qml/EmptyState.qml")
            .qml_file("qml/SectionHeader.qml")
            .qml_file("qml/PagePlaceholder.qml")
            .qml_file("qml/ThinScrollBar.qml")
            .qml_file("qml/GridKeys.qml")
            .qml_file("qml/PageScroll.qml")
            .qml_file("qml/Rail.qml")
            .qml_file("qml/StatusStrip.qml")
            .qml_file("qml/SearchField.qml")
            .qml_file("qml/Pager.qml")
            .qml_file("qml/Card.qml")
            .qml_file("qml/LibraryRow.qml")
            .qml_file("qml/LibraryPage.qml")
            .qml_file("qml/HomePage.qml")
            .qml_file("qml/PosterTone.qml")
            .qml_file("qml/StoneFact.qml")
            .qml_file("qml/FeedPage.qml")
            .qml_file("qml/WatchingPage.qml")
            .qml_file("qml/MetadataPage.qml")
            .qml_file("qml/MatchPanel.qml")
            .qml_file("qml/EpisodesPane.qml")
            .qml_file("qml/ConnectSheet.qml")
            .qml_file("qml/TrackerNudge.qml")
            .qml_file("qml/ConnectButton.qml")
            .qml_file("qml/PosterPool.qml")
            .qml_file("qml/StatusDot.qml")
            .qml_file("qml/ScorePicker.qml")
            .qml_file("qml/EpisodeRow.qml")
            .qml_file("qml/PersonCard.qml")
            .qml_file("qml/RecommendationCard.qml")
            .qml_file("qml/SeriesPage.qml")
            .qml_file("qml/PlayerButton.qml")
            .qml_file("qml/SeekBar.qml")
            .qml_file("qml/TrackPicker.qml")
            .qml_file("qml/Notice.qml")
            .qml_file("qml/KeyHelp.qml")
            .qml_file("qml/PlayerChrome.qml")
            .qml_file("qml/PlayerKeys.qml")
            .qml_file("qml/PlayerPage.qml")
            .qml_file("qml/ActivityDrawer.qml")
            .qml_file("qml/Note.qml")
            .qml_file("qml/Tile.qml")
            .qml_file("qml/Tiles.qml")
            .qml_file("qml/UsageBar.qml")
            .qml_file("qml/SettingsTab.qml")
            .qml_file("qml/SettingsPair.qml")
            .qml_file("qml/SourceRow.qml")
            .qml_file("qml/TrackerRow.qml")
            .qml_file("qml/SettingsPage.qml")
            .qml_file("qml/SettingsLibraryTab.qml")
            .qml_file("qml/LookPane.qml")
            .qml_file("qml/LookPreview.qml")
            .qml_file("qml/SettingsAppearanceTab.qml")
            .qml_file("qml/SubtitlePreview.qml")
            .qml_file("qml/SettingsPlaybackTab.qml")
            .qml_file("qml/SettingsDownloadsTab.qml")
            .qml_file("qml/DownloadsPage.qml")
            .qml_file("qml/ReleaseTable.qml")
            .qml_file("qml/SearchProgress.qml")
            .qml_file("qml/ShowCard.qml")
            .qml_file("qml/ShowRow.qml")
            .qml_file("qml/ShowDetail.qml")
            .qml_file("qml/RemoveSheet.qml")
            .qml_file("qml/TorrentRow.qml")
            .qml_file("qml/SettingsDataTab.qml"),
    )
    .qt_module("Quick")
    .files([
        "src/bridge/door.rs",
        "src/bridge/fmt.rs",
        "src/bridge/helpers.rs",
        "src/bridge/model.rs",
        "src/bridge/player.rs",
        "src/bridge/shell.rs",
        "src/bridge/theme.rs",
    ])
    .include_dir("cpp")
    // mpvqt_export.h includes mpvqt_version.h bare; CMake's target used to supply this.
    .include_dir(format!("{mpvqt_prefix}/include/MpvQt"))
    .cpp_files(["cpp/helpers.cpp", "cpp/videoitem.h", "cpp/videoitem.cpp"])
    // qrc:/qt/qml/com/marcusrosado/AniBeam/assets/...; a name left out of this list is a
    // blank icon at runtime, not a build error, so it is generated:
    // ls apps/linux/assets/icons/*.svg | sed 's|apps/linux/||;s|.*|        "&",|'
    .qrc_resources([
        "assets/icon.png",
        "assets/fonts/Archivo-Variable-Regular.ttf",
        "assets/fonts/Archivo-Variable-SemiBold.ttf",
        "assets/fonts/Archivo-Variable-ExtraBold.ttf",
        "assets/icons/activity.svg",
        "assets/icons/archive.svg",
        "assets/icons/arrow-down.svg",
        "assets/icons/arrow-left.svg",
        "assets/icons/arrow-right.svg",
        "assets/icons/arrow-up-right.svg",
        "assets/icons/arrow-up.svg",
        "assets/icons/audio-lines.svg",
        "assets/icons/badge-check.svg",
        "assets/icons/ban.svg",
        "assets/icons/bell.svg",
        "assets/icons/book-open.svg",
        "assets/icons/bookmark.svg",
        "assets/icons/calendar-clock.svg",
        "assets/icons/captions.svg",
        "assets/icons/case-sensitive.svg",
        "assets/icons/chart-pie.svg",
        "assets/icons/check-check.svg",
        "assets/icons/check.svg",
        "assets/icons/chevron-down.svg",
        "assets/icons/chevron-left.svg",
        "assets/icons/chevron-right.svg",
        "assets/icons/chevron-up.svg",
        "assets/icons/chevrons-right.svg",
        "assets/icons/circle-alert.svg",
        "assets/icons/circle-check.svg",
        "assets/icons/circle-play.svg",
        "assets/icons/circle-question-mark.svg",
        "assets/icons/circle-x.svg",
        "assets/icons/clapperboard.svg",
        "assets/icons/clock.svg",
        "assets/icons/copy.svg",
        "assets/icons/database.svg",
        "assets/icons/download.svg",
        "assets/icons/external-link.svg",
        "assets/icons/eye-off.svg",
        "assets/icons/eye.svg",
        "assets/icons/file-down.svg",
        "assets/icons/file-up.svg",
        "assets/icons/film.svg",
        "assets/icons/folder-open.svg",
        "assets/icons/folder-plus.svg",
        "assets/icons/folder-search.svg",
        "assets/icons/folder-x.svg",
        "assets/icons/folder.svg",
        "assets/icons/git-branch.svg",
        "assets/icons/globe.svg",
        "assets/icons/hard-drive.svg",
        "assets/icons/heart.svg",
        "assets/icons/house.svg",
        "assets/icons/image.svg",
        "assets/icons/info.svg",
        "assets/icons/keyboard.svg",
        "assets/icons/languages.svg",
        "assets/icons/layers.svg",
        "assets/icons/link.svg",
        "assets/icons/layout-grid.svg",
        "assets/icons/list-filter.svg",
        "assets/icons/list-video.svg",
        "assets/icons/list.svg",
        "assets/icons/log-in.svg",
        "assets/icons/log-out.svg",
        "assets/icons/maximize.svg",
        "assets/icons/minimize.svg",
        "assets/icons/monitor.svg",
        "assets/icons/moon.svg",
        "assets/icons/palette.svg",
        "assets/icons/pause.svg",
        "assets/icons/pencil.svg",
        "assets/icons/play.svg",
        "assets/icons/plus.svg",
        "assets/icons/radio.svg",
        "assets/icons/refresh-cw.svg",
        "assets/icons/rotate-ccw.svg",
        "assets/icons/rotate-cw.svg",
        "assets/icons/rss.svg",
        "assets/icons/scan.svg",
        "assets/icons/search.svg",
        "assets/icons/settings.svg",
        "assets/icons/shapes.svg",
        "assets/icons/skip-back.svg",
        "assets/icons/skip-forward.svg",
        "assets/icons/sliders-horizontal.svg",
        "assets/icons/sparkles.svg",
        "assets/icons/square-check.svg",
        "assets/icons/square.svg",
        "assets/icons/star.svg",
        "assets/icons/step-back.svg",
        "assets/icons/step-forward.svg",
        "assets/icons/sun.svg",
        "assets/icons/trash-2.svg",
        "assets/icons/trash.svg",
        "assets/icons/triangle-alert.svg",
        "assets/icons/tv.svg",
        "assets/icons/type.svg",
        "assets/icons/upload.svg",
        "assets/icons/user-check.svg",
        "assets/icons/users.svg",
        "assets/icons/volume-2.svg",
        "assets/icons/volume-x.svg",
        "assets/icons/wrench.svg",
        "assets/icons/x.svg",
    ]);
    // include_dir() copies a directory's headers into the build rather than adding a -I, and
    // skips names with no extension (MpvQt/MpvAbstractItem), so a prefix the compiler does
    // not search already goes in as a real -I.
    for prefix in &extra {
        let dir = format!("{prefix}/include");
        // SAFETY: only adds an include path to cxx-qt's own cc::Build.
        builder = unsafe {
            builder.cc_builder(|cc| {
                cc.include(&dir);
            })
        };
    }
    builder.build();

    for prefix in &extra {
        println!("cargo:rustc-link-search=native={prefix}/lib");
        // Both install names are @rpath/... on a Mac; the bundle step rewrites them to the
        // app's own Frameworks, and this rpath is how a bare cargo run finds them.
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{prefix}/lib");
        }
    }
    println!("cargo:rustc-link-lib=MpvQt");
    println!("cargo:rustc-link-lib=mpv");
}
