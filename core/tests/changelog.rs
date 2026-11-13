//! Each app's CHANGELOG.md, held to the shape What's new reads: the newest heading is the
//! app's own version, the sections are New, Changed, Fixed and Removed, and every line fits
//! in two lines on a phone. It sits in the core's tests because CI runs those on every pull
//! request, and builds neither app. The rules for writing a line are in CONTRIBUTING.md.
//!
//! The change files in each app's changes/ count too: each is held to its own shape, the
//! notes they fold into are held to the file's, and `.github/changelog-fold.sh` has to fold
//! them into the file exactly as the apps read them.

use std::path::Path;

use anibeam_core::changelog;

fn read(path: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path);
    std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

/// The change files waiting in `app`/changes, in name order, as the apps build them in.
fn changes(app: &str) -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(app)
        .join("changes");
    changelog::change_files(&dir)
        .into_iter()
        .map(|p| {
            let name = format!("{app}/changes/{}", p.file_name().unwrap().to_string_lossy());
            (name, std::fs::read_to_string(&p).unwrap())
        })
        .collect()
}

fn check(app: &str, version: &str) {
    let path = format!("{app}/CHANGELOG.md");
    let problems = changelog::lint(&read(&path), version);
    assert!(problems.is_empty(), "{path}:\n  {}", problems.join("\n  "));
    for (name, text) in changes(app) {
        let problems = changelog::lint_change(&text);
        assert!(problems.is_empty(), "{name}:\n  {}", problems.join("\n  "));
    }
    #[cfg(unix)]
    folds_as_read(app, version, &changes(app));
}

/// Folds `app`'s notes with `extra` change files through the release's script, in a copy,
/// and holds the result to what the apps show and to the file's shape.
#[cfg(unix)]
fn folds_as_read(app: &str, version: &str, extra: &[(String, String)]) {
    let notes = read(&format!("{app}/CHANGELOG.md"));
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("changes")).unwrap();
    std::fs::write(dir.path().join("CHANGELOG.md"), &notes).unwrap();
    for (name, text) in extra {
        let file = name.rsplit('/').next().unwrap();
        std::fs::write(dir.path().join("changes").join(file), text).unwrap();
    }
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/changelog-fold.sh");
    let out = std::process::Command::new("bash")
        .arg(&script)
        .arg(dir.path())
        .output()
        .expect("bash runs changelog-fold.sh");
    assert!(
        out.status.success(),
        "changelog-fold.sh: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let folded = std::fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
    let texts: Vec<&str> = extra
        .iter()
        .filter(|(name, _)| !name.rsplit('/').next().unwrap().starts_with('.'))
        .map(|(_, t)| t.as_str())
        .collect();
    assert_eq!(
        changelog::parse(&folded),
        changelog::parse_with(&notes, &texts),
        "{app}: changelog-fold.sh folds the change files unlike the apps read them:\n{folded}"
    );
    let problems = changelog::lint(&folded, version);
    assert!(
        problems.is_empty(),
        "{app} folded:\n  {}",
        problems.join("\n  ")
    );
    assert!(
        changelog::change_files(&dir.path().join("changes")).is_empty(),
        "changelog-fold.sh left change files behind"
    );
}
/// `git` in `dir`, which has to work.
#[cfg(unix)]
fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The Release workflow folds what the promoted nightly had, as it had it: a change file
/// merged since stays, and so does a line added since to one that shipped.
#[cfg(unix)]
#[test]
fn the_release_folds_only_what_the_shipped_commit_had() {
    let repo = tempfile::tempdir().unwrap();
    let app = repo.path().join("app");
    std::fs::create_dir_all(app.join("changes")).unwrap();
    std::fs::write(
        app.join("CHANGELOG.md"),
        "# Notes\n\n## 2.5.0 · 2026-11-02\n\n## 2.4.0 · 2026-10-30\n\n### New\n- Old\n",
    )
    .unwrap();
    std::fs::write(app.join("changes/1-a.md"), "### New\n- Shipped\n").unwrap();
    git(repo.path(), &["init", "-q"]);
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-qm", "nightly"]);
    let shipped = git(repo.path(), &["rev-parse", "HEAD"]);
    std::fs::write(
        app.join("changes/1-a.md"),
        "### New\n- Shipped\n\n### Fixed\n- Later fix\n",
    )
    .unwrap();
    std::fs::write(app.join("changes/2-b.md"), "### Fixed\n- Later\n").unwrap();
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-qm", "after"]);
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/changelog-fold.sh");
    let out = std::process::Command::new("bash")
        .arg(&script)
        .arg(&app)
        .arg(&shipped)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let notes = changelog::parse(&std::fs::read_to_string(app.join("CHANGELOG.md")).unwrap());
    assert_eq!(notes[0].version, "2.5.0");
    assert_eq!(notes[0].sections.len(), 1);
    assert_eq!(notes[0].sections[0].lines, ["Shipped"]);
    assert_eq!(
        std::fs::read_to_string(app.join("changes/1-a.md")).unwrap(),
        "### Fixed\n- Later fix\n"
    );
    assert!(app.join("changes/2-b.md").exists());
    // A change file that shipped whole is gone from the index too, so the release commit
    // takes its deletion; one with lines left stays tracked.
    std::fs::write(app.join("changes/1-a.md"), "### New\n- Shipped\n").unwrap();
    git(repo.path(), &["commit", "-qam", "drop the later fix"]);
    let out = std::process::Command::new("bash")
        .arg(&script)
        .arg(&app)
        .arg(git(repo.path(), &["rev-parse", "HEAD"]))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        git(repo.path(), &["diff", "--cached", "--name-status"]),
        "D\tapp/changes/1-a.md\nD\tapp/changes/2-b.md"
    );
}

#[cfg(unix)]
#[test]
fn the_release_folds_change_files_under_the_newest_version_as_the_apps_read_them() {
    let extra = [
        (
            "apps/linux/changes/1-a.md".to_string(),
            "### New\n- A first thing\n\n### Removed\n- An old thing\n".to_string(),
        ),
        (
            "apps/linux/changes/2-b.md".to_string(),
            "<!-- a note to the reviewer -->\n### Changed\n- A changed thing\n\n### Fixed\n- A fixed thing  \n".to_string(),
        ),
        // A hidden file is no change file, for the fold as for the apps.
        (
            "apps/linux/changes/.3-c.md".to_string(),
            "### New\n- Never shown\n".to_string(),
        ),
    ];
    for app in ["apps/linux", "apps/android"] {
        let version = changelog::parse(&read(&format!("{app}/CHANGELOG.md")))[0]
            .version
            .clone();
        folds_as_read(app, &version, &extra);
    }
}

#[test]
fn the_desktop_notes_are_whole_and_name_its_version() {
    // The desktop shell takes the workspace version, as the core does.
    check("apps/linux", env!("CARGO_PKG_VERSION"));
}

#[test]
fn the_phone_notes_are_whole_and_name_its_version() {
    let cargo = read("apps/android/Cargo.toml");
    let version = cargo
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"'))
        .expect("a version in apps/android/Cargo.toml");
    // The APK takes its version from Cargo.toml alone (build.sh); the manifest names none,
    // so nothing can drift from it.
    let manifest = read("apps/android/android/AndroidManifest.xml");
    assert!(
        !manifest.contains("android:versionName") && !manifest.contains("android:versionCode"),
        "the manifest names a version; build.sh writes it from Cargo.toml"
    );
    check("apps/android", version);
}
