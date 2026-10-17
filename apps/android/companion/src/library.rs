//! "On disk": what the host has downloaded, and its torrents.

use crate::api::{Kind, Series, Snapshot};
use crate::format::{self, Tone};

#[derive(Clone, Debug, PartialEq)]
pub struct DiskEpisode {
    pub code: String,
    pub watched: bool,
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
            None => out.push((e.season, &e.key, DiskEpisode { code: e.code.clone(), watched: e.watched })),
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
        assert_eq!(rows[0].episodes[6], DiskEpisode { code: "E07".into(), watched: false });
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
