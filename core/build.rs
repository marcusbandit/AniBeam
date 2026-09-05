use std::process::Command;

fn main() {
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
