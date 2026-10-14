//! Ties a download folder to the library series it fills. anirss and the
//! torrent client speak in save paths; the library speaks in series. A
//! folder links by its exact path first, then by its last component, then
//! by that component with case and everything but letters and digits
//! dropped: a client's save path can name the library folder under another
//! mount (`/srv/media/Anime/X` for `/mnt/media/Anime/X`), and a rule's
//! name can spell the folder a little differently ("Ranma 1 2" for
//! "Ranma 1_2").

use crate::contract::*;
use crate::library::snapshot::Snapshot;
use rusqlite::Connection;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::time::SystemTime;

/// The library's folders, indexed three ways.
pub struct Linker {
    by_path: HashMap<String, u64>,
    by_name: HashMap<String, u64>,
    by_key: HashMap<String, u64>,
}

impl Linker {
    /// Every series still on disk. A missing one is left out: a folder
    /// that is gone cannot be what a torrent is filling.
    pub fn load(conn: &Connection) -> Result<Linker, CoreError> {
        let mut stmt = conn.prepare("SELECT id, path FROM series WHERE missing_since IS NULL")?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, i64>(0)? as u64, r.get::<_, String>(1)?))
        })?;
        let mut pairs = Vec::new();
        for row in rows {
            pairs.push(row?);
        }
        Ok(Linker::from_pairs(pairs))
    }

    pub fn from_pairs(pairs: Vec<(u64, String)>) -> Linker {
        let mut linker = Linker {
            by_path: HashMap::new(),
            by_name: HashMap::new(),
            by_key: HashMap::new(),
        };
        for (id, path) in pairs {
            let path = trim(&path).to_string();
            let name = last(&path).to_string();
            // The first one in wins a clash: two library folders with one
            // name under two sources is ambiguous, and the lower id was
            // there first.
            linker.by_key.entry(key(&name)).or_insert(id);
            linker.by_name.entry(name).or_insert(id);
            linker.by_path.insert(path, id);
        }
        linker
    }

    /// The series a folder fills, or None.
    pub fn find(&self, path: &str) -> Option<u64> {
        let path = trim(path);
        if path.is_empty() {
            return None;
        }
        if let Some(id) = self.by_path.get(path) {
            return Some(*id);
        }
        let name = last(path);
        if let Some(id) = self.by_name.get(name) {
            return Some(*id);
        }
        let k = key(name);
        if k.is_empty() {
            return None;
        }
        self.by_key.get(&k).copied()
    }

    /// A torrent's series: its save folder, else what it wrote there (a
    /// folder torrent saved straight into the base names the show by its
    /// content, a film by its file).
    pub fn find_torrent(&self, t: &Torrent) -> Option<u64> {
        self.find(&t.save_path)
            .or_else(|| self.find(&t.content_path))
    }
}

fn trim(path: &str) -> &str {
    let t = path.trim();
    let t = t.trim_end_matches('/');
    if t.is_empty() && path.trim().starts_with('/') {
        "/"
    } else {
        t
    }
}

fn last(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Letters and digits, lower case. Empty for a name of only punctuation,
/// which then matches nothing.
pub fn key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// The named series as the page draws them, by id. One scoped snapshot
/// load: only these series, their files and their airing.
pub fn shows(
    conn: &Connection,
    images_dir: &Path,
    now: SystemTime,
    ids: &[u64],
) -> Result<HashMap<u64, ShowLink>, CoreError> {
    let mut wanted: Vec<u64> = ids.to_vec();
    wanted.sort_unstable();
    wanted.dedup();
    if wanted.is_empty() {
        return Ok(HashMap::new());
    }
    let snap = Snapshot::load(conn, images_dir, now, Some(&wanted))?;
    let mut out = HashMap::new();
    for id in wanted {
        let Some(card) = snap.card(id) else { continue };
        let numbers: Vec<f64> = snap
            .files
            .get(&id)
            .map(|files| {
                files
                    .iter()
                    .filter(|f| f.is_episode)
                    .map(|f| f.number)
                    .collect()
            })
            .unwrap_or_default();
        out.insert(id, show_of(&card, &numbers));
    }
    Ok(out)
}

/// A card and its episode files' numbers as a `ShowLink`.
pub fn show_of(card: &SeriesCard, numbers: &[f64]) -> ShowLink {
    let distinct: BTreeSet<u64> = numbers
        .iter()
        .filter(|n| n.is_finite() && **n >= 1.0 && n.fract() == 0.0)
        .map(|n| *n as u64)
        .collect();
    let known_total = card
        .total_episodes
        .filter(|t| *t > 0 && !card.total_is_estimate);
    let missing: Vec<u32> = match known_total {
        Some(total) => (1..=total)
            .filter(|n| !distinct.contains(&(*n as u64)))
            .collect(),
        None => Vec::new(),
    };
    let finished = matches!(
        card.status,
        Some(AiringStatus::Finished) | Some(AiringStatus::Cancelled)
    );
    let complete = match card.kind {
        SeriesKind::Movie => !card.missing,
        SeriesKind::Show => finished && known_total.is_some() && missing.is_empty(),
    };
    ShowLink {
        series_id: card.id,
        kind: card.kind,
        title: card.title.clone(),
        path: card.path.clone(),
        poster: card.poster.clone(),
        status: card.status,
        episodes_on_disk: distinct.len() as u64,
        total_episodes: card.total_episodes,
        total_is_estimate: card.total_is_estimate,
        next_airing: card.next_airing.clone(),
        complete,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linker() -> Linker {
        Linker::from_pairs(vec![
            (1, "/mnt/media/Anime/Frieren".into()),
            (2, "/mnt/media/Anime/Ranma 1_2 (2024) 3rd Season".into()),
            (3, "/mnt/media/TV/Avatar Seven Havens/".into()),
            (4, "/mnt/media/Films/Perfect Blue (1997).mkv".into()),
        ])
    }

    #[test]
    fn a_folder_links_by_path_then_name_then_key() {
        let l = linker();
        assert_eq!(l.find("/mnt/media/Anime/Frieren"), Some(1));
        assert_eq!(
            l.find("/mnt/media/Anime/Frieren/"),
            Some(1),
            "a trailing slash is the same folder"
        );
        assert_eq!(
            l.find("/srv/media/Anime/Frieren"),
            Some(1),
            "another mount, the same folder name"
        );
        assert_eq!(
            l.find("/srv/media/Anime/Ranma 1 2 (2024) 3rd Season"),
            Some(2),
            "punctuation aside"
        );
        assert_eq!(l.find("/mnt/media/TV/Avatar Seven Havens"), Some(3));
        assert_eq!(l.find("/mnt/media/Anime"), None, "the base is no show");
        assert_eq!(l.find(""), None);
        assert_eq!(
            l.find("/srv/media/Anime/!!!"),
            None,
            "a key of nothing matches nothing"
        );
    }

    #[test]
    fn a_torrent_falls_back_to_what_it_wrote() {
        let l = linker();
        let mut t = Torrent {
            hash: "h".into(),
            name: "n".into(),
            state: "uploading".into(),
            progress: 1.0,
            size: 0,
            dlspeed: 0,
            upspeed: 0,
            eta: 0,
            num_seeds: 0,
            num_leechs: 0,
            ratio: 0.0,
            save_path: "/mnt/media/Films".into(),
            content_path: "/mnt/media/Films/Perfect Blue (1997).mkv".into(),
            added_on: 0,
            completion_on: 0,
            tags: String::new(),
            category: String::new(),
            series_id: None,
        };
        assert_eq!(l.find_torrent(&t), Some(4));
        t.save_path = "/mnt/media/Anime/Frieren".into();
        assert_eq!(l.find_torrent(&t), Some(1), "the save folder first");
    }

    fn card(
        kind: SeriesKind,
        status: Option<AiringStatus>,
        total: Option<u32>,
        estimate: bool,
    ) -> SeriesCard {
        let mut c: SeriesCard = serde_json::from_value(serde_json::json!({
            "id": 7, "kind": "Show", "path": "/a/Frieren", "title": "Frieren",
            "titles": {"romaji": null, "english": null, "native": null, "folder": "Frieren"},
            "poster": "/img/7.jpg", "format": null, "status": null, "genres": [],
            "hidden": false, "missing": false, "match_info": null,
            "episodes_on_disk": 0, "extras_on_disk": 0, "total_episodes": null,
            "total_is_estimate": false, "code": null, "watched": null,
            "watched_state": "Unknown", "strip": {"watched": 0.0, "aired_unwatched": 0.0, "unknown": 0.0},
            "community_score": null, "my_score": null, "list_status": null, "next_airing": null,
            "last_viewed_at": {"secs_since_epoch": 0, "nanos_since_epoch": 0},
            "latest_activity_at": {"secs_since_epoch": 0, "nanos_since_epoch": 0}
        }))
        .expect("a card");
        c.kind = kind;
        c.status = status;
        c.total_episodes = total;
        c.total_is_estimate = estimate;
        c
    }

    #[test]
    fn complete_means_finished_and_every_number_on_disk() {
        let all: Vec<f64> = (1..=12).map(|n| n as f64).collect();
        let finished = card(
            SeriesKind::Show,
            Some(AiringStatus::Finished),
            Some(12),
            false,
        );
        let s = show_of(&finished, &all);
        assert!(s.complete);
        assert_eq!(s.episodes_on_disk, 12);
        assert!(s.missing.is_empty());

        // a v2 beside its v1 and a recap .5 do not stand in for episode 7
        let mut gap: Vec<f64> = (1..=12).filter(|n| *n != 7).map(|n| n as f64).collect();
        gap.extend([6.0, 6.5]);
        let s = show_of(&finished, &gap);
        assert!(!s.complete);
        assert_eq!(s.missing, vec![7]);
        assert_eq!(s.episodes_on_disk, 11);

        let airing = card(
            SeriesKind::Show,
            Some(AiringStatus::Releasing),
            Some(12),
            false,
        );
        assert!(!show_of(&airing, &all).complete, "still airing");
        let estimated = card(
            SeriesKind::Show,
            Some(AiringStatus::Finished),
            Some(12),
            true,
        );
        assert!(
            !show_of(&estimated, &all).complete,
            "an estimated total proves nothing"
        );
        let unknown = card(SeriesKind::Show, Some(AiringStatus::Finished), None, false);
        assert!(!show_of(&unknown, &all).complete);
        let cancelled = card(
            SeriesKind::Show,
            Some(AiringStatus::Cancelled),
            Some(12),
            false,
        );
        assert!(
            show_of(&cancelled, &all).complete,
            "cancelled is as final as finished"
        );
        let film = card(SeriesKind::Movie, None, None, false);
        assert!(show_of(&film, &[]).complete, "a film on disk is complete");
    }
}
