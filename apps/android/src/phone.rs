//! This phone's own library, read from its core, in the shapes the Slint pages take.

use std::time::{SystemTime, UNIX_EPOCH};

use anibeam_companion::calendar;
use anibeam_companion::format;
use anibeam_core::{SeriesCard, SeriesDetail, SeriesKind};
use chrono::{Datelike, Days, NaiveDate};

use crate::posters;
use crate::remote::Art;
use crate::{Day, EpisodeLine, Hero, SeriesHead, Tile};

/// How many tiles a Home shelf holds.
const SHELF: usize = 12;

pub fn secs(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// The core's cube letters (`library::cards::cubes`) as Theme.cube's codes.
pub fn cube_codes(cubes: &str) -> Vec<i32> {
    cubes.chars().map(cube_code).collect()
}

fn cube_code(c: char) -> i32 {
    match c {
        'w' => 0,
        'l' => 1,
        'd' => 2,
        'm' | 'e' => 3,
        'f' => 5,
        't' => 6,
        _ => 4,
    }
}

/// "6 Oct", with the year when it is not this one: "6 Oct 2023".
fn date(at: i64, now: i64) -> String {
    let year = |t: i64| chrono::DateTime::from_timestamp(t, 0).map(|d| d.with_timezone(&chrono::Local).year());
    if year(at) == year(now) {
        format::day_month(at)
    } else {
        format!("{} {}", format::day_month(at), year(at).unwrap_or_default())
    }
}

fn id(series: u64) -> i32 {
    i32::try_from(series).unwrap_or(-1)
}

fn title(card: &SeriesCard) -> &str {
    &card.title
}

fn tile(card: &SeriesCard, art: &Art) -> Tile {
    let (poster, has_poster) = art.poster(Some(card.id));
    Tile {
        id: id(card.id),
        title: title(card).into(),
        poster,
        has_poster,
        initials: format::initials(title(card)).into(),
        cubes: std::rc::Rc::new(slint::VecModel::from(cube_codes(&card.cubes))).into(),
        ..Default::default()
    }
}

fn next_code(card: &SeriesCard) -> Option<String> {
    card.next_airing.as_ref().map(|a| format::episode_code(a.season, a.episode))
}

fn on_phone(card: &SeriesCard) -> String {
    match card.episodes_on_disk {
        0 => "nothing on phone".to_string(),
        1 => "1 on phone".to_string(),
        n => format!("{n} on phone"),
    }
}

/// The library's tile: the next episode and when it airs, else how many are on the phone.
fn library_tile(card: &SeriesCard, art: &Art, now: i64) -> Tile {
    let mut t = tile(card, art);
    if let Some(next) = card.next_airing.as_ref().filter(|a| secs(a.at) > now) {
        t.badge = format::episode_code(next.season, next.episode).into();
        t.ledger = format::countdown(secs(next.at) - now).into();
        t.ledger_tone = 1;
    } else {
        t.ledger = on_phone(card).into();
    }
    if let Some(d) = card.downloading.first() {
        t.flag = format!("{}%", (d.progress * 100.0).round() as i64).into();
    }
    t
}

pub fn library<'a>(cards: impl IntoIterator<Item = &'a SeriesCard>, art: &Art, now: i64) -> Vec<Tile> {
    cards.into_iter().map(|c| library_tile(c, art, now)).collect()
}

/// Home, in one read of the cards: the hero, the week, the picked day's tiles, and the
/// recently watched and recently downloaded shelves.
pub struct HomeView {
    pub hero: Hero,
    pub days: Vec<Day>,
    pub day_title: String,
    pub day_tiles: Vec<Tile>,
    pub watched: Vec<Tile>,
    pub downloaded: Vec<Tile>,
}

pub fn home(cards: &[SeriesCard], art: &Art, now: i64, day: usize) -> HomeView {
    let upcoming: Vec<(&SeriesCard, i64)> = cards
        .iter()
        .filter_map(|c| c.next_airing.as_ref().map(|a| (c, secs(a.at))))
        .filter(|(_, at)| *at > now)
        .collect();

    let hero = upcoming.iter().min_by_key(|(_, at)| *at).map(|(c, at)| hero(c, *at, art, now)).unwrap_or_default();

    let monday = calendar::week_start(now, 0);
    let today = calendar::today(now);
    let dates: Vec<NaiveDate> = (0..7).map(|i| monday + Days::new(i)).collect();
    let bounds = |d: NaiveDate| (calendar::midnight(d), calendar::midnight(d + Days::new(1)));
    let days = dates
        .iter()
        .map(|&d| {
            let (start, end) = bounds(d);
            Day {
                weekday: d.format("%a").to_string().into(),
                date: d.day().to_string().into(),
                count: upcoming.iter().filter(|(_, at)| *at >= start && *at < end).count() as i32,
                today: d == today,
            }
        })
        .collect();
    let picked = dates[day.min(6)];
    let (start, end) = bounds(picked);
    let mut on_day: Vec<&(&SeriesCard, i64)> = upcoming.iter().filter(|(_, at)| *at >= start && *at < end).collect();
    on_day.sort_by_key(|(_, at)| *at);
    let day_tiles: Vec<Tile> = on_day
        .iter()
        .map(|(c, at)| {
            let mut t = tile(c, art);
            t.badge = format::clock(*at).into();
            t.ledger = format!("{} {}", next_code(c).unwrap_or_default(), format::countdown(at - now)).into();
            t.ledger_tone = 1;
            t
        })
        .collect();
    let day_title = match (picked == today, day_tiles.len()) {
        (true, 0) => "Nothing airs today.".to_string(),
        (false, 0) => format!("Nothing airs {}.", picked.format("%A")),
        (true, n) => format!("Today, {n} airing"),
        (false, n) => format!("{}, {n} airing", picked.format("%A %-d %B")),
    };

    let mut watched: Vec<&SeriesCard> = cards.iter().filter(|c| c.last_viewed_at.is_some()).collect();
    watched.sort_by_key(|c| std::cmp::Reverse(c.last_viewed_at.map(secs)));
    let watched = watched
        .into_iter()
        .take(SHELF)
        .map(|c| {
            let mut t = tile(c, art);
            if let Some(code) = &c.code {
                t.badge = code.clone().into();
            }
            t.ledger = c.last_viewed_at.map(|at| format::ago(now - secs(at))).unwrap_or_default().into();
            t
        })
        .collect();

    let mut downloaded: Vec<&SeriesCard> = cards.iter().filter(|c| c.downloaded_at.is_some()).collect();
    downloaded.sort_by_key(|c| std::cmp::Reverse(c.downloaded_at.map(secs)));
    let downloaded = downloaded
        .into_iter()
        .take(SHELF)
        .map(|c| {
            let mut t = tile(c, art);
            t.ledger = c.downloaded_at.map(|at| format::ago(now - secs(at))).unwrap_or_default().into();
            if let Some(d) = c.downloading.first() {
                t.flag = format!("{}%", (d.progress * 100.0).round() as i64).into();
            }
            t
        })
        .collect();

    HomeView { hero, days, day_title, day_tiles, watched, downloaded }
}

fn hero(card: &SeriesCard, at: i64, art: &Art, now: i64) -> Hero {
    let (poster, has_poster) = art.poster(Some(card.id));
    let tone = art.tone(card.id);
    Hero {
        show: true,
        id: id(card.id),
        title: title(card).into(),
        poster,
        has_poster,
        initials: format::initials(title(card)).into(),
        episode: format!("{}  {}", next_code(card).unwrap_or_default(), format::when(at, now)).into(),
        countdown: format!("in {}", format::countdown(at - now)).into(),
        band: posters::band(tone),
        ink: posters::ink(tone),
    }
}

/// A series page from the core's detail: the band, then every episode in number order,
/// on the phone or not, aired or still to come.
pub fn series(detail: &SeriesDetail, art: &Art, now: i64) -> (SeriesHead, Vec<EpisodeLine>) {
    let card = &detail.card;
    let (poster, has_poster) = art.poster(Some(card.id));
    let tone = art.tone(card.id);
    let movie = card.kind == SeriesKind::Movie;

    let alt = [&card.titles.english, &card.titles.romaji, &card.titles.native]
        .into_iter()
        .flatten()
        .find(|t| t.as_str() != card.title)
        .cloned()
        .unwrap_or_default();
    let mut facts: Vec<String> = Vec::new();
    if let Some(f) = &card.format {
        facts.push(f.clone());
    }
    if let Some(s) = &card.season {
        facts.push(s.clone());
    }
    if let Some(total) = card.total_episodes {
        facts.push(format!("{total}{} episodes", if card.total_is_estimate { "?" } else { "" }));
    }
    let next = card
        .next_airing
        .as_ref()
        .filter(|a| secs(a.at) > now)
        .map(|a| format!("Episode {} airs in {}", a.episode, format::countdown_full(secs(a.at) - now)))
        .unwrap_or_default();
    let watched = detail.progress.watched.unwrap_or(0);
    let progress_text = match (movie, detail.progress.total) {
        (true, _) => if watched > 0 { "Watched" } else { "Not watched yet" }.to_string(),
        (false, Some(total)) if total > 0 => {
            format!("{} of {total} watched, {} on this phone", watched.min(total), detail.progress.on_disk)
        }
        _ => format!("{} on this phone", detail.progress.on_disk),
    };
    // Play opens the next one up, the desktop hero's rule, when it is whole on the phone.
    let play = detail.episodes.iter().find(|e| e.next_up && e.downloading.is_none());
    let head = SeriesHead {
        id: id(card.id),
        title: card.title.clone().into(),
        alt_title: alt.into(),
        facts: facts.join("  ").into(),
        poster,
        has_poster,
        initials: format::initials(&card.title).into(),
        band: posters::band(tone),
        ink: posters::ink(tone),
        next: next.into(),
        progress_text: progress_text.into(),
        cubes: std::rc::Rc::new(slint::VecModel::from(cube_codes(&card.cubes))).into(),
        synopsis: detail.synopsis.clone().into(),
        play_file: play.map(|e| e.file as i32).unwrap_or(0),
        play_label: play
            .map(|e| {
                let what = if movie { "the film".to_string() } else { format!("episode {}", e.number) };
                if e.resume.is_some() { format!("Resume {what}") } else { format!("Play {what}") }
            })
            .unwrap_or_default()
            .into(),
    };

    // Sorted by season, then number: a show with S1 and S2 lists each season in order.
    let mut lines: Vec<((u32, f64), EpisodeLine)> = Vec::new();
    for e in &detail.episodes {
        let note = if let Some(p) = e.downloading {
            format!("Downloading {}%", (p * 100.0).round() as i64)
        } else if e.next_up {
            "Next up".to_string()
        } else if let Some(r) = &e.resume {
            format!("Resume at {}", clock(r.position))
        } else {
            String::new()
        };
        let title = e.title.clone().unwrap_or_else(|| if movie { card.title.clone() } else { format!("Episode {}", e.number) });
        lines.push((
            (e.season.unwrap_or(1), e.number),
            EpisodeLine {
                code: e.code.clone().into(),
                title: title.into(),
                date: e.air_date.map(|t| date(secs(t), now)).unwrap_or_default().into(),
                note: note.into(),
                cube: e.cube.chars().next().map(cube_code).unwrap_or(2),
                // A file a torrent is still writing is on disk but not playable.
                file: if e.downloading.is_none() { e.file as i32 } else { 0 },
            },
        ));
    }
    for a in &detail.absent {
        lines.push((
            (a.season.unwrap_or(1), f64::from(a.number)),
            EpisodeLine {
                code: a.code.clone().into(),
                title: a.title.clone().unwrap_or_else(|| format!("Episode {}", a.number)).into(),
                date: a.air_date.map(|t| date(secs(t), now)).unwrap_or_default().into(),
                note: "Missing".into(),
                cube: a.cube.chars().next().map(cube_code).unwrap_or(3),
                file: 0,
            },
        ));
    }
    for u in detail.upcoming.iter().take(6) {
        let at = secs(u.at);
        let date = if u.predicted {
            format!("about {}", format::day_month(at))
        } else if at - now < 7 * 86_400 {
            format!("in {}", format::countdown(at - now))
        } else {
            format::day_month(at)
        };
        lines.push((
            (u.season.unwrap_or(1), f64::from(u.number)),
            EpisodeLine {
                code: u.code.clone().into(),
                title: u.title.clone().unwrap_or_else(|| format!("Episode {}", u.number)).into(),
                date: date.into(),
                note: Default::default(),
                cube: 5,
                file: 0,
            },
        ));
    }
    lines.sort_by(|a, b| a.0.0.cmp(&b.0.0).then(a.0.1.total_cmp(&b.0.1)));
    (head, lines.into_iter().map(|(_, l)| l).collect())
}

/// m:ss, or h:mm:ss past an hour: the desktop's Fmt.clock.
fn clock(secs: f64) -> String {
    let t = secs.max(0.0) as u64;
    let (h, m, s) = (t / 3600, t % 3600 / 60, t % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_core_cube_letter_has_a_colour_code() {
        assert_eq!(cube_codes("wldmgft"), vec![0, 1, 2, 3, 4, 5, 6]);
    }
}
