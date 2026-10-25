//! One series page: the episodes, the extras, the characters and the
//! recommendations. Built off the same `Snapshot` a card is, so a page and
//! the card behind it can never disagree.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::{Connection, params_from_iter};
use serde_json::Value;

use crate::contract::*;
use crate::library::cards::{next_up, upcoming};
use crate::library::classifier;
use crate::library::snapshot::{FileRow, MediaRow, Snapshot, season_of};
use crate::library::{labels, titles};
use crate::playback::session::MARK_FRACTION;
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
        let (latest_aired, next_scheduled) = self.aired_and_next(row, now);
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

        let disk: Vec<(u64, u32, f64)> = episodes
            .iter()
            .map(|f| (f.id, season_of(f), f.number))
            .collect();
        let next = next_up(&disk, self.last_completed(id), card.watched);

        // Files numbered past a known episode count are almost always
        // misnamed, duplicates or stray specials; `past_the_end` decides
        // which list each file is judged against.
        let unmatched: HashSet<u64> = match (row.kind, total) {
            (SeriesKind::Show, Some(t)) => episodes
                .iter()
                .filter(|f| self.past_the_end(row, f, t))
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
            let air = self.episode_airing(row, f);
            let tracked = card.watched.is_some_and(|w| f.number <= f64::from(w));
            let here = completed_keys.contains(f.episode_key.as_str());
            Episode {
                file: f.id,
                number: f.number,
                season: f.season,
                code: labels::episode_code(f.season, f.number),
                // The provider's title, else whatever the release name
                // carries after its episode token.
                title: air.and_then(|a| a.title.clone()).or_else(|| {
                    Path::new(&f.path)
                        .file_name()
                        .and_then(|n| classifier::file_episode_title(&n.to_string_lossy()))
                }),
                air_date: air.and_then(|a| a.aired_at).map(time::from_secs),
                path: f.path.clone(),
                sidecars: f.sidecars.clone(),
                resume: self.resume_of(id, &f.episode_key),
                stopped_short: self.stopped_short(id, &f.episode_key),
                watched: here || tracked,
                next_up: next == Some(f.id),
                downloading: None,
                cube: if tracked {
                    "w"
                } else if here {
                    "l"
                } else {
                    "d"
                }
                .to_string(),
                fixed: f.fixed,
            }
        };

        // The aired episodes with no file, read off the card's own cube row
        // so the page and every cube row agree on what is missing. Those the
        // tracker covers ('w') are left out: a series watched elsewhere would
        // otherwise list every episode it never had. A record numbering
        // without seasons takes the newest file's, so the rows sort and code
        // like the files around them (S04E09 after S04E08).
        let season = self
            .card_season(row)
            .or_else(|| episodes.last().and_then(|f| f.season));
        let airing = self.airing_of(row);
        // Every aired episode with no file here, a watched one included: watched elsewhere is
        // still not on this device (#156). Its cube stays watched, so it reads as tracked.
        let absent: Vec<AbsentEpisode> = card
            .cubes
            .chars()
            .zip(1u32..)
            .filter(|(c, n)| {
                matches!(c, 'm' | 'g')
                    || (matches!(c, 'w' | 'l') && !card.episodes_here.contains(n))
            })
            .map(|(c, number)| {
                let air = airing.iter().find(|a| a.number == number);
                AbsentEpisode {
                    number,
                    season,
                    code: labels::episode_code(season, f64::from(number)),
                    title: air.and_then(|a| a.title.clone()),
                    air_date: air.and_then(|a| a.aired_at).map(time::from_secs),
                    cube: c.to_string(),
                }
            })
            .collect();
        let released = self.released_of(
            row,
            &episodes,
            total,
            latest_aired,
            next_scheduled,
            card.watched,
        );
        let upcoming = upcoming(airing, released, total, card.status, now)
            .into_iter()
            .map(|(number, at, predicted)| UpcomingEpisode {
                number,
                season,
                code: labels::episode_code(season, f64::from(number)),
                title: airing
                    .iter()
                    .find(|a| a.number == number)
                    .and_then(|a| a.title.clone()),
                at: time::from_secs(at),
                predicted,
            })
            .collect();

        let absent_numbers: Vec<u32> = absent.iter().map(|a| a.number).collect();
        let suggested_fix = suggest(
            row.kind,
            media.and_then(|m| m.format.as_deref()),
            total,
            row.provider == Some(Provider::Tvmaze),
            files,
            &absent_numbers,
        );

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
            absent,
            upcoming,
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
                        index: f.extra_index,
                        fixed: f.fixed,
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
            suggested_fix,
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

    /// Whether the episode's resume point sits before its end line: the
    /// outro's start when the skip windows name one, else the tracker
    /// mark's 85 percent. The same line `playback::session` marks on.
    fn stopped_short(&self, series: u64, key: &str) -> bool {
        let k = (series, key.to_string());
        let Some((position, duration)) = self.resume.get(&k) else {
            return false;
        };
        let line = self
            .outro
            .get(&k)
            .copied()
            .unwrap_or(duration * MARK_FRACTION);
        *position < line
    }

    /// The season and number of the most recently completed episode,
    /// resolved through the file that carries its key, or the key itself
    /// when no file does.
    fn last_completed(&self, id: u64) -> Option<(u32, f64)> {
        let (key, _) = self.completed.get(&id)?.iter().max_by_key(|(_, at)| *at)?;
        self.files_of(id)
            .iter()
            .find(|f| f.is_episode && f.episode_key == *key)
            .map(|f| (season_of(f), f.number))
            .or_else(|| classifier::parse_episode_key(key))
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
                    (SELECT tm.status FROM tracker_entries tm WHERE tm.tracker = 'mal' AND tm.media_id = m.mal_id),
                    m.title_native
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
            let native: Option<String> = r.get(7)?;
            let (main, other) = match self.main_tracker {
                Tracker::Anilist => (anilist_status, mal_status),
                Tracker::Mal => (mal_status, anilist_status),
            };
            Ok(Recommendation {
                anilist_id: r.get::<_, i64>(0)? as u64,
                title: titles::resolve(
                    self.lang,
                    romaji.as_deref(),
                    english.as_deref(),
                    native.as_deref(),
                    "",
                ),
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

/// A misreading of a series' files the page can see on its own (#170), and
/// the fixes that would undo it. The first of these that fits, never touching
/// a file the user already fixed:
///
/// - A one-episode record (a film in its own folder, a lone OVA) holding one
///   file, an episode or a special, that is not read as that episode:
///   `Dai 63-kai ... - Movie` read as episode 63.
/// - Numbered specials where the episodes they number are missing and no
///   regular episode is on disk.
/// - Every episode numbered past the record's count, two or more in one
///   unbroken run no longer than the count: a later season numbered on from
///   the first, 13 to 24 for this record's 1 to 12. A TVmaze match counts
///   within its season and is left out.
fn suggest(
    kind: SeriesKind,
    format: Option<&str>,
    total: Option<u32>,
    tvmaze: bool,
    files: &[FileRow],
    absent: &[u32],
) -> Option<SuggestedFix> {
    if kind != SeriesKind::Show || files.is_empty() || files.iter().any(|f| f.fixed) {
        return None;
    }
    let to_episode = |f: &FileRow, number: u32| FileFix {
        file: f.id,
        reading: Some(FileReading::Episode {
            number: f64::from(number),
            season: None,
        }),
    };

    if let ([only], Some(1)) = (files, total)
        && !(only.is_episode && only.number == 1.0)
        && (only.is_episode || only.extra_kind == Some(ExtraKind::Sp))
    {
        let read = if only.is_episode {
            format!("episode {}", classifier::format_number(only.number))
        } else {
            only.label.to_lowercase()
        };
        let what = if format == Some("MOVIE") {
            "the film"
        } else {
            "its one episode"
        };
        return Some(SuggestedFix {
            text: format!("The one file here reads as {read}. It looks like {what}."),
            fixes: vec![to_episode(only, 1)],
        });
    }

    let specials: Vec<&FileRow> = files
        .iter()
        .filter(|f| !f.is_episode && f.extra_kind == Some(ExtraKind::Sp))
        .collect();
    let indexes: Vec<u32> = specials.iter().filter_map(|f| f.extra_index).collect();
    let distinct: HashSet<u32> = indexes.iter().copied().collect();
    // Only where no regular episode is on disk at all: beside episodes, a
    // numbered special is a special, whatever is missing around it.
    if !specials.is_empty()
        && files.iter().all(|f| !f.is_episode)
        && indexes.len() == specials.len()
        && distinct.len() == indexes.len()
        && indexes.iter().all(|n| absent.contains(n))
    {
        let mut numbers = indexes.clone();
        numbers.sort_unstable();
        return Some(SuggestedFix {
            text: format!(
                "{} look like the missing {} {}.",
                plural(specials.len(), "special", "specials"),
                if numbers.len() == 1 {
                    "episode"
                } else {
                    "episodes"
                },
                runs(&numbers)
            ),
            fixes: specials
                .iter()
                .map(|f| to_episode(f, f.extra_index.unwrap_or(0)))
                .collect(),
        });
    }

    let episodes: Vec<&FileRow> = files.iter().filter(|f| f.is_episode).collect();
    if let (Some(total), false, false) = (total, tvmaze, episodes.is_empty()) {
        let whole: Option<Vec<u32>> = episodes
            .iter()
            .map(|f| {
                (f.number.fract() == 0.0 && f.season.is_none_or(|s| s <= 1))
                    .then_some(f.number as u32)
            })
            .collect();
        if let Some(mut numbers) = whole {
            numbers.sort_unstable();
            let first = numbers[0];
            let last = numbers[numbers.len() - 1];
            let unbroken = numbers.windows(2).all(|w| w[1] == w[0] + 1);
            if numbers.len() > 1 && first > total && unbroken && last - first < total {
                let shift = first - 1;
                return Some(SuggestedFix {
                    text: format!(
                        "Episodes {} look like this record's {}, numbered on from an earlier season.",
                        runs(&numbers),
                        runs(&numbers.iter().map(|n| n - shift).collect::<Vec<_>>())
                    ),
                    fixes: episodes
                        .iter()
                        .map(|f| to_episode(f, f.number as u32 - shift))
                        .collect(),
                });
            }
        }
    }
    None
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// Sorted whole numbers in words: `1 to 6`, `2, 4 and 7`, `1 and 2`, `3`.
fn runs(numbers: &[u32]) -> String {
    match numbers {
        [] => String::new(),
        [only] => only.to_string(),
        [first, _, .., last] if numbers.windows(2).all(|w| w[1] == w[0] + 1) => {
            format!("{first} to {last}")
        }
        [init @ .., last] => format!(
            "{} and {last}",
            init.iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn file(id: u64, name: &str, episode: Option<f64>, special: Option<Option<u32>>) -> FileRow {
        FileRow {
            id,
            series_id: 1,
            path: format!("/lib/Show/{name}"),
            mtime: 0,
            is_episode: episode.is_some(),
            number: episode.unwrap_or(0.0),
            season: None,
            extra_kind: special.map(|_| ExtraKind::Sp),
            extra_index: special.flatten(),
            label: if episode.is_some() {
                name.to_string()
            } else {
                "Special".to_string()
            },
            episode_key: name.to_string(),
            sidecars: Vec::new(),
            fixed: false,
        }
    }

    fn numbers(fix: &SuggestedFix) -> Vec<(u64, f64)> {
        fix.fixes
            .iter()
            .map(|f| match f.reading {
                Some(FileReading::Episode { number, .. }) => (f.file, number),
                _ => panic!("not an episode"),
            })
            .collect()
    }

    #[test]
    fn a_film_folder_whose_one_file_reads_as_another_number() {
        // Girls und Panzer der Film: `Dai 63-kai ... - Movie` read as episode 63.
        let files = [file(7, "GuP - Dai 63-kai - Movie.mkv", Some(63.0), None)];
        let fix = suggest(
            SeriesKind::Show,
            Some("MOVIE"),
            Some(1),
            false,
            &files,
            &[1],
        )
        .unwrap();
        assert_eq!(
            fix.text,
            "The one file here reads as episode 63. It looks like the film."
        );
        assert_eq!(numbers(&fix), vec![(7, 1.0)]);
        // Already episode 1, or a loose film file: nothing to say.
        let files = [file(7, "Film.mkv", Some(1.0), None)];
        assert!(suggest(SeriesKind::Show, Some("MOVIE"), Some(1), false, &files, &[]).is_none());
        let files = [file(7, "Film 2019.mkv", Some(63.0), None)];
        assert!(
            suggest(
                SeriesKind::Movie,
                Some("MOVIE"),
                Some(1),
                false,
                &files,
                &[]
            )
            .is_none()
        );
    }

    fn ovas() -> Vec<FileRow> {
        (1..=6)
            .map(|n| file(n, &format!("OVA {n}.mkv"), None, Some(Some(n as u32))))
            .collect()
    }

    #[test]
    fn numbered_specials_where_their_episodes_are_missing() {
        let all = [1, 2, 3, 4, 5, 6];
        let fix = suggest(SeriesKind::Show, Some("TV"), Some(6), false, &ovas(), &all).unwrap();
        assert_eq!(
            fix.text,
            "6 specials look like the missing episodes 1 to 6."
        );
        assert_eq!(numbers(&fix)[5], (6, 6.0));
        // A special whose episode is on disk, or one with no number, is no such case.
        assert!(
            suggest(
                SeriesKind::Show,
                Some("TV"),
                Some(6),
                false,
                &ovas(),
                &[1, 2, 3]
            )
            .is_none()
        );
        let mut loose = ovas();
        loose[0].extra_index = None;
        assert!(suggest(SeriesKind::Show, Some("TV"), Some(6), false, &loose, &all).is_none());
        // Nor a series the user already fixed.
        let mut fixed = ovas();
        fixed[2].fixed = true;
        assert!(suggest(SeriesKind::Show, Some("TV"), Some(6), false, &fixed, &all).is_none());
    }

    fn numbered_on() -> Vec<FileRow> {
        (13..=24)
            .map(|n| file(n, &format!("Show - {n}.mkv"), Some(n as f64), None))
            .collect()
    }

    #[test]
    fn a_season_numbered_on_from_the_last() {
        let fix = suggest(
            SeriesKind::Show,
            Some("TV"),
            Some(12),
            false,
            &numbered_on(),
            &[],
        )
        .unwrap();
        assert_eq!(
            fix.text,
            "Episodes 13 to 24 look like this record's 1 to 12, numbered on from an earlier season."
        );
        assert_eq!(numbers(&fix)[0], (13, 1.0));
        // A gap, a run longer than the count, a TVmaze match or one file in range: no.
        assert!(
            suggest(
                SeriesKind::Show,
                Some("TV"),
                Some(12),
                true,
                &numbered_on(),
                &[]
            )
            .is_none()
        );
        assert!(
            suggest(
                SeriesKind::Show,
                Some("TV"),
                Some(11),
                false,
                &numbered_on(),
                &[]
            )
            .is_none()
        );
        let mut gap = numbered_on();
        gap.remove(3);
        assert!(suggest(SeriesKind::Show, Some("TV"), Some(12), false, &gap, &[]).is_none());
        let mut mixed = numbered_on();
        mixed.push(file(99, "Show - 01.mkv", Some(1.0), None));
        assert!(suggest(SeriesKind::Show, Some("TV"), Some(12), false, &mixed, &[]).is_none());
    }

    #[test]
    fn no_suggestion_for_what_is_likely_right() {
        // Recap specials beside the series' own episodes, 1 and 2 deleted after watching.
        let mut beside: Vec<FileRow> = (3..=12)
            .map(|n| file(n, &format!("Show - {n}.mkv"), Some(n as f64), None))
            .collect();
        beside.push(file(21, "Show SP1.mkv", None, Some(Some(1))));
        beside.push(file(22, "Show SP2.mkv", None, Some(Some(2))));
        assert!(
            suggest(
                SeriesKind::Show,
                Some("TV"),
                Some(12),
                false,
                &beside,
                &[1, 2]
            )
            .is_none()
        );
        // One file past the end is a bonus episode as likely as a renumbering.
        let lone = [file(13, "Show - 13.mkv", Some(13.0), None)];
        assert!(suggest(SeriesKind::Show, Some("TV"), Some(12), false, &lone, &[]).is_none());
        // A lone opening is not the film.
        let mut opening = file(7, "Show NCOP1.mkv", None, None);
        opening.extra_kind = Some(ExtraKind::Op);
        assert!(
            suggest(
                SeriesKind::Show,
                Some("MOVIE"),
                Some(1),
                false,
                &[opening],
                &[1]
            )
            .is_none()
        );
    }

    #[test]
    fn numbers_in_words() {
        assert_eq!(runs(&[3]), "3");
        assert_eq!(runs(&[1, 2, 3]), "1 to 3");
        assert_eq!(runs(&[2, 4, 7]), "2, 4 and 7");
        assert_eq!(runs(&[1, 2]), "1 and 2");
    }
}
