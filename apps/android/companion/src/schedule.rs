//! "Next up": what is subscribed or airing, in the order it airs.

use std::collections::{BTreeMap, HashSet};

use crate::api::{Airing, Series, Subscription};
use crate::store::{AniListCache, Cache};

/// What a row leads with.
#[derive(Clone, Debug, PartialEq)]
pub enum Headline {
    /// The next episode still to air.
    Upcoming(Airing),
    /// An episode that has aired and was not on the host's disk at the last sync.
    /// `at` is unknown for an episode the host listed as missing without a time.
    Aired { season: Option<u32>, episode: u32, at: Option<i64> },
    /// Subscribed or releasing, but nobody knows when the next one airs.
    Unscheduled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NextUp {
    /// Stable across refreshes: `s<series id>` or `r<rule name>`.
    pub key: String,
    /// The series for the poster, when the row has one.
    pub series_id: Option<u64>,
    pub title: String,
    /// An anirss rule downloads it.
    pub subscribed: bool,
    /// The rule is switched off in qBittorrent.
    pub paused: bool,
    /// The rule has every episode and the show has finished.
    pub complete: bool,
    pub headline: Headline,
    /// Every aired episode not on disk, oldest first; the headline is the last of them.
    pub missing: Vec<(Option<u32>, u32)>,
    /// The next one to air, on a row that leads with an aired episode.
    pub then: Option<Airing>,
}

impl NextUp {
    /// Aired rows first (they are the past), then by air time, then the unscheduled.
    fn order(&self) -> (u8, i64, String) {
        let title = self.title.to_lowercase();
        match &self.headline {
            Headline::Aired { at, .. } => (0, at.unwrap_or(i64::MIN), title),
            Headline::Upcoming(a) => (0, a.at, title),
            Headline::Unscheduled => (1, 0, title),
        }
    }
}

fn season_of(a: &Airing) -> u32 {
    a.season.unwrap_or(1)
}

/// The schedule to count down through: the host's, unless AniList answered after the
/// host last did. AniList lists only what is still to air, so a host entry before the
/// first of those has aired since, and a host entry at or past it gives way to AniList's
/// time for the same episode (a delay moves it).
pub fn schedule_for(series: &Series, synced_at: i64, anilist: &AniListCache) -> Vec<Airing> {
    let mut host = series.upcoming.clone();
    if let Some(next) = &series.next_airing
        && !host.iter().any(|a| a.episode == next.episode && season_of(a) == season_of(next))
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

fn on_disk(series: &Series) -> HashSet<(u32, u32)> {
    series
        .episodes
        .iter()
        .filter(|e| e.number.fract() == 0.0 && e.number >= 0.0)
        .map(|e| (e.season.unwrap_or(1), e.number as u32))
        .collect()
}

fn is_airing(series: &Series, schedule: &[Airing]) -> bool {
    series.status.as_deref() == Some("Releasing") || !schedule.is_empty()
}

fn row(
    key: String,
    series_id: Option<u64>,
    title: String,
    subscription: Option<&Subscription>,
    schedule: &[Airing],
    disk: &HashSet<(u32, u32)>,
    now: i64,
) -> NextUp {
    // On the schedule, aired by now, and not on disk at the last sync. The host lists only
    // what is still to come, so these aired after it made the snapshot.
    let mut missing: BTreeMap<(u32, u32), Option<i64>> = BTreeMap::new();
    for a in schedule.iter().filter(|a| a.at <= now) {
        if !disk.contains(&(season_of(a), a.episode)) {
            missing.insert((season_of(a), a.episode), Some(a.at));
        }
    }
    // What the host itself said was missing when it made the snapshot.
    for &episode in subscription.map(|s| s.missing.as_slice()).unwrap_or_default() {
        if !disk.contains(&(1, episode)) {
            let at = schedule.iter().find(|a| season_of(a) == 1 && a.episode == episode).map(|a| a.at);
            missing.entry((1, episode)).or_insert(at);
        }
    }
    let mut missing: Vec<((u32, u32), Option<i64>)> = missing.into_iter().collect();
    missing.sort_by_key(|((s, e), at)| (at.unwrap_or(i64::MIN), *s, *e));

    let next = schedule.iter().find(|a| a.at > now).cloned();
    let (headline, then) = match missing.last() {
        Some(&((season, episode), at)) => {
            let season = (season > 1).then_some(season);
            (Headline::Aired { season, episode, at }, next)
        }
        None => match next {
            Some(a) => (Headline::Upcoming(a), None),
            None => (Headline::Unscheduled, None),
        },
    };
    NextUp {
        key,
        series_id,
        title,
        subscribed: subscription.is_some(),
        paused: subscription.is_some_and(|s| !s.enabled),
        complete: subscription.is_some_and(|s| s.complete),
        headline,
        missing: missing.into_iter().map(|((s, e), _)| ((s > 1).then_some(s), e)).collect(),
        then,
    }
}

/// Every anirss subscription plus every releasing series in the library, one row each,
/// in the order they air.
pub fn next_up(cache: &Cache, anilist: &AniListCache, now: i64) -> Vec<NextUp> {
    let snapshot = &cache.snapshot;
    let mut subscribed: BTreeMap<u64, &Subscription> = BTreeMap::new();
    let mut rows = Vec::new();

    for sub in &snapshot.subscriptions {
        match sub.series_id.filter(|id| snapshot.series.iter().any(|s| s.id == *id)) {
            Some(id) => {
                subscribed.entry(id).or_insert(sub);
            }
            None => {
                let schedule: Vec<Airing> = sub.next_airing.iter().cloned().collect();
                rows.push(row(
                    format!("r{}", sub.name),
                    None,
                    sub.title.clone(),
                    Some(sub),
                    &schedule,
                    &HashSet::new(),
                    now,
                ));
            }
        }
    }

    for series in &snapshot.series {
        let schedule = schedule_for(series, cache.synced_at, anilist);
        let sub = subscribed.get(&series.id).copied();
        if sub.is_none() && !is_airing(series, &schedule) {
            continue;
        }
        rows.push(row(
            format!("s{}", series.id),
            Some(series.id),
            series.title.clone(),
            sub,
            &schedule,
            &on_disk(series),
            now,
        ));
    }

    rows.sort_by_key(NextUp::order);
    rows
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
            serde_json::from_str(include_str!("../../../../host-api/fixtures/snapshot.json")).unwrap();
        Cache {
            address: "banditbox:8790".into(),
            host: "banditbox".into(),
            host_version: "2.1.0".into(),
            synced_at: snapshot.generated_at + 5,
            snapshot,
        }
    }

    fn airing(episode: u32, at: i64) -> Airing {
        Airing { episode, season: None, at }
    }

    #[test]
    fn the_fixture_has_frieren_up_next_and_leaves_finished_shows_out() {
        let cache = fixture();
        let rows = next_up(&cache, &AniListCache::default(), cache.synced_at + 60);
        assert_eq!(rows.len(), 1, "Mushishi and Perfect Blue are finished and not subscribed");
        let frieren = &rows[0];
        assert_eq!(frieren.key, "s12");
        assert!(frieren.subscribed);
        assert_eq!(frieren.headline, Headline::Upcoming(airing(8, 1_792_414_800)));
        assert!(frieren.missing.is_empty());
    }

    #[test]
    fn an_episode_that_airs_while_the_host_is_away_is_marked_aired() {
        let cache = fixture();
        // Two hours after episode 8 aired, with no sync since.
        let now = 1_792_414_800 + 7_200;
        let rows = next_up(&cache, &AniListCache::default(), now);
        let frieren = &rows[0];
        assert_eq!(frieren.headline, Headline::Aired { season: None, episode: 8, at: Some(1_792_414_800) });
        assert_eq!(frieren.then, Some(airing(9, 1_793_019_600)));
        assert_eq!(frieren.missing, vec![(None, 8)]);
    }

    #[test]
    fn rows_run_aired_then_by_air_time_then_unscheduled() {
        let mut cache = fixture();
        let now = cache.synced_at + 60;
        let mut later = cache.snapshot.series[0].clone();
        later.id = 50;
        later.title = "Later Show".into();
        later.status = Some("Releasing".into());
        later.next_airing = Some(airing(3, now + 10 * 86_400));
        later.upcoming = vec![airing(3, now + 10 * 86_400)];
        later.episodes.clear();
        let mut sooner = later.clone();
        sooner.id = 51;
        sooner.title = "Sooner Show".into();
        sooner.next_airing = Some(airing(2, now + 3_600));
        sooner.upcoming = vec![airing(2, now + 3_600)];
        cache.snapshot.series.extend([later, sooner]);
        cache.snapshot.subscriptions.push(Subscription {
            name: "Batch".into(),
            title: "A batch".into(),
            enabled: true,
            ..Default::default()
        });
        cache.snapshot.subscriptions.push(Subscription {
            name: "Old gap".into(),
            title: "Old gap".into(),
            enabled: false,
            series_id: None,
            missing: vec![3],
            ..Default::default()
        });
        let rows = next_up(&cache, &AniListCache::default(), now);
        let keys: Vec<&str> = rows.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(keys, vec!["rOld gap", "s51", "s12", "s50", "rBatch"]);
        assert!(rows[0].paused);
        assert_eq!(rows[0].headline, Headline::Aired { season: None, episode: 3, at: None });
        assert_eq!(rows[4].headline, Headline::Unscheduled);
    }

    #[test]
    fn a_fresher_anilist_answer_moves_a_delayed_episode() {
        let cache = fixture();
        let fetched_at = 1_792_414_800 + 3_600;
        let mut anilist = AniListCache { fetched_at, attempted_at: fetched_at, ..Default::default() };
        // AniList says episode 8 slipped a week, and so did the rest.
        anilist.shows.insert(
            182255,
            Schedule { status: Some("RELEASING".into()), upcoming: vec![airing(8, 1_793_019_600), airing(9, 1_793_624_400)] },
        );
        let rows = next_up(&cache, &anilist, fetched_at + 60);
        assert_eq!(rows[0].headline, Headline::Upcoming(airing(8, 1_793_019_600)));

        // An AniList answer older than the snapshot changes nothing.
        let stale = AniListCache { fetched_at: cache.synced_at - 1, ..anilist.clone() };
        let rows = next_up(&cache, &stale, cache.synced_at + 60);
        assert_eq!(rows[0].headline, Headline::Upcoming(airing(8, 1_792_414_800)));
    }

    #[test]
    fn anilist_keeps_what_aired_before_its_answer() {
        let cache = fixture();
        let fetched_at = 1_792_414_800 + 3_600;
        let mut anilist = AniListCache { fetched_at, ..Default::default() };
        anilist.shows.insert(182255, Schedule { status: None, upcoming: vec![airing(9, 1_793_019_600)] });
        let schedule = schedule_for(&cache.snapshot.series[0], cache.synced_at, &anilist);
        assert_eq!(schedule, vec![airing(8, 1_792_414_800), airing(9, 1_793_019_600)]);
    }

    #[test]
    fn only_airing_shows_are_asked_about() {
        assert_eq!(anilist_ids(&fixture()), vec![182255]);
    }
}
