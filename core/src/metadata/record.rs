//! A reply becomes rows. Nothing here fetches anything: `build` turns a
//! `Media` and its `Enrichment` into the values one `anilist_media` row
//! holds, `merge_episodes` turns a schedule and Jikan's episode list into
//! `anilist_episodes` rows, and the `write_*` half puts them in a
//! transaction the caller opened.
//!
//! The rules are Electron's, carried over one for one: the padded fuzzy
//! dates and the `extraLarge` cover of `anilistHandler.formatMetadata`,
//! the studio priority and the caps of `main.ts:433-451`, the streaming
//! title parse of `anilistHandler.ts:586-610`, the episode merge of
//! `posterMatch.fetchEpisodeAirDates` plus `main.ts:382-406`, and Jikan's
//! `Episode N` placeholders from `malHandler.ts:96-124`.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use rusqlite::types::Value;
use rusqlite::{Transaction, params, params_from_iter};
use serde::{Deserialize, Serialize};

use crate::contract::CoreError;
use crate::net::anilist::{
    CharacterEdge, Enrichment, FuzzyDate, Media, RelatedNode, Schedule, StreamingEpisode,
    StudioEdge, TagNode,
};
use crate::net::jikan::JikanEpisode;
use crate::store::sql::{as_i64, placeholders};

/// AniList answers with twelve characters and twelve recommendations; the
/// series page shows these many, so these many are what the row keeps.
const CHARACTER_CAP: usize = 10;
const RECOMMENDATION_CAP: usize = 8;

/// The first one to three digit number bounded by non-digits. The prefix
/// in front of it is whatever the streaming site felt like: "Episode ",
/// "S2E", nothing at all.
static STREAM_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[^\d])(\d{1,3})(?:\D|$)").unwrap());

/// What both providers write when they have no title: AniList in a
/// streaming entry, Jikan in its episode list.
static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^Episode\s+\d+$").unwrap());

/// One studio as the row keeps it. The flat `studio` column is one of
/// these names; the JSON keeps the rest, so a later view can show the
/// production committee without another fetch.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Studio {
    pub id: u64,
    pub name: String,
    pub main: bool,
    pub animation: bool,
}

/// AniList's two spoiler flags collapse into one: the reader only ever
/// asks whether a tag is a spoiler, never which kind.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagJson {
    pub name: String,
    pub rank: Option<u32>,
    pub spoiler: bool,
    pub adult: bool,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterJson {
    pub id: u64,
    pub name: Option<String>,
    pub role: Option<String>,
    pub image_url: Option<String>,
}

/// Everything one fetched `anilist_media` row holds, plus the neighbours
/// that row implies. Not a contract record: it never leaves the core.
#[derive(Clone, Debug, Default)]
pub struct MediaWrite {
    pub id: u64,
    pub mal_id: Option<u64>,
    pub media_type: Option<String>,
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub title_native: Option<String>,
    pub synonyms: Vec<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub season: Option<String>,
    pub year: Option<u32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub episodes: Option<u32>,
    pub duration: Option<u32>,
    pub description: Option<String>,
    pub average_score: Option<u32>,
    pub genres: Vec<String>,
    pub studios: Vec<Studio>,
    pub studio: Option<String>,
    pub tags: Vec<TagJson>,
    pub characters: Vec<CharacterJson>,
    pub cover_url: Option<String>,
    pub banner_url: Option<String>,
    pub site_url: Option<String>,
    /// Rank, the target, and AniList's rating. Rank is the position the
    /// `recommendations` row keeps, so the series page draws them in the
    /// order AniList's readers voted for.
    pub recommendations: Vec<(u64, RelatedNode, i64)>,
    /// `relationType` and the other end. Every edge, `CHARACTER` and
    /// `OTHER` included: the crawl is what refuses to walk those, not the
    /// write.
    pub relations: Vec<(String, RelatedNode)>,
}

/// A row the library knows of but has never fetched: a relation's other
/// end, a recommendation's target, a watching entry, an import. `episodes`
/// and `average_score` ride along because the watching query answers with
/// them and nothing else would ever fill them in.
#[derive(Clone, Debug, Default)]
pub struct StubWrite {
    pub id: u64,
    pub mal_id: Option<u64>,
    pub media_type: Option<String>,
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub year: Option<u32>,
    pub cover_url: Option<String>,
    pub site_url: Option<String>,
    pub episodes: Option<u32>,
    pub average_score: Option<u32>,
}

/// One `anilist_episodes` row: the number, whatever title either provider
/// had for it, and when it aired or will air.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EpisodeRow {
    pub number: u32,
    pub title: Option<String>,
    pub aired_at: Option<i64>,
}

/// `YYYY-MM-DD`, month and day defaulting to 01. A date with no year is no
/// date at all: AniList uses one for "announced, no schedule yet".
pub fn format_date(d: Option<&FuzzyDate>) -> Option<String> {
    let d = d?;
    let year = d.year?;
    let month = d.month.filter(|m| *m > 0).unwrap_or(1);
    let day = d.day.filter(|d| *d > 0).unwrap_or(1);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// The studio the detail page names, and the whole list behind it in the
/// same priority order: the edge flagged both main and animation, then any
/// animation studio, then the main one, then the rest. The sort is stable,
/// so AniList's own order survives inside each band.
pub fn pick_studio(edges: &[StudioEdge]) -> (Option<String>, Vec<Studio>) {
    let mut sorted: Vec<&StudioEdge> = edges.iter().collect();
    sorted.sort_by_key(|e| studio_rank(e));
    let list: Vec<Studio> = sorted
        .iter()
        .map(|e| Studio {
            id: e.node.id,
            name: e.node.name.clone(),
            main: e.is_main,
            animation: e.node.is_animation_studio,
        })
        .collect();
    let studio = list
        .iter()
        .find(|s| s.main && s.animation)
        .or_else(|| list.iter().find(|s| s.animation))
        .or_else(|| list.first())
        .map(|s| s.name.clone());
    (studio, list)
}

fn studio_rank(e: &StudioEdge) -> u8 {
    match (e.is_main, e.node.is_animation_studio) {
        (true, true) => 0,
        (_, true) => 1,
        (true, false) => 2,
        _ => 3,
    }
}

/// One streaming entry's episode number and its real title. The shapes are
/// "Episode 1 - Ordinary Person", "1 - Ordinary Person", "S2 Episode 3 -
/// ..." and the bare "Episode 1", which carries no title and is dropped
/// rather than persisted as a placeholder.
pub fn parse_streaming_title(raw: &str) -> Option<(u32, String)> {
    let trimmed = raw.trim();
    let number: u32 = STREAM_NUMBER
        .captures(trimmed)?
        .get(1)?
        .as_str()
        .parse()
        .ok()?;
    if number == 0 {
        return None;
    }
    // Everything up to and including the first " - " is the prefix. With
    // no separator there is no title, only the numbering.
    let separator = trimmed.find(" - ")?;
    let title = trimmed[separator + 3..].trim();
    if title.is_empty() || PLACEHOLDER.is_match(title) {
        return None;
    }
    Some((number, title.to_string()))
}

/// The titles AniList's Watch tab carries, one per episode. Aggregators
/// list the same episode once per streaming site, so the first entry for a
/// number wins.
pub fn streaming_titles(episodes: &[StreamingEpisode]) -> Vec<(u32, String)> {
    let mut seen: HashSet<u32> = HashSet::new();
    let mut out = Vec::new();
    for e in episodes {
        let Some((number, title)) = e.title.as_deref().and_then(parse_streaming_title) else {
            continue;
        };
        if seen.insert(number) {
            out.push((number, title));
        }
    }
    out
}

/// The episode rows for one series. AniList's schedule is the source of
/// dates; Jikan fills the dates AniList had none for and the titles
/// everywhere, its `Episode N` placeholders dropped; AniList's own
/// streaming titles win over Jikan's; the numbers are the union, since the
/// schedule covers the airing batch while Jikan covers the whole run. The
/// next broadcast folds in last and wins its own episode's date: the
/// schedule is one page of 25, so for a long runner the episode actually
/// airing next is missing from every list above it.
pub fn merge_episodes(
    schedule: Option<&Schedule>,
    titles: &[(u32, String)],
    jikan: &[JikanEpisode],
) -> Vec<EpisodeRow> {
    let mut by_number: BTreeMap<u32, EpisodeRow> = BTreeMap::new();

    for node in schedule
        .iter()
        .filter_map(|s| s.airing_schedule.as_ref())
        .flat_map(|s| s.nodes.iter())
    {
        if node.episode == 0 || node.airing_at <= 0 {
            continue;
        }
        by_number.insert(
            node.episode,
            EpisodeRow {
                number: node.episode,
                title: None,
                aired_at: Some(node.airing_at),
            },
        );
    }

    for e in jikan {
        if e.number == 0 {
            continue;
        }
        let row = by_number.entry(e.number).or_insert_with(|| EpisodeRow {
            number: e.number,
            ..Default::default()
        });
        row.title = e
            .title
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty() && !PLACEHOLDER.is_match(t))
            .map(str::to_string);
        if row.aired_at.is_none() {
            row.aired_at = e.aired.as_deref().and_then(parse_aired);
        }
    }

    for (number, title) in titles {
        if *number == 0 {
            continue;
        }
        let row = by_number.entry(*number).or_insert_with(|| EpisodeRow {
            number: *number,
            ..Default::default()
        });
        row.title = Some(title.clone());
    }

    if let Some(next) = schedule.and_then(|s| s.next_airing_episode)
        && next.episode > 0
        && next.airing_at > 0
    {
        let row = by_number.entry(next.episode).or_insert_with(|| EpisodeRow {
            number: next.episode,
            ..Default::default()
        });
        row.aired_at = Some(next.airing_at);
    }

    by_number.into_values().collect()
}

/// Jikan's `aired` is RFC 3339 in practice and a bare date in the older
/// rows. Anything else is no date rather than a failure: one unparseable
/// string must not cost the series its whole episode list.
fn parse_aired(raw: &str) -> Option<i64> {
    let raw = raw.trim();
    if let Ok(t) =
        ::time::OffsetDateTime::parse(raw, &::time::format_description::well_known::Rfc3339)
    {
        return Some(t.unix_timestamp());
    }
    let ymd = ::time::macros::format_description!("[year]-[month]-[day]");
    let date = ::time::Date::parse(raw, ymd).ok()?;
    Some(date.midnight().assume_utc().unix_timestamp())
}

/// The reply as fetched, all four parts under their own key, so a later
/// migration can mine a field this schema has no column for without asking
/// the providers again. The parts arrive as the JSON that came off the
/// wire; a part that was never fetched is null.
pub fn raw_bundle(
    media: Option<&serde_json::Value>,
    enrichment: Option<&serde_json::Value>,
    schedule: Option<&serde_json::Value>,
    jikan: Option<&serde_json::Value>,
) -> serde_json::Value {
    let part = |v: Option<&serde_json::Value>| v.cloned().unwrap_or(serde_json::Value::Null);
    serde_json::json!({
        "media": part(media),
        "enrichment": part(enrichment),
        "schedule": part(schedule),
        "jikan": part(jikan),
    })
}

/// One reply's worth of row. `media` carries the series itself;
/// `enrichment` carries the series page, and everything it alone knows
/// (the studios, the tags, the characters, the neighbours) is empty
/// without it.
pub fn build(media: &Media, enrichment: Option<&Enrichment>) -> MediaWrite {
    let enrichment_title = enrichment.and_then(|e| e.title.as_ref());
    let (studio, studios) = match enrichment.and_then(|e| e.studios.as_ref()) {
        Some(s) => pick_studio(&s.edges),
        None => (None, Vec::new()),
    };
    MediaWrite {
        id: media.id,
        mal_id: media.id_mal.or_else(|| enrichment.and_then(|e| e.id_mal)),
        // Every `Media` query filters on `type: ANIME`, so a reply with no
        // enrichment beside it is an anime by construction. Manga only
        // ever arrives as a relation's other end, through a stub.
        media_type: enrichment
            .and_then(|e| e.type_.clone())
            .or_else(|| Some("ANIME".to_string())),
        title_romaji: media
            .title
            .romaji
            .clone()
            .or_else(|| enrichment_title.and_then(|t| t.romaji.clone())),
        title_english: media
            .title
            .english
            .clone()
            .or_else(|| enrichment_title.and_then(|t| t.english.clone())),
        title_native: media.title.native.clone(),
        synonyms: media.synonyms.clone(),
        format: media
            .format
            .clone()
            .or_else(|| enrichment.and_then(|e| e.format.clone())),
        status: media
            .status
            .clone()
            .or_else(|| enrichment.and_then(|e| e.status.clone())),
        season: media.season.clone(),
        year: year_of(media, enrichment),
        start_date: format_date(media.start_date.as_ref()),
        end_date: format_date(media.end_date.as_ref()),
        episodes: media.episodes,
        duration: media.duration,
        description: media.description.clone(),
        average_score: media.average_score,
        genres: media.genres.clone(),
        studios,
        studio,
        tags: enrichment
            .map(|e| e.tags.iter().map(tag_json).collect())
            .unwrap_or_default(),
        characters: enrichment
            .and_then(|e| e.characters.as_ref())
            .map(|c| {
                c.edges
                    .iter()
                    .take(CHARACTER_CAP)
                    .map(character_json)
                    .collect()
            })
            .unwrap_or_default(),
        cover_url: media
            .cover_image
            .as_ref()
            .and_then(|c| c.extra_large.clone().or_else(|| c.large.clone()))
            .or_else(|| {
                enrichment
                    .and_then(|e| e.cover_image.as_ref())
                    .and_then(|c| c.large.clone())
            }),
        banner_url: media.banner_image.clone(),
        site_url: enrichment.and_then(|e| e.site_url.clone()),
        recommendations: top_recommendations(enrichment),
        relations: enrichment
            .and_then(|e| e.relations.as_ref())
            .map(|r| {
                r.edges
                    .iter()
                    .filter(|e| e.node.id != 0)
                    .map(|e| (e.relation_type.clone(), e.node.clone()))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// `seasonYear`, else the year the run started.
fn year_of(media: &Media, enrichment: Option<&Enrichment>) -> Option<u32> {
    media
        .season_year
        .or_else(|| media.start_date.as_ref().and_then(|d| d.year))
        .or_else(|| enrichment.and_then(|e| e.season_year))
        .or_else(|| {
            enrichment
                .and_then(|e| e.start_date.as_ref())
                .and_then(|d| d.year)
        })
}

fn tag_json(t: &TagNode) -> TagJson {
    TagJson {
        name: t.name.clone(),
        rank: t.rank,
        spoiler: t.is_media_spoiler || t.is_general_spoiler,
        adult: t.is_adult,
        category: t.category.clone(),
    }
}

/// AniList sorts its characters by role then relevance, so the first rows
/// are the ones worth keeping. The portrait is the large one, the medium
/// where there is no large.
fn character_json(e: &CharacterEdge) -> CharacterJson {
    CharacterJson {
        id: e.node.id,
        name: e.node.name.as_ref().and_then(|n| n.full.clone()),
        role: e.role.clone(),
        image_url: e
            .node
            .image
            .as_ref()
            .and_then(|i| i.large.clone().or_else(|| i.medium.clone())),
    }
}

/// The eight AniList's readers rated highest. A recommendation whose
/// target AniList has since deleted arrives with a null media and is
/// dropped rather than stored as an id nothing answers.
fn top_recommendations(enrichment: Option<&Enrichment>) -> Vec<(u64, RelatedNode, i64)> {
    let Some(edges) = enrichment.and_then(|e| e.recommendations.as_ref()) else {
        return Vec::new();
    };
    let mut picks: Vec<(RelatedNode, i64)> = edges
        .edges
        .iter()
        .filter_map(|e| {
            e.node
                .media_recommendation
                .as_ref()
                .map(|m| (m.clone(), e.node.rating.unwrap_or(0)))
        })
        .filter(|(m, _)| m.id != 0)
        .collect();
    // The query already asks for RATING_DESC; sorting again costs nothing
    // and keeps the top eight right whatever order the reply arrived in.
    // The sort is stable, so equal ratings keep AniList's own order.
    picks.sort_by_key(|(_, rating)| Reverse(*rating));
    let mut seen: HashSet<u64> = HashSet::new();
    let mut out = Vec::new();
    let mut rank: u64 = 0;
    for (node, rating) in picks {
        if !seen.insert(node.id) {
            continue;
        }
        out.push((rank, node, rating));
        rank += 1;
        if out.len() == RECOMMENDATION_CAP {
            break;
        }
    }
    out
}

/// Every picture the row names, in the order a series page draws them.
/// Task 16's fill walks this list.
pub fn image_urls(w: &MediaWrite) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    urls.extend(w.cover_url.clone());
    urls.extend(w.banner_url.clone());
    for c in &w.characters {
        urls.extend(c.image_url.clone());
    }
    for (_, node, _) in &w.recommendations {
        urls.extend(node.cover_image.as_ref().and_then(|c| c.large.clone()));
    }
    urls
}

/// `relations_fetched_at` and `raw` ride along with the rest: a row this
/// wrote is a row whose edges are known, so the crawl has nothing left to
/// owe it and any deferral it was carrying is spent.
const MEDIA_UPSERT: &str = "INSERT INTO anilist_media (
        id, mal_id, media_type, title_romaji, title_english, title_native, synonyms, format, status, season,
        year, start_date, end_date, episodes, duration, description, average_score, genres, studios, studio,
        tags, characters, cover_url, banner_url, site_url, fetched_at, raw, relations_fetched_at
     ) VALUES (
        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
        ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
        ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?26
     )
     ON CONFLICT(id) DO UPDATE SET
        mal_id = excluded.mal_id,
        media_type = excluded.media_type,
        title_romaji = excluded.title_romaji,
        title_english = excluded.title_english,
        title_native = excluded.title_native,
        synonyms = excluded.synonyms,
        format = excluded.format,
        status = excluded.status,
        season = excluded.season,
        year = excluded.year,
        start_date = excluded.start_date,
        end_date = excluded.end_date,
        episodes = excluded.episodes,
        duration = excluded.duration,
        description = excluded.description,
        average_score = excluded.average_score,
        genres = excluded.genres,
        studios = excluded.studios,
        studio = excluded.studio,
        tags = excluded.tags,
        characters = excluded.characters,
        cover_url = excluded.cover_url,
        banner_url = excluded.banner_url,
        site_url = excluded.site_url,
        fetched_at = excluded.fetched_at,
        raw = excluded.raw,
        relations_fetched_at = ?26,
        crawl_deferred_until = NULL";

/// A stub only ever fills blanks. The row it lands on may be a fetched
/// one, and a neighbour's thin copy of a title must never replace it, nor
/// null a column the fetch had filled.
const STUB_UPSERT: &str = "INSERT INTO anilist_media (
        id, mal_id, media_type, title_romaji, title_english, format, status, year, cover_url, site_url,
        episodes, average_score
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
     ON CONFLICT(id) DO UPDATE SET
        mal_id = coalesce(anilist_media.mal_id, excluded.mal_id),
        media_type = coalesce(anilist_media.media_type, excluded.media_type),
        title_romaji = coalesce(anilist_media.title_romaji, excluded.title_romaji),
        title_english = coalesce(anilist_media.title_english, excluded.title_english),
        format = coalesce(anilist_media.format, excluded.format),
        status = coalesce(anilist_media.status, excluded.status),
        year = coalesce(anilist_media.year, excluded.year),
        cover_url = coalesce(anilist_media.cover_url, excluded.cover_url),
        site_url = coalesce(anilist_media.site_url, excluded.site_url),
        episodes = coalesce(anilist_media.episodes, excluded.episodes),
        average_score = coalesce(anilist_media.average_score, excluded.average_score)";

/// The whole row, its neighbours as stubs, and its two edge tables
/// replaced. One call, inside the caller's transaction: a half-written
/// series would draw a page with recommendations and no relations.
pub fn write_media(
    tx: &Transaction,
    w: &MediaWrite,
    raw: &serde_json::Value,
    now: i64,
) -> Result<(), CoreError> {
    let id = as_i64(w.id);
    tx.execute(
        MEDIA_UPSERT,
        params![
            id,
            w.mal_id.map(as_i64),
            w.media_type,
            w.title_romaji,
            w.title_english,
            w.title_native,
            serde_json::to_string(&w.synonyms)?,
            w.format,
            w.status,
            w.season,
            w.year.map(i64::from),
            w.start_date,
            w.end_date,
            w.episodes.map(i64::from),
            w.duration.map(i64::from),
            w.description,
            w.average_score.map(i64::from),
            serde_json::to_string(&w.genres)?,
            serde_json::to_string(&w.studios)?,
            w.studio,
            serde_json::to_string(&w.tags)?,
            serde_json::to_string(&w.characters)?,
            w.cover_url,
            w.banner_url,
            w.site_url,
            now,
            raw.to_string(),
        ],
    )?;

    // Both edge tables reference `anilist_media`, so every neighbour needs
    // its row before the edge can point at it.
    for node in w
        .recommendations
        .iter()
        .map(|(_, n, _)| n)
        .chain(w.relations.iter().map(|(_, n)| n))
    {
        if node.id != w.id {
            write_stub(tx, &stub_from_node(node))?;
        }
    }

    tx.execute(
        "DELETE FROM recommendations WHERE anilist_id = ?1",
        params![id],
    )?;
    for (rank, node, rating) in &w.recommendations {
        tx.execute(
            "INSERT OR REPLACE INTO recommendations (anilist_id, recommended_id, rank, rating) VALUES (?1, ?2, ?3, ?4)",
            params![id, as_i64(node.id), as_i64(*rank), rating],
        )?;
    }

    // Only the edges leaving this node: an edge some other series drew to
    // it is that series' row to replace, not this one's.
    tx.execute("DELETE FROM relations WHERE from_id = ?1", params![id])?;
    for (relation, node) in &w.relations {
        tx.execute(
            "INSERT OR REPLACE INTO relations (from_id, to_id, relation) VALUES (?1, ?2, ?3)",
            params![id, as_i64(node.id), relation],
        )?;
    }
    Ok(())
}

pub fn write_stub(tx: &Transaction, s: &StubWrite) -> Result<(), CoreError> {
    tx.execute(
        STUB_UPSERT,
        params![
            as_i64(s.id),
            s.mal_id.map(as_i64),
            s.media_type,
            s.title_romaji,
            s.title_english,
            s.format,
            s.status,
            s.year.map(i64::from),
            s.cover_url,
            s.site_url,
            s.episodes.map(i64::from),
            s.average_score.map(i64::from),
        ],
    )?;
    Ok(())
}

/// What a relation's other end or a recommendation's target knows about
/// itself. `episodes` and `average_score` are not in that reply, so they
/// stay blank for the watching refresh to fill.
pub fn stub_from_node(node: &RelatedNode) -> StubWrite {
    StubWrite {
        id: node.id,
        mal_id: node.id_mal,
        media_type: node.type_.clone(),
        title_romaji: node.title.as_ref().and_then(|t| t.romaji.clone()),
        title_english: node.title.as_ref().and_then(|t| t.english.clone()),
        format: node.format.clone(),
        status: node.status.clone(),
        year: node
            .season_year
            .or_else(|| node.start_date.as_ref().and_then(|d| d.year)),
        cover_url: node.cover_image.as_ref().and_then(|c| c.large.clone()),
        site_url: node.site_url.clone(),
        episodes: None,
        average_score: None,
    }
}

const EPISODE_UPSERT_KEEP: &str =
    "INSERT INTO anilist_episodes (anilist_id, number, title, aired_at) VALUES (?1, ?2, ?3, ?4)
     ON CONFLICT(anilist_id, number) DO UPDATE SET
        title = coalesce(anilist_episodes.title, excluded.title),
        aired_at = coalesce(excluded.aired_at, anilist_episodes.aired_at)";

const EPISODE_UPSERT_FRESH: &str =
    "INSERT INTO anilist_episodes (anilist_id, number, title, aired_at) VALUES (?1, ?2, ?3, ?4)
     ON CONFLICT(anilist_id, number) DO UPDATE SET
        title = coalesce(excluded.title, anilist_episodes.title),
        aired_at = coalesce(excluded.aired_at, anilist_episodes.aired_at)";

/// The schedule and the titles for one series. `keep_titles` is the airing
/// refresh's rule: that job asks AniList for dates alone, so a title
/// already in the table is the best one there will ever be and nothing it
/// writes may replace it. A full metadata fetch clears the flag and its
/// titles win.
///
/// A row in the future that this list does not mention is a scheduled
/// episode the provider has dropped, so it goes. An empty list is a fetch
/// that found nothing, never an instruction to empty the schedule.
pub fn write_episodes(
    tx: &Transaction,
    anilist_id: u64,
    rows: &[EpisodeRow],
    keep_titles: bool,
    now: i64,
) -> Result<(), CoreError> {
    let id = as_i64(anilist_id);
    let sql = if keep_titles {
        EPISODE_UPSERT_KEEP
    } else {
        EPISODE_UPSERT_FRESH
    };
    for r in rows {
        tx.execute(sql, params![id, i64::from(r.number), r.title, r.aired_at])?;
    }
    if rows.is_empty() {
        return Ok(());
    }
    let mut binds: Vec<Value> = Vec::with_capacity(rows.len() + 2);
    binds.push(Value::from(id));
    binds.push(Value::from(now));
    binds.extend(rows.iter().map(|r| Value::from(i64::from(r.number))));
    let sql = format!(
        "DELETE FROM anilist_episodes WHERE anilist_id = ? AND aired_at > ? AND number NOT IN ({})",
        placeholders(rows.len())
    );
    tx.execute(&sql, params_from_iter(binds))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::anilist::*;
    use crate::net::jikan::JikanEpisode;

    #[test]
    fn dates_pad_missing_parts() {
        assert_eq!(
            format_date(Some(&FuzzyDate {
                year: Some(2023),
                month: Some(9),
                day: Some(29)
            })),
            Some("2023-09-29".into())
        );
        assert_eq!(
            format_date(Some(&FuzzyDate {
                year: Some(2023),
                month: None,
                day: None
            })),
            Some("2023-01-01".into())
        );
        assert_eq!(
            format_date(Some(&FuzzyDate {
                year: None,
                month: Some(9),
                day: None
            })),
            None
        );
        assert_eq!(format_date(None), None);
    }

    #[test]
    fn studio_priority() {
        let edges = vec![
            StudioEdge {
                is_main: false,
                node: StudioNode {
                    id: 1,
                    name: "Aniplex".into(),
                    is_animation_studio: false,
                },
            },
            StudioEdge {
                is_main: false,
                node: StudioNode {
                    id: 2,
                    name: "Madhouse".into(),
                    is_animation_studio: true,
                },
            },
            StudioEdge {
                is_main: true,
                node: StudioNode {
                    id: 3,
                    name: "Main Anim".into(),
                    is_animation_studio: true,
                },
            },
        ];
        let (studio, list) = pick_studio(&edges);
        assert_eq!(studio.as_deref(), Some("Main Anim"));
        assert_eq!(
            list.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["Main Anim", "Madhouse", "Aniplex"]
        );
        assert_eq!(pick_studio(&[]).0, None);
    }

    #[test]
    fn streaming_titles_parse_and_first_wins() {
        assert_eq!(
            parse_streaming_title("Episode 1 - Ordinary Person"),
            Some((1, "Ordinary Person".into()))
        );
        assert_eq!(
            parse_streaming_title("1 - Ordinary Person"),
            Some((1, "Ordinary Person".into()))
        );
        assert_eq!(parse_streaming_title("Episode 1"), None);
        assert_eq!(
            parse_streaming_title("S2 Episode 3 - The Heroine"),
            Some((2, "The Heroine".into()))
        );
        assert_eq!(parse_streaming_title("Episode 5 - Episode 5"), None);
        let eps = vec![
            StreamingEpisode {
                title: Some("Episode 1 - A".into()),
                thumbnail: None,
                url: None,
                site: None,
            },
            StreamingEpisode {
                title: Some("Episode 1 - A (HiDive)".into()),
                thumbnail: None,
                url: None,
                site: None,
            },
            StreamingEpisode {
                title: Some("Episode 2 - B".into()),
                thumbnail: None,
                url: None,
                site: None,
            },
        ];
        assert_eq!(
            streaming_titles(&eps),
            vec![(1, "A".into()), (2, "B".into())]
        );
    }

    #[test]
    fn episodes_merge_anilist_dates_win_jikan_fills_next_broadcast_last() {
        let schedule = Schedule {
            next_airing_episode: Some(AiringNode {
                episode: 4,
                airing_at: 4000,
            }),
            airing_schedule: Some(AiringNodes {
                nodes: vec![
                    AiringNode {
                        episode: 1,
                        airing_at: 1000,
                    },
                    AiringNode {
                        episode: 2,
                        airing_at: 2000,
                    },
                    AiringNode {
                        episode: 4,
                        airing_at: 3999,
                    },
                ],
            }),
        };
        let jikan = vec![
            JikanEpisode {
                number: 1,
                title: Some("One".into()),
                aired: Some("1970-01-01T00:16:40+00:00".into()),
                synopsis: None,
            },
            JikanEpisode {
                number: 2,
                title: Some("Episode 2".into()),
                aired: None,
                synopsis: None,
            },
            JikanEpisode {
                number: 3,
                title: Some("Three".into()),
                aired: Some("1970-01-01T00:50:00+00:00".into()),
                synopsis: None,
            },
        ];
        let rows = merge_episodes(Some(&schedule), &[(2, "Two (AniList)".into())], &jikan);
        assert_eq!(
            rows.iter()
                .map(|r| (r.number, r.title.clone(), r.aired_at))
                .collect::<Vec<_>>(),
            vec![
                (1, Some("One".into()), Some(1000)),
                (2, Some("Two (AniList)".into()), Some(2000)),
                (3, Some("Three".into()), Some(3000)),
                (4, None, Some(4000)),
            ]
        );
    }

    /// A schedule AniList has nothing for, and a Jikan list with a blank
    /// title where the real one should be: the numbers still come through,
    /// with no title rather than an empty one.
    #[test]
    fn episodes_merge_with_no_schedule_and_a_blank_jikan_title() {
        let jikan = vec![
            JikanEpisode {
                number: 1,
                title: Some("   ".into()),
                aired: Some("2023-09-29".into()),
                synopsis: None,
            },
            JikanEpisode {
                number: 2,
                title: None,
                aired: None,
                synopsis: None,
            },
        ];
        let rows = merge_episodes(None, &[(3, "Streaming only".into())], &jikan);
        assert_eq!(
            rows,
            vec![
                EpisodeRow {
                    number: 1,
                    title: None,
                    aired_at: Some(1_695_945_600)
                },
                EpisodeRow {
                    number: 2,
                    title: None,
                    aired_at: None
                },
                EpisodeRow {
                    number: 3,
                    title: Some("Streaming only".into()),
                    aired_at: None
                },
            ]
        );
    }

    #[test]
    fn a_bare_date_parses_and_nonsense_is_no_date() {
        assert_eq!(parse_aired("2023-09-29"), Some(1_695_945_600));
        assert_eq!(
            parse_aired("2023-09-29T00:00:00+00:00"),
            Some(1_695_945_600)
        );
        assert_eq!(parse_aired("not a date"), None);
        assert_eq!(parse_aired(""), None);
    }

    #[test]
    fn an_empty_reply_still_builds_a_row() {
        let w = build(
            &Media {
                id: 7,
                ..Default::default()
            },
            None,
        );
        assert_eq!(w.id, 7);
        assert_eq!(w.media_type.as_deref(), Some("ANIME"));
        assert!(w.studios.is_empty());
        assert!(w.tags.is_empty());
        assert!(w.characters.is_empty());
        assert!(w.recommendations.is_empty());
        assert!(w.relations.is_empty());
        assert!(image_urls(&w).is_empty());
    }

    /// The cap is eight of twelve, and a recommendation AniList has since
    /// deleted arrives with a null target.
    #[test]
    fn recommendations_are_the_top_eight_and_dead_targets_are_dropped() {
        let edge = |id: u64, rating: i64| RecommendationEdge {
            node: RecommendationNode {
                rating: Some(rating),
                media_recommendation: Some(RelatedNode {
                    id,
                    ..Default::default()
                }),
            },
        };
        let mut edges: Vec<RecommendationEdge> = (1..=12u64).map(|i| edge(i, as_i64(i))).collect();
        edges.push(RecommendationEdge {
            node: RecommendationNode {
                rating: Some(999),
                media_recommendation: None,
            },
        });
        let enrichment = Enrichment {
            recommendations: Some(RecommendationEdges { edges }),
            ..Default::default()
        };
        let w = build(&Media::default(), Some(&enrichment));
        assert_eq!(
            w.recommendations
                .iter()
                .map(|(rank, n, _)| (*rank, n.id))
                .collect::<Vec<_>>(),
            vec![
                (0, 12),
                (1, 11),
                (2, 10),
                (3, 9),
                (4, 8),
                (5, 7),
                (6, 6),
                (7, 5),
            ]
        );
    }

    /// A relation type the crawl will not walk is still a relation the
    /// page draws.
    #[test]
    fn every_relation_edge_is_kept() {
        let edge = |relation: &str, id: u64| RelationEdge {
            relation_type: relation.to_string(),
            node: RelatedNode {
                id,
                ..Default::default()
            },
        };
        let enrichment = Enrichment {
            relations: Some(RelationEdges {
                edges: vec![
                    edge("SEQUEL", 2),
                    edge("CHARACTER", 3),
                    edge("OTHER", 4),
                    edge("", 5),
                ],
            }),
            ..Default::default()
        };
        let w = build(&Media::default(), Some(&enrichment));
        assert_eq!(
            w.relations
                .iter()
                .map(|(r, n)| (r.as_str(), n.id))
                .collect::<Vec<_>>(),
            vec![("SEQUEL", 2), ("CHARACTER", 3), ("OTHER", 4), ("", 5)]
        );
    }
}
