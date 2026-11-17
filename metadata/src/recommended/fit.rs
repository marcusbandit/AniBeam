//! The fit: the models learnt from the user's list, every candidate ranked,
//! and each pick's evidence. Pure, and quick: a list of hundreds fits in
//! well under a second, so a run refits from scratch and keeps nothing of
//! the model itself.

use std::collections::HashSet;

use anibeam_recs::eval::evaluate;
use anibeam_recs::model::{Fit, Model, Weights};
use anibeam_recs::recommend::{Entries, Mood, Query, recommend};
use anibeam_recs::signal::Signal;
use anibeam_recs::why::Index;
use anibeam_recs::{Catalog, Neighbor, ShowId};
use serde::{Deserialize, Serialize};

use super::mine::Mine;

/// Fewer rated shows than this and holding a fifth of them out says little,
/// so the blend keeps its default weights.
const TUNE_FROM: usize = 30;
/// How many ranked shows a fit keeps.
pub const KEPT: usize = 300;

/// One of the user's shows a pick is close to, and their score of it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Near {
    pub id: ShowId,
    /// Out of 10.
    pub score: f32,
}

/// What a fit stores per pick beside its rank.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Why {
    pub near: Vec<Near>,
    /// How many spoiler tags the user loves are behind the pick. Counted,
    /// never named.
    pub hidden: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ranked {
    pub id: ShowId,
    pub forecast: u8,
    /// 0 to 100.
    pub predicted: f32,
    pub why: Why,
}

/// The blend's weights and how traits are scored, tuned on the user's own
/// list by holding a fifth of it out at a time.
fn tuned(
    signals: &[Signal],
    mean: f32,
    catalog: &Catalog,
    neighbours: &[Neighbor],
) -> (Weights, Fit) {
    if signals.iter().filter(|s| !s.vote).count() < TUNE_FROM {
        return (Weights::default(), Fit::Mean);
    }
    let report = evaluate(signals, mean, catalog, neighbours, 5);
    (report.best, report.fit)
}

/// Every candidate the user has yet to watch, best first: shows that start
/// something, and the next season of something finished. A show only
/// AniList's "if you liked this" put forward, or one the user chose
/// already (on the list or in the library), may rank without anyone with
/// the user's taste having rated it.
pub fn rank(
    mine: &Mine,
    signals: &[Signal],
    mean: f32,
    catalog: &Catalog,
    neighbours: &[Neighbor],
    trusted: &[ShowId],
) -> Vec<Ranked> {
    let (weights, how) = tuned(signals, mean, catalog, neighbours);
    let model = Model::fit_with(signals, mean, catalog, neighbours, how);
    let me = mine.for_ranking(|id| catalog.get(&id).and_then(|s| s.episodes));
    let mut trust: HashSet<ShowId> = trusted.iter().copied().collect();
    trust.extend(mine.owned.keys());
    trust.extend(
        me.iter()
            .filter(|e| e.status == anibeam_recs::Status::Planning)
            .map(|e| e.show),
    );
    let query = |entries| Query {
        mood: Mood::Any,
        max_minutes: None,
        top: KEPT,
        planning: true,
        max_popularity: None,
        exclude: mine.hidden.iter().copied().collect(),
        entries,
        trusted: trust.clone(),
    };
    let mut picks = recommend(
        &model,
        &weights,
        &me,
        signals,
        catalog,
        &query(Entries::First),
    );
    picks.extend(recommend(
        &model,
        &weights,
        &me,
        signals,
        catalog,
        &query(Entries::NextSeason),
    ));
    picks.sort_by(|a, b| {
        b.predicted
            .total_cmp(&a.predicted)
            .then(a.show.cmp(&b.show))
    });
    let mut seen = HashSet::new();
    picks.retain(|p| seen.insert(p.show));
    picks.truncate(KEPT);

    let index = Index::new(signals, mean, catalog);
    picks
        .iter()
        .filter_map(|p| {
            let why = index.why(&model, &weights, catalog, catalog.get(&p.show)?);
            Some(Ranked {
                id: p.show,
                forecast: why.match_pct,
                predicted: p.predicted,
                why: Why {
                    near: why
                        .near
                        .iter()
                        .map(|n| Near {
                            id: n.id,
                            score: n.score,
                        })
                        .collect(),
                    hidden: why.hidden.map_or(0, |h| h.count),
                },
            })
        })
        .collect()
}
