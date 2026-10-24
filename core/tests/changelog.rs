//! Each app's CHANGELOG.md, held to the shape What's new reads: the newest heading is the
//! app's own version, the sections are New, Changed, Fixed and Removed, and every line fits
//! in two lines on a phone. It sits in the core's tests because CI runs those on every pull
//! request, and builds neither app. The rules for writing a line are in CONTRIBUTING.md.

use std::path::Path;

use anibeam_core::changelog;

fn read(path: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path);
    std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

fn check(path: &str, version: &str) {
    let problems = changelog::lint(&read(path), version);
    assert!(problems.is_empty(), "{path}:\n  {}", problems.join("\n  "));
}

#[test]
fn the_desktop_notes_are_whole_and_name_its_version() {
    // The desktop shell takes the workspace version, as the core does.
    check("apps/linux/CHANGELOG.md", env!("CARGO_PKG_VERSION"));
}

#[test]
fn the_phone_notes_are_whole_and_name_its_version() {
    let manifest = read("apps/android/Cargo.toml");
    let version = manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"'))
        .expect("a version in apps/android/Cargo.toml");
    check("apps/android/CHANGELOG.md", version);
}
