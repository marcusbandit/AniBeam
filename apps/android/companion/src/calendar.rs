//! The calendar: every episode that airs, aired or will air in a span of days, from the
//! cached schedules and the episodes on disk.

use chrono::{Datelike, Days, Local, NaiveDate, TimeZone};

use crate::api::Airing;
use crate::schedule::{self, season_of};
use crate::store::{AniListCache, Cache};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Still to air.
    Upcoming,
    /// Aired, and not on the host's disk at the last sync.
    Missing,
    /// On the host's disk.
    OnDisk { watched: bool },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The series, for the poster and the series page; `None` for a subscription with no
    /// library series behind it.
    pub series_id: Option<u64>,
    pub title: String,
    pub season: Option<u32>,
    pub episode: u32,
    pub at: i64,
    pub state: State,
    pub subscribed: bool,
}

/// Every entry with `from <= at < to`, in air order.
pub fn entries(cache: &Cache, anilist: &AniListCache, from: i64, to: i64, now: i64) -> Vec<Entry> {
    let snapshot = &cache.snapshot;
    let subscribed = |id: u64| snapshot.subscriptions.iter().any(|s| s.series_id == Some(id));
    let mut out = Vec::new();

    for series in &snapshot.series {
        let disk = schedule::on_disk_episodes(series);
        let mut seen = std::collections::HashSet::new();
        for a in schedule::schedule_for(series, cache.synced_at, anilist) {
            if a.at < from || a.at >= to {
                continue;
            }
            let key = (season_of(&a), a.episode);
            seen.insert(key);
            let state = match disk.get(&key) {
                Some(&watched) => State::OnDisk { watched },
                None if a.at <= now => State::Missing,
                None => State::Upcoming,
            };
            out.push(entry(Some(series.id), &series.title, &a, state, subscribed(series.id)));
        }
        // What is on disk carries its own air date; the schedule only lists what is to come.
        for e in &series.episodes {
            let Some(at) = e.aired_at.filter(|at| *at >= from && *at < to) else { continue };
            if e.number.fract() != 0.0 || e.number < 0.0 {
                continue;
            }
            let key = (e.season.unwrap_or(1), e.number as u32);
            if !seen.insert(key) {
                continue;
            }
            let a = Airing { episode: key.1, season: e.season, at };
            let watched = disk.get(&key).copied().unwrap_or(e.watched);
            out.push(entry(Some(series.id), &series.title, &a, State::OnDisk { watched }, subscribed(series.id)));
        }
    }
    for sub in snapshot.subscriptions.iter().filter(|s| s.series_id.is_none()) {
        if let Some(a) = sub.next_airing.as_ref().filter(|a| a.at >= from && a.at < to) {
            let state = if a.at <= now { State::Missing } else { State::Upcoming };
            out.push(entry(None, &sub.title, a, state, true));
        }
    }
    out.sort_by(|a, b| (a.at, &a.title, a.episode).cmp(&(b.at, &b.title, b.episode)));
    out
}

fn entry(series_id: Option<u64>, title: &str, a: &Airing, state: State, subscribed: bool) -> Entry {
    Entry {
        series_id,
        title: title.to_string(),
        season: a.season.filter(|s| *s > 1),
        episode: a.episode,
        at: a.at,
        state,
        subscribed,
    }
}

/// The Monday of the week `weeks` away from the one `now` falls in, in `zone`.
pub fn week_start_in<Tz: TimeZone>(now: i64, weeks: i64, zone: &Tz) -> NaiveDate {
    let today = zone.timestamp_opt(now, 0).single().map(|t| t.date_naive()).unwrap_or_default();
    let monday = today - Days::new(u64::from(today.weekday().num_days_from_monday()));
    if weeks >= 0 {
        monday + Days::new(7 * weeks as u64)
    } else {
        monday - Days::new(7 * weeks.unsigned_abs())
    }
}

/// Local midnight at the start of `date`, as Unix seconds. Across a DST change the
/// earlier of two midnights counts.
pub fn midnight_in<Tz: TimeZone>(date: NaiveDate, zone: &Tz) -> i64 {
    let naive = date.and_hms_opt(0, 0, 0).expect("midnight exists");
    zone.from_local_datetime(&naive)
        .earliest()
        .map(|t| t.timestamp())
        .unwrap_or_else(|| naive.and_utc().timestamp())
}

pub fn today_in<Tz: TimeZone>(now: i64, zone: &Tz) -> NaiveDate {
    zone.timestamp_opt(now, 0).single().map(|t| t.date_naive()).unwrap_or_default()
}

pub fn week_start(now: i64, weeks: i64) -> NaiveDate {
    week_start_in(now, weeks, &Local)
}

pub fn midnight(date: NaiveDate) -> i64 {
    midnight_in(date, &Local)
}

pub fn today(now: i64) -> NaiveDate {
    today_in(now, &Local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Snapshot;
    use chrono::FixedOffset;

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

    #[test]
    fn a_week_runs_monday_to_monday_in_the_phones_zone() {
        let cet = FixedOffset::east_opt(2 * 3600).unwrap();
        // Saturday 2026-10-17 15:40 CEST.
        let now = 1_792_244_400;
        assert_eq!(week_start_in(now, 0, &cet), NaiveDate::from_ymd_opt(2026, 10, 12).unwrap());
        assert_eq!(week_start_in(now, 1, &cet), NaiveDate::from_ymd_opt(2026, 10, 19).unwrap());
        assert_eq!(week_start_in(now, -1, &cet), NaiveDate::from_ymd_opt(2026, 10, 5).unwrap());
        assert_eq!(midnight_in(NaiveDate::from_ymd_opt(2026, 10, 17).unwrap(), &cet), now - (15 * 3600 + 40 * 60));
        assert_eq!(today_in(now, &cet), NaiveDate::from_ymd_opt(2026, 10, 17).unwrap());
    }

    #[test]
    fn the_calendar_holds_whats_on_disk_and_whats_to_come() {
        let cache = fixture();
        let now = cache.synced_at + 60;
        let all = entries(&cache, &AniListCache::default(), 0, i64::MAX, now);
        // Frieren: 7 on disk with air dates, then 8 to 12 scheduled. Mushishi and Perfect
        // Blue have no air dates on their files.
        assert_eq!(all.len(), 12);
        assert_eq!(all[0].state, State::OnDisk { watched: true });
        assert_eq!(all[6].state, State::OnDisk { watched: false });
        assert_eq!((all[7].episode, all[7].state), (8, State::Upcoming));
        assert!(all.iter().all(|e| e.series_id == Some(12) && e.subscribed));
    }

    #[test]
    fn a_span_cuts_and_a_passed_airing_is_missing() {
        let cache = fixture();
        // Episode 8 airs at 1_792_414_800; look at that day only, an hour after.
        let rows = entries(&cache, &AniListCache::default(), 1_792_414_800 - 3600, 1_792_414_800 + 3600, 1_792_414_800 + 60);
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].episode, rows[0].state), (8, State::Missing));
    }
}
