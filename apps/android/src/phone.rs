//! This phone's own library, read from its core, in the shapes the Slint pages take.

use std::time::{SystemTime, UNIX_EPOCH};

use anibeam_companion::calendar;
use anibeam_companion::format;
use anibeam_core::{SeriesCard, SeriesDetail, SeriesKind, WeekDrop};
use chrono::{Datelike, Days, NaiveDate};

use crate::posters;
use crate::remote::Art;
use crate::{CalDay, CalItem, EpisodeLine, Hero, SeriesHead, Tile};

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

/// Home, in one read of the cards: the hero, the recently watched and recently downloaded
/// shelves, and the calendar with the few episodes that air next.
pub struct HomeView {
    pub hero: Hero,
    pub watched: Vec<Tile>,
    pub downloaded: Vec<Tile>,
    /// Each day, and what airs on it apart, so the app only rebuilds the days that changed.
    pub calendar: Vec<(CalDay, Vec<CalItem>)>,
    pub coming: Vec<CalItem>,
}

/// The calendar's span: two days back, today, and a week ahead. The core's week airings
/// reach from three days before the moment the cards were read to eight after, so these
/// whole days are inside that window all day.
pub const CAL_BACK: usize = 2;
const CAL_DAYS: usize = CAL_BACK + 8;
/// How many of the next episodes Home lists.
const COMING: usize = 4;

pub fn home(cards: &[SeriesCard], art: &Art, now: i64) -> HomeView {
    let upcoming: Vec<(&SeriesCard, i64)> = cards
        .iter()
        .filter_map(|c| c.next_airing.as_ref().map(|a| (c, secs(a.at))))
        .filter(|(_, at)| *at > now)
        .collect();

    let hero = upcoming.iter().min_by_key(|(_, at)| *at).map(|(c, at)| hero(c, *at, art, now)).unwrap_or_default();
    let hero_at = upcoming.iter().min_by_key(|(_, at)| *at).map(|(c, at)| (c.id, *at));
    let (calendar, coming) = calendar(cards, art, now, hero_at);

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

    HomeView { hero, watched, downloaded, calendar, coming }
}

/// "Episode 5", or "Episodes 5 to 7" for a drop of several at once.
fn episodes(drop: &WeekDrop) -> String {
    let code = |e: u32| match drop.season {
        Some(s) if s > 1 => format!("S{s} E{e}"),
        _ => format!("{e}"),
    };
    match drop.through.filter(|t| *t > drop.episode) {
        Some(t) => format!("Episodes {} to {}", code(drop.episode), code(t)),
        None => format!("Episode {}", code(drop.episode)),
    }
}

/// Every day from three back to a week ahead with what airs on it, in air order, and the
/// next few still to air across all of them.
fn calendar(cards: &[SeriesCard], art: &Art, now: i64, hero: Option<(u64, i64)>) -> (Vec<(CalDay, Vec<CalItem>)>, Vec<CalItem>) {
    let today = calendar::today(now);
    let first = today - Days::new(CAL_BACK as u64);
    let mut drops: Vec<(&SeriesCard, &WeekDrop, i64)> =
        cards.iter().flat_map(|c| c.week_airings.iter().map(move |d| (c, d, secs(d.at)))).collect();
    drops.sort_by_key(|(c, _, at)| (*at, c.id));
    let next_at = drops.iter().map(|(_, _, at)| *at).find(|at| *at > now);

    let item = |c: &SeriesCard, d: &WeekDrop, at: i64, with_day: bool| {
        let (poster, has_poster) = art.poster(Some(c.id));
        let state = if at > now {
            0
        } else if d.here {
            1
        } else {
            2
        };
        let when = match state {
            0 => format::countdown(at - now),
            1 => "On the phone".to_string(),
            _ => format!("Out {}", format::ago(now - at)),
        };
        CalItem {
            id: id(c.id),
            title: title(c).into(),
            poster,
            has_poster,
            initials: format::initials(title(c)).into(),
            time: format::clock(at).into(),
            // Home's list spans days, so its rows say which.
            episode: if with_day { format!("{}, {}", episodes(d), day_word(calendar::today(at), today)) } else { episodes(d) }.into(),
            when: when.into(),
            state,
            soonest: Some(at) == next_at,
        }
    };

    let mut days: Vec<(CalDay, Vec<CalItem>)> = (0..CAL_DAYS)
        .map(|i| {
            let date = first + Days::new(i as u64);
            let (start, end) = (calendar::midnight(date), calendar::midnight(date + Days::new(1)));
            let items: Vec<CalItem> =
                drops.iter().filter(|(_, _, at)| *at >= start && *at < end).map(|(c, d, at)| item(c, d, *at, false)).collect();
            let day = CalDay {
                weekday: date.format("%a").to_string().into(),
                date: date.day().to_string().into(),
                title: day_word(date, today).into(),
                full: date.format("%A %-d %B").to_string().into(),
                today: date == today,
                past: date < today,
                count: items.len() as i32,
                ..Default::default()
            };
            (day, items)
        })
        .collect();
    let mut next = -1;
    for (i, (day, items)) in days.iter_mut().enumerate().rev() {
        day.next = next;
        if !items.is_empty() {
            next = i as i32;
        }
    }
    // Home's hero is the next to air; the list leaves it out rather than say it twice.
    let coming = drops.iter().filter(|(c, _, at)| *at > now && Some((c.id, *at)) != hero).take(COMING).map(|(c, d, at)| item(c, d, *at, true)).collect();
    (days, coming)
}

/// "Today", "Tomorrow", "Yesterday", else the weekday: "Thursday".
fn day_word(date: NaiveDate, today: NaiveDate) -> String {
    match (date - today).num_days() {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        -1 => "Yesterday".to_string(),
        _ => date.format("%A").to_string(),
    }
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
        countdown: format::countdown(at - now).into(),
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
        play_remote: false,
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
                picked: false,
                remote: false,
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
                // Watched somewhere else: tracked, but no file on this phone.
                note: if matches!(a.cube.as_str(), "w" | "l") { "Not on this phone" } else { "Missing" }.into(),
                cube: a.cube.chars().next().map(cube_code).unwrap_or(3),
                file: 0,
                picked: false,
                remote: false,
            },
        ));
    }
    for u in detail.upcoming.iter().take(6) {
        let at = secs(u.at);
        let date = if u.predicted {
            format!("about {}", format::day_month(at))
        } else if at - now < 7 * 86_400 {
            format::countdown(at - now)
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
                picked: false,
                remote: false,
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
