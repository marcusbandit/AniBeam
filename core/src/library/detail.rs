//! One series page: the episodes, the extras, the characters and the
//! recommendations. Built off the same `Snapshot` a card is, so a page and
//! the card behind it can never disagree.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::{Connection, params_from_iter};
use serde_json::Value;

use crate::contract::*;
use crate::library::cards::{aired_and_scheduled, next_up};
use crate::library::snapshot::{FileRow, MediaRow, Snapshot};
use crate::library::{labels, titles};
use crate::store::sql::placeholders;
use crate::time;

impl Snapshot {
    /// The series page. Takes the connection because the character images
    /// and the recommendations are one query each, and neither belongs in a
    /// whole-library snapshot.
    pub fn detail(&self, conn: &Connection, id: u64) -> Result<Option<SeriesDetail>, CoreError> {
        let Some(row) = self.row(id) else {
            return Ok(None);
        };
        let card = self.card_of(row);
        let media = row.media.as_ref();
        let now = time::to_secs(self.now);
        let airing = self.airing_of(row);
        let (latest_aired, _) = aired_and_scheduled(airing, now);
        let files = self.files_of(id);
        let episodes = self.episodes_on_disk(id);
        let entry = self.tracker(row);

        let total = media.and_then(|m| m.episodes).filter(|t| *t > 0);
        let on_disk = episodes.len() as u64;
        let (progress_total, estimate) = match total {
            Some(t) => (Some(t), false),
            None if latest_aired > 0 => (Some(latest_aired.max(card.watched.unwrap_or(0))), true),
            None => (Some(on_disk as u32), false),
        };

        let disk: Vec<(u64, f64)> = episodes.iter().map(|f| (f.id, f.number)).collect();
        let next = next_up(&disk, self.last_completed_number(id), card.watched);

        // Files numbered past a known episode count are almost always
        // misnamed, duplicates or stray specials; only the main season is
        // judged, so a multi-season folder is never false-flagged.
        let unmatched: HashSet<u64> = match (row.kind, total) {
            (SeriesKind::Show, Some(t)) => episodes
                .iter()
                .filter(|f| f.season.is_none_or(|s| s <= 1) && f.number > f64::from(t))
                .map(|f| f.id)
                .collect(),
            _ => HashSet::new(),
        };

        let completed_keys: HashSet<&str> = self
            .completed
            .get(&id)
            .map(|rows| rows.iter().map(|(k, _)| k.as_str()).collect())
            .unwrap_or_default();
        let episode = |f: &FileRow| {
            let air = airing.iter().find(|a| f64::from(a.number) == f.number);
            Episode {
                file: f.id,
                number: f.number,
                season: f.season,
                code: labels::episode_code(f.season, f.number),
                title: air.and_then(|a| a.title.clone()),
                air_date: air.and_then(|a| a.aired_at).map(time::from_secs),
                path: f.path.clone(),
                sidecars: f.sidecars.clone(),
                resume: self.resume_of(id, &f.episode_key),
                watched: completed_keys.contains(f.episode_key.as_str())
                    || card.watched.is_some_and(|w| f.number <= f64::from(w)),
                next_up: next == Some(f.id),
            }
        };

        let mut extras: Vec<&FileRow> = files.iter().filter(|f| !f.is_episode).collect();
        extras.sort_by(|a, b| {
            extra_order(a.extra_kind)
                .cmp(&extra_order(b.extra_kind))
                .then_with(|| a.extra_index.unwrap_or(0).cmp(&b.extra_index.unwrap_or(0)))
                .then_with(|| a.label.cmp(&b.label))
                .then_with(|| a.path.cmp(&b.path))
        });

        Ok(Some(SeriesDetail {
            banner: row.banner_path.clone(),
            synopsis: media
                .and_then(|m| m.description.clone())
                .unwrap_or_default(),
            year: media.and_then(|m| m.year),
            studio: media.and_then(|m| m.studio.clone()),
            genres: media.map(|m| m.genres.clone()).unwrap_or_default(),
            tags: media.map(|m| tags_of(&m.tags)).unwrap_or_default(),
            rewatch_count: entry.map(|e| e.repeat).filter(|r| *r > 0),
            site_url: media.and_then(|m| m.site_url.clone()),
            progress: ProgressLine {
                watched: card.watched,
                total: progress_total,
                estimate,
                on_disk,
            },
            next_up: next,
            episodes: episodes
                .iter()
                .filter(|f| !unmatched.contains(&f.id))
                .map(|f| episode(f))
                .collect(),
            extras: extras
                .iter()
                .map(|f| {
                    let kind = f.extra_kind.unwrap_or(ExtraKind::Other);
                    Extra {
                        file: f.id,
                        kind,
                        code: labels::extra_code_with_index(kind, f.extra_index),
                        label: f.label.clone(),
                        path: f.path.clone(),
                        sidecars: f.sidecars.clone(),
                        resume: self.resume_of(id, &f.episode_key),
                    }
                })
                .collect(),
            unmatched_files: episodes
                .iter()
                .filter(|f| unmatched.contains(&f.id))
                .map(|f| episode(f))
                .collect(),
            characters: self.characters(conn, media)?,
            recommendations: self.recommendations(conn, row.anilist_id)?,
            has_graph: row
                .anilist_id
                .is_some_and(|a| self.graph_seeds.contains(&a)),
            card,
        }))
    }

    fn resume_of(&self, series: u64, key: &str) -> Option<ResumePoint> {
        self.resume
            .get(&(series, key.to_string()))
            .map(|(position, duration)| ResumePoint {
                position: *position,
                duration: *duration,
            })
    }

    /// The number of the most recently completed episode, resolved through
    /// the file that carries its key, or the key itself when no file does.
    fn last_completed_number(&self, id: u64) -> Option<f64> {
        let (key, _) = self.completed.get(&id)?.iter().max_by_key(|(_, at)| *at)?;
        self.files_of(id)
            .iter()
            .find(|f| f.is_episode && f.episode_key == *key)
            .map(|f| f.number)
            .or_else(|| key.parse::<f64>().ok())
    }

    /// The top characters, their portraits resolved through the image cache
    /// in one query. A portrait that has not been fetched yet is None.
    fn characters(
        &self,
        conn: &Connection,
        media: Option<&MediaRow>,
    ) -> Result<Vec<Person>, CoreError> {
        let Some(media) = media else {
            return Ok(Vec::new());
        };
        let people = json_array(&media.characters);
        if people.is_empty() {
            return Ok(Vec::new());
        }
        let urls: Vec<String> = people
            .iter()
            .filter_map(|c| {
                c.get("image_url")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect();
        let mut cached: HashMap<String, String> = HashMap::new();
        if !urls.is_empty() {
            let sql = format!(
                "SELECT url, path FROM images WHERE url IN ({})",
                placeholders(urls.len())
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_from_iter(urls.iter()), |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (url, path) = row?;
                cached.insert(url, path);
            }
        }
        Ok(people
            .iter()
            .map(|c| Person {
                name: c
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                image: c
                    .get("image_url")
                    .and_then(Value::as_str)
                    .and_then(|u| cached.get(u))
                    .map(|p| {
                        Path::new(&self.images_dir)
                            .join(p)
                            .to_string_lossy()
                            .into_owned()
                    }),
                role: c
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect())
    }

    /// AniList's recommendations for this series, in AniList's own order,
    /// each one told whether the library owns it and where it sits on the
    /// list.
    fn recommendations(
        &self,
        conn: &Connection,
        anilist_id: Option<u64>,
    ) -> Result<Vec<Recommendation>, CoreError> {
        let Some(anilist_id) = anilist_id else {
            return Ok(Vec::new());
        };
        let mut stmt = conn.prepare(
            "SELECT r.recommended_id, m.title_romaji, m.title_english, i.path,
                    (SELECT s.id FROM series s WHERE s.anilist_id = m.id ORDER BY s.id LIMIT 1),
                    (SELECT ta.status FROM tracker_entries ta WHERE ta.tracker = 'anilist' AND ta.media_id = m.id),
                    (SELECT tm.status FROM tracker_entries tm WHERE tm.tracker = 'mal' AND tm.media_id = m.mal_id)
             FROM recommendations r
             JOIN anilist_media m ON m.id = r.recommended_id
             LEFT JOIN images i ON i.url = m.cover_url
             WHERE r.anilist_id = ?1
             ORDER BY r.rank",
        )?;
        let rows = stmt.query_map([anilist_id as i64], |r| {
            let romaji: Option<String> = r.get(1)?;
            let english: Option<String> = r.get(2)?;
            let anilist_status: Option<String> = r.get(5)?;
            let mal_status: Option<String> = r.get(6)?;
            let (main, other) = match self.main_tracker {
                Tracker::Anilist => (anilist_status, mal_status),
                Tracker::Mal => (mal_status, anilist_status),
            };
            Ok(Recommendation {
                anilist_id: r.get::<_, i64>(0)? as u64,
                title: titles::resolve(self.lang, romaji.as_deref(), english.as_deref(), ""),
                poster: r.get::<_, Option<String>>(3)?.map(|p| {
                    Path::new(&self.images_dir)
                        .join(p)
                        .to_string_lossy()
                        .into_owned()
                }),
                owned: r.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                list_status: main.or(other).and_then(|s| ListStatus::from_column(&s)),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

/// Openings, then endings, previews, specials and the rest.
fn extra_order(kind: Option<ExtraKind>) -> u8 {
    match kind.unwrap_or(ExtraKind::Other) {
        ExtraKind::Op => 0,
        ExtraKind::Ed => 1,
        ExtraKind::Pv => 2,
        ExtraKind::Sp => 3,
        ExtraKind::Other => 4,
    }
}

fn tags_of(raw: &Value) -> Vec<Tag> {
    json_array(raw)
        .iter()
        .map(|t| Tag {
            name: t
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            rank: t.get("rank").and_then(Value::as_u64).unwrap_or(0) as u32,
            spoiler: t.get("spoiler").and_then(Value::as_bool).unwrap_or(false),
            adult: t.get("adult").and_then(Value::as_bool).unwrap_or(false),
        })
        .collect()
}

fn json_array(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}
