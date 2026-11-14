//! The library module: classifying a file name, rendering its display
//! labels, and the read side that turns the tables into cards, details and
//! metadata rows. `BRACKETS` and `SPACES` live here because Task 8's folder
//! walk shares them with the classifier.

use std::sync::LazyLock;

use regex::Regex;

pub(crate) static BRACKETS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*\[[^\]]*\]\s*").unwrap());
pub(crate) static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub(crate) mod act;
pub mod cards;
pub mod classifier;
pub mod delete;
pub mod detail;
pub mod fixes;
pub(crate) mod history;
pub(crate) mod hostview;
pub mod labels;
pub mod readings;
pub mod reads;
pub mod report;
pub mod scan;
pub mod schedule;
pub mod schedule_text;
pub mod sidecar;
pub mod snapshot;
pub mod sort;
pub mod walk;
pub mod watcher;
