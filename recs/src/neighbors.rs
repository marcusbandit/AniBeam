//! People like you: AniList users whose ratings move with yours. Their scores on shows you have
//! not seen, measured against their own mean, predict yours.

use std::collections::HashMap;

use crate::signal::{Signal, signals};
use crate::types::{DropReasons, Neighbor, ShowId};

/// Shows two people must share before their agreement means anything.
pub const MIN_OVERLAP: usize = 12;
/// Similarity is shrunk by overlap / (overlap + this), so a perfect match on twelve shows counts
/// for less than a strong match on a hundred.
pub const OVERLAP_SHRINK: f32 = 25.0;

#[derive(Clone, Debug)]
pub struct Peer {
    pub user: u64,
    pub sim: f32,
    pub overlap: u32,
    pub mean: f32,
    dev: HashMap<ShowId, f32>,
}

#[derive(Clone, Debug, Default)]
pub struct Neighborhood {
    pub peers: Vec<Peer>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PeerPrediction {
    /// Points above your mean, 0 to 100 scale.
    pub dev: f32,
    /// How many similar people have rated it.
    pub raters: u32,
    /// Their plain average, 0 to 100, for showing a person.
    pub avg: f32,
    /// How much they love it, unshrunk: the similarity-weighted mean of their distances from
    /// their own means. Three people who adore a show score as high here as thirty would.
    pub love: f32,
}

impl Neighborhood {
    /// `me` is the training signal only, so an evaluation can hold shows out of it.
    pub fn build(me: &[Signal], my_mean: f32, others: &[Neighbor], keep: usize) -> Neighborhood {
        let mine: HashMap<ShowId, f32> = me.iter().map(|s| (s.show, s.value - my_mean)).collect();
        let none = DropReasons::new();
        let mut peers = Vec::new();
        for o in others {
            let Some((mean, sig)) = signals(&o.entries, &none) else {
                continue;
            };
            let dev: HashMap<ShowId, f32> = sig.iter().map(|s| (s.show, s.value - mean)).collect();
            let (mut ab, mut aa, mut bb, mut n) = (0.0f32, 0.0f32, 0.0f32, 0usize);
            for (show, a) in &mine {
                if let Some(b) = dev.get(show) {
                    ab += a * b;
                    aa += a * a;
                    bb += b * b;
                    n += 1;
                }
            }
            if n < MIN_OVERLAP || aa == 0.0 || bb == 0.0 {
                continue;
            }
            let r = ab / (aa.sqrt() * bb.sqrt());
            let sim = r * n as f32 / (n as f32 + OVERLAP_SHRINK);
            if sim > 0.05 {
                peers.push(Peer {
                    user: o.user,
                    sim,
                    overlap: n as u32,
                    mean,
                    dev,
                });
            }
        }
        peers.sort_by(|a, b| b.sim.total_cmp(&a.sim));
        peers.truncate(keep);
        Neighborhood { peers }
    }

    pub fn predict(&self, show: ShowId) -> Option<PeerPrediction> {
        let (mut num, mut den, mut raw, mut n) = (0.0f32, 0.0f32, 0.0f32, 0u32);
        for p in &self.peers {
            if let Some(d) = p.dev.get(&show) {
                num += p.sim * d;
                den += p.sim;
                raw += d + p.mean;
                n += 1;
            }
        }
        if n < 2 {
            return None;
        }
        // Shrink thin evidence: two lukewarm peers should not outweigh the content model.
        Some(PeerPrediction {
            dev: num / (den + 0.5),
            raters: n,
            avg: raw / n as f32,
            love: num / den,
        })
    }

    /// Every show the neighbourhood has an opinion on, with how many peers rated it.
    pub fn rated(&self) -> HashMap<ShowId, u32> {
        let mut out = HashMap::new();
        for p in &self.peers {
            for show in p.dev.keys() {
                *out.entry(*show).or_insert(0) += 1;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Entry, Status};

    fn list(scores: &[(ShowId, f32)]) -> Vec<Entry> {
        scores
            .iter()
            .map(|(show, s)| Entry {
                show: *show,
                status: Status::Completed,
                score: Some(*s),
                repeat: 0,
            })
            .collect()
    }

    #[test]
    fn a_like_minded_user_predicts_and_an_opposite_one_is_ignored() {
        let mine: Vec<(ShowId, f32)> = (1..=20)
            .map(|i| (i, if i % 2 == 0 { 90.0 } else { 50.0 }))
            .collect();
        let (mean, me) = signals(&list(&mine), &DropReasons::new()).unwrap();
        let mut twin = mine.clone();
        twin.push((99, 95.0));
        let mut twin2 = mine.clone();
        twin2.push((99, 90.0));
        let opposite: Vec<(ShowId, f32)> = mine
            .iter()
            .map(|(i, s)| (*i, 140.0 - s))
            .chain([(99, 10.0)])
            .collect();
        let others = vec![
            Neighbor {
                user: 1,
                entries: list(&twin),
            },
            Neighbor {
                user: 2,
                entries: list(&twin2),
            },
            Neighbor {
                user: 3,
                entries: list(&opposite),
            },
        ];
        let hood = Neighborhood::build(&me, mean, &others, 50);
        assert_eq!(hood.peers.len(), 2);
        let p = hood.predict(99).unwrap();
        assert!(p.dev > 0.0, "{p:?}");
        assert_eq!(p.raters, 2);
    }
}
