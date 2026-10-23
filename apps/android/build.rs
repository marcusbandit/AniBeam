fn main() {
    // The manifest owns the version; the app shows the build number from it in Settings.
    let manifest = std::fs::read_to_string("android/AndroidManifest.xml").expect("android/AndroidManifest.xml");
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
        let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendor/mpv").join(abi);
        assert!(dir.join("libmpv.so").is_file(), "{} has no libmpv.so: run scripts/fetch-mpv.sh", dir.display());
        println!("cargo:rustc-link-search=native={}", dir.display());
        println!("cargo:rustc-link-lib=dylib=mpv");
        println!("cargo:rustc-link-lib=dylib=avcodec");
        println!("cargo:rustc-link-lib=dylib=EGL");
        println!("cargo:rustc-link-lib=dylib=GLESv2");
        println!("cargo:rerun-if-changed={}", dir.join(".tag").display());
    }
    slint_build::compile("ui/app.slint").unwrap();
}
