fn main() {
    // `phone_os`, the switch for a phone's target, is made in net/build.rs and
    // library/build.rs, beside the code that reads it; nothing left in the core does.
    // The bundled credentials are read in trackers/build.rs and playback/build.rs, beside
    // the code that reads them.
    // A build CI ships says what it is: 2.4.0, 2.4.0-nightly.20261030.12 or 2.4.0-pr.280.3.
    // That string names the build and decides which updates it is offered; a build without
    // it (a checkout, the AUR) counts as the workspace version. `VERSION`, the describe
    // string, is worked out in net/build.rs.
    println!("cargo:rerun-if-env-changed=ANIBEAM_RELEASE_VERSION");
    let shipped = std::env::var("ANIBEAM_RELEASE_VERSION")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let fallback = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    println!(
        "cargo:rustc-env=ANIBEAM_RELEASE={}",
        shipped.as_deref().unwrap_or(&fallback)
    );
}
