fn main() {
    changes();
    // scripts/build.sh works out the version the APK ships as (2.4.3-nightly.20261113.41 on a
    // nightly) and its versionCode; the app shows both in More. A plain cargo run, the tests
    // say, has neither and shows the crate's version.
    for (key, fallback) in [
        (
            "ANIBEAM_PHONE_VERSION",
            std::env::var("CARGO_PKG_VERSION").unwrap(),
        ),
        ("ANIBEAM_VERSION_CODE", "0".to_string()),
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        let value = std::env::var(key).unwrap_or(fallback);
        println!("cargo:rustc-env={key}={value}");
    }
    // The player links libmpv out of vendor/mpv/<abi>, which scripts/fetch-mpv.sh fills.
    // Its ffmpeg libraries load by name from the APK beside it; libavcodec is linked too
    // because the app hands it the Java VM before mpv starts. `cfg(libmpv)` tells mpv.rs,
    // video.rs and thumbs.rs that libmpv, EGL and GLES are there; without it they stand in
    // for a player that never opens.
    println!("cargo::rustc-check-cfg=cfg(libmpv)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo::rustc-cfg=libmpv");
        let abi = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
            Ok("aarch64") => "arm64-v8a",
            Ok("x86_64") => "x86_64",
            other => panic!("no libmpv for {other:?}"),
        };
        let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("vendor/mpv")
            .join(abi);
        assert!(
            dir.join("libmpv.so").is_file(),
            "{} has no libmpv.so: run scripts/fetch-mpv.sh",
            dir.display()
        );
        println!("cargo:rustc-link-search=native={}", dir.display());
        println!("cargo:rustc-link-lib=dylib=mpv");
        println!("cargo:rustc-link-lib=dylib=avcodec");
        println!("cargo:rustc-link-lib=dylib=EGL");
        println!("cargo:rustc-link-lib=dylib=GLESv2");
        println!("cargo:rerun-if-changed={}", dir.join(".tag").display());
    }
    // A debug build, the tests' among them, keeps each element's name, which lets a test find
    // the elements of a page laid out headless and measure them (#874).
    let mut config = slint_build::CompilerConfiguration::new();
    if std::env::var("PROFILE").as_deref() == Ok("debug") {
        config = config.with_debug_info(true);
    }
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}

include!("../../core/src/changelog_files.rs");

/// The patch notes waiting in changes/ for the next version, built in beside CHANGELOG.md:
/// What's new shows them under the newest heading until the release folds them into the
/// file (#504). A slice of `include_str!`s in name order, in OUT_DIR/changes.rs.
fn changes() {
    println!("cargo:rerun-if-changed=changes");
    let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("changes");
    let list: String = change_files(&dir)
        .iter()
        .map(|p| format!("include_str!({:?}),", p.display().to_string()))
        .collect();
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("changes.rs");
    std::fs::write(out, format!("&[{list}]")).unwrap();
}
