//! The `anibeam-export` document, the Export job that writes one and the
//! Import job that merges one in.
//!
//! One format serves two purposes: it is what Electron's last feature
//! wrote, and it is the native line's own backup. The core reads every
//! version from 1 up and writes the current one, so a file from either
//! world lands here.

pub mod export;
pub mod format;
pub mod import;

/// Path containment, the same rule `library::scan` reconciles with: a path
/// is under a root when it is the root or sits below it. String prefixes
/// are not enough, or `/lib2` would count as inside `/lib`.
pub(crate) fn under(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{}/", root.trim_end_matches('/')))
}

/// A trailing separator is not part of a path's identity, so it never
/// reaches a column: `/lib/` and `/lib` are one path, not two. A bare root
/// stays itself. The same normalisation `AddSource` puts a source through.
pub(crate) fn normalise(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

/// The last segment of a path: a show's folder name, a film's file name.
pub(crate) fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_is_a_path_prefix_not_a_string_prefix() {
        assert!(under("/lib/Show", "/lib"));
        assert!(under("/lib", "/lib"));
        assert!(!under("/lib2/Show", "/lib"));
        assert!(under("/lib/Show", "/lib/"));
    }

    #[test]
    fn a_trailing_separator_is_not_part_of_a_path() {
        assert_eq!(normalise("/lib/"), "/lib");
        assert_eq!(normalise("/lib"), "/lib");
        assert_eq!(normalise("/"), "/");
        assert_eq!(normalise("  /lib/anime/  "), "/lib/anime");
    }

    #[test]
    fn the_file_name_is_the_last_segment() {
        assert_eq!(file_name("/lib/Show"), "Show");
        assert_eq!(file_name("/lib/Movies/Film (2001).mkv"), "Film (2001).mkv");
    }
}
