//! Schedule notes (#418): what the user knows about a series' schedule that
//! its provider does not say, read by rule from a pasted release list and
//! kept per series and season. The predictor in `cards::upcoming` reads
//! them; this module parses, stores and loads them.

use rusqlite::{Connection, params};

use crate::contract::*;
use crate::core::Core;
use crate::library::cards;
use crate::library::schedule_text::{SPAN_MAX, fold, parse};
use crate::time as clock;

/// The highest episode number a note may name.
const EPISODE_MAX: u32 = 10_000;

/// When the episodes a note covers land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    /// Unix seconds.
    On(i64),
    /// Some time in this month: the year, then the month 1 to 12.
    Month(i32, u8),
    /// Some time in this year.
    Year(i32),
    /// No date known.
    Unknown,
}

impl When {
    /// The note's `at`, `year` and `month` columns, the one mapping both
    /// ways between them and `When`.
    pub fn columns(self) -> (Option<i64>, Option<i32>, Option<u8>) {
        match self {
            When::On(at) => (Some(at), None, None),
            When::Month(y, m) => (None, Some(y), Some(m)),
            When::Year(y) => (None, Some(y), None),
            When::Unknown => (None, None, None),
        }
    }

    /// The reverse of `columns`; a month outside 1 to 12 reads as the year.
    pub fn from_columns(at: Option<i64>, year: Option<i32>, month: Option<u8>) -> When {
        match (at, year, month) {
            (Some(at), _, _) => When::On(at),
            (None, Some(y), Some(m)) if (1..=12).contains(&m) => When::Month(y, m),
            (None, Some(y), _) => When::Year(y),
            (None, None, _) => When::Unknown,
        }
    }
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
        let when = When::from_columns(
            n.at.map(clock::to_secs),
            n.year,
            n.month.and_then(|m| u8::try_from(m).ok()),
        );
        NoteRow {
            first: n.first.min(n.last),
            last: n.first.max(n.last),
            when,
        }
    }

    pub fn to_wire(self) -> ScheduleNote {
        let (at, year, month) = self.when.columns();
        ScheduleNote {
            first: self.first,
            last: self.last,
            at: at.map(clock::from_secs),
            year,
            month: month.map(u32::from),
        }
    }
}

/// Every note of the given series, with its season, by series id.
pub(crate) fn load(
    conn: &Connection,
    series: Option<&[u64]>,
) -> Result<Vec<(u64, Option<u32>, NoteRow)>, CoreError> {
    if series.is_some_and(<[u64]>::is_empty) {
        return Ok(Vec::new());
    }
    let mut sql =
        "SELECT series_id, season, first, last, at, year, month FROM schedule_notes".to_string();
    if let Some(ids) = series {
        let list: Vec<String> = ids.iter().map(u64::to_string).collect();
        sql.push_str(&format!(" WHERE series_id IN ({})", list.join(",")));
    }
    sql.push_str(" ORDER BY series_id, season, first");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        let at: Option<i64> = r.get(4)?;
        let year: Option<i64> = r.get(5)?;
        let month: Option<i64> = r.get(6)?;
        Ok((
            r.get::<_, i64>(0)? as u64,
            r.get::<_, Option<i64>>(1)?.map(|s| s as u32),
            NoteRow {
                first: r.get::<_, i64>(2)? as u32,
                last: r.get::<_, i64>(3)? as u32,
                when: When::from_columns(
                    at,
                    year.and_then(|y| i32::try_from(y).ok()),
                    month.and_then(|m| u8::try_from(m).ok()),
                ),
            },
        ))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// `Call::ParseSchedule`.
pub fn parse_call(text: &str) -> Reply {
    let (season, notes) = parse(text, clock::now_secs());
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
        .filter(|n| n.first > 0 && n.last <= EPISODE_MAX && n.last - n.first < SPAN_MAX)
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
            let (at, year, month) = r.when.columns();
            tx.execute(
                "INSERT INTO schedule_notes (series_id, season, first, last, at, year, month, set_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    series as i64,
                    season.map(i64::from),
                    i64::from(r.first),
                    i64::from(r.last),
                    at,
                    year.map(i64::from),
                    month.map(i64::from),
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
