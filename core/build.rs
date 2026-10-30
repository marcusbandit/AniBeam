use std::path::Path;
use std::process::Command;

/// The tracker credentials a build bundles, and its Discord application id. Electron's
/// Vite read them out of the workspace's `.env.local`; Cargo has no such convention, so
/// this does the same read itself and hands each one to `option_env!` in
/// `trackers/accounts.rs` and `presence/mod.rs`.
const CREDENTIALS: [&str; 4] = [
    "ANIBEAM_ANILIST_CLIENT_ID",
    "ANIBEAM_MAL_CLIENT_ID",
    "ANIBEAM_MAL_CLIENT_SECRET",
    "ANIBEAM_DISCORD_CLIENT_ID",
];

fn main() {
    credentials();
    watch_git();
    rerun_when_head_moves();
    let fallback = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    // Each app describes itself from its own tags only: the desktop's v2.3.0, the phone's
    // android-v2.1.1. Any other tag (the preview release's) never names a version.
    let android = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "android");
    let (pattern, prefix) = if android {
        ("android-v[0-9]*", "android-v")
    } else {
        ("v[0-9]*", "v")
    };
    let describe = Command::new("git")
        .args(["describe", "--tags", "--match", pattern, "--dirty"])
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

/// Each credential from the environment first, so a packaging script or CI can set one
/// without a file, then from `.env.local` at the workspace root. A value that comes out
/// empty is not passed on at all, which `option_env!` reads as no credential.
fn credentials() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env.local");
    // Watched only when it exists: Cargo reads a missing file as changed, which would rerun
    // this script and rebuild the core on every build of a clone without one. The cost is
    // that a newly created .env.local goes unnoticed until `touch core/build.rs`.
    if file.exists() {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    let local = std::fs::read_to_string(&file).unwrap_or_default();
    for name in CREDENTIALS {
        println!("cargo:rerun-if-env-changed={name}");
        let value = std::env::var(name)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| dotenv_value(&local, name))
            .unwrap_or_default();
        if !value.is_empty() {
            println!("cargo:rustc-env={name}={value}");
        }
    }
}

/// One `KEY=VALUE` line's value, the way a dotenv file is read: comments and blank lines
/// skipped, an `export ` prefix allowed, whitespace trimmed, one pair of matching quotes
/// around the value stripped, and on an unquoted value a ` #` comment after it dropped.
/// The last line naming the key wins, as it does for Vite.
fn dotenv_value(text: &str, name: &str) -> Option<String> {
    let mut found = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != name {
            continue;
        }
        let value = value.trim();
        let quoted = ['"', '\'']
            .iter()
            .find_map(|q| value.strip_prefix(*q).and_then(|v| v.strip_suffix(*q)));
        let value = match quoted {
            Some(inner) => inner,
            None => value.split_once(" #").map_or(value, |(v, _)| v),
        };
        found = Some(value.trim().to_string());
    }
    found.filter(|v| !v.is_empty())
}
