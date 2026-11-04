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
        let when = match (n.at, n.year, n.month) {
            (Some(at), _, _) => When::On(clock::to_secs(at)),
            (None, Some(y), Some(m)) if (1..=12).contains(&m) => When::Month(y, m as u8),
            (None, Some(y), _) => When::Year(y),
            (None, None, _) => When::Unknown,
        };
        NoteRow {
            first: n.first.min(n.last),
            last: n.first.max(n.last),
            when,
        }
    }

    pub fn to_wire(self) -> ScheduleNote {
        let (at, year, month) = match self.when {
            When::On(at) => (Some(clock::from_secs(at)), None, None),
            When::Month(y, m) => (None, Some(y), Some(u32::from(m))),
            When::Year(y) => (None, Some(y), None),
            When::Unknown => (None, None, None),
        };
        ScheduleNote {
            first: self.first,
            last: self.last,
            at,
            year,
            month,
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
                when: match (at, year, month) {
                    (Some(at), _, _) => When::On(at),
                    (None, Some(y), Some(m)) => When::Month(y as i32, m as u8),
                    (None, Some(y), None) => When::Year(y as i32),
                    (None, None, _) => When::Unknown,
                },
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
            let (at, year, month) = match r.when {
                When::On(at) => (Some(at), None, None),
                When::Month(y, m) => (None, Some(i64::from(y)), Some(i64::from(m))),
                When::Year(y) => (None, Some(i64::from(y)), None),
                When::Unknown => (None, None, None),
            };
            tx.execute(
                "INSERT INTO schedule_notes (series_id, season, first, last, at, year, month, set_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    series as i64,
                    season.map(i64::from),
                    i64::from(r.first),
                    i64::from(r.last),
                    at,
                    year,
                    month,
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
