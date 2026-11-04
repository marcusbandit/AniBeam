//! Schedule notes (#418): what the user knows about a series' schedule that
//! its provider does not say, read by rule from a pasted release list and
//! kept per series and season. The predictor in `cards::upcoming` reads
//! them; this module parses, stores and loads them.

use std::sync::LazyLock;

use regex::Regex;
use rusqlite::{Connection, params};
use time::{Date, Month, PrimitiveDateTime, Time};

use crate::contract::*;
use crate::core::Core;
use crate::library::cards;
use crate::time as clock;

/// When the episodes a note covers land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// Unix seconds.
    On(i64),
    /// Some time in this year.
    Year(i32),
    /// No date known.
    Unknown,
}

/// One note as the snapshot holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteRow {
    pub first: u32,
    pub last: u32,
    pub when: When,
}

impl NoteRow {
    pub fn covers(&self, n: u32) -> bool {
        (self.first..=self.last).contains(&n)
    }

    pub fn from_wire(n: &ScheduleNote) -> NoteRow {
        let when = match (n.at, n.year) {
            (Some(at), _) => When::On(clock::to_secs(at)),
            (None, Some(y)) => When::Year(y),
            (None, None) => When::Unknown,
        };
        NoteRow {
            first: n.first.min(n.last),
            last: n.first.max(n.last),
            when,
        }
    }

    pub fn to_wire(self) -> ScheduleNote {
        let (at, year) = match self.when {
            When::On(at) => (Some(clock::from_secs(at)), None),
            When::Year(y) => (None, Some(y)),
            When::Unknown => (None, None),
        };
        ScheduleNote {
            first: self.first,
            last: self.last,
            at,
            year,
        }
    }
}

/// The most episodes one note may span, so a typo ("Episode 1 - 9999")
/// cannot plan thousands of rows.
const SPAN_MAX: u32 = 500;

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<[^>]*>").unwrap());
static BLOCK_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)</(li|p|div|tr|h[1-6])>|<br\s*/?>").unwrap());
static SEASON: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bseason\s+(\d{1,2})\b|\bs(\d{1,2})\s*e\d{1,4}\b").unwrap());
static EPISODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:\bs\d{1,2}\s*e|\b(?:episodes?|eps?\.?|e)\s*#?\s*)(\d{1,4})(?:\s*(?:-|\x{2013}|\x{2014}|to|through|thru|&|and)\s*(?:episodes?\s*|eps?\.?\s*|e)?(\d{1,4}))?",
    )
    .unwrap()
});
const MONTHS: &str = "jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sep(?:t(?:ember)?)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?";
static ISO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{4})-(\d{1,2})-(\d{1,2})\b").unwrap());
static DAY_MONTH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)\b(\d{{1,2}})(?:st|nd|rd|th)?\s+(?:of\s+)?({MONTHS})\.?,?(?:\s+(\d{{4}}))?\b"
    ))
    .unwrap()
});
static MONTH_DAY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)\b({MONTHS})\.?\s+(\d{{1,2}})(?:st|nd|rd|th)?,?(?:\s+(\d{{4}}))?\b"
    ))
    .unwrap()
});
static YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b((?:19|20)\d{2})\b").unwrap());
static UNDATED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(tbc|tba|tbd|to be (?:confirmed|announced|determined)|unknown|later)\b")
        .unwrap()
});
/// What may follow a number that is a day, not the end of a range.
static DATE_AHEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)^(?:st|nd|rd|th)?\s*(?:of\s+)?(?:{MONTHS})\b|^[/.]\d"
    ))
    .unwrap()
});

/// A pasted release list read into notes, and the season it names, if any.
/// One line per entry; a line with no episode number or no date is skipped.
/// Neighbouring episodes landing at the same moment fold into one note.
pub fn parse(text: &str) -> (Option<u32>, Vec<NoteRow>) {
    let text = BLOCK_END.replace_all(text, "\n");
    let text = TAG.replace_all(&text, " ");
    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&ndash;", "-")
        .replace("&mdash;", "-");
    let season = SEASON.captures(&text).and_then(|c| {
        c.get(1)
            .or_else(|| c.get(2))
            .and_then(|m| m.as_str().parse().ok())
    });
    // A day and month with no year borrow the first year the text names
    let fallback_year = YEAR.captures(&text).and_then(|c| c[1].parse::<i32>().ok());

    let mut rows: Vec<NoteRow> = Vec::new();
    for line in text.lines() {
        let Some((first, last, rest)) = episodes_of(line) else {
            continue;
        };
        let Some(when) = when_of(rest, fallback_year) else {
            continue;
        };
        rows.push(NoteRow { first, last, when });
    }
    rows.sort_by_key(|r| r.first);
    (season, fold(rows))
}

/// The episode or range a line names, and the text after it.
fn episodes_of(line: &str) -> Option<(u32, u32, &str)> {
    let c = EPISODE.captures(line)?;
    let first: u32 = c[1].parse().ok()?;
    let whole = c.get(0)?;
    if let Some(end) = c.get(2) {
        let last: u32 = end.as_str().parse().ok()?;
        let after = &line[end.end()..];
        if last > first && last - first < SPAN_MAX && !DATE_AHEAD.is_match(after.trim_start()) {
            return Some((first, last, after));
        }
        // "Episode 1 - 14 October": the number after the dash is a day
        let one = c.get(1)?;
        return Some((first, first, &line[one.end()..]));
    }
    Some((first, first, &line[whole.end()..]))
}

/// When the rest of a line says the episodes land: a date, a year, or no
/// date at all. None when it says nothing a schedule can use.
fn when_of(rest: &str, fallback_year: Option<i32>) -> Option<When> {
    if let Some(c) = ISO.captures(rest) {
        let month = Month::try_from(c[2].parse::<u8>().ok()?).ok()?;
        return day(c[1].parse().ok()?, month, c[3].parse().ok()?);
    }
    if let Some(c) = DAY_MONTH.captures(rest) {
        let year = c
            .get(3)
            .and_then(|y| y.as_str().parse().ok())
            .or(fallback_year)?;
        return day(year, month_of(&c[2])?, c[1].parse().ok()?);
    }
    if let Some(c) = MONTH_DAY.captures(rest) {
        let year = c
            .get(3)
            .and_then(|y| y.as_str().parse().ok())
            .or(fallback_year)?;
        return day(year, month_of(&c[1])?, c[2].parse().ok()?);
    }
    if let Some(c) = YEAR.captures(rest) {
        return Some(When::Year(c[1].parse().ok()?));
    }
    UNDATED.is_match(rest).then_some(When::Unknown)
}

/// Noon UTC on a date: a release list names the day, rarely the hour.
fn day(year: i32, month: Month, d: u8) -> Option<When> {
    let date = Date::from_calendar_date(year, month, d).ok()?;
    let at = PrimitiveDateTime::new(date, Time::from_hms(12, 0, 0).ok()?)
        .assume_utc()
        .unix_timestamp();
    Some(When::On(at))
}

fn month_of(name: &str) -> Option<Month> {
    let n = match name.get(..3)?.to_ascii_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    Month::try_from(n).ok()
}

/// Neighbouring notes landing at the same moment, as one note each; a later
/// line for an episode already read gives way to the first.
fn fold(rows: Vec<NoteRow>) -> Vec<NoteRow> {
    let mut out: Vec<NoteRow> = Vec::new();
    for r in rows {
        if let Some(prev) = out.last_mut() {
            if r.first <= prev.last {
                if r.last > prev.last && r.when == prev.when {
                    prev.last = r.last;
                }
                continue;
            }
            if r.when == prev.when && r.first == prev.last + 1 {
                prev.last = r.last;
                continue;
            }
        }
        out.push(r);
    }
    out
}

/// Every note of the given series, with its season, by series id.
pub(crate) fn load(
    conn: &Connection,
    series: Option<&[u64]>,
) -> Result<Vec<(u64, Option<u32>, NoteRow)>, CoreError> {
    if series.is_some_and(<[u64]>::is_empty) {
        return Ok(Vec::new());
    }
    let mut sql = "SELECT series_id, season, first, last, at, year FROM schedule_notes".to_string();
    if let Some(ids) = series {
        let list: Vec<String> = ids.iter().map(u64::to_string).collect();
        sql.push_str(&format!(" WHERE series_id IN ({})", list.join(",")));
    }
    sql.push_str(" ORDER BY series_id, season, first");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        let at: Option<i64> = r.get(4)?;
        let year: Option<i64> = r.get(5)?;
        Ok((
            r.get::<_, i64>(0)? as u64,
            r.get::<_, Option<i64>>(1)?.map(|s| s as u32),
            NoteRow {
                first: r.get::<_, i64>(2)? as u32,
                last: r.get::<_, i64>(3)? as u32,
                when: match (at, year) {
                    (Some(at), _) => When::On(at),
                    (None, Some(y)) => When::Year(y as i32),
                    (None, None) => When::Unknown,
                },
            },
        ))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// `Call::ParseSchedule`.
pub fn parse_call(text: &str) -> Reply {
    let (season, notes) = parse(text);
    Reply::Schedule {
        season,
        notes: notes.into_iter().map(NoteRow::to_wire).collect(),
    }
}

/// `Call::SetScheduleNotes`: replaces one season's notes of a series.
pub fn set_notes(
    core: &Core,
    series: u64,
    season: Option<u32>,
    notes: Vec<ScheduleNote>,
) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let card = core
        .store
        .read(|c| cards::cards_for(c, &images_dir, &[series]))?
        .into_iter()
        .next()
        .ok_or(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        })?;
    // `count_season` is the card's own season, Some only for a record
    // numbering within seasons (TVmaze), so AniList keeps NULL.
    let season = card.count_season.map(|own| season.unwrap_or(own));
    let mut rows: Vec<NoteRow> = notes
        .iter()
        .map(NoteRow::from_wire)
        .filter(|n| n.first > 0 && n.last - n.first < SPAN_MAX)
        .collect();
    rows.sort_by_key(|r| r.first);
    let rows = fold(rows);
    let count = rows.len();
    let now = clock::now_secs();
    core.store.write(move |c| {
        let tx = c.transaction()?;
        tx.execute(
            "DELETE FROM schedule_notes WHERE series_id = ?1 AND season IS ?2",
            params![series as i64, season.map(i64::from)],
        )?;
        for r in &rows {
            let (at, year) = match r.when {
                When::On(at) => (Some(at), None),
                When::Year(y) => (None, Some(i64::from(y))),
                When::Unknown => (None, None),
            };
            tx.execute(
                "INSERT INTO schedule_notes (series_id, season, first, last, at, year, set_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    series as i64,
                    season.map(i64::from),
                    i64::from(r.first),
                    i64::from(r.last),
                    at,
                    year,
                    now
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    })?;
    let cards = core
        .store
        .read(|c| cards::cards_for(c, &images_dir, &[series]))?;
    let title = cards.first().map_or_else(String::new, |c| c.title.clone());
    let (said, what) = if count == 0 {
        (
            format!("Cleared the schedule notes of {title}."),
            format!("{title}'s schedule notes cleared"),
        )
    } else {
        (
            format!("Saved {count} schedule notes for {title}."),
            format!("{title}'s schedule set by hand"),
        )
    };
    core.bus.say(
        Level::Info,
        Stage::Library,
        said,
        what,
        EventBody::SeriesChanged { series: cards },
    );
    Ok(Reply::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noon(y: i32, m: u8, d: u8) -> When {
        day(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn reads_a_release_list_with_dates_and_a_tbc_year() {
        let text = "Helluva Boss season 3 Part 1 releases its seven episodes across three Wednesdays in October 2026:\n\
            Episode 1 \u{2013} Wednesday 14 October 2026 (out now)\n\
            Episode 2 \u{2013} Wednesday 14 October 2026 (out now)\n\
            Episode 3 \u{2013} Wednesday 14 October 2026 (out now)\n\
            Episode 4 \u{2013} Wednesday 21 October 2026 (out now)\n\
            Episode 5 \u{2013} Wednesday 21 October 2026 (out now)\n\
            Episode 6 \u{2013} Wednesday 28 October 2026 (out now)\n\
            Episode 7 \u{2013} Wednesday 28 October 2026 (out now)\n\
            Episode 8 - TBC in 2027\n\
            Episode 9 - TBC in 2027\n\
            Episode 15 - TBC in 2027\n";
        let (season, notes) = parse(text);
        assert_eq!(season, Some(3));
        assert_eq!(
            notes,
            vec![
                NoteRow {
                    first: 1,
                    last: 3,
                    when: noon(2026, 10, 14)
                },
                NoteRow {
                    first: 4,
                    last: 5,
                    when: noon(2026, 10, 21)
                },
                NoteRow {
                    first: 6,
                    last: 7,
                    when: noon(2026, 10, 28)
                },
                NoteRow {
                    first: 8,
                    last: 9,
                    when: When::Year(2027)
                },
                NoteRow {
                    first: 15,
                    last: 15,
                    when: When::Year(2027)
                },
            ]
        );
    }

    #[test]
    fn reads_the_html_a_browser_copies() {
        let html = "<ul><li><strong>Episode 7 \u{2013} Wednesday 28 October 2026 (out now)</strong></li><li><strong>Episode 8 - TBC in 2027 </strong></li></ul>";
        let (_, notes) = parse(html);
        assert_eq!(
            notes,
            vec![
                NoteRow {
                    first: 7,
                    last: 7,
                    when: noon(2026, 10, 28)
                },
                NoteRow {
                    first: 8,
                    last: 8,
                    when: When::Year(2027)
                },
            ]
        );
    }

    #[test]
    fn a_range_and_other_date_shapes() {
        let text = "Episodes 8-15: TBC\nS02E03 2026-12-01\nEp 4: December 8th, 2026\nE5 - 15 Dec";
        let (season, notes) = parse(text);
        assert_eq!(season, Some(2));
        assert_eq!(
            notes,
            vec![
                NoteRow {
                    first: 3,
                    last: 3,
                    when: noon(2026, 12, 1)
                },
                NoteRow {
                    first: 4,
                    last: 4,
                    when: noon(2026, 12, 8)
                },
                NoteRow {
                    first: 5,
                    last: 5,
                    when: noon(2026, 12, 15)
                },
                NoteRow {
                    first: 8,
                    last: 15,
                    when: When::Unknown
                },
            ]
        );
    }

    #[test]
    fn a_day_after_a_dash_is_not_a_range() {
        let (_, notes) = parse("Episode 1 - 14 October 2026");
        assert_eq!(
            notes,
            vec![NoteRow {
                first: 1,
                last: 1,
                when: noon(2026, 10, 14)
            }]
        );
    }

    #[test]
    fn lines_with_no_date_or_no_episode_are_skipped() {
        let (_, notes) = parse("Episode 3 is the best one\nComing 2027 maybe");
        assert!(notes.is_empty());
    }
}
