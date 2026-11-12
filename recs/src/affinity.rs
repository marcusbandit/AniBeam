//! Content model: for every feature, how far above or below your own mean the shows carrying it
//! sit. Shrunk toward zero so one 10/10 with a rare director can't outvote thirty ratings.

use std::collections::HashMap;

use crate::features::{Side, features};
use crate::signal::Signal;
use crate::types::{Catalog, Show};

/// Pseudo-count for shrinkage: a feature needs about this much weight before it says much.
pub const SHRINK: f32 = 6.0;
/// The most judge-page votes together may move any single feature, in points. A poster is
/// thin evidence, and a run of swipes must not mark down a whole studio.
pub const VOTE_CAP: f32 = 10.0;

#[derive(Clone, Debug, Default)]
struct Stat {
    /// Weighted distance from the mean, from list entries.
    dev: f32,
    /// The same from judge-page votes, kept apart so its total can be capped.
    votes: f32,
    w: f32,
    /// Shows on the list carrying the feature. Votes never count: a swiped poster is not a
    /// show you rated.
    n: u32,
}

#[derive(Clone, Debug)]
pub struct Affinity {
    side: Side,
    stats: HashMap<String, Stat>,
    /// Traits set aside for now: they say nothing until brought back.
    muted: std::collections::HashSet<String>,
    /// Points per trait from the joint model, when it is the one in use. Then a show's
    /// prediction is the sum of its traits' points rather than their average.
    joint: Option<std::sync::Arc<HashMap<String, f32>>>,
}

/// One feature's part in a prediction, kept so a pick can say why.
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    pub key: String,
    pub label: String,
    /// Shrunk points above (or below) your mean, 0 to 100 scale.
    pub aff: f32,
    /// Shows of yours that carry the feature.
    pub n: u32,
    pub w: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prediction {
    /// Points above your mean this side expects, 0 to 100 scale.
    pub dev: f32,
    /// Strongest features first.
    pub evidence: Vec<Evidence>,
}

impl Affinity {
    pub fn fit(signals: &[Signal], mean: f32, catalog: &Catalog, side: Side) -> Affinity {
        let mut stats: HashMap<String, Stat> = HashMap::new();
        for s in signals {
            let Some(show) = catalog.get(&s.show) else {
                continue;
            };
            let sw = match side {
                Side::Taste => s.taste_w,
                Side::Look => s.look_w,
            };
            for f in features(show, side) {
                let st = stats.entry(f.key).or_default();
                let dev = sw * f.w * (s.value - mean);
                if s.vote {
                    st.votes += dev;
                } else {
                    st.dev += dev;
                    st.n += 1;
                }
                st.w += sw * f.w;
            }
        }
        Affinity {
            side,
            stats,
            muted: Default::default(),
            joint: None,
        }
    }

    /// The same side, scored by the joint model's points instead of one-at-a-time averages.
    /// The counts of your shows per trait still come from the list, for explaining.
    pub fn with_joint(mut self, coef: std::sync::Arc<HashMap<String, f32>>) -> Affinity {
        self.joint = Some(coef);
        self
    }

    pub fn mute(&mut self, keys: &std::collections::HashSet<String>) {
        self.muted.extend(keys.iter().cloned());
    }

    pub fn get(&self, key: &str) -> Option<(f32, u32)> {
        if self.muted.contains(key) {
            return None;
        }
        if let Some(coef) = &self.joint {
            let n = self.stats.get(key).map_or(0, |s| s.n);
            return coef.get(key).map(|c| (*c, n));
        }
        self.stats.get(key).map(|s| {
            let den = s.w + SHRINK;
            // Votes' share of the numerator capped at VOTE_CAP * den caps their effect at
            // VOTE_CAP points, however many there are.
            let votes = s.votes.clamp(-VOTE_CAP * den, VOTE_CAP * den);
            ((s.dev + votes) / den, s.n)
        })
    }

    pub fn predict(&self, show: &Show) -> Prediction {
        let mut num = 0.0;
        let mut den = 0.0;
        let mut evidence = Vec::new();
        for f in features(show, self.side) {
            let Some((aff, n)) = self.get(&f.key) else {
                continue;
            };
            num += f.w * aff;
            den += f.w;
            evidence.push(Evidence {
                key: f.key,
                label: f.label,
                aff,
                n,
                w: f.w,
            });
        }
        evidence.sort_by(|a, b| (b.w * b.aff.abs()).total_cmp(&(a.w * a.aff.abs())));
        // Joint points add up; one-at-a-time affinities are averaged, and the +1 keeps a
        // show with one known feature from inheriting its whole affinity.
        let dev = if self.joint.is_some() {
            num
        } else if den > 0.0 {
            num / (den + 1.0)
        } else {
            0.0
        };
        Prediction { dev, evidence }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Credit, Format, LookRole, Show};

    #[test]
    fn votes_together_move_a_feature_at_most_the_cap() {
        let show = Show {
            id: 1,
            title: "A".into(),
            format: Format::Tv,
            episodes: None,
            minutes: None,
            year: None,
            genres: vec![],
            tags: vec![],
            studios: vec![],
            credits: vec![Credit {
                staff: 9,
                name: "D".into(),
                role: LookRole::Director,
            }],
            community: None,
            popularity: None,
            relations: vec![],
            adult: false,
            status: None,
        };
        let catalog = Catalog::from([(1, show)]);
        // Thirty nahs on one director's show: uncapped they would move the director by about
        // nine points. Votes also never count as shows rated.
        let nah = Signal {
            show: 1,
            value: 58.0,
            taste_w: 0.2,
            look_w: 0.5,
            dropped: false,
            vote: true,
        };
        let a = Affinity::fit(&vec![nah; 30], 70.0, &catalog, Side::Look);
        let (aff, n) = a.get("staff:9").unwrap();
        assert!(aff < -3.0, "the cap should bind here: {aff}");
        assert_eq!(n, 0);
        assert!(aff >= -VOTE_CAP - 1e-4, "{aff}");
    }
}
