use std::path::Path;
use std::process::Command;

/// The tracker credentials a build bundles. Electron's Vite read them out of the
/// workspace's `.env.local`; Cargo has no such convention, so this does the same read
/// itself and hands each one to `option_env!` in `trackers/accounts.rs`.
const CREDENTIALS: [&str; 3] = [
    "ANIBEAM_ANILIST_CLIENT_ID",
    "ANIBEAM_MAL_CLIENT_ID",
    "ANIBEAM_MAL_CLIENT_SECRET",
];

fn main() {
    credentials();
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/tags");
    let fallback = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let describe = Command::new("git")
        .args(["describe", "--tags", "--dirty"])
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
            let d = d.strip_prefix('v').unwrap_or(&d).to_string();
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

/// Each credential from the environment first, so a packaging script or CI can set one
/// without a file, then from `.env.local` at the workspace root. A value that comes out
/// empty is not passed on at all, which `option_env!` reads as no credential.
fn credentials() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env.local");
    println!("cargo:rerun-if-changed={}", file.display());
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
/// around the value stripped. The last line naming the key wins, as it does for Vite.
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
        let unquoted = ['"', '\'']
            .iter()
            .find_map(|q| value.strip_prefix(*q).and_then(|v| v.strip_suffix(*q)))
            .unwrap_or(value);
        found = Some(unquoted.trim().to_string());
    }
    found.filter(|v| !v.is_empty())
}
