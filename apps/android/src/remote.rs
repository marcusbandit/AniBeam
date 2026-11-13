//! Another AniBeam's library, read from its tailnet snapshot, in the shapes the Slint pages
//! take. The phone never changes that library; its episodes stream from there.

use std::collections::HashMap;

use anibeam_companion::api::Airing;
use anibeam_companion::format;
use anibeam_companion::library::{self, Tile as LibraryTile};
use anibeam_companion::store::{AniListCache, Cache};
use slint::Image;

use crate::posters;
use crate::{CalDay, CalItem, EpisodeLine, EpisodeVersionLine, SeriesHead, Tile};

/// Decoded posters and their hues, by series id.
#[derive(Default)]
pub struct Art {
    pub posters: HashMap<u64, Image>,
    pub tones: HashMap<u64, Option<[u8; 3]>>,
}

impl Art {
    pub fn poster(&self, id: Option<u64>) -> (Image, bool) {
        match id.and_then(|id| self.posters.get(&id)) {
            Some(image) => (image.clone(), true),
            None => (Image::default(), false),
        }
    }

    pub fn tone(&self, id: u64) -> Option<[u8; 3]> {
        self.tones.get(&id).copied().flatten()
    }
}

fn id(series_id: Option<u64>) -> i32 {
    series_id
        .and_then(|id| i32::try_from(id).ok())
        .unwrap_or(-1)
}

fn tile(art: &Art, series_id: Option<u64>, title: &str) -> Tile {
    let (poster, has_poster) = art.poster(series_id);
    Tile {
        id: id(series_id),
        title: title.into(),
        poster,
        has_poster,
        initials: format::initials(title).into(),
        ..Default::default()
    }
}

fn code(a: &Airing) -> String {
    format::episode_code(a.season, a.episode)
}

fn library_tile(t: &LibraryTile, art: &Art, now: i64) -> Tile {
    let mut out = tile(art, Some(t.id), &t.title);
    if let Some(next) = &t.next {
        out.badge = code(next).into();
        out.ledger = format::countdown(next.at - now).into();
        out.ledger_tone = 1;
    } else if t.movie {
        out.ledger = if t.watched > 0 {
            tr!("remote-watched").into()
        } else {
            tr!("remote-movie").into()
        };
    } else if let Some(total) = t.total.filter(|n| *n > 0) {
        out.ledger = tr!(
            "remote-watched-of",
            done = t.watched.min(total).to_string(),
            total = total.to_string()
        )
        .into();
    } else if t.on_disk > 0 {
        out.ledger = tr!("remote-on-disk", count = t.on_disk.to_string()).into();
    }
    out
}

/// The device's library as one grid, airing first, then by title, narrowed by `query`.
pub fn library(
    cache: Option<&Cache>,
    anilist: &AniListCache,
    art: &Art,
    now: i64,
    query: &str,
) -> Vec<Tile> {
    let Some(cache) = cache else {
        return Vec::new();
    };
    let query = query.trim().to_lowercase();
    let (airing, rest): (Vec<_>, Vec<_>) = library::tiles(cache, anilist, now)
        .into_iter()
        .filter(|t| query.is_empty() || t.title.to_lowercase().contains(&query))
        .partition(|t| t.airing);
    airing
        .iter()
        .chain(rest.iter())
        .map(|t| library_tile(t, art, now))
        .collect()
}

/// One series page: the band and the episode rows.
pub fn series(
    cache: Option<&Cache>,
    anilist: &AniListCache,
    art: &Art,
    series_id: u64,
    now: i64,
) -> Option<(SeriesHead, Vec<EpisodeLine>)> {
    let cache = cache?;
    let page = library::detail(cache, anilist, series_id, now)?;
    let (poster, has_poster) = art.poster(Some(series_id));
    let tone = art.tone(series_id);
    let movie = page.movie;

    let progress_text = if movie {
        let seen = page.episodes.iter().any(|e| e.watched) || page.watched > 0;
        if seen {
            tr!("remote-watched")
        } else {
            tr!("remote-not-watched-yet")
        }
    } else {
        match page.total.filter(|n| *n > 0) {
            Some(total) => tr!(
                "remote-watched-of",
                done = page.watched.min(total).to_string(),
                total = total.to_string()
            ),
            None if page.watched > 0 => {
                tr!("remote-watched-count", count = page.watched.to_string())
            }
            None => String::new(),
        }
    };

    // Aired, listed by the rule as missing, and not on disk.
    let on_disk: Vec<(u32, u32)> = page
        .episodes
        .iter()
        .filter(|e| e.number.fract() == 0.0)
        .map(|e| (e.season.unwrap_or(1), e.number as u32))
        .collect();
    let next_number = page.upcoming.first().map(|a| a.episode);
    let missing: Vec<u32> = page
        .subscription
        .iter()
        .flat_map(|s| s.missing.iter().copied())
        .filter(|n| !on_disk.contains(&(1, *n)) && next_number.is_none_or(|next| *n < next))
        .collect();

    let head = SeriesHead {
        id: id(Some(series_id)),
        title: page.title.clone().into(),
        alt_title: page.alt_title.clone().unwrap_or_default().into(),
        facts: page.facts.join("  ").into(),
        poster,
        has_poster,
        initials: format::initials(&page.title).into(),
        band: posters::band(tone),
        ink: posters::ink(tone),
        next: page
            .upcoming
            .first()
            .map(|a| {
                let countdown = format::countdown_full(a.at - now);
                match a.season.filter(|s| *s > 1) {
                    Some(s) => tr!(
                        "remote-airs-in-season",
                        season = s.to_string(),
                        n = a.episode.to_string(),
                        countdown = countdown
                    ),
                    None => tr!(
                        "remote-airs-in",
                        n = a.episode.to_string(),
                        countdown = countdown
                    ),
                }
            })
            .unwrap_or_default()
            .into(),
        progress_text: progress_text.into(),
        ..Default::default()
    };
    // Play opens the first episode not watched yet, or the first of all when every one is.
    let next = page
        .episodes
        .iter()
        .find(|e| !e.watched && e.file.is_some());
    let up = next.or_else(|| page.episodes.iter().find(|e| e.file.is_some()));
    let head = match up.and_then(|e| Some((e, i32::try_from(e.file?).ok()?))) {
        Some((e, file)) => SeriesHead {
            play_file: file,
            play_remote: true,
            play_label: if movie {
                tr!("common-play").into()
            } else if next.is_some() {
                tr!("remote-play-episode", n = e.number.to_string()).into()
            } else {
                tr!("remote-watch-again", n = e.number.to_string()).into()
            },
            ..head
        },
        None => head,
    };

    let mut lines: Vec<((u32, f64), EpisodeLine)> = page
        .episodes
        .iter()
        .map(|e| {
            let title = e.title.clone().unwrap_or_else(|| {
                if movie {
                    page.title.clone()
                } else {
                    tr!("common-episode-n", n = e.number.to_string())
                }
            });
            let line = EpisodeLine {
                code: e.code.clone().into(),
                title: title.into(),
                date: e.aired_at.map(format::day_month).unwrap_or_default().into(),
                // A host too old to stream lists no files; the row says what to do.
                note: if e.file.is_none() {
                    tr!("remote-update-host")
                } else {
                    String::new()
                }
                .into(),
                note_kind: 0,
                cube: if e.watched { 0 } else { 2 },
                // Streamed from the other device, from a host that can.
                file: e.file.and_then(|f| i32::try_from(f).ok()).unwrap_or(0),
                picked: false,
                remote: e.file.is_some(),
                versions: version_lines(e),
                ..Default::default()
            };
            ((e.season.unwrap_or(1), e.number), line)
        })
        .collect();
    for n in &missing {
        let line = EpisodeLine {
            code: format::episode_code(None, *n).into(),
            title: tr!("common-episode-n", n = n.to_string()).into(),
            date: tr!("remote-aired").into(),
            note: tr!("remote-missing").into(),
            note_kind: 2,
            cube: 3,
            file: 0,
            picked: false,
            remote: false,
            versions: Default::default(),
            ..Default::default()
        };
        lines.push(((1, f64::from(*n)), line));
    }
    lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut lines: Vec<EpisodeLine> = lines.into_iter().map(|(_, l)| l).collect();
    for a in page.upcoming.iter().take(6) {
        let soon = a.at - now < 7 * 86_400;
        lines.push(EpisodeLine {
            code: code(a).into(),
            title: tr!("common-episode-n", n = a.episode.to_string()).into(),
            date: if soon {
                format::countdown(a.at - now)
            } else {
                format::day_month(a.at)
            }
            .into(),
            note: Default::default(),
            note_kind: 0,
            cube: 5,
            file: 0,
            picked: false,
            remote: false,
            versions: Default::default(),
            ..Default::default()
        });
    }
    Some((head, lines))
}

/// The versions the other device keeps of an episode, streamed from there at a tap.
fn version_lines(e: &library::DiskEpisode) -> slint::ModelRc<EpisodeVersionLine> {
    let list: Vec<(u64, String)> = e
        .versions
        .iter()
        .map(|v| (v.file, v.label.clone()))
        .collect();
    crate::versions::model(crate::versions::offered(&list, |_| true))
}

/// The computer's calendar over the same days as this phone's, from its cached snapshot:
/// what it has on disk, what aired and it lacks, and what is still to air.
pub fn calendar(
    cache: Option<&Cache>,
    anilist: &AniListCache,
    art: &Art,
    now: i64,
) -> Vec<(CalDay, Vec<CalItem>)> {
    use anibeam_companion::calendar::{self, State};
    let Some(cache) = cache else {
        return crate::phone::days_of(Vec::new(), now);
    };
    let today = calendar::today(now);
    let from = calendar::midnight(today - chrono::Days::new(crate::phone::CAL_BACK as u64));
    let to = calendar::midnight(
        today + chrono::Days::new((crate::phone::CAL_DAYS - crate::phone::CAL_BACK) as u64),
    );
    let entries = calendar::entries(cache, anilist, from, to, now);
    let next_at = entries.iter().map(|e| e.at).find(|at| *at > now);
    let timed = entries
        .iter()
        .map(|e| {
            let (poster, has_poster) = art.poster(e.series_id);
            let (state, when) = match e.state {
                State::Upcoming => (0, format::countdown(e.at - now)),
                State::OnDisk { .. } => (1, tr!("remote-on-computer")),
                State::Missing => (2, tr!("remote-out", ago = format::ago(now - e.at))),
            };
            let item = CalItem {
                id: id(e.series_id),
                title: e.title.clone().into(),
                poster,
                has_poster,
                initials: format::initials(&e.title).into(),
                time: format::clock(e.at).into(),
                episode: match e.season {
                    Some(s) if s > 1 => tr!(
                        "remote-season-episode",
                        season = s.to_string(),
                        n = e.episode.to_string()
                    ),
                    _ => tr!("common-episode-n", n = e.episode.to_string()),
                }
                .into(),
                when: when.into(),
                state,
                soonest: Some(e.at) == next_at,
                remote: true,
            };
            (e.at, item)
        })
        .collect();
    crate::phone::days_of(timed, now)
}
