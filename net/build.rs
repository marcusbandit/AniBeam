use std::path::Path;
use std::process::Command;

/// `phone_os`, as in library/build.rs, and the build's version for `VERSION` in
/// `lib.rs`. The version is worked out here, the lowest crate that names it, because every
/// provider client sends it in its user agent; the core hands it on. A commit moves it, so
/// it rebuilds this crate and the ones above, never the foundation (#971).
fn main() {
    // `phone_os`: the target is a phone's, Android or iOS. The phone's clients resolve
    // over HTTPS and keep a front's cookies (lib.rs). library/build.rs makes the same
    // switch for the library; a new phone platform is one line in each.
    println!("cargo::rustc-check-cfg=cfg(phone_os)");
    let phone = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "android" || os == "ios");
    if phone {
        println!("cargo::rustc-cfg=phone_os");
    }
    watch_git();
    rerun_when_head_moves();
    let fallback = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    // A build CI ships says what it is: 2.4.0, 2.4.0-nightly.20261030.12 or 2.4.0-pr.280.3,
    // and is named by it; core/build.rs reads the same variable for `RELEASE`. A build
    // without it (a checkout, the AUR) describes itself from git.
    println!("cargo:rerun-if-env-changed=ANIBEAM_RELEASE_VERSION");
    let shipped = std::env::var("ANIBEAM_RELEASE_VERSION")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    // Each app describes itself from its own tags only: the desktop's v2.3.0, the phone's
    // android-v2.1.1 (on Android and the iPhone alike). Any other tag never names a version.
    // A nightly's or test build's tag (v2.4.0-nightly.20261030.12) is never one to describe
    // from: only a stable version is.
    let (pattern, exclude, prefix) = if phone {
        ("android-v[0-9]*", "android-v*-*", "android-v")
    } else {
        ("v[0-9]*", "v*-*", "v")
    };
    let describe = Command::new("git")
        .args([
            "describe",
            "--tags",
            "--match",
            pattern,
            "--exclude",
            exclude,
            "--dirty",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    // The same rewrite the PKGBUILD's pkgver() applies: v2.0.0-14-g1a2b3c4-dirty
    // becomes 2.0.0.r14.g1a2b3c4.dirty, so the binary and pacman agree.
    let version = describe
        .map(|d| {
            let d = d.strip_prefix(prefix).unwrap_or(&d).to_string();
            let d = d.replace("-dirty", ".dirty");
            let d = match d.find("-g") {
                Some(i) => {
                    let (head, tail) = d.split_at(i);
                    let head = match head.rfind('-') {
                        Some(j) => format!("{}.r{}", &head[..j], &head[j + 1..]),
                        None => head.to_string(),
                    };
                    format!("{head}.{}", &tail[1..])
                }
                None => d,
            };
            d.replace('-', ".")
        })
        .unwrap_or(fallback);
    let version = shipped.unwrap_or(version);
    println!("cargo:rustc-env=ANIBEAM_VERSION={version}");
}

/// On a branch, `.git/HEAD` names the ref and never changes when a commit lands; the ref
/// file does. Watch HEAD in the git dir (a worktree's own), the ref it points to in the
/// common dir, and packed-refs, so the described hash is never stale. Only files that
/// exist are named: cargo reruns a script on every build for a missing one.
fn rerun_when_head_moves() {
    let rev_parse = |arg: &str| {
        Command::new("git")
            .args(["rev-parse", arg])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| std::path::PathBuf::from(s.trim()))
    };
    let (Some(git_dir), Some(common)) = (rev_parse("--git-dir"), rev_parse("--git-common-dir"))
    else {
        return;
    };
    let head = git_dir.join("HEAD");
    let mut watch = vec![head.clone(), common.join("packed-refs")];
    if let Ok(text) = std::fs::read_to_string(&head)
        && let Some(r) = text.trim().strip_prefix("ref: ")
    {
        watch.push(common.join(r));
    }
    for path in watch.into_iter().filter(|p| p.exists()) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

/// The version below changes with HEAD and the tags, so those are watched. Asked of git
/// rather than spelled `../.git/HEAD`: in a worktree `.git` is a file and HEAD lives under
/// the main checkout's `.git/worktrees/`, and a path that does not exist reads to Cargo as
/// changed on every build. Without git there is nothing to watch.
fn watch_git() {
    let paths = Command::new("git")
        .args(["rev-parse", "--git-path", "HEAD", "--git-path", "refs/tags"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    for path in paths.lines().map(str::trim).filter(|p| !p.is_empty()) {
        if Path::new(path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}
