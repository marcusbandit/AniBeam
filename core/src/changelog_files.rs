// The change files waiting in an app's `changes/`, in name order (#504): every Markdown
// file but the README and a hidden one, which `.github/changelog-fold.sh`'s `*.md` skips
// too. One listing for the core's tests and both apps' build.rs, which `include!` this
// file, so the apps, the tests and the fold always agree on which files count.

/// The change files in `dir`, sorted by name; none when it cannot be read.
pub fn change_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    name.ends_with(".md") && name != "README.md" && !name.starts_with('.')
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}
