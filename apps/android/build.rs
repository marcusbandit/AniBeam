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
    // The player links libmpv. On Android it comes out of vendor/mpv/<abi>, which
    // scripts/fetch-mpv.sh fills; its ffmpeg libraries load by name from the APK beside it,
    // and libavcodec is linked too because the app hands it the Java VM before mpv starts. On
    // the iPhone it is static, out of vendor/mpv-ios/{device,sim}, which
    // scripts/fetch-mpv-ios.sh fills. `cfg(libmpv)` tells mpv.rs, video.rs and thumbs.rs that
    // libmpv and the platform's GL are there; without it they stand in for a player that
    // never opens.
    println!("cargo::rustc-check-cfg=cfg(libmpv)");
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("android") => android_mpv(&manifest),
        Ok("ios") => ios_mpv(&manifest),
        _ => {}
    }
    slint_build::compile("ui/app.slint").unwrap();
}

fn android_mpv(manifest: &std::path::Path) {
    println!("cargo::rustc-cfg=libmpv");
    let abi = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "arm64-v8a",
        Ok("x86_64") => "x86_64",
        other => panic!("no libmpv for {other:?}"),
    };
    let dir = manifest.join("vendor/mpv").join(abi);
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

/// MPVKit's static archives, every one in the slice's folder, then the system frameworks
/// and libraries they and video/metal.rs call. ld64 reads static archives in any order.
fn ios_mpv(manifest: &std::path::Path) {
    println!("cargo::rustc-cfg=libmpv");
    let root = manifest.join("vendor/mpv-ios");
    let slice = if std::env::var("CARGO_CFG_TARGET_ABI").as_deref() == Ok("sim") {
        "sim"
    } else {
        "device"
    };
    let dir = root.join(slice);
    let mut archives: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            Some(name.strip_prefix("lib")?.strip_suffix(".a")?.to_string())
        })
        .collect();
    archives.sort();
    assert!(
        archives.iter().any(|a| a == "Libmpv"),
        "{} has no libLibmpv.a: run scripts/fetch-mpv-ios.sh",
        dir.display()
    );
    println!("cargo:rerun-if-changed={}", root.join(".tag").display());
    println!("cargo:rustc-link-search=native={}", dir.display());
    for a in &archives {
        println!("cargo:rustc-link-lib=static:-bundle={a}");
    }
    for f in [
        "AudioToolbox",
        "AVFoundation",
        "CoreAudio",
        "CoreFoundation",
        "CoreGraphics",
        "CoreMedia",
        "CoreText",
        "CoreVideo",
        "Foundation",
        "IOSurface",
        "Metal",
        "OpenGLES",
        "QuartzCore",
        "Security",
        "UIKit",
        "VideoToolbox",
    ] {
        println!("cargo:rustc-link-lib=framework={f}");
    }
    for l in ["bz2", "c++", "expat", "iconv", "resolv", "xml2", "z"] {
        println!("cargo:rustc-link-lib=dylib={l}");
    }
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
