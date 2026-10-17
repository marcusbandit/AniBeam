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
    slint_build::compile("ui/app.slint").unwrap();
}
