//! All traits at once. Scoring each tag on its own credits a tag that merely rides along on
//! shows you love for other reasons. A ridge regression over every trait together gives each
//! one only the credit the others don't already explain.

use std::collections::HashMap;

use crate::affinity::VOTE_CAP;
use crate::features::{Side, features};
use crate::signal::Signal;
use crate::types::Catalog;

/// How strongly coefficients are pulled toward zero. Larger means a trait needs more shows
/// behind it before it counts.
pub const LAMBDA: f32 = 30.0;
const PASSES: usize = 40;

/// Each trait's points, 0 to 100 scale: how much it adds to a show's predicted score.
pub fn fit(signals: &[Signal], mean: f32, catalog: &Catalog, lambda: f32) -> HashMap<String, f32> {
    let list: Vec<&Signal> = signals.iter().filter(|s| !s.vote).collect();
    let base = solve(&list, mean, catalog, lambda);
    if list.len() == signals.len() {
        return base;
    }
    // Votes may only move any trait VOTE_CAP points from where the list alone puts it.
    let all: Vec<&Signal> = signals.iter().collect();
    let with = solve(&all, mean, catalog, lambda);
    let mut out = base.clone();
    for (key, c) in with {
        let b = base.get(&key).copied().unwrap_or(0.0);
        out.insert(key, b + (c - b).clamp(-VOTE_CAP, VOTE_CAP));
    }
    out
}

/// Weighted ridge regression by coordinate descent: one trait at a time, against what the
/// others leave unexplained, until it settles.
fn solve(signals: &[&Signal], mean: f32, catalog: &Catalog, lambda: f32) -> HashMap<String, f32> {
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut columns: Vec<Vec<(usize, f32)>> = Vec::new();
    let mut residual = Vec::new();
    let mut weight = Vec::new();
    for s in signals {
        let Some(show) = catalog.get(&s.show) else {
            continue;
        };
        let i = residual.len();
        residual.push(s.value - mean);
        weight.push((s.taste_w + s.look_w) / 2.0);
        for side in [Side::Taste, Side::Look] {
            for f in features(show, side) {
                let next = columns.len();
                let j = *index.entry(f.key).or_insert(next);
                if j == columns.len() {
                    columns.push(Vec::new());
                }
                columns[j].push((i, f.w));
            }
        }
    }
    let mut coef = vec![0.0f32; columns.len()];
    for _ in 0..PASSES {
        for (j, col) in columns.iter().enumerate() {
            let (mut num, mut den) = (0.0f32, lambda);
            for &(i, x) in col {
                num += weight[i] * x * (residual[i] + x * coef[j]);
                den += weight[i] * x * x;
            }
            let new = num / den;
            let step = new - coef[j];
            if step != 0.0 {
                for &(i, x) in col {
                    residual[i] -= x * step;
                }
                coef[j] = new;
            }
        }
    }
    index.into_iter().map(|(k, j)| (k, coef[j])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Credit, Format, LookRole, Show, Tag};

    fn show(id: u64, director: u64, tags: &[&str]) -> Show {
        Show {
            id,
            title: String::new(),
            format: Format::Tv,
            episodes: None,
            minutes: None,
            year: None,
            genres: vec![],
            tags: tags
                .iter()
                .map(|t| Tag {
                    name: (*t).into(),
                    rank: 90,
                    technical: false,
                    spoiler: false,
                })
                .collect(),
            studios: vec![],
            credits: vec![Credit {
                staff: director,
                name: String::new(),
                role: LookRole::Director,
            }],
            community: None,
            popularity: None,
            relations: vec![],
            adult: false,
            status: None,
        }
    }

    fn signal(show: u64, value: f32) -> Signal {
        Signal {
            show,
            value,
            taste_w: 1.0,
            look_w: 1.0,
            dropped: false,
            vote: false,
        }
    }

    #[test]
    fn a_tag_that_only_rides_along_gets_little_credit() {
        // Everything by director 1 is loved; "Rider" happens to sit on those shows, but on
        // shows by others it means nothing.
        let mut catalog = Catalog::new();
        let mut sig = Vec::new();
        for i in 0..8 {
            catalog.insert(i, show(i, 1, &["Rider"]));
            sig.push(signal(i, 95.0));
        }
        for i in 8..16 {
            catalog.insert(i, show(i, 100 + i, &["Rider"]));
            sig.push(signal(i, 70.0));
        }
        for i in 16..24 {
            catalog.insert(i, show(i, 100 + i, &[]));
            sig.push(signal(i, 70.0));
        }
        let mean = sig.iter().map(|s| s.value).sum::<f32>() / sig.len() as f32;
        let c = fit(&sig, mean, &catalog, LAMBDA);
        let director = c["staff:1"];
        let rider = c["tag:Rider"];
        // Eight shows is little evidence at the real pull toward zero, so the director's
        // points are modest; what matters is that it, not the tag, gets them.
        assert!(director > 2.0, "the director carries it: {director}");
        assert!(
            rider.abs() < director / 3.0,
            "the tag rides along: {rider} vs {director}"
        );
    }
}
