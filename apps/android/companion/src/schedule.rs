//! The airing schedule for a series, merged from the host and AniList.

use std::collections::HashMap;

use crate::api::{Airing, Series};
use crate::store::{AniListCache, Cache};

pub(crate) fn season_of(a: &Airing) -> u32 {
    a.season.unwrap_or(1)
}

/// The schedule to count down through: the host's, unless AniList answered after the
/// host last did. AniList lists only what is still to air, so a host entry before the
/// first of those has aired since, and a host entry at or past it gives way to AniList's
/// time for the same episode (a delay moves it).
pub fn schedule_for(series: &Series, synced_at: i64, anilist: &AniListCache) -> Vec<Airing> {
    let mut host = series.upcoming.clone();
    if let Some(next) = &series.next_airing
        && !host
            .iter()
            .any(|a| a.episode == next.episode && season_of(a) == season_of(next))
    {
        host.push(next.clone());
    }
    host.sort_by_key(|a| (a.at, season_of(a), a.episode));

    // AniList numbers one entry per season, so it only speaks for single-season schedules.
    let single_season = host.iter().all(|a| season_of(a) == 1);
    let fresh = series
        .anilist_id
        .filter(|_| anilist.fetched_at > synced_at && single_season)
        .and_then(|id| anilist.shows.get(&id));
    let Some(fresh) = fresh else { return host };

    let first_left = fresh.upcoming.first().map(|a| a.episode);
    let mut merged: Vec<Airing> = host
        .into_iter()
        .filter(|a| a.at <= anilist.fetched_at && first_left.is_none_or(|n| a.episode < n))
        .collect();
    merged.extend(fresh.upcoming.iter().cloned());
    merged.sort_by_key(|a| (a.at, a.episode));
    merged
}

/// The whole-numbered episodes on disk by (season, episode), each watched when any of
/// its files is.
pub(crate) fn on_disk_episodes(series: &Series) -> HashMap<(u32, u32), bool> {
    let mut out = HashMap::new();
    for e in series
        .episodes
        .iter()
        .filter(|e| e.number.fract() == 0.0 && e.number >= 0.0)
    {
        *out.entry((e.season.unwrap_or(1), e.number as u32))
            .or_insert(false) |= e.watched;
    }
    out
}

/// The AniList ids worth asking about: shows that are airing or have a schedule.
pub fn anilist_ids(cache: &Cache) -> Vec<u64> {
    let mut ids: Vec<u64> = cache
        .snapshot
        .series
        .iter()
        .filter(|s| {
            s.status.as_deref() == Some("Releasing")
                || s.status.as_deref() == Some("NotYetReleased")
                || !s.upcoming.is_empty()
                || s.next_airing.is_some()
        })
        .filter_map(|s| s.anilist_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anilist::Schedule;
    use crate::api::Snapshot;

    fn fixture() -> Cache {
        let snapshot: Snapshot =
            serde_json::from_str(include_str!("../../../../host-api/fixtures/snapshot.json"))
                .unwrap();
        Cache {
            address: "desktop:8790".into(),
            host: "desktop".into(),
            host_version: "2.1.0".into(),
            synced_at: snapshot.generated_at + 5,
            snapshot,
            peers: Vec::new(),
        }
    }

    fn airing(episode: u32, at: i64) -> Airing {
        Airing {
            episode,
            season: None,
            at,
        }
    }

    #[test]
    fn anilist_keeps_what_aired_before_its_answer() {
        let cache = fixture();
        let fetched_at = 1_792_414_800 + 3_600;
        let mut anilist = AniListCache {
            fetched_at,
            ..Default::default()
        };
        anilist.shows.insert(
            182255,
            Schedule {
                status: None,
                upcoming: vec![airing(9, 1_793_019_600)],
            },
        );
        let schedule = schedule_for(&cache.snapshot.series[0], cache.synced_at, &anilist);
        assert_eq!(
            schedule,
            vec![airing(8, 1_792_414_800), airing(9, 1_793_019_600)]
        );
    }

    #[test]
    fn only_airing_shows_are_asked_about() {
        assert_eq!(anilist_ids(&fixture()), vec![182255]);
    }
}
