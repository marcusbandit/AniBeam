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
    fn the_file_name_is_the_last_segment() {
        assert_eq!(file_name("/lib/Show"), "Show");
        assert_eq!(file_name("/lib/Movies/Film (2001).mkv"), "Film (2001).mkv");
    }
}
