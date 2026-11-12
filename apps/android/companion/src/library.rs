//! The library: every series as a tile, and one series in full.

use crate::api::{Airing, EpisodeVersion, Kind, Series};
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
    /// The host's file for it, for streaming; the first copy when there are two. None
    /// from a host older than streaming.
    pub file: Option<u64>,
    /// Every file of the episode when the host keeps it in more than one release, each
    /// named by its release; empty for one file, or a host older than versions.
    pub versions: Vec<EpisodeVersion>,
}

/// One entry per episode, in the host's order. The host lists every file, so an episode
/// kept in two releases comes twice; it counts as watched when either copy is. Copies are
/// one episode by season and number, a missing season read as the first, the way the host
/// groups their versions.
fn episodes(series: &Series) -> Vec<DiskEpisode> {
    let mut out: Vec<((u32, u64), DiskEpisode)> = Vec::new();
    for e in &series.episodes {
        let key = (e.season.unwrap_or(1), e.number.to_bits());
        match out.iter_mut().find(|(k, _)| *k == key) {
            Some((_, seen)) => {
                seen.watched |= e.watched;
                seen.file = seen.file.or(e.file);
                if seen.versions.is_empty() {
                    seen.versions = e.versions.clone();
                }
            }
            None => out.push((
                key,
                DiskEpisode {
                    season: e.season,
                    number: e.number,
                    code: e.code.clone(),
                    watched: e.watched,
                    title: e.title.clone(),
                    aired_at: e.aired_at,
                    file: e.file,
                    versions: e.versions.clone(),
                },
            )),
        }
    }
    out.into_iter().map(|(_, e)| e).collect()
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
    series
        .progress
        .unwrap_or_else(|| episodes.iter().filter(|e| e.watched).count() as u32)
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
                on_disk: if eps.is_empty() {
                    s.episodes_on_disk as u32
                } else {
                    eps.len() as u32
                },
                subscribed: snapshot
                    .subscriptions
                    .iter()
                    .any(|sub| sub.series_id == Some(s.id)),
            };
            (tile, s.latest_activity_at)
        })
        .collect();
    tiles.sort_by_key(|(t, latest)| {
        let soon = t.next.as_ref().map(|a| a.at).unwrap_or(i64::MAX);
        (
            !t.airing,
            if t.airing { soon } else { -latest },
            t.title.to_lowercase(),
        )
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
    let alt_title = s
        .titles
        .english
        .clone()
        .filter(|t| !t.is_empty() && *t != s.title);
    let subscription = snapshot
        .subscriptions
        .iter()
        .find(|sub| sub.series_id == Some(id))
        .map(|sub| SubscriptionInfo {
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
        upcoming: schedule::schedule_for(s, cache.synced_at, anilist)
            .into_iter()
            .filter(|a| a.at > now)
            .collect(),
        watched: watched_count(s, &episodes),
        total: s.total_episodes,
        episodes,
        subscription,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Snapshot;

    fn fixture() -> Snapshot {
        serde_json::from_str(include_str!("../../../../host-api/fixtures/snapshot.json")).unwrap()
    }

    #[test]
    fn episodes_keep_the_hosts_order_and_fields() {
        let snapshot = fixture();
        let rows = episodes(&snapshot.series[0]);
        assert_eq!(rows.len(), 7);
        assert_eq!(rows[6].code, "E07");
        assert!(!rows[6].watched);
        assert_eq!(rows[6].aired_at, Some(1_791_205_200));
    }

    #[test]
    fn an_episode_kept_twice_is_one_episode() {
        let mut snapshot = fixture();
        let frieren = &mut snapshot.series[0];
        let mut copy = frieren.episodes[6].clone();
        copy.watched = true;
        frieren.episodes.push(copy);
        frieren.episodes_on_disk = 8;
        let rows = episodes(&snapshot.series[0]);
        assert_eq!(rows.len(), 7);
        assert!(rows[6].watched, "the watched copy counts");
    }

    #[test]
    fn an_episode_kept_twice_carries_its_versions_once() {
        let mut snapshot = fixture();
        let frieren = &mut snapshot.series[0];
        let versions = vec![
            EpisodeVersion {
                file: 70,
                label: "Group A, 1080p".into(),
            },
            EpisodeVersion {
                file: 71,
                label: "Group B, 720p".into(),
            },
        ];
        frieren.episodes[6].file = Some(70);
        frieren.episodes[6].versions = versions.clone();
        let mut copy = frieren.episodes[6].clone();
        copy.file = Some(71);
        frieren.episodes.push(copy);
        let rows = episodes(&snapshot.series[0]);
        let seventh = &rows[6];
        assert_eq!(rows.len(), 7);
        assert_eq!(seventh.file, Some(70), "the first copy plays");
        assert_eq!(seventh.versions, versions);
        assert!(rows[0].versions.is_empty());
    }

    #[test]
    fn a_copy_with_no_season_and_one_in_the_first_are_one_episode() {
        let mut snapshot = fixture();
        let frieren = &mut snapshot.series[0];
        frieren.episodes[6].season = None;
        let mut copy = frieren.episodes[6].clone();
        copy.season = Some(1);
        frieren.episodes.push(copy);
        assert_eq!(episodes(&snapshot.series[0]).len(), 7);
    }

    fn cache() -> Cache {
        let snapshot = fixture();
        Cache {
            synced_at: snapshot.generated_at + 5,
            snapshot,
            ..Default::default()
        }
    }

    #[test]
    fn tiles_put_airing_shows_first_then_the_most_recent() {
        let cache = cache();
        let tiles = tiles(&cache, &AniListCache::default(), cache.synced_at + 60);
        let order: Vec<(u64, bool)> = tiles.iter().map(|t| (t.id, t.airing)).collect();
        assert_eq!(order, vec![(12, true), (40, false), (3, false)]);
        assert_eq!(
            (tiles[0].watched, tiles[0].total, tiles[0].on_disk),
            (6, Some(12), 7)
        );
        assert_eq!(tiles[0].next.as_ref().map(|a| a.episode), Some(8));
        assert!(tiles[0].subscribed);
        assert!(tiles[1].movie);
    }

    #[test]
    fn a_series_page_has_facts_episodes_and_its_rule() {
        let cache = cache();
        let page = detail(&cache, &AniListCache::default(), 12, cache.synced_at + 60).unwrap();
        assert_eq!(page.facts, vec!["TV", "Fall 2026", "Airing", "12 episodes"]);
        assert_eq!(
            page.alt_title.as_deref(),
            Some("Frieren: Beyond Journey's End Season 2")
        );
        assert_eq!(page.upcoming.len(), 5);
        assert_eq!(page.episodes.len(), 7);
        assert_eq!(page.subscription.as_ref().map(|s| s.files), Some(7));
        assert!(detail(&cache, &AniListCache::default(), 999, 0).is_none());
    }
}
