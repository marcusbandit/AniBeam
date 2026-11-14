//! `ActOnSeries` (#208): one action over several picked series at once.

use std::collections::HashSet;

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{started, started_or_ok};
use crate::library::{reads, scan};
use crate::metadata::automatch;
use crate::trackers::writes::{self, ManyWrite};

/// Every id is checked first, so a pick that is gone fails the whole call
/// before anything is written.
pub(crate) fn act_on_series(
    core: &Core,
    series: Vec<u64>,
    action: SeriesAction,
) -> Result<Reply, CoreError> {
    let mut seen = HashSet::with_capacity(series.len());
    let picks: Vec<u64> = series.into_iter().filter(|id| seen.insert(*id)).collect();
    if picks.is_empty() {
        return Err(CoreError::invalid(
            "series",
            crate::i18n::tr("core-core-nothing-picked", &[]),
        ));
    }
    // One read for every id, however many were picked
    let there: HashSet<u64> = core.store.read(|c| {
        let sql = format!(
            "SELECT id FROM series WHERE id IN ({})",
            crate::store::sql::placeholders(picks.len())
        );
        let mut stmt = c.prepare(&sql)?;
        let ids = stmt
            .query_map(
                rusqlite::params_from_iter(picks.iter().map(|id| *id as i64)),
                |r| r.get::<_, i64>(0),
            )?
            .map(|id| id.map(|id| id as u64))
            .collect::<Result<_, _>>()?;
        Ok(ids)
    })?;
    if let Some(&id) = picks.iter().find(|id| !there.contains(id)) {
        return Err(CoreError::NotFound {
            what: Entity::Series,
            id,
        });
    }
    match action {
        SeriesAction::Hide { hidden } => reads::set_hidden_many(core, &picks, hidden),
        SeriesAction::Rescan => scan::rescan_many(core, &picks),
        // Every pick kept on this device is Ok with no job (#213)
        SeriesAction::Score { score } => {
            writes::start_many(core, &picks, ManyWrite::Score(score)).map(started_or_ok)
        }
        SeriesAction::ListStatus { status } => {
            writes::start_many(core, &picks, ManyWrite::Status(status)).map(started_or_ok)
        }
        SeriesAction::FindMatch => Ok(started(automatch::start_for(&core.owner()?, picks))),
    }
}
