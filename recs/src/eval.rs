//! Does it work? Hide a fifth of your list, predict it from the rest, compare. Five times, so
//! every show gets hidden once. Nothing here touches the network.

use crate::model::{Fit, Model, Parts, Weights};
use crate::signal::Signal;
use crate::types::{Catalog, Neighbor};

#[derive(Clone, Debug)]
pub struct Held {
    pub show: u64,
    pub actual: f32,
    pub dropped: bool,
    pub parts: Parts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    pub name: String,
    /// Rank agreement between predicted and real scores, -1 to 1.
    pub spearman: f32,
    /// Chance a random drop is ranked below a random show you liked, 0.5 is a coin flip.
    pub drop_auc: f32,
    pub n: usize,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub mean: f32,
    pub held: Vec<Held>,
    pub rows: Vec<Metrics>,
    pub best: Weights,
    /// The best blend that ignores the community score, for hidden gems.
    pub gems: Weights,
    /// Anything a person should know about the fit, such as a blend that failed to calibrate.
    pub notes: Vec<String>,
    /// The way of scoring traits that predicted better, and that the weights belong to.
    pub fit: Fit,
}

fn fold_of(show: u64, folds: u64) -> u64 {
    // A fixed scramble, so the split is the same every run and not tied to id order.
    show.wrapping_mul(2_654_435_761) % folds
}

pub fn holdout(
    signals: &[Signal],
    mean: f32,
    catalog: &Catalog,
    neighbors: &[Neighbor],
    folds: u64,
    how: Fit,
) -> Vec<Held> {
    let usable: Vec<&Signal> = signals
        .iter()
        .filter(|s| catalog.contains_key(&s.show))
        .collect();
    let mut held = Vec::new();
    for f in 0..folds {
        let train: Vec<Signal> = usable
            .iter()
            .filter(|s| fold_of(s.show, folds) != f)
            .map(|s| (*s).clone())
            .collect();
        let model = Model::fit_with(&train, mean, catalog, neighbors, how);
        for s in usable.iter().filter(|s| fold_of(s.show, folds) == f) {
            let parts = model.parts(&catalog[&s.show]);
            held.push(Held {
                show: s.show,
                actual: s.value,
                dropped: s.dropped,
                parts,
            });
        }
    }
    held
}

fn ranks(xs: &[f32]) -> Vec<f32> {
    let mut idx: Vec<usize> = (0..xs.len()).collect();
    idx.sort_by(|a, b| xs[*a].total_cmp(&xs[*b]));
    let mut r = vec![0.0; xs.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && xs[idx[j + 1]] == xs[idx[i]] {
            j += 1;
        }
        let avg = (i + j) as f32 / 2.0;
        for k in i..=j {
            r[idx[k]] = avg;
        }
        i = j + 1;
    }
    r
}

pub fn spearman(a: &[f32], b: &[f32]) -> f32 {
    let (ra, rb) = (ranks(a), ranks(b));
    let n = ra.len() as f32;
    if n < 3.0 {
        return 0.0;
    }
    let (ma, mb) = (ra.iter().sum::<f32>() / n, rb.iter().sum::<f32>() / n);
    let (mut ab, mut aa, mut bb) = (0.0, 0.0, 0.0);
    for (x, y) in ra.iter().zip(&rb) {
        ab += (x - ma) * (y - mb);
        aa += (x - ma).powi(2);
        bb += (y - mb).powi(2);
    }
    if aa == 0.0 || bb == 0.0 {
        0.0
    } else {
        ab / (aa * bb).sqrt()
    }
}

/// Chance a random `low` scores below a random `high`, ties counting half. 0.5 when either
/// side is empty, since there is nothing to compare.
pub fn auc(low: &[f32], high: &[f32]) -> f32 {
    if low.is_empty() || high.is_empty() {
        return 0.5;
    }
    let mut wins = 0.0;
    for l in low {
        for h in high {
            wins += if l < h {
                1.0
            } else if l == h {
                0.5
            } else {
                0.0
            };
        }
    }
    wins / (low.len() * high.len()) as f32
}

pub fn drop_auc(held: &[Held], predicted: &[f32], mean: f32) -> f32 {
    let drops: Vec<f32> = held
        .iter()
        .zip(predicted)
        .filter(|(h, _)| h.dropped)
        .map(|(_, p)| *p)
        .collect();
    let liked: Vec<f32> = held
        .iter()
        .zip(predicted)
        .filter(|(h, _)| !h.dropped && h.actual >= mean + 5.0)
        .map(|(_, p)| *p)
        .collect();
    auc(&drops, &liked)
}

fn metrics(name: &str, held: &[Held], mean: f32, w: &Weights) -> Metrics {
    let predicted: Vec<f32> = held.iter().map(|h| w.score(mean, &h.parts)).collect();
    let actual: Vec<f32> = held.iter().map(|h| h.actual).collect();
    Metrics {
        name: name.into(),
        spearman: spearman(&predicted, &actual),
        drop_auc: drop_auc(held, &predicted, mean),
        n: held.len(),
    }
}

/// Least squares on the held-out shows: the scale that makes predicted distance from your mean
/// match the real distance. Ranks don't change, only how big the numbers are.
/// A blend that moves against your real scores gets no scale at all, and a note saying so,
/// rather than a clamp that would quietly flatten every prediction to your mean.
pub fn calibrate(held: &[Held], mean: f32, w: Weights) -> Result<Weights, String> {
    let raw = Weights { scale: 1.0, ..w };
    let (mut xy, mut xx) = (0.0f32, 0.0f32);
    for h in held {
        let x = raw.score(mean, &h.parts) - mean;
        xy += x * (h.actual - mean);
        xx += x * x;
    }
    if xx == 0.0 || xy <= 0.0 {
        return Err("predictions do not move with your real scores; left uncalibrated".into());
    }
    Ok(Weights {
        scale: (xy / xx).min(5.0),
        ..w
    })
}

/// One number to climb: rank agreement, plus how much better than a coin flip drops are caught.
fn objective(m: &Metrics) -> f32 {
    m.spearman + (m.drop_auc - 0.5)
}

/// One way of scoring traits, tuned: its rows, its best blends, and how good the best is.
struct Tuned {
    held: Vec<Held>,
    rows: Vec<Metrics>,
    best: Weights,
    gems: Weights,
    notes: Vec<String>,
    score: f32,
}

fn tune(held: Vec<Held>, mean: f32, label: &str) -> Tuned {
    let only = |taste, look, peers, crowd| Weights {
        taste,
        look,
        peers,
        crowd,
        scale: 1.0,
    };
    let mut rows = vec![
        metrics(
            &format!("story, {label}"),
            &held,
            mean,
            &only(1.0, 0.0, 0.0, 0.0),
        ),
        metrics(
            &format!("look, {label}"),
            &held,
            mean,
            &only(0.0, 1.0, 0.0, 0.0),
        ),
    ];
    let steps = [0.0, 0.5, 1.0, 1.5, 2.0, 3.0];
    let mut best = (f32::MIN, Weights::default());
    let mut gems = (f32::MIN, Weights::default());
    for &taste in &steps {
        for &look in &steps {
            for &peers in &steps {
                for &crowd in &[0.0, 0.25, 0.5, 1.0] {
                    let w = only(taste, look, peers, crowd);
                    let m = metrics("", &held, mean, &w);
                    if objective(&m) > best.0 {
                        best = (objective(&m), w);
                    }
                    if crowd == 0.0 && objective(&m) > gems.0 {
                        gems = (objective(&m), w);
                    }
                }
            }
        }
    }
    let mut notes = Vec::new();
    let mut fit = |name: &str, w: Weights| {
        calibrate(&held, mean, w).unwrap_or_else(|e| {
            notes.push(format!("{name}, {label}: {e}"));
            w
        })
    };
    let (score, best) = (best.0, fit("tuned blend", best.1));
    let gems = fit("gems blend", gems.1);
    rows.push(metrics(&format!("blend, {label}"), &held, mean, &best));
    rows.push(metrics(&format!("gems blend, {label}"), &held, mean, &gems));
    Tuned {
        held,
        rows,
        best,
        gems,
        notes,
        score,
    }
}

/// Scores both ways of fitting traits, one at a time and all together, and keeps whichever
/// predicts the held-out shows better.
pub fn evaluate(
    signals: &[Signal],
    mean: f32,
    catalog: &Catalog,
    neighbors: &[Neighbor],
    folds: u64,
) -> Report {
    let one = tune(
        holdout(signals, mean, catalog, neighbors, folds, Fit::Mean),
        mean,
        "one trait at a time",
    );
    let all = tune(
        holdout(signals, mean, catalog, neighbors, folds, Fit::Joint),
        mean,
        "all traits together",
    );
    let story = tune(
        holdout(signals, mean, catalog, neighbors, folds, Fit::Story),
        mean,
        "tags together, staff apart",
    );
    let only = |taste, look, peers, crowd| Weights {
        taste,
        look,
        peers,
        crowd,
        scale: 1.0,
    };
    let mut rows = vec![
        metrics(
            "crowd score only",
            &one.held,
            mean,
            &only(0.0, 0.0, 0.0, 1.0),
        ),
        metrics(
            "people like you",
            &one.held,
            mean,
            &only(0.0, 0.0, 1.0, 0.0),
        ),
    ];
    rows.extend(one.rows.iter().cloned());
    rows.extend(all.rows.iter().cloned());
    rows.extend(story.rows.iter().cloned());
    let (win, fit) = [(one, Fit::Mean), (all, Fit::Joint), (story, Fit::Story)]
        .into_iter()
        .max_by(|a, b| a.0.score.total_cmp(&b.0.score))
        .expect("three candidates");
    Report {
        mean,
        held: win.held,
        rows,
        best: win.best,
        gems: win.gems,
        notes: win.notes,
        fit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spearman_is_one_for_the_same_order_and_minus_one_reversed() {
        let a = [1.0, 2.0, 3.0, 4.0];
        assert!((spearman(&a, &[10.0, 20.0, 30.0, 40.0]) - 1.0).abs() < 1e-6);
        assert!((spearman(&a, &[4.0, 3.0, 2.0, 1.0]) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn calibration_shrinks_a_stretched_blend_back_to_real_size() {
        use crate::affinity::Prediction;
        use crate::model::Parts;
        // Real distances from the mean are a third of what the look side claims.
        let held: Vec<Held> = [-30.0f32, -6.0, 9.0, 15.0]
            .iter()
            .map(|dev| Held {
                show: 0,
                actual: 70.0 + dev / 3.0,
                dropped: false,
                parts: Parts {
                    taste: Prediction::default(),
                    look: Prediction {
                        dev: *dev,
                        evidence: vec![],
                    },
                    peers: None,
                    crowd: 0.0,
                },
            })
            .collect();
        let w = calibrate(
            &held,
            70.0,
            Weights {
                taste: 0.0,
                look: 1.0,
                peers: 0.0,
                crowd: 0.0,
                scale: 1.0,
            },
        )
        .unwrap();
        assert!((w.scale - 1.0 / 3.0).abs() < 1e-4, "{}", w.scale);
    }

    #[test]
    fn folds_cover_everything_once() {
        let mut counts = [0; 5];
        for id in 0..1000u64 {
            counts[fold_of(id, 5) as usize] += 1;
        }
        assert!(counts.iter().all(|c| *c > 150), "{counts:?}");
    }
}
