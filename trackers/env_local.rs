// The bundled credentials' reader, shared by the build scripts of the crates that compile
// one in: this crate's (the tracker apps) and the core's (Discord). Each `include!`s it, so
// `.env.local` is read one way. It is not a module of the crate; only build scripts use it.

use std::path::Path;

/// Each of `names` from the environment first, so a packaging script or CI can set one
/// without a file, then from `.env.local` at the workspace root, handed to the crate this
/// build script compiles for `option_env!`. A value that comes out empty is not passed on
/// at all, which `option_env!` reads as no credential.
fn bundle(names: &[&str]) {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env.local");
    // Watched only when it exists: Cargo reads a missing file as changed, which would rerun
    // this script and rebuild the crate on every build of a clone without one. The cost is
    // that a newly created .env.local goes unnoticed until the build script is touched.
    if file.exists() {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    let local = std::fs::read_to_string(&file).unwrap_or_default();
    for &name in names {
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
