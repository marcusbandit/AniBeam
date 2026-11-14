use std::path::Path;

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
    // `phone_os`: the target is a phone's, Android or iOS. Every switch in the core that
    // means "the phone" reads it, so a new phone platform is one line here.
    println!("cargo::rustc-check-cfg=cfg(phone_os)");
    let phone = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "android" || os == "ios");
    if phone {
        println!("cargo::rustc-cfg=phone_os");
    }
    credentials();
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
