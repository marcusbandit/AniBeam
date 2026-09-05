//! The library module: classifying a file name and rendering its display
//! labels. `BRACKETS` and `SPACES` live here because Task 8's folder walk
//! shares them with the classifier.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

pub(crate) static BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\[[^\]]*\]\s*").unwrap());
pub(crate) static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

/// A name's extension, lowercased, checked against a list. Shared by
/// `walk::is_video` and `sidecar::is_subtitle` so the two never drift.
pub(crate) fn has_extension(name: &str, list: &[&str]) -> bool {
    Path::new(name).extension().is_some_and(|e| list.contains(&e.to_string_lossy().to_lowercase().as_str()))
}

pub mod classifier;
pub mod labels;
pub mod sidecar;
pub mod walk;
