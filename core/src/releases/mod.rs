//! anirss's release logic, ported from Python into the core so every shell (the desktop
//! and the phone) runs one implementation: endpoints and their search URLs, erai's
//! specifics, reading a feed into releases, the seeders a feed leaves out, title parsing,
//! best fit, and the refine steps. Each module names the anirss file it came from; the
//! behaviour, and the tests, follow anirss's.
//!
//! The core's contract types are anirss's wire types already: `Release` is anirss's
//! `Item`, `ReleaseGroup` its `Group`, `BestFit` and `Refinement` its answers.

pub mod bestfit;
pub mod endpoints;
pub mod erai;
pub mod fetch;
pub mod refinement;
pub mod stats;
pub mod titles;

/// A search endpoint (anirss `endpoints.Endpoint`, without the on-demand start and stop
/// commands, which the core keeps elsewhere).
#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub name: String,
    /// `nyaa` or `rss`.
    pub kind: String,
    pub url: String,
    /// nyaa only.
    pub category: String,
    /// nyaa only.
    pub filter: String,
    /// Seconds a fetch may take.
    pub timeout: f64,
}

impl Endpoint {
    pub fn new(name: &str, kind: &str, url: &str) -> Endpoint {
        Endpoint {
            name: name.to_string(),
            kind: kind.to_string(),
            url: url.to_string(),
            category: "1_0".to_string(),
            filter: "0".to_string(),
            timeout: 10.0,
        }
    }

    /// True when `url` points at this endpoint's feed (everything in the template before
    /// `{query}` matches).
    pub fn owns_url(&self, url: &str) -> bool {
        let base = self.url.split("{query}").next().unwrap_or("");
        !base.is_empty() && url.starts_with(base)
    }
}

/// anirss `config.BestfitConfig`, with anirss's defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct BestfitConfig {
    pub preferred_groups: Vec<String>,
    pub source_order: Vec<String>,
    /// `highest`, or a target like `1080`.
    pub preferred_resolution: String,
    pub target_mib_per_episode: f64,
    pub max_size_ratio: f64,
    pub assumed_source: String,
}

impl Default for BestfitConfig {
    fn default() -> Self {
        BestfitConfig {
            preferred_groups: Vec::new(),
            source_order: ["BluRay", "WEB-DL", "WEB", "WEBRip", "HDTV"]
                .map(String::from)
                .to_vec(),
            preferred_resolution: "highest".to_string(),
            target_mib_per_episode: 1000.0,
            max_size_ratio: 3.0,
            assumed_source: "WEB".to_string(),
        }
    }
}
