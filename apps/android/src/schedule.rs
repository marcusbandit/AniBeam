//! Episodes and dates (#746): the desktop's schedule sheet (#418, #432) on the phone. A season
//! is told what its provider does not say: every episode is a tile with its date under its
//! number, the last one is marked, and nothing past it is predicted. Tap tiles (Pick a run
//! takes everything between two taps, the desktop's shift-click), then one row of actions
//! dates them, ends the season there or clears them. Paste a list reads a release list onto
//! the tiles. Save runs the desktop's own calls: `SetScheduleNotes`, then `SetEpisodeCount`
//! when the end moved. The desktop's Ask an agent is left out: a phone has no coding agent.
//!
//! The draft lives here, in Rust; the sheet draws the tiles this module builds and calls back
//! for every tap.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use anibeam_core::library::schedule::{NoteRow, When};
use anibeam_core::time::to_secs;
use anibeam_core::{Call, Provider, Reply, SeriesDetail, SeriesKind};
use chrono::{Local, TimeZone};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, DateTile, ScheduleData, ShowData};

/// The most tiles the sheet draws, the desktop's limit.
const MOST: u32 = 200;
const WEEK: i64 = 7 * 86_400;

/// The sheet number in `ShowData.sheet`.
pub const SHEET: i32 = 7;

/// What the open show's page says, kept for the sheet to open from.
#[derive(Clone, Debug, Default, PartialEq)]
struct Page {
    series: u64,
    title: String,
    season: Option<u32>,
    notes: Vec<NoteRow>,
    up_to: u32,
    aired: Vec<u32>,
    end_at: u32,
    /// What the record says, by episode: its air date and whether that is a guess.
    known: BTreeMap<u32, (i64, bool)>,
    site: String,
}

/// The sheet's draft, the desktop sheet's properties.
#[derive(Clone, Debug, Default)]
struct Sheet {
    series: u64,
    season: Option<u32>,
    saved: Vec<NoteRow>,
    draft: Vec<NoteRow>,
    up_to: u32,
    aired: Vec<u32>,
    picked: Vec<u32>,
    anchor: u32,
    weekly: bool,
    run: bool,
    typed: Option<When>,
    end_at: u32,
    saved_end: u32,
    known: BTreeMap<u32, (i64, bool)>,
    site: String,
}

thread_local! {
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
    static SHEET_STATE: RefCell<Sheet> = RefCell::new(Sheet::default());
    static TILES: RefCell<Option<Rc<VecModel<DateTile>>>> = const { RefCell::new(None) };
}

/// The desktop's `expand`: one entry an episode.
fn expand(notes: &[NoteRow]) -> BTreeMap<u32, When> {
    let mut by = BTreeMap::new();
    for n in notes {
        for e in n.first..=n.last {
            by.insert(e, n.when);
        }
    }
    by
}

/// The desktop's `collapse`: runs of neighbours with the same date as one note.
fn collapse(by: &BTreeMap<u32, When>) -> Vec<NoteRow> {
    let mut out: Vec<NoteRow> = Vec::new();
    for (&e, &when) in by {
        match out.last_mut() {
            Some(last) if last.last + 1 == e && last.when == when => last.last = e,
            _ => out.push(NoteRow {
                first: e,
                last: e,
                when,
            }),
        }
    }
    out
}

fn note_for(draft: &[NoteRow], ep: u32) -> Option<&NoteRow> {
    draft.iter().find(|n| n.covers(ep))
}

/// 0 none, 1 a day, 2 a month, 3 a year, 4 no date yet: the tile's colour.
fn kind_of(note: Option<&NoteRow>) -> i32 {
    match note.map(|n| n.when) {
        None => 0,
        Some(When::On(_)) => 1,
        Some(When::Month(..)) => 2,
        Some(When::Year(_)) => 3,
        Some(When::Unknown) => 4,
    }
}

fn local(at: i64, pattern: &str) -> String {
    Local
        .timestamp_opt(at, 0)
        .single()
        .map(|t| t.format(pattern).to_string())
        .unwrap_or_default()
}

fn month_name(year: i32, month: u8, pattern: &str) -> String {
    chrono::NaiveDate::from_ymd_opt(year, u32::from(month), 1)
        .map(|d| d.format(pattern).to_string())
        .unwrap_or_else(|| year.to_string())
}

impl Sheet {
    fn open(page: &Page) -> Sheet {
        let draft = collapse(&expand(&page.notes));
        let top = page.notes.iter().map(|n| n.last).max().unwrap_or(0);
        Sheet {
            series: page.series,
            season: page.season,
            saved: page.notes.clone(),
            draft,
            up_to: page.up_to.max(top).clamp(12, MOST),
            aired: page.aired.clone(),
            end_at: page.end_at,
            saved_end: page.end_at,
            known: page.known.clone(),
            site: page.site.clone(),
            ..Sheet::default()
        }
    }

    fn out(&self, ep: u32) -> bool {
        self.aired.contains(&ep)
    }

    /// The record's real dates, without its guesses.
    fn record_dates(&self) -> BTreeMap<u32, i64> {
        self.known
            .iter()
            .filter(|(_, (_, guess))| !guess)
            .map(|(&e, &(at, _))| (e, at))
            .collect()
    }

    /// The desktop's preview of the weekly guesses the core makes from what is set here.
    fn guesses(&self) -> BTreeMap<u32, i64> {
        let mut out = BTreeMap::new();
        let by = expand(&self.draft);
        let mut dated = self.record_dates();
        for (&e, &when) in &by {
            if let When::On(at) = when {
                dated.insert(e, at);
            }
        }
        let nums: Vec<u32> = dated.keys().copied().collect();
        let (Some(&first), Some(&last)) = (nums.first(), nums.last()) else {
            return out;
        };
        // The cadence: the first gap between two neighbours, in whole weeks, at least one.
        let mut step = WEEK;
        for pair in nums.windows(2) {
            if pair[1] == pair[0] + 1 {
                let gap = (dated[&pair[1]] - dated[&pair[0]]) as f64 / WEEK as f64;
                step = (gap.round() as i64).max(1) * WEEK;
                break;
            }
        }
        let to = if self.end_at > 0 {
            self.end_at
        } else {
            last + 1
        };
        let mut anchor: Option<(u32, i64)> = None;
        for n in first..=to {
            if let Some(&at) = dated.get(&n) {
                anchor = Some((n, at));
                continue;
            }
            if by.contains_key(&n) {
                anchor = None;
                continue;
            }
            if let Some((a, at)) = anchor
                && !self.out(n)
            {
                out.insert(n, at + i64::from(n - a) * step);
            }
        }
        out
    }

    /// What a tile says under its number.
    fn tile_text(
        &self,
        ep: u32,
        record: &BTreeMap<u32, i64>,
        guesses: &BTreeMap<u32, i64>,
    ) -> String {
        if self.end_at > 0 && ep > self.end_at {
            return String::new();
        }
        let note = note_for(&self.draft, ep);
        if note.is_none()
            && let Some(&at) = record.get(&ep)
            && !self.out(ep)
        {
            return local(at, "%-d %b");
        }
        if note.is_none()
            && let Some(&at) = guesses.get(&ep)
        {
            return format!("≈ {}", local(at, "%-d %b"));
        }
        if let Some(n) = note {
            return match n.when {
                When::On(at) => local(at, "%-d %b"),
                When::Month(y, m) => month_name(y, m, "%b %Y"),
                When::Year(y) => y.to_string(),
                When::Unknown => "No date".to_string(),
            };
        }
        if self.end_at == ep {
            return "Last".to_string();
        }
        if self.out(ep) {
            "Out".to_string()
        } else {
            String::new()
        }
    }

    fn tiles(&self) -> Vec<DateTile> {
        let record = self.record_dates();
        let guesses = self.guesses();
        (1..=self.up_to)
            .map(|ep| {
                let note = note_for(&self.draft, ep);
                DateTile {
                    ep: ep as i32,
                    date: self.tile_text(ep, &record, &guesses).into(),
                    kind: kind_of(note),
                    picked: self.picked.contains(&ep),
                    out: self.out(ep),
                    end: self.end_at == ep,
                    past: self.end_at > 0 && ep > self.end_at,
                    record: record.contains_key(&ep) && !self.out(ep),
                }
            })
            .collect()
    }

    /// The footer's one line: where the season ends and how much is dated.
    fn summary(&self) -> String {
        let set: u32 = self.draft.iter().map(|n| n.last - n.first + 1).sum();
        let mut bits = Vec::new();
        if self.end_at > 0 {
            bits.push(format!("Ends at episode {}", self.end_at));
        }
        let from_record = self
            .record_dates()
            .keys()
            .filter(|&&e| !self.out(e) && note_for(&self.draft, e).is_none())
            .count();
        if from_record > 0 {
            let site = if self.site.is_empty() {
                "the record"
            } else {
                &self.site
            };
            bits.push(format!("{from_record} dated by {site}"));
        }
        if set > 0 {
            bits.push(format!("{set} set by you"));
        }
        let guessed = self.guesses().len();
        if guessed > 0 {
            bits.push(format!("{guessed} guessed weekly"));
        }
        if bits.is_empty() {
            "Nothing set yet".to_string()
        } else {
            bits.join(" · ")
        }
    }

    fn dirty(&self) -> bool {
        self.draft != collapse(&expand(&self.saved)) || self.end_at != self.saved_end
    }

    /// A tap toggles one; with Pick a run on, a tap picks the run from the last tap.
    fn tap(&mut self, ep: u32) {
        if self.run && self.anchor > 0 {
            let (a, b) = (self.anchor.min(ep), self.anchor.max(ep));
            for e in a..=b {
                if !self.picked.contains(&e) {
                    self.picked.push(e);
                }
            }
        } else if let Some(at) = self.picked.iter().position(|&e| e == ep) {
            self.picked.remove(at);
        } else {
            self.picked.push(ep);
        }
        self.anchor = ep;
        self.picked.sort_unstable();
    }

    /// Gives the picked episodes a date; weekly spreads a day one week apart.
    fn apply(&mut self, when: When) {
        if self.picked.is_empty() {
            return;
        }
        let mut by = expand(&self.draft);
        for (i, &ep) in self.picked.iter().enumerate() {
            let when = match when {
                When::On(at) if self.weekly => When::On(at + i as i64 * WEEK),
                other => other,
            };
            by.insert(ep, when);
        }
        self.draft = collapse(&by);
        self.picked.clear();
        self.typed = None;
    }

    fn unset(&mut self) {
        let mut by = expand(&self.draft);
        for ep in &self.picked {
            by.remove(ep);
        }
        self.draft = collapse(&by);
        self.picked.clear();
    }

    /// Marks the one picked episode as the season's last, or clears the mark when it is.
    fn mark_last(&mut self) {
        let [ep] = self.picked[..] else { return };
        self.end_at = if self.end_at == ep { 0 } else { ep };
        // Nothing past the end keeps a date.
        let mut by = expand(&self.draft);
        if self.end_at > 0 {
            by.retain(|&e, _| e <= self.end_at);
        }
        self.draft = collapse(&by);
        self.picked.clear();
    }

    /// A paste replaces the draft with what it reads; picking then fixes what it missed.
    fn pasted(&mut self, notes: &[NoteRow]) {
        self.draft = collapse(&expand(notes));
        let top = self.draft.iter().map(|n| n.last).max().unwrap_or(0);
        // A list naming episodes past the marked end moves the end out of their way.
        if self.end_at > 0 && top > self.end_at {
            self.end_at = 0;
        }
        self.up_to = self.up_to.max(top + 1).min(MOST);
    }

    fn more(&mut self) {
        self.up_to = (self.up_to + 12).min(MOST);
    }
}

/// What the open show's page holds, as the sheet opens from it: the desktop's
/// `openSchedule`. Rows run over the card's season.
fn page(detail: &SeriesDetail) -> Page {
    let card = &detail.card;
    let season = card.count_season;
    let ours = |s: Option<u32>| season.is_none_or(|own| s.unwrap_or(1) == own);
    let notes: Vec<NoteRow> = detail
        .schedule_notes
        .iter()
        .map(NoteRow::from_wire)
        .collect();
    let noted = |n: u32| notes.iter().any(|x| x.covers(n));
    let mut top = 0;
    let mut aired = Vec::new();
    let mut known = BTreeMap::new();
    let mut mark = |n: u32, at: Option<i64>, guess: bool| {
        // A date from the user's own notes is not the record's.
        if let Some(at) = at
            && !noted(n)
        {
            known.insert(n, (at, guess));
        }
    };
    for e in detail.episodes.iter().filter(|e| ours(e.season)) {
        let n = e.number.max(0.0).floor() as u32;
        top = top.max(n);
        aired.push(n);
        mark(n, e.air_date.map(to_secs), false);
    }
    for a in detail.absent.iter().filter(|a| ours(a.season)) {
        top = top.max(a.number);
        aired.push(a.number);
        mark(a.number, a.air_date.map(to_secs), false);
    }
    for u in detail.upcoming.iter().filter(|u| ours(u.season)) {
        top = top.max(u.number);
        mark(u.number, Some(to_secs(u.at)), u.predicted);
    }
    aired.sort_unstable();
    aired.dedup();
    let end_at = card.episodes_set.unwrap_or(0);
    let up_to = card
        .total_episodes
        .unwrap_or(0)
        .max(top + 8)
        .max(16)
        .max(end_at + 2);
    let provider = card
        .match_info
        .as_ref()
        .map(|m| m.provider)
        .unwrap_or(Provider::Anilist);
    Page {
        series: card.id,
        title: card.title.clone(),
        season,
        notes,
        up_to,
        aired,
        end_at,
        known,
        site: provider_name(provider).to_string(),
    }
}

/// A provider as the desktop names it, `frame.providerName`.
fn provider_name(p: Provider) -> &'static str {
    match p {
        Provider::Anilist => "AniList",
        Provider::Mal => "MyAnimeList",
        Provider::Tmdb => "TMDB",
        Provider::Tvmaze => "TVmaze",
    }
}

/// The page's line on what the user told the season (#432): the episode count set by hand and
/// a break in the schedule, the desktop's `countRow` text. Empty when neither is set.
fn told_line(detail: &SeriesDetail) -> String {
    let card = &detail.card;
    let held = detail
        .schedule_notes
        .iter()
        .find(|n| n.at.is_none())
        .map(|n| {
            let eps = if n.first == n.last {
                format!("Episode {}", n.first)
            } else {
                format!("Episodes {} to {}", n.first, n.last)
            };
            let due = match (n.year, n.month.and_then(|m| u8::try_from(m).ok())) {
                (None, _) => " not dated yet".to_string(),
                (Some(y), Some(m)) if (1..=12).contains(&m) => {
                    format!(" due in {}", month_name(y, m, "%B %Y"))
                }
                (Some(y), _) => format!(" due in {y}"),
            };
            eps + &due
        })
        .unwrap_or_default();
    let Some(set) = card.episodes_set else {
        return if held.is_empty() {
            held
        } else {
            format!("{held}, set by you")
        };
    };
    let gave_way = !card.episodes_set_applied;
    if gave_way && held.is_empty() {
        return format!("You set {set} episodes; that no longer applies");
    }
    let count = if gave_way {
        String::new()
    } else {
        match card.count_season {
            Some(s) => format!("Season {s} ends at episode {set}"),
            None => format!("Ends at episode {set}"),
        }
    };
    let words: Vec<&str> = [count.as_str(), held.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    format!("{}, set by you", words.join(" · "))
}

/// Keeps the open show's page for the sheet to open from, and puts its line on the page.
pub fn track(window: &AppWindow, detail: Option<&SeriesDetail>) {
    let show = window.global::<ShowData>();
    let Some(detail) = detail else {
        PAGE.with(|p| *p.borrow_mut() = None);
        show.set_told(Default::default());
        show.set_movie(false);
        return;
    };
    let fresh = page(detail);
    PAGE.with(|p| {
        let mut p = p.borrow_mut();
        if p.as_ref() != Some(&fresh) {
            *p = Some(fresh);
        }
    });
    show.set_told(told_line(detail).into());
    show.set_movie(detail.card.kind == SeriesKind::Movie);
}

/// Draws the sheet from the draft.
fn draw(window: &AppWindow) {
    let data = window.global::<ScheduleData>();
    SHEET_STATE.with(|s| {
        let s = s.borrow();
        TILES.with(|t| {
            if let Some(model) = t.borrow().as_ref() {
                crate::app::update(model, s.tiles());
            }
        });
        let n = s.picked.len();
        data.set_picked_count(n as i32);
        data.set_picked_word(
            match s.picked[..] {
                [one] => format!("Episode {one}"),
                _ => format!("{n} episodes"),
            }
            .into(),
        );
        data.set_typed_ok(s.typed.is_some());
        data.set_weekly(s.weekly);
        data.set_run(s.run);
        data.set_is_last(matches!(s.picked[..], [one] if one == s.end_at));
        data.set_can_clear(s.picked.iter().any(|&e| note_for(&s.draft, e).is_some()));
        data.set_summary(s.summary().into());
        data.set_dirty(s.dirty());
        data.set_more(s.up_to < MOST);
    });
}

/// Reads text by the core's own rules, the call the desktop's paste box and date field run.
fn parse(text: &str) -> Option<(Option<u32>, Vec<NoteRow>)> {
    let core = crate::local::core()?;
    match core.call(Call::ParseSchedule {
        text: text.to_string(),
    }) {
        Ok(Reply::Schedule { season, notes }) => {
            Some((season, notes.iter().map(NoteRow::from_wire).collect()))
        }
        _ => None,
    }
}

fn with_sheet(window: &AppWindow, f: impl FnOnce(&mut Sheet)) {
    SHEET_STATE.with(|s| f(&mut s.borrow_mut()));
    draw(window);
}

/// Hooks the sheet's callbacks to the draft and the core.
pub fn install(window: &AppWindow) {
    let data = window.global::<ScheduleData>();
    let tiles = Rc::new(VecModel::default());
    data.set_tiles(ModelRc::from(tiles.clone()));
    TILES.with(|t| *t.borrow_mut() = Some(tiles));
    let weak = window.as_weak();
    let on = move |f: fn(&mut Sheet)| {
        let weak = weak.clone();
        move || {
            if let Some(window) = weak.upgrade() {
                with_sheet(&window, f);
            }
        }
    };
    window.global::<ShowData>().on_open_dates({
        let weak = window.as_weak();
        move || {
            let Some(window) = weak.upgrade() else { return };
            let Some(page) = PAGE.with(|p| p.borrow().clone()) else {
                return;
            };
            let data = window.global::<ScheduleData>();
            data.set_line(
                match page.season {
                    Some(s) => format!("{} · Season {s}", page.title),
                    None => page.title.clone(),
                }
                .into(),
            );
            data.set_typed_text(Default::default());
            data.set_paste_text(Default::default());
            data.set_pasting(false);
            with_sheet(&window, |s| *s = Sheet::open(&page));
            window.global::<ShowData>().set_sheet(SHEET);
        }
    });
    data.on_tap({
        let weak = window.as_weak();
        move |ep| {
            let (Some(window), Ok(ep)) = (weak.upgrade(), u32::try_from(ep)) else {
                return;
            };
            with_sheet(&window, |s| s.tap(ep));
        }
    });
    data.on_typed({
        let weak = window.as_weak();
        move |text| {
            let Some(window) = weak.upgrade() else { return };
            let text = text.trim().to_string();
            // A typed date read by the same parser as a paste: "7 Jan", "March 2027", "TBC".
            let typed = if text.is_empty() {
                None
            } else {
                parse(&format!("Episode 1 - {text}"))
                    .and_then(|(_, notes)| notes.first().map(|n| n.when))
            };
            with_sheet(&window, |s| s.typed = typed);
        }
    });
    data.on_set_date({
        let weak = window.as_weak();
        move || {
            let Some(window) = weak.upgrade() else { return };
            let mut done = false;
            with_sheet(&window, |s| {
                if let Some(when) = s.typed {
                    s.apply(when);
                    done = true;
                }
            });
            if done {
                window
                    .global::<ScheduleData>()
                    .set_typed_text(Default::default());
            }
        }
    });
    data.on_no_date(on(|s| s.apply(When::Unknown)));
    data.on_toggle_weekly(on(|s| s.weekly = !s.weekly));
    data.on_toggle_run(on(|s| {
        s.run = !s.run;
        s.anchor = s.picked.last().copied().unwrap_or(0);
    }));
    data.on_mark_last(on(Sheet::mark_last));
    data.on_clear_date(on(Sheet::unset));
    data.on_more_tiles(on(Sheet::more));
    data.on_paste({
        let weak = window.as_weak();
        move |text| {
            let Some(window) = weak.upgrade() else { return };
            if text.trim().is_empty() {
                return;
            }
            if let Some((_, notes)) = parse(&text) {
                with_sheet(&window, |s| s.pasted(&notes));
            }
        }
    });
    data.on_save({
        let weak = window.as_weak();
        move || {
            let Some(window) = weak.upgrade() else { return };
            let s = SHEET_STATE.with(|s| s.borrow().clone());
            window.global::<ShowData>().set_sheet(0);
            save(s);
        }
    });
}

/// Saves the draft through the desktop's calls, off the UI thread, and says how it went.
fn save(s: Sheet) {
    let Some(core) = crate::local::core() else {
        crate::show::say("AniBeam is still starting. Try again in a moment.".to_string());
        return;
    };
    let _ = std::thread::Builder::new()
        .name("schedule".into())
        .spawn(move || {
            let notes = s.draft.iter().map(|n| n.to_wire()).collect::<Vec<_>>();
            let any = !notes.is_empty() || s.end_at > 0;
            let mut line = match core.call(Call::SetScheduleNotes {
                series: s.series,
                // Always the season the page is about, the one its count and tiles belong to.
                season: s.season,
                notes,
            }) {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            };
            if line.is_none() && s.end_at != s.saved_end {
                let count = (s.end_at > 0).then_some(s.end_at);
                if let Err(e) = core.call(Call::SetEpisodeCount {
                    series: s.series,
                    count,
                }) {
                    line = Some(e.to_string());
                }
            }
            let line = line.unwrap_or_else(|| {
                if any {
                    "Saved."
                } else {
                    "Schedule notes cleared."
                }
                .to_string()
            });
            let _ = slint::invoke_from_event_loop(move || crate::show::say(line));
        });
}

/// Unix seconds as the wire's time, for tests.
#[cfg(test)]
fn at(secs: i64) -> std::time::SystemTime {
    anibeam_core::time::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;
    /// A Monday, noon UTC: 2 Nov 2026.
    const T0: i64 = 1_793_620_800;

    fn note(first: u32, last: u32, when: When) -> NoteRow {
        NoteRow { first, last, when }
    }

    fn sheet(up_to: u32) -> Sheet {
        Sheet::open(&Page {
            series: 1,
            up_to,
            ..Page::default()
        })
    }

    #[test]
    fn expand_and_collapse_join_neighbours_with_one_date() {
        let notes = vec![
            note(1, 2, When::Year(2027)),
            note(3, 3, When::Year(2027)),
            note(5, 5, When::Year(2027)),
            note(6, 6, When::Unknown),
        ];
        assert_eq!(
            collapse(&expand(&notes)),
            vec![
                note(1, 3, When::Year(2027)),
                note(5, 5, When::Year(2027)),
                note(6, 6, When::Unknown),
            ]
        );
    }

    #[test]
    fn a_tap_toggles_and_a_run_takes_everything_between_two_taps() {
        let mut s = sheet(12);
        s.tap(3);
        s.tap(3);
        assert!(s.picked.is_empty());
        s.tap(2);
        s.run = true;
        s.tap(6);
        assert_eq!(s.picked, vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn weekly_spreads_a_day_one_week_apart() {
        let mut s = sheet(12);
        for e in [4, 5, 6] {
            s.tap(e);
        }
        s.weekly = true;
        s.apply(When::On(T0));
        assert_eq!(
            s.draft,
            vec![
                note(4, 4, When::On(T0)),
                note(5, 5, When::On(T0 + WEEK)),
                note(6, 6, When::On(T0 + 2 * WEEK)),
            ]
        );
        assert!(s.picked.is_empty());
        assert!(s.dirty());
    }

    #[test]
    fn the_last_episode_drops_every_date_past_it_and_a_second_tap_clears_it() {
        let mut s = sheet(12);
        s.draft = vec![note(8, 11, When::Year(2027))];
        s.tap(9);
        s.mark_last();
        assert_eq!(s.end_at, 9);
        assert_eq!(s.draft, vec![note(8, 9, When::Year(2027))]);
        s.tap(9);
        s.mark_last();
        assert_eq!(s.end_at, 0);
    }

    #[test]
    fn clearing_picked_episodes_keeps_the_rest() {
        let mut s = sheet(12);
        s.draft = vec![note(1, 4, When::Unknown)];
        s.tap(2);
        s.unset();
        assert_eq!(
            s.draft,
            vec![note(1, 1, When::Unknown), note(3, 4, When::Unknown)]
        );
    }

    #[test]
    fn guesses_run_weekly_from_the_last_date_to_the_end_and_stop_at_a_held_note() {
        let mut s = sheet(12);
        s.known.insert(1, (T0, false));
        s.known.insert(2, (T0 + WEEK, false));
        s.aired = vec![1, 2];
        s.end_at = 5;
        let g = s.guesses();
        assert_eq!(g.get(&3), Some(&(T0 + 2 * WEEK)));
        assert_eq!(g.get(&5), Some(&(T0 + 4 * WEEK)));
        assert_eq!(g.get(&6), None);
        // A note with no day holds the run until the next date.
        s.draft = vec![note(4, 4, When::Unknown)];
        let g = s.guesses();
        assert_eq!(g.get(&3), Some(&(T0 + 2 * WEEK)));
        assert_eq!(g.get(&4), None);
        assert_eq!(g.get(&5), None);
    }

    #[test]
    fn the_record_s_own_guesses_are_not_dates() {
        let mut s = sheet(12);
        s.known.insert(3, (T0, true));
        assert!(s.record_dates().is_empty());
        assert_eq!(s.summary(), "Nothing set yet");
    }

    #[test]
    fn the_summary_counts_what_is_set_where() {
        let mut s = sheet(12);
        s.site = "TVmaze".into();
        s.known.insert(1, (T0, false));
        s.draft = vec![note(3, 4, When::Year(2027))];
        s.end_at = 4;
        assert_eq!(
            s.summary(),
            "Ends at episode 4 · 1 dated by TVmaze · 2 set by you · 1 guessed weekly"
        );
    }

    #[test]
    fn a_paste_past_the_end_moves_the_end_and_grows_the_grid() {
        let mut s = sheet(12);
        s.end_at = 10;
        s.pasted(&[note(14, 15, When::On(T0 + DAY))]);
        assert_eq!(s.end_at, 0);
        assert_eq!(s.up_to, 16);
    }

    #[test]
    fn tiles_say_the_date_kind_and_end() {
        let mut s = sheet(12);
        s.aired = vec![1];
        s.end_at = 6;
        s.draft = vec![note(4, 4, When::Year(2027)), note(5, 5, When::Unknown)];
        let tiles = s.tiles();
        assert_eq!(tiles.len(), 12);
        assert_eq!(tiles[0].date, "Out");
        assert_eq!(tiles[3].date, "2027");
        assert_eq!(tiles[3].kind, 3);
        assert_eq!(tiles[4].date, "No date");
        assert_eq!(tiles[5].date, "Last");
        assert!(tiles[5].end);
        assert!(tiles[6].past);
        assert_eq!(tiles[6].date, "");
    }

    #[test]
    fn the_wire_round_trips_a_day() {
        let n = note(2, 3, When::On(T0));
        assert_eq!(n.to_wire().at, Some(at(T0)));
        assert_eq!(NoteRow::from_wire(&n.to_wire()), n);
    }
}
