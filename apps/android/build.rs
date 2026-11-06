fn main() {
    changes();
    // The manifest owns the version; the app shows the build number from it in Settings.
    let manifest = std::fs::read_to_string("android/AndroidManifest.xml")
        .expect("android/AndroidManifest.xml");
    let code = manifest
        .split("android:versionCode=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("versionCode in the manifest");
    println!("cargo:rustc-env=ANIBEAM_VERSION_CODE={code}");
    println!("cargo:rerun-if-changed=android/AndroidManifest.xml");
    // The player links libmpv out of vendor/mpv/<abi>, which scripts/fetch-mpv.sh fills.
    // Its ffmpeg libraries load by name from the APK beside it; libavcodec is linked too
    // because the app hands it the Java VM before mpv starts.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
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
    slint_build::compile("ui/app.slint").unwrap();
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
