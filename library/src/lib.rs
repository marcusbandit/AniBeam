//! The library team (#854): the sources, the scans, the folder walk and the
//! watcher over them, the classifier and its labels, and the reads that turn
//! the tables into cards, details and the snapshot. `BRACKETS` and `SPACES`
//! live here because the folder walk shares them with the classifier.

use std::sync::LazyLock;

use regex::Regex;

pub(crate) static BRACKETS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*\[[^\]]*\]\s*").unwrap());
pub(crate) static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

pub mod cards;
pub mod classifier;
pub mod delete;
pub mod detail;
pub mod fixes;
pub mod franchises;
pub mod history;
pub mod hostview;
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
mod team;
pub mod trash;
pub mod walk;
pub mod watcher;

pub use team::Incoming;
pub use team::Library;
