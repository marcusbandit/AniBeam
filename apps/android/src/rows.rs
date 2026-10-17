//! The companion crate's rows, in the shape the Slint screens take.

use std::collections::HashMap;

use anibeam_companion::format::{self, Tone};
use anibeam_companion::library;
use anibeam_companion::schedule::{self, Headline, NextUp};
use anibeam_companion::store::{AniListCache, Cache};
use slint::{Image, ModelRc, SharedString, VecModel};

use crate::{DiskEpisode, DiskRow, NextUpRow, TorrentRow};

/// Episode chips past this many show only the newest, with a count of the rest.
const MOST_CHIPS: usize = 24;
const MOST_TORRENTS: usize = 12;

fn initials(title: &str) -> SharedString {
    format::initials(title).into()
}

fn poster(posters: &HashMap<u64, Image>, id: Option<u64>) -> (Image, bool) {
    match id.and_then(|id| posters.get(&id)) {
        Some(image) => (image.clone(), true),
        None => (Image::default(), false),
    }
}

fn next_up_row(row: &NextUp, posters: &HashMap<u64, Image>, now: i64) -> NextUpRow {
    let (poster, has_poster) = poster(posters, row.series_id);
    let mut out = NextUpRow {
        title: row.title.clone().into(),
        poster,
        has_poster,
        initials: initials(&row.title),
        subscribed: row.subscribed,
        ..Default::default()
    };
    match &row.headline {
        Headline::Upcoming(airing) => {
            out.tone = 0;
            out.countdown = format::countdown(airing.at - now).into();
            out.detail = format!("{}  {}", format::episode_code(airing.season, airing.episode), format::when(airing.at, now)).into();
        }
        Headline::Aired { season, episode, at } => {
            out.tone = 1;
            let code = format::episode_code(*season, *episode);
            match at {
                Some(at) => {
                    out.countdown = format!("aired {}", format::ago(now - at)).into();
                    out.detail = format!("{code}  {}", format::when(*at, now)).into();
                }
                None => {
                    out.countdown = "aired".into();
                    out.detail = code.into();
                }
            }
            out.flag = match row.missing.len() {
                0 | 1 => "Not on disk".into(),
                n => format!("{n} not on disk").into(),
            };
        }
        Headline::Unscheduled => {
            out.tone = 2;
            out.countdown = if row.complete {
                "Complete".into()
            } else if row.finished {
                "Finished airing".into()
            } else {
                "No air time".into()
            };
        }
    }
    if let Some(then) = &row.then {
        out.note = format!("next {} {}", format::episode_code(then.season, then.episode), format::countdown(then.at - now)).into();
    }
    if row.paused {
        out.flag = "Paused".into();
    }
    out
}

pub fn next_up(cache: Option<&Cache>, anilist: &AniListCache, posters: &HashMap<u64, Image>, now: i64) -> Vec<NextUpRow> {
    let Some(cache) = cache else { return Vec::new() };
    schedule::next_up(cache, anilist, now).iter().map(|row| next_up_row(row, posters, now)).collect()
}

pub fn disk(cache: Option<&Cache>, posters: &HashMap<u64, Image>) -> Vec<DiskRow> {
    let Some(cache) = cache else { return Vec::new() };
    library::on_disk(&cache.snapshot)
        .into_iter()
        .map(|series| {
            let (poster, has_poster) = poster(posters, Some(series.id));
            let skip = series.episodes.len().saturating_sub(MOST_CHIPS);
            let episodes: Vec<DiskEpisode> = series.episodes[skip..]
                .iter()
                .map(|e| DiskEpisode { code: e.code.clone().into(), watched: e.watched })
                .collect();
            DiskRow {
                initials: initials(&series.title),
                title: series.title.into(),
                poster,
                has_poster,
                summary: series.summary.into(),
                episodes: ModelRc::new(VecModel::from(episodes)),
                more: if skip > 0 { format!("Showing the newest {MOST_CHIPS}; {skip} earlier").into() } else { "".into() },
            }
        })
        .collect()
}

/// The torrent rows, and whether the host could read its torrent client at all.
pub fn torrents(cache: Option<&Cache>) -> (Vec<TorrentRow>, bool) {
    let Some(cache) = cache else { return (Vec::new(), true) };
    match library::torrents(&cache.snapshot, MOST_TORRENTS) {
        None => (Vec::new(), false),
        Some(lines) => (
            lines
                .into_iter()
                .map(|t| TorrentRow {
                    name: t.name.into(),
                    state: t.state.into(),
                    facts: t.facts.into(),
                    progress: t.progress,
                    tone: match t.tone {
                        Tone::Active => 0,
                        Tone::Quiet => 1,
                        Tone::Trouble => 2,
                    },
                })
                .collect(),
            true,
        ),
    }
}
