//! The library module: classifying a file name, rendering its display
//! labels, and the read side that turns the tables into cards, details and
//! metadata rows. `BRACKETS` and `SPACES` live here because Task 8's folder
//! walk shares them with the classifier.

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
mod team;
pub mod trash;
pub mod walk;
pub mod watcher;

pub(crate) use team::Incoming;
pub use team::Library;
