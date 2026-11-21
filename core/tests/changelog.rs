//! Each app's CHANGELOG.md, held to the shape What's new reads: the newest heading is the
//! app's own version, headings and dates are valid, there are no em dashes, and no release
//! balloons into a commit dump. It sits in the core's tests because CI runs those on every
//! pull request, and builds neither app. The rules for writing release notes are in
//! CONTRIBUTING.md.

use std::path::Path;

use anibeam_core::changelog;

fn read(path: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path);
    std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

fn check(app: &str, version: &str) {
    let path = format!("{app}/CHANGELOG.md");
    let problems = changelog::lint(&read(&path), version);
    assert!(problems.is_empty(), "{path}:\n  {}", problems.join("\n  "));
}

#[test]
fn the_desktop_notes_are_whole_and_name_its_version() {
    // The desktop shell takes the workspace version, as the core does.
    check("apps/linux", env!("CARGO_PKG_VERSION"));
}

#[test]
fn the_phone_notes_are_whole_and_name_its_version() {
    let cargo = read("apps/phone/Cargo.toml");
    let version = cargo
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"'))
        .expect("a version in apps/phone/Cargo.toml");
    // The APK takes its version from Cargo.toml alone (build.sh); the manifest names none,
    // so nothing can drift from it.
    let manifest = read("apps/phone/android/AndroidManifest.xml");
    assert!(
        !manifest.contains("android:versionName") && !manifest.contains("android:versionCode"),
        "the manifest names a version; build.sh writes it from Cargo.toml"
    );
    check("apps/phone", version);
}
