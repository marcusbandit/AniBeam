//! The three models side by side, and the blend that turns their opinions into one number.

use serde::{Deserialize, Serialize};

use crate::affinity::{Affinity, Prediction};
use crate::features::Side;
use crate::neighbors::{Neighborhood, PeerPrediction};
use crate::signal::Signal;
use crate::types::{Catalog, Neighbor, Show};

/// Peers kept per neighbourhood.
pub const PEERS: usize = 60;
/// AniList averages sit around here, so a show at the crowd mean adds nothing.
pub const CROWD_MEAN: f32 = 75.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Weights {
    pub taste: f32,
    pub look: f32,
    pub peers: f32,
    pub crowd: f32,
    /// Shrinks the blend back to the size of real score differences. Tuning picks weights for
    /// ranking, which can stretch the numbers; `eval` fits this so a 9.0 means a 9.0.
    #[serde(default = "one")]
    pub scale: f32,
}

fn one() -> f32 {
    1.0
}

impl Default for Weights {
    fn default() -> Weights {
        Weights {
            taste: 0.5,
            look: 0.8,
            peers: 1.0,
            crowd: 0.25,
            scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parts {
    pub taste: Prediction,
    pub look: Prediction,
    pub peers: Option<PeerPrediction>,
    /// Community score minus the crowd mean.
    pub crowd: f32,
}

impl Weights {
    /// Predicted score, 0 to 100.
    pub fn score(&self, mean: f32, p: &Parts) -> f32 {
        mean + self.scale
            * (self.taste * p.taste.dev
                + self.look * p.look.dev
                + self.peers * p.peers.as_ref().map_or(0.0, |p| p.dev)
                + self.crowd * p.crowd)
    }
}

/// How the story and look sides score traits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fit {
    /// Each trait on its own: the average of your shows that carry it.
    Mean,
    /// All traits together: each gets only the credit the others don't explain.
    Joint,
    /// Story traits from the joint fit, so tags stop taking credit that belongs elsewhere;
    /// look traits one at a time, which predicts the look side better.
    Story,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub mean: f32,
    pub taste: Affinity,
    pub look: Affinity,
    pub peers: Neighborhood,
}

impl Model {
    pub fn fit(signals: &[Signal], mean: f32, catalog: &Catalog, neighbors: &[Neighbor]) -> Model {
        Model::fit_with(signals, mean, catalog, neighbors, Fit::Mean)
    }

    pub fn fit_with(
        signals: &[Signal],
        mean: f32,
        catalog: &Catalog,
        neighbors: &[Neighbor],
        how: Fit,
    ) -> Model {
        let mut taste = Affinity::fit(signals, mean, catalog, Side::Taste);
        let mut look = Affinity::fit(signals, mean, catalog, Side::Look);
        // The joint fit always sees every trait, so a tag can't take credit that belongs to the
        // director. Which sides then read its points is the choice.
        if how != Fit::Mean {
            let coef = std::sync::Arc::new(crate::joint::fit(
                signals,
                mean,
                catalog,
                crate::joint::LAMBDA,
            ));
            taste = taste.with_joint(coef.clone());
            if how == Fit::Joint {
                look = look.with_joint(coef);
            }
        }
        Model {
            mean,
            taste,
            look,
            peers: Neighborhood::build(signals, mean, neighbors, PEERS),
        }
    }

    /// Set traits aside for now, such as a tag or a studio the person is tired of.
    pub fn mute(mut self, keys: &std::collections::HashSet<String>) -> Model {
        self.taste.mute(keys);
        self.look.mute(keys);
        self
    }

    pub fn parts(&self, show: &Show) -> Parts {
        Parts {
            taste: self.taste.predict(show),
            look: self.look.predict(show),
            peers: self.peers.predict(show.id),
            crowd: show.community.map_or(0.0, |c| c as f32 - CROWD_MEAN),
        }
    }
}
