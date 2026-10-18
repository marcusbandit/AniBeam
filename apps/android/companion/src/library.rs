//! The library: every series as a tile, one series in full, what is on disk, and the
//! torrents.

use crate::api::{Airing, Kind, Series, Snapshot};
use crate::format::{self, Tone};
use crate::schedule;
use crate::store::{AniListCache, Cache};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiskEpisode {
    pub season: Option<u32>,
    pub number: f64,
    pub code: String,
    pub watched: bool,
    pub title: Option<String>,
    pub aired_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiskSeries {
    pub id: u64,
    pub title: String,
    /// "7 of 12 on disk, 6 watched".
    pub summary: String,
    pub episodes: Vec<DiskEpisode>,
}

/// One entry per episode, in the host's order. The host lists every file, so an episode
/// kept in two releases comes twice; it counts as watched when either copy is.
fn episodes(series: &Series) -> Vec<DiskEpisode> {
    let mut out: Vec<(Option<u32>, &str, DiskEpisode)> = Vec::new();
    for e in &series.episodes {
        match out.iter_mut().find(|(season, key, _)| *season == e.season && *key == e.key) {
            Some((_, _, seen)) => seen.watched |= e.watched,
            None => out.push((
                e.season,
                &e.key,
                DiskEpisode {
                    season: e.season,
                    number: e.number,
                    code: e.code.clone(),
                    watched: e.watched,
                    title: e.title.clone(),
                    aired_at: e.aired_at,
                },
            )),
        }
    }
    out.into_iter().map(|(_, _, e)| e).collect()
}

fn summary(series: &Series, episodes: &[DiskEpisode]) -> String {
    let watched = episodes.iter().filter(|e| e.watched).count();
    // The host's own count is of files; without a file list it is all there is.
    let on_disk = if episodes.is_empty() { series.episodes_on_disk } else { episodes.len() as u64 };
    if series.kind == Kind::Movie {
        return if watched > 0 { "On disk, watched".into() } else { "On disk, not watched yet".into() };
    }
    let have = match series.total_episodes {
        Some(total) if u64::from(total) > on_disk => format!("{on_disk} of {total} on disk"),
        _ => format!("{on_disk} on disk"),
    };
    let seen = if watched == 0 {
        "none watched".to_string()
    } else if watched == episodes.len() {
        "all watched".to_string()
    } else {
        format!("{watched} watched")
    };
    format!("{have}, {seen}")
}

/// Every series with something on disk, the most recently active first.
pub fn on_disk(snapshot: &Snapshot) -> Vec<DiskSeries> {
    let mut series: Vec<&Series> = snapshot
        .series
        .iter()
        .filter(|s| s.episodes_on_disk > 0 || !s.episodes.is_empty())
        .collect();
    series.sort_by_key(|s| std::cmp::Reverse(s.latest_activity_at));
    series
        .into_iter()
        .map(|s| {
            let episodes = episodes(s);
            DiskSeries { id: s.id, title: s.title.clone(), summary: summary(s, &episodes), episodes }
        })
        .collect()
}

/// One poster in the library grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub id: u64,
    pub title: String,
    pub movie: bool,
    /// Releasing or with a schedule: it goes in the Airing section.
    pub airing: bool,
    pub next: Option<Airing>,
    /// The tracker's count when there is one, else the watched episodes on disk.
    pub watched: u32,
    pub total: Option<u32>,
    /// Episodes on disk, not files.
    pub on_disk: u32,
    pub subscribed: bool,
}

fn watched_count(series: &Series, episodes: &[DiskEpisode]) -> u32 {
    series.progress.unwrap_or_else(|| episodes.iter().filter(|e| e.watched).count() as u32)
}

/// Every series: the airing ones first, soonest first, then the rest, the most recently
/// active first.
pub fn tiles(cache: &Cache, anilist: &AniListCache, now: i64) -> Vec<Tile> {
    let snapshot = &cache.snapshot;
    let mut tiles: Vec<(Tile, i64)> = snapshot
        .series
        .iter()
        .map(|s| {
            let plan = schedule::schedule_for(s, cache.synced_at, anilist);
            let next = plan.into_iter().find(|a| a.at > now);
            let eps = episodes(s);
            let tile = Tile {
                id: s.id,
                title: s.title.clone(),
                movie: s.kind == Kind::Movie,
                airing: next.is_some() || s.status.as_deref() == Some("Releasing"),
                next,
                watched: watched_count(s, &eps),
                total: s.total_episodes,
                on_disk: if eps.is_empty() { s.episodes_on_disk as u32 } else { eps.len() as u32 },
                subscribed: snapshot.subscriptions.iter().any(|sub| sub.series_id == Some(s.id)),
            };
            (tile, s.latest_activity_at)
        })
        .collect();
    tiles.sort_by_key(|(t, latest)| {
        let soon = t.next.as_ref().map(|a| a.at).unwrap_or(i64::MAX);
        (!t.airing, if t.airing { soon } else { -latest }, t.title.to_lowercase())
    });
    tiles.into_iter().map(|(t, _)| t).collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubscriptionInfo {
    pub name: String,
    pub query: String,
    pub enabled: bool,
    pub complete: bool,
    pub files: u64,
    /// Aired episodes the rule has not found, as the host saw it.
    pub missing: Vec<u32>,
}

/// One series in full, for its page.
#[derive(Clone, Debug, PartialEq)]
pub struct Detail {
    pub id: u64,
    pub title: String,
    /// The English title when it differs from the shown one.
    pub alt_title: Option<String>,
    /// "TV", "2026", "Fall 2026", "Airing", "12 episodes".
    pub facts: Vec<String>,
    /// Still to air, soonest first.
    pub upcoming: Vec<Airing>,
    pub watched: u32,
    pub total: Option<u32>,
    pub episodes: Vec<DiskEpisode>,
    pub subscription: Option<SubscriptionInfo>,
}

fn status_word(status: &str) -> &str {
    match status {
        "Releasing" => "Airing",
        "NotYetReleased" => "Not yet aired",
        other => other,
    }
}

fn format_word(format: &str) -> String {
    match format {
        "TV" | "OVA" | "ONA" => format.to_string(),
        "TV_SHORT" => "TV short".to_string(),
        "MOVIE" => "Movie".to_string(),
        "SPECIAL" => "Special".to_string(),
        "MUSIC" => "Music".to_string(),
        other => other.to_string(),
    }
}

pub fn detail(cache: &Cache, anilist: &AniListCache, id: u64, now: i64) -> Option<Detail> {
    let snapshot = &cache.snapshot;
    let s = snapshot.series.iter().find(|s| s.id == id)?;
    let episodes = episodes(s);
    let mut facts = Vec::new();
    facts.extend(s.format.as_deref().map(format_word));
    facts.extend(s.season.clone().or_else(|| s.year.map(|y| y.to_string())));
    facts.extend(s.status.as_deref().map(|st| status_word(st).to_string()));
    if let Some(total) = s.total_episodes.filter(|t| *t > 1) {
        facts.push(format!("{total} episodes"));
    }
    let alt_title = s.titles.english.clone().filter(|t| !t.is_empty() && *t != s.title);
    let subscription = snapshot.subscriptions.iter().find(|sub| sub.series_id == Some(id)).map(|sub| SubscriptionInfo {
        name: sub.name.clone(),
        query: sub.query.clone(),
        enabled: sub.enabled,
        complete: sub.complete,
        files: sub.file_count,
        missing: sub.missing.clone(),
    });
    Some(Detail {
        id,
        title: s.title.clone(),
        alt_title,
        facts,
        upcoming: schedule::schedule_for(s, cache.synced_at, anilist).into_iter().filter(|a| a.at > now).collect(),
        watched: watched_count(s, &episodes),
        total: s.total_episodes,
        episodes,
        subscription,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct TorrentLine {
    pub name: String,
    pub state: &'static str,
    pub tone: Tone,
    /// 0 to 1.
    pub progress: f32,
    pub facts: String,
}

/// The torrents still moving first, then the newest. `None` when the host could not
/// read its torrent client, which is not the same as an empty list.
pub fn torrents(snapshot: &Snapshot, limit: usize) -> Option<Vec<TorrentLine>> {
    let mut all: Vec<_> = snapshot.torrents.as_ref()?.iter().collect();
    all.sort_by_key(|t| {
        let (_, tone) = format::torrent_state(&t.state);
        let moving = t.progress < 1.0 && tone != Tone::Quiet;
        (std::cmp::Reverse(moving), std::cmp::Reverse(t.added_on))
    });
    Some(
        all.into_iter()
            .take(limit)
            .map(|t| {
                let (state, tone) = format::torrent_state(&t.state);
                TorrentLine {
                    name: t.name.clone(),
                    state,
                    tone,
                    progress: t.progress.clamp(0.0, 1.0) as f32,
                    facts: format::torrent_facts(t.progress, t.size, t.download_speed, t.eta),
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Torrent;

    fn fixture() -> Snapshot {
        serde_json::from_str(include_str!("../../../../host-api/fixtures/snapshot.json")).unwrap()
    }

    #[test]
    fn series_on_disk_newest_first_with_a_summary() {
        let rows = on_disk(&fixture());
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, vec!["Sousou no Frieren 2nd Season", "Perfect Blue", "Mushishi"]);
        assert_eq!(rows[0].summary, "7 of 12 on disk, 6 watched");
        assert_eq!(rows[1].summary, "On disk, not watched yet");
        assert_eq!(rows[2].summary, "26 on disk, all watched");
        assert_eq!(rows[0].episodes[6].code, "E07");
        assert!(!rows[0].episodes[6].watched);
        assert_eq!(rows[0].episodes[6].aired_at, Some(1_791_205_200));
    }

    #[test]
    fn an_episode_kept_twice_is_one_episode() {
        let mut snapshot = fixture();
        let frieren = &mut snapshot.series[0];
        let mut copy = frieren.episodes[6].clone();
        copy.watched = true;
        frieren.episodes.push(copy);
        frieren.episodes_on_disk = 8;
        let rows = on_disk(&snapshot);
        assert_eq!(rows[0].episodes.len(), 7);
        assert!(rows[0].episodes[6].watched, "the watched copy counts");
        assert_eq!(rows[0].summary, "7 of 12 on disk, all watched");
    }

    fn cache() -> Cache {
        let snapshot = fixture();
        Cache { synced_at: snapshot.generated_at + 5, snapshot, ..Default::default() }
    }

    #[test]
    fn tiles_put_airing_shows_first_then_the_most_recent() {
        let cache = cache();
        let tiles = tiles(&cache, &AniListCache::default(), cache.synced_at + 60);
        let order: Vec<(u64, bool)> = tiles.iter().map(|t| (t.id, t.airing)).collect();
        assert_eq!(order, vec![(12, true), (40, false), (3, false)]);
        assert_eq!((tiles[0].watched, tiles[0].total, tiles[0].on_disk), (6, Some(12), 7));
        assert_eq!(tiles[0].next.as_ref().map(|a| a.episode), Some(8));
        assert!(tiles[0].subscribed);
        assert!(tiles[1].movie);
    }

    #[test]
    fn a_series_page_has_facts_episodes_and_its_rule() {
        let cache = cache();
        let page = detail(&cache, &AniListCache::default(), 12, cache.synced_at + 60).unwrap();
        assert_eq!(page.facts, vec!["TV", "Fall 2026", "Airing", "12 episodes"]);
        assert_eq!(page.alt_title.as_deref(), Some("Frieren: Beyond Journey's End Season 2"));
        assert_eq!(page.upcoming.len(), 5);
        assert_eq!(page.episodes.len(), 7);
        assert_eq!(page.subscription.as_ref().map(|s| s.files), Some(7));
        assert!(detail(&cache, &AniListCache::default(), 999, 0).is_none());
    }

    #[test]
    fn moving_torrents_come_first() {
        let mut snapshot = fixture();
        snapshot.torrents.as_mut().unwrap().push(Torrent {
            hash: "b".into(),
            name: "older but downloading".into(),
            state: "downloading".into(),
            progress: 0.5,
            size: 1_000_000_000,
            download_speed: 2_000_000,
            eta: Some(250),
            added_on: 1,
            ..Default::default()
        });
        let lines = torrents(&snapshot, 10).unwrap();
        assert_eq!(lines[0].name, "older but downloading");
        assert_eq!(lines[0].state, "Downloading");
        assert_eq!(lines[1].state, "Seeding");
        assert_eq!(lines[1].facts, "1.4 GB");

        snapshot.torrents = None;
        assert!(torrents(&snapshot, 10).is_none());
    }
}
