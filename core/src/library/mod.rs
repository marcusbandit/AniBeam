//! The library module: classifying a file name and rendering its display
//! labels. `BRACKETS` and `SPACES` live here because Task 8's folder walk
//! shares them with the classifier.

use std::sync::LazyLock;

use regex::Regex;

pub(crate) static BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\[[^\]]*\]\s*").unwrap());
pub(crate) static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub mod classifier;
pub mod labels;
