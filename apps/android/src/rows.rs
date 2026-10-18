//! The companion crate's rows, in the shapes the Slint pages take.

use std::collections::HashMap;

use anibeam_companion::api::Airing;
use anibeam_companion::calendar::{self, State};
use anibeam_companion::format::{self, Tone};
use anibeam_companion::library::{self, Tile as LibraryTile};
use anibeam_companion::schedule::{self, Headline, NextUp};
use anibeam_companion::store::{AniListCache, Cache};
use chrono::{Datelike, Days, NaiveDate};
use slint::{Image, SharedString};

use crate::posters;
use crate::{Day, EpisodeLine, Hero, RuleRow, SeriesHead, Tile, TorrentRow};

/// Decoded posters and their hues, by series id.
#[derive(Default)]
pub struct Art {
    pub posters: HashMap<u64, Image>,
    pub tones: HashMap<u64, Option<[u8; 3]>>,
}

impl Art {
    fn poster(&self, id: Option<u64>) -> (Image, bool) {
        match id.and_then(|id| self.posters.get(&id)) {
            Some(image) => (image.clone(), true),
            None => (Image::default(), false),
        }
    }

    fn tone(&self, id: u64) -> Option<[u8; 3]> {
        self.tones.get(&id).copied().flatten()
    }
}

fn id(series_id: Option<u64>) -> i32 {
    series_id.and_then(|id| i32::try_from(id).ok()).unwrap_or(-1)
}

fn tile(art: &Art, series_id: Option<u64>, title: &str) -> Tile {
    let (poster, has_poster) = art.poster(series_id);
    Tile {
        id: id(series_id),
        title: title.into(),
        poster,
        has_poster,
        initials: format::initials(title).into(),
        progress: -1.0,
        ..Default::default()
    }
}

/// A shelf before it becomes a model: the app keeps one tile model per shelf title, so a
/// tick updates tiles in place rather than rebuilding the grid.
pub struct ShelfRows {
    pub title: &'static str,
    pub tone: i32,
    pub tiles: Vec<Tile>,
}

fn shelf(title: &'static str, tone: i32, tiles: Vec<Tile>) -> ShelfRows {
    ShelfRows { title, tone, tiles }
}

fn code(a: &Airing) -> String {
    format::episode_code(a.season, a.episode)
}

/// Next up: the soonest episode to air as the hero, then what aired and is not on disk,
/// what comes after, and what has no air time.
pub fn next_up(cache: Option<&Cache>, anilist: &AniListCache, art: &Art, now: i64) -> (Hero, Vec<ShelfRows>) {
    let Some(cache) = cache else { return (Hero::default(), Vec::new()) };
    let rows = schedule::next_up(cache, anilist, now);
    let hero_key = rows.iter().find(|r| matches!(r.headline, Headline::Upcoming(_))).map(|r| r.key.clone());

    let mut hero = Hero::default();
    let (mut aired, mut coming, mut unscheduled) = (Vec::new(), Vec::new(), Vec::new());
    for row in &rows {
        match &row.headline {
            Headline::Upcoming(a) if Some(&row.key) == hero_key.as_ref() => hero = hero_of(row, a, art, now),
            Headline::Upcoming(a) => {
                let mut t = tile(art, row.series_id, &row.title);
                t.badge = code(a).into();
                t.ledger = format::countdown(a.at - now).into();
                t.ledger_tone = 1;
                coming.push(t);
            }
            Headline::Aired { season, episode, at } => {
                let mut t = tile(art, row.series_id, &row.title);
                t.badge = format::episode_code(*season, *episode).into();
                t.ledger = match (row.missing.len(), at) {
                    (n, _) if n > 1 => format!("{n} not on disk"),
                    (_, Some(at)) => format!("aired {}", format::ago(now - at)),
                    _ => "not on disk".to_string(),
                }
                .into();
                t.ledger_tone = 2;
                aired.push(t);
            }
            Headline::Unscheduled => {
                let mut t = tile(art, row.series_id, &row.title);
                t.ledger = if row.paused {
                    "Paused"
                } else if row.complete {
                    "Complete"
                } else if row.finished {
                    "Finished airing"
                } else {
                    "No air time"
                }
                .into();
                unscheduled.push(t);
            }
        }
    }
    let mut shelves = Vec::new();
    if !aired.is_empty() {
        shelves.push(shelf("Aired, not on disk", 2, aired));
    }
    if !coming.is_empty() {
        shelves.push(shelf("Coming up", 1, coming));
    }
    if !unscheduled.is_empty() {
        shelves.push(shelf("No air time", 0, unscheduled));
    }
    (hero, shelves)
}

fn hero_of(row: &NextUp, a: &Airing, art: &Art, now: i64) -> Hero {
    let (poster, has_poster) = art.poster(row.series_id);
    let tone = row.series_id.and_then(|id| art.tone(id));
    let episode = match a.season.filter(|s| *s > 1) {
        Some(s) => format!("S{s} Episode {}", a.episode),
        None => format!("Episode {}", a.episode),
    };
    Hero {
        show: true,
        id: id(row.series_id),
        title: row.title.clone().into(),
        poster,
        has_poster,
        initials: format::initials(&row.title).into(),
        episode: format!("{episode}  {}", format::when(a.at, now)).into(),
        countdown: format::countdown(a.at - now).into(),
        band: posters::band(tone),
        ink: posters::ink(tone),
        subscribed: row.subscribed,
    }
}

/// The calendar's week strip and the picked day's episodes.
pub struct CalendarView {
    pub label: String,
    pub note: String,
    pub current: bool,
    pub days: Vec<Day>,
    pub day_title: String,
    pub day_note: String,
    pub tiles: Vec<Tile>,
}

pub fn calendar(cache: Option<&Cache>, anilist: &AniListCache, art: &Art, now: i64, week: i64, day: usize) -> CalendarView {
    let monday = calendar::week_start(now, week);
    let today = calendar::today(now);
    let dates: Vec<NaiveDate> = (0..7).map(|i| monday + Days::new(i)).collect();
    let bounds = |d: NaiveDate| (calendar::midnight(d), calendar::midnight(d + Days::new(1)));
    let (from, _) = bounds(dates[0]);
    let (_, to) = bounds(dates[6]);
    let entries = cache.map(|c| calendar::entries(c, anilist, from, to, now)).unwrap_or_default();

    let days = dates
        .iter()
        .map(|&d| {
            let (start, end) = bounds(d);
            Day {
                weekday: d.format("%a").to_string().into(),
                date: d.day().to_string().into(),
                count: entries.iter().filter(|e| e.at >= start && e.at < end).count() as i32,
                today: d == today,
            }
        })
        .collect();

    let picked = dates[day.min(6)];
    let (start, end) = bounds(picked);
    let tiles: Vec<Tile> = entries
        .iter()
        .filter(|e| e.at >= start && e.at < end)
        .map(|e| {
            let mut t = tile(art, e.series_id, &e.title);
            t.badge = format::clock(e.at).into();
            let code = format::episode_code(e.season, e.episode);
            let (ledger, tone) = match e.state {
                State::Upcoming => (format!("{code} {}", format::countdown(e.at - now)), 1),
                State::Missing => (format!("{code} not on disk"), 2),
                State::OnDisk { watched: true } => (format!("{code} watched"), 0),
                State::OnDisk { watched: false } => (format!("{code} on disk"), 3),
            };
            t.ledger = ledger.into();
            t.ledger_tone = tone;
            t
        })
        .collect();

    let note = match week {
        0 => "This week".to_string(),
        1 => "Next week".to_string(),
        -1 => "Last week".to_string(),
        n if n > 1 => format!("In {n} weeks"),
        n => format!("{} weeks ago", -n),
    };
    let label = format!("{} to {}", dates[0].format("%-d %b"), dates[6].format("%-d %b"));
    let day_title = if picked == today { format!("Today, {}", picked.format("%-d %B")) } else { picked.format("%A %-d %B").to_string() };
    let day_note = match tiles.len() {
        0 => String::new(),
        1 => "1 episode".to_string(),
        n => format!("{n} episodes"),
    };
    CalendarView { label, note, current: week == 0, days, day_title, day_note, tiles }
}

fn library_tile(t: &LibraryTile, art: &Art, now: i64) -> Tile {
    let mut out = tile(art, Some(t.id), &t.title);
    if let Some(next) = &t.next {
        out.badge = code(next).into();
        out.ledger = format::countdown(next.at - now).into();
        out.ledger_tone = 1;
    } else if t.movie {
        out.ledger = if t.watched > 0 { "Watched".into() } else { "Movie".into() };
    } else if let Some(total) = t.total.filter(|n| *n > 0) {
        out.ledger = format!("{} of {total} watched", t.watched.min(total)).into();
    } else if t.on_disk > 0 {
        out.ledger = format!("{} on disk", t.on_disk).into();
    }
    out.progress = if t.movie {
        if t.watched > 0 { 1.0 } else { 0.0 }
    } else {
        match t.total.filter(|n| *n > 0).or(Some(t.on_disk).filter(|n| *n > 0)) {
            Some(of) => (t.watched as f32 / of as f32).min(1.0),
            None => -1.0,
        }
    };
    out
}

/// The library as two shelves: airing, then everything else.
pub fn library(cache: Option<&Cache>, anilist: &AniListCache, art: &Art, now: i64) -> Vec<ShelfRows> {
    let Some(cache) = cache else { return Vec::new() };
    let (airing, rest): (Vec<_>, Vec<_>) = library::tiles(cache, anilist, now).into_iter().partition(|t| t.airing);
    let mut shelves = Vec::new();
    if !airing.is_empty() {
        shelves.push(shelf("Airing", 1, airing.iter().map(|t| library_tile(t, art, now)).collect()));
    }
    if !rest.is_empty() {
        shelves.push(shelf("Library", 0, rest.iter().map(|t| library_tile(t, art, now)).collect()));
    }
    shelves
}

/// One series page: the band and the episode rows.
pub fn series(cache: Option<&Cache>, anilist: &AniListCache, art: &Art, series_id: u64, now: i64) -> Option<(SeriesHead, Vec<EpisodeLine>)> {
    let cache = cache?;
    let page = library::detail(cache, anilist, series_id, now)?;
    let (poster, has_poster) = art.poster(Some(series_id));
    let tone = art.tone(series_id);
    let movie = page.facts.first().is_some_and(|f| f == "Movie");

    let (progress_text, progress) = if movie {
        let seen = page.episodes.iter().any(|e| e.watched) || page.watched > 0;
        (if seen { "Watched" } else { "Not watched yet" }.to_string(), if seen { 1.0 } else { 0.0 })
    } else {
        match page.total.filter(|n| *n > 0) {
            Some(total) => (format!("{} of {total} watched", page.watched.min(total)), page.watched.min(total) as f32 / total as f32),
            None if page.watched > 0 => (format!("{} watched", page.watched), -1.0),
            None => (String::new(), -1.0),
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

    let (rule, rule_note) = match &page.subscription {
        Some(sub) => {
            let mut rule = format!("Subscribed with anirss: {}", if sub.query.is_empty() { &sub.name } else { &sub.query });
            if !sub.enabled {
                rule.push_str(". The rule is paused.");
            } else if sub.complete {
                rule.push_str(". Complete.");
            }
            let note = match missing.len() {
                0 => String::new(),
                1 => format!("Episode {} aired and is not on disk yet.", missing[0]),
                n => format!("{n} aired episodes are not on disk yet."),
            };
            (rule, note)
        }
        None => (String::new(), String::new()),
    };

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
                let ep = match a.season.filter(|s| *s > 1) {
                    Some(s) => format!("S{s} Episode {}", a.episode),
                    None => format!("Episode {}", a.episode),
                };
                format!("{ep} airs in {}", format::countdown_full(a.at - now))
            })
            .unwrap_or_default()
            .into(),
        progress_text: progress_text.into(),
        progress,
        rule: rule.into(),
        rule_note: rule_note.into(),
    };

    let mut lines: Vec<((u32, f64), EpisodeLine)> = page
        .episodes
        .iter()
        .map(|e| {
            let title = e.title.clone().unwrap_or_else(|| {
                if movie { page.title.clone() } else { format!("Episode {}", e.number) }
            });
            let line = EpisodeLine {
                code: e.code.clone().into(),
                title: title.into(),
                date: e.aired_at.map(format::day_month).unwrap_or_default().into(),
                state: if e.watched { 0 } else { 1 },
            };
            ((e.season.unwrap_or(1), e.number), line)
        })
        .collect();
    for n in &missing {
        let line = EpisodeLine {
            code: format::episode_code(None, *n).into(),
            title: "Not on disk".into(),
            date: "aired".into(),
            state: 3,
        };
        lines.push(((1, f64::from(*n)), line));
    }
    lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut lines: Vec<EpisodeLine> = lines.into_iter().map(|(_, l)| l).collect();
    for a in page.upcoming.iter().take(6) {
        let soon = a.at - now < 7 * 86_400;
        lines.push(EpisodeLine {
            code: code(a).into(),
            title: format!("Episode {}", a.episode).into(),
            date: if soon { format::countdown(a.at - now) } else { format::day_month(a.at) }.into(),
            state: 2,
        });
    }
    Some((head, lines))
}

/// Downloads: the torrents, whether the host could read them, and the subscriptions.
pub fn downloads(cache: Option<&Cache>, anilist: &AniListCache, now: i64) -> (Vec<TorrentRow>, bool, Vec<RuleRow>) {
    let Some(cache) = cache else { return (Vec::new(), true, Vec::new()) };
    let (torrents, known) = match library::torrents(&cache.snapshot, 24) {
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
    };

    // A rule's own next airing, or its series' schedule when it has one.
    let next_up: HashMap<String, NextUp> =
        schedule::next_up(cache, anilist, now).into_iter().map(|r| (r.key.clone(), r)).collect();
    let mut rules: Vec<(u8, String, RuleRow)> = cache
        .snapshot
        .subscriptions
        .iter()
        .map(|sub| {
            let key = match sub.series_id {
                Some(id) if cache.snapshot.series.iter().any(|s| s.id == id) => format!("s{id}"),
                _ => format!("r{}", sub.name),
            };
            let row = next_up.get(&key);
            let missing = row.map(|r| r.missing.len()).unwrap_or(0);
            let mut facts = vec![format!("{} files", sub.file_count)];
            match row.map(|r| &r.headline) {
                Some(Headline::Upcoming(a)) => facts.push(format!("next {} {}", code(a), format::countdown(a.at - now))),
                _ => {
                    if let Some(then) = row.and_then(|r| r.then.as_ref()) {
                        facts.push(format!("next {} {}", code(then), format::countdown(then.at - now)));
                    }
                }
            }
            if missing > 0 {
                facts.push(format!("{missing} not found"));
            }
            let (state, tone, order) = if !sub.enabled {
                ("Paused", 0, 2)
            } else if sub.complete {
                ("Complete", 0, 1)
            } else if missing > 0 {
                ("Behind", 2, 0)
            } else {
                ("Airing", 1, 0)
            };
            let row = RuleRow {
                id: id(sub.series_id),
                title: sub.title.clone().into(),
                state: state.into(),
                facts: facts.join("  ").into(),
                tone,
            };
            (order, sub.title.to_lowercase(), row)
        })
        .collect();
    rules.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    (torrents, known, rules.into_iter().map(|(_, _, r)| r).collect())
}

pub fn problems(cache: Option<&Cache>) -> Vec<SharedString> {
    cache.map(|c| c.snapshot.problems.iter().map(|p| SharedString::from(p.as_str())).collect()).unwrap_or_default()
}
