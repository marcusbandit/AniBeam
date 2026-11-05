//! Reading a pasted release list into schedule notes (#418), by rule and by
//! intent rather than one fixed shape. Each line is read for two things: a
//! date (a day, a month, a year, or plainly none yet) and the episodes it
//! is about (a marker such as "Episode 8", "Ep. 8-10", "S03E08", "3x08",
//! "#8", "第8話", or a bare number leading a table row). The date is read
//! first and cut out, so a day after a dash is never taken for a range. A
//! line naming episodes and no date takes the date from the next line or
//! two, the way a site's episode card stacks them. "Weekly from" spreads a
//! range one week apart.

use std::sync::LazyLock;

use regex::{Captures, Regex};
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

use super::schedule::{NoteRow, When};

/// The most episodes one note may span, so a typo ("Episode 1 - 9999")
/// cannot plan thousands of rows.
pub(crate) const SPAN_MAX: u32 = 500;

/// Month names in English, Spanish, French and German, each a prefix
/// `month_num` reads back.
const MONTHS: &str = "jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sep(?:t(?:ember)?)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?|enero|febrero|marzo|abril|mayo|junio|julio|agosto|septiembre|setiembre|octubre|noviembre|diciembre|janvier|février|fevrier|mars|avril|mai|juin|juillet|août|aout|septembre|octobre|novembre|décembre|decembre|januar|februar|märz|maerz|juni|juli|oktober|dezember|gennaio|febbraio|aprile|maggio|giugno|luglio|settembre|ottobre|dicembre|janeiro|fevereiro|março|marco|maio|junho|julho|setembro|outubro|dezembro";
const DAYS: &str = "(?:(?:mon|tue|wed|thu|fri|sat|sun)[a-z]*|lunes|martes|miércoles|jueves|viernes|sábado|domingo|lundi|mardi|mercredi|jeudi|vendredi|samedi|dimanche|montag|dienstag|mittwoch|donnerstag|freitag|samstag|sonntag)\\.?,?\\s+(?:the\\s+|le\\s+|el\\s+|den\\s+)?";

fn re(s: &str) -> Regex {
    Regex::new(s).unwrap()
}

static TAG: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)<[^>]*>"));
static BLOCK_END: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)</(li|p|div|tr|h[1-6]|dt|dd)>|<br\s*/?>"));
static CELL_END: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)</t[dh]>"));
static ENTITY: LazyLock<Regex> = LazyLock::new(|| re(r"&#(x[0-9a-fA-F]+|\d+);"));
static SEASON: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(?:season|staffel|temporada|saison|series)\s*(\d{1,2})\b|\bs(\d{1,2})\s*[ .]?e\d{1,4}\b|\b(\d{1,2})x\d{1,3}\b|第\s*(\d{1,2})\s*期",
    )
});

// Dates. Every pattern is tried; the longest match wins where two overlap,
// so "Episode 2 Jan 21 2027" reads Jan 21 2027, not the 2nd of January.
static JP_DAY: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?:(\d{4})\s*[年년]\s*)?(\d{1,2})\s*[月월]\s*(\d{1,2})\s*[日일]"));
static JP_MONTH: LazyLock<Regex> = LazyLock::new(|| re(r"(\d{4})\s*[年년]\s*(\d{1,2})\s*[月월]"));
static ISO: LazyLock<Regex> = LazyLock::new(|| {
    re(r"\b(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})(?:[T ]\d{1,2}:\d{2}(?::\d{2})?Z?)?\b")
});
static NUMERIC: LazyLock<Regex> =
    LazyLock::new(|| re(r"\b(\d{1,2})([/.])(\d{1,2})[/.](\d{4}|\d{2})\b"));
static DAY_MONTH: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?i)(?:{DAYS})?\b(\d{{1,2}})(?:st|nd|rd|th|er|\.)? +(?:of +|de +)?({MONTHS})\b\.?,?(?: +(?:de +)?(\d{{4}}))?"
    ))
});
static MONTH_DAY: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?i)(?:{DAYS})?\b({MONTHS})\b\.?\s+(\d{{1,2}})(?:st|nd|rd|th)?\b,?(?:\s+(\d{{4}}))?"
    ))
});
static MONTH_YEAR: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?i)\b({MONTHS})\b\.?,?\s+(?:of\s+|de\s+)?(\d{{4}})\b"
    ))
});
/// A month opening a line on its own, as a heading or a label: "October:",
/// "Later June -". The year comes from around it.
static MONTH_ALONE: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?i)^[\s*•-]*(?:later|early|mid|late|end of)?\s*({MONTHS})\b\.?\s*[:-]"
    ))
});
/// An episode named by its place: "the sixth episode", "episode six".
static ORDINAL: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth|eleventh|twelfth|thirteenth|\d{1,3}(?:st|nd|rd|th))\s+episode\b|\bepisode\s+(one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen)\b",
    )
});
const ORDINALS: [&str; 13] = [
    "first",
    "second",
    "third",
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
];
const CARDINALS: [&str; 13] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve", "thirteen",
];
static VAGUE_YEAR: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(?:early|mid|late|end of|spring|summer|fall|autumn|winter|q[1-4]|h[12]|first half of|second half of|printemps|été|ete|automne|hiver|frühling|fruehling|frühjahr|sommer|herbst|primavera|verano|otoño|invierno|estate|autunno|inverno|verão|verao|outono)[\s-]*(?:of\s+|de\s+|du\s+)?(\d{4})\b",
    )
});
static YEAR: LazyLock<Regex> = LazyLock::new(|| re(r"(?:\b|[年년])((?:19|20)\d{2})(?:\b|[年년])"));
static UNDATED: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(?:tbc|tba|tbd|to be (?:confirmed|announced|determined|decided)|not yet (?:announced|dated|scheduled|known|aired|released)|yet to (?:air|be announced|be dated)|no (?:release )?date(?: yet)?|date unknown|unknown|coming soon|on hiatus|delayed indefinitely|n/a|null|in planung|termin folgt|noch offen|unbekannt|date inconnue|à venir|a venir|à confirmer|por confirmar|por anunciar|sin fecha|próximamente|proximamente|data da definire|da definire|da annunciare|sem data(?: confirmada)?|a definir|em breve|prossimamente)\b|未定|待定|近日|미정",
    )
});
static WEEKLY: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(weekly|every week|each week|a week apart|one a week|every other week|biweekly|fortnightly|every two weeks|wöchentlich|semanal(?:mente)?|chaque semaine)\b",
    )
});
static FORTNIGHT: LazyLock<Regex> =
    LazyLock::new(|| re(r"(?i)every other week|biweekly|fortnightly|every two weeks"));

// Episodes
const EP_WORD: &str = r"(?:episodes?|eps?\.?|e|no\.?|folge|episodio|épisode|episode|aflevering|odcinek|episódio|cap(?:ítulo|itulo)?\.?|puntata)";
static MARKED: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?i)(?:\bs\.?\s*\d{{1,2}}\s*[ .,]?\s*e\.?\s*|\b\d{{1,2}}x|\b{EP_WORD}\s*[#=:]?\s*|#\s*)(\d{{1,4}})\b((?:\s*(?:-|~|to|through|thru|until|bis|al|à|a|,|&|\+|and|y|et|und)\s*(?:{EP_WORD}\s*|#\s*)?\d{{1,4}}\b)*)"
    ))
});
/// An episode word ending the text before a number: that number is the
/// episode's.
static EP_BEFORE: LazyLock<Regex> =
    LazyLock::new(|| re(&format!(r"(?i)(?:\b{EP_WORD}|#)\s*[#=:]?\s*$")));
static TAIL: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"(?i)(-|~|to|through|thru|until|bis|al|à|a|,|&|\+|and|y|et|und)\s*(?:{EP_WORD}\s*|#\s*)?(\d{{1,4}})"
    ))
});
/// An episode named by a counter after its number: 第13話, 13話, 13集, 13화,
/// with a range in the same style (13화부터 15화까지, 13〜15話).
static SUFFIXED: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?:第\s*)?(\d{1,4})\s*[話话集화회](?:\s*(?:-|~|〜|부터|から)\s*(?:第\s*)?(\d{1,4})\s*[話话集화회]?)?",
    )
});
/// A clause correcting a date: the date after these words is the one meant.
static CORRECTION: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(?:now|moved to|delayed to|delayed until|pushed to|pushed back to|postponed to|postponed until|rescheduled (?:for|to)|instead|changed to|updated to)\b",
    )
});
/// A delay by a number of weeks, from the date said with it.
static DELAY: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(?:delayed|pushed back|postponed|moved back)\s+(?:by\s+)?(a|one|1|two|2|three|3)\s+weeks?\b",
    )
});
static LEADING: LazyLock<Regex> =
    LazyLock::new(|| re(r"^[\s|]*(\d{1,4})\b(?:[\s|.):-]+(\d{1,4})\b(?:\s*[|\t]|\s*$))?"));

/// What places a day given without a year: a year on the same line, the
/// last year named above, the day read before it (a list runs forward),
/// any year the text names, and last of all `now`.
#[derive(Clone, Copy)]
struct Ctx {
    line_year: Option<i32>,
    year_seen: Option<i32>,
    prev: Option<i64>,
    doc_year: Option<i32>,
    now: i64,
}

/// A date found on a line: where it sits, how exact it is (0 a day, 1 a
/// month, 2 a year, 3 none at all), and when.
#[derive(Clone, Copy)]
struct Found {
    start: usize,
    end: usize,
    rank: u8,
    when: When,
}

/// Part of a line: the episodes it names and the date said with them.
struct Clause {
    episodes: Vec<u32>,
    when: Option<When>,
    weekly: Option<i64>,
    /// The last day of a range of days said with a run of episodes
    /// ("第48話～第54話：1月8日～2月12日"), which spreads them a week apart
    /// up to it.
    until: Option<i64>,
}

/// A pasted release list read into notes, and the season it names, if any.
/// `now` places a date given without a year: this year, or next year when
/// that would put it more than half a year behind.
pub fn parse(text: &str, now: i64) -> (Option<u32>, Vec<NoteRow>) {
    let text = clean(text);
    let season = SEASON.captures(&text).and_then(|c| {
        (1..=4)
            .find_map(|i| c.get(i))
            .and_then(|m| m.as_str().parse().ok())
    });
    let mut rows: Vec<NoteRow> = Vec::new();
    // Episodes still waiting for a date, and how many lines they wait
    let mut waiting: Option<(Vec<u32>, u8)> = None;
    // A day heading the lines under it, until the next date or other text
    let mut heading: Option<When> = None;
    let mut ctx = Ctx {
        line_year: None,
        year_seen: None,
        prev: None,
        doc_year: YEAR.captures(&text).and_then(|c| c[1].parse().ok()),
        now,
    };
    for raw in text.lines() {
        if raw.trim().is_empty() {
            continue;
        }
        ctx.line_year = YEAR.captures(raw).and_then(|c| c[1].parse().ok());
        let (clauses, line_when, heads) = read_line(raw, ctx);
        if ctx.line_year.is_some() {
            ctx.year_seen = ctx.line_year;
        }
        let latest = clauses
            .iter()
            .filter_map(|c| c.when)
            .chain(line_when)
            .filter_map(|w| match w {
                When::On(at) => Some(at),
                _ => None,
            })
            .max();
        if latest.is_some() {
            ctx.prev = latest;
        }
        if clauses.is_empty() {
            match line_when {
                Some(when) => {
                    if let Some((eps, _)) = waiting.take() {
                        push(&mut rows, &eps, when, None);
                        heading = None;
                    } else {
                        heading = (matches!(when, When::On(_)) && heads).then_some(when);
                    }
                }
                None => {
                    waiting = waiting.and_then(|(eps, left)| (left > 1).then(|| (eps, left - 1)));
                    // A line of other text ends a heading's run, a short
                    // one (a runtime, a rating) under a heading does not
                    if raw.trim().len() > 24 {
                        heading = None;
                    }
                }
            }
            continue;
        }
        let mut undated: Vec<u32> = Vec::new();
        for c in clauses {
            match c.when.or(heading) {
                Some(when) => match (when, c.until) {
                    (When::On(start), Some(end)) => spread(&mut rows, &c.episodes, start, end),
                    _ => push(&mut rows, &c.episodes, when, c.weekly),
                },
                None => undated.extend(c.episodes),
            }
        }
        waiting = (!undated.is_empty()).then_some((undated, WAIT_LINES));
    }
    rows.sort_by_key(|r| r.first);
    (season, fold(rows))
}

/// How many lines an episode with no date waits for one: a site's episode
/// card puts a title, a runtime and a rating between them.
const WAIT_LINES: u8 = 5;

/// Notes for each episode, one week (or `weekly` seconds) apart when the
/// line said so. A later line about an episode replaces an earlier one, so
/// an edit or an update further down wins.
fn push(rows: &mut Vec<NoteRow>, episodes: &[u32], when: When, weekly: Option<i64>) {
    rows.retain(|r| !episodes.contains(&r.first));
    for (i, &n) in episodes.iter().enumerate() {
        let when = match (when, weekly) {
            (When::On(at), Some(step)) => When::On(at + step * i as i64),
            _ => when,
        };
        rows.push(NoteRow {
            first: n,
            last: n,
            when,
        });
    }
}

/// A run of episodes a week apart from `start`, the last on `end`; when
/// the run is longer than the weeks between, the extra ones open together
/// (a double premiere).
fn spread(rows: &mut Vec<NoteRow>, episodes: &[u32], start: i64, end: i64) {
    let week = 7 * 86_400;
    let weeks = ((end - start) as f64 / week as f64).round() as usize;
    let extra = (episodes.len() - 1).saturating_sub(weeks);
    rows.retain(|r| !episodes.contains(&r.first));
    for (i, &n) in episodes.iter().enumerate() {
        let step = i.saturating_sub(extra) as i64;
        rows.push(NoteRow {
            first: n,
            last: n,
            when: When::On(start + step * week),
        });
    }
}

/// HTML to lines of text: a row or a list item ends a line, a cell is a
/// tab, tags go, and every dash is a hyphen.
fn clean(text: &str) -> String {
    let text = BLOCK_END.replace_all(text, "\n");
    let text = CELL_END.replace_all(&text, "\t");
    let text = TAG.replace_all(&text, " ");
    let text = ENTITY.replace_all(&text, |c: &Captures| {
        let code = &c[1];
        let n = match code.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok(),
            None => code.parse().ok(),
        };
        n.and_then(char::from_u32)
            .map(String::from)
            .unwrap_or_default()
    });
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&ndash;", "-")
        .replace("&mdash;", "-")
        .chars()
        .map(|c| match c {
            '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{FF0D}' => '-',
            '\u{00A0}' | '\u{3000}' => ' ',
            '\u{FF5E}' | '\u{301C}' => '~',
            '\u{FF10}'..='\u{FF19}' => char::from_u32(c as u32 - 0xFF10 + '0' as u32).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// A line split into clauses, one per episode mention, each with the date
/// said in its own stretch of the line (or, for the first, a date said
/// before it); and the line's best date for a line that names no episode.
fn read_line(raw: &str, ctx: Ctx) -> (Vec<Clause>, Option<When>, bool) {
    let dates = dates_of(raw, ctx);
    let best = |list: &mut dyn Iterator<Item = &Found>| {
        list.min_by_key(|f| (f.rank, f.start)).map(|f| f.when)
    };
    let line_when = best(&mut dates.iter());
    // The dates' own digits are out of the way before episodes are read
    let mut rest = raw.to_string();
    for f in &dates {
        rest.replace_range(f.start..f.end, &" ".repeat(f.end - f.start));
    }
    // Words left once the dates are out: a weekday in brackets is fine
    let words = rest.chars().filter(|c| c.is_alphabetic()).count();
    let mut mentions = mentions_of(&rest);
    if mentions.is_empty()
        && line_when.is_some()
        && let Some(c) = LEADING.captures(&rest)
        && let Some(n) = c
            .get(2)
            .or_else(|| c.get(1))
            .and_then(|m| num32(m.as_str()))
    {
        mentions.push((0, vec![n]));
    }
    let weekly_of = |s: &str| {
        WEEKLY
            .is_match(s)
            .then(|| if FORTNIGHT.is_match(s) { 14 } else { 7 } * 86_400)
    };
    let mut clauses = Vec::new();
    for (i, (start, episodes)) in mentions.iter().enumerate() {
        let end = mentions.get(i + 1).map_or(raw.len(), |m| m.0);
        let from = if i == 0 { 0 } else { *start };
        // Corrections and delays read in the mention's own stretch only
        let text = raw.get(*start..end).unwrap_or(raw);
        // "Originally Nov 12, now Nov 19": the date after the correction
        let after = CORRECTION.find(text).map(|m| start + m.end());
        let own = |f: &&Found| f.start >= *start && f.start < end;
        let when = after
            .and_then(|p| best(&mut dates.iter().filter(own).filter(|f| f.start >= p)))
            .or_else(|| best(&mut dates.iter().filter(own)))
            .or_else(|| best(&mut dates.iter().filter(|f| f.start >= from && f.start < *start)));
        // "Was Oct 30 but is delayed by a week"
        let when = match (when, DELAY.captures(text).filter(|_| after.is_none())) {
            (Some(When::On(at)), Some(c)) => {
                let weeks = match &c[1].to_lowercase()[..] {
                    "two" | "2" => 2,
                    "three" | "3" => 3,
                    _ => 1,
                };
                Some(When::On(at + weeks * 7 * 86_400))
            }
            (w, _) => w,
        };
        // Two days joined by "~", "-" or "to" are a range the run spreads over
        let days: Vec<&Found> = dates
            .iter()
            .filter(own)
            .filter(|f| matches!(f.when, When::On(_)))
            .collect();
        let range = days.windows(2).find(|w| {
            let between = raw
                .get(w[0].end..w[1].start)
                .unwrap_or("")
                .trim()
                .to_lowercase();
            matches!(
                between.as_str(),
                "~" | "-" | "to" | "until" | "through" | "thru" | "bis" | "al" | "à"
            )
        });
        let (when, until) = match range {
            Some(w) if episodes.len() > 1 && after.is_none() => match (w[0].when, w[1].when) {
                (When::On(a), When::On(b)) if b > a => (Some(When::On(a)), Some(b)),
                _ => (when, None),
            },
            _ => (when, None),
        };
        clauses.push(Clause {
            episodes: episodes.clone(),
            when,
            weekly: weekly_of(raw.get(from..end).unwrap_or(raw)),
            until,
        });
    }
    // A line that opens on its date can head the lines under it
    let leads = dates
        .first()
        .is_some_and(|f| raw[..f.start].trim().is_empty());
    (clauses, line_when, words <= 6 || leads)
}

/// Every date-like stretch of a line, the longest kept where two overlap.
fn dates_of(line: &str, ctx: Ctx) -> Vec<Found> {
    let mut all: Vec<(usize, usize, u8, Option<When>)> = Vec::new();
    let mut add = |m: regex::Match, rank: u8, when: Option<When>| {
        all.push((m.start(), m.end(), rank, when));
    };
    for c in JP_DAY.captures_iter(line) {
        let year = c.get(1).and_then(|y| y.as_str().parse().ok());
        let w = num(&c[2]).and_then(|m| dated(year, ctx, m, num(&c[3])?));
        add(c.get(0).unwrap(), 0, w);
    }
    for c in ISO.captures_iter(line) {
        let w = num(&c[2]).and_then(|m| dated(c[1].parse().ok(), ctx, m, num(&c[3])?));
        add(c.get(0).unwrap(), 0, w);
    }
    for c in NUMERIC.captures_iter(line) {
        let (a, b) = (num(&c[1]).unwrap_or(0), num(&c[3]).unwrap_or(0));
        // A slash reads month first, as US sites write it, a dot day first;
        // either flips when the first number cannot be what it says
        let (m, d) = match &c[2] {
            "/" if a <= 12 => (a, b),
            "/" => (b, a),
            _ if b <= 12 => (b, a),
            _ => (a, b),
        };
        let y: i32 = c[4].parse().unwrap_or(0);
        let y = if y < 100 { 2000 + y } else { y };
        add(c.get(0).unwrap(), 0, dated(Some(y), ctx, m, d));
    }
    for c in DAY_MONTH.captures_iter(line) {
        // "Episode 8 March 2027": the number is the episode's, not a day
        if EP_BEFORE.is_match(&line[..c.get(0).unwrap().start()]) {
            continue;
        }
        let year = c.get(3).and_then(|y| y.as_str().parse().ok());
        let w = month_num(&c[2]).and_then(|m| dated(year, ctx, m, num(&c[1])?));
        add(c.get(0).unwrap(), 0, w);
    }
    for c in MONTH_DAY.captures_iter(line) {
        let year = c.get(3).and_then(|y| y.as_str().parse().ok());
        let w = month_num(&c[1]).and_then(|m| dated(year, ctx, m, num(&c[2])?));
        add(c.get(0).unwrap(), 0, w);
    }
    for c in JP_MONTH.captures_iter(line) {
        let w = c[1]
            .parse()
            .ok()
            .zip(num(&c[2]).filter(|m| (1..=12).contains(m)));
        add(c.get(0).unwrap(), 1, w.map(|(y, m)| When::Month(y, m)));
    }
    for c in MONTH_ALONE.captures_iter(line) {
        let m = c.get(1).unwrap();
        let w = month_num(m.as_str()).and_then(|mo| match dated(None, ctx, mo, 1)? {
            When::On(at) => OffsetDateTime::from_unix_timestamp(at)
                .ok()
                .map(|d| When::Month(d.year(), mo)),
            _ => None,
        });
        add(m, 1, w);
    }
    for c in MONTH_YEAR.captures_iter(line) {
        let w = c[2].parse().ok().zip(month_num(&c[1]));
        add(c.get(0).unwrap(), 1, w.map(|(y, m)| When::Month(y, m)));
    }
    for c in VAGUE_YEAR.captures_iter(line) {
        add(c.get(0).unwrap(), 2, c[1].parse().ok().map(When::Year));
    }
    for c in YEAR.captures_iter(line) {
        add(c.get(1).unwrap(), 2, c[1].parse().ok().map(When::Year));
    }
    for m in UNDATED.find_iter(line) {
        add(m, 3, Some(When::Unknown));
    }
    // Longest first, then the more exact, then the earlier
    all.sort_by_key(|(a, b, rank, _)| (std::cmp::Reverse(b - a), *rank, *a));
    let mut kept: Vec<(usize, usize, u8, Option<When>)> = Vec::new();
    for d in all {
        if kept.iter().all(|k| d.1 <= k.0 || k.1 <= d.0) {
            kept.push(d);
        }
    }
    let mut out: Vec<Found> = kept
        .into_iter()
        .filter_map(|(start, end, rank, when)| {
            when.map(|when| Found {
                start,
                end,
                rank,
                when,
            })
        })
        .collect();
    out.sort_by_key(|f| f.start);
    out
}

/// Every marked episode mention on a line, with where it starts and the
/// numbers it names, ranges and lists spelled out.
fn mentions_of(rest: &str) -> Vec<(usize, Vec<u32>)> {
    let mut out: Vec<(usize, Vec<u32>)> = Vec::new();
    for c in MARKED.captures_iter(rest) {
        let Some(first) = num32(&c[1]) else { continue };
        let mut eps = vec![first];
        let mut prev = first;
        for t in TAIL.captures_iter(c.get(2).map_or("", |m| m.as_str())) {
            let Some(n) = num32(&t[2]) else { continue };
            let range = matches!(
                t[1].to_lowercase().as_str(),
                "-" | "~" | "〜" | "to" | "through" | "thru" | "until" | "bis" | "al" | "à" | "a"
            );
            if range && n > prev && n - prev < SPAN_MAX {
                eps.extend(prev + 1..=n);
            } else {
                eps.push(n);
            }
            prev = n;
        }
        eps.retain(|n| *n > 0);
        if !eps.is_empty() {
            out.push((c.get(0).unwrap().start(), eps));
        }
    }
    let marked: Vec<(usize, usize)> = MARKED
        .find_iter(rest)
        .map(|m| (m.start(), m.end()))
        .collect();
    for c in SUFFIXED.captures_iter(rest) {
        let m = c.get(0).unwrap();
        if marked.iter().any(|(a, b)| m.start() < *b && *a < m.end()) {
            continue;
        }
        let Some(first) = num32(&c[1]).filter(|n| *n > 0) else {
            continue;
        };
        let last = c.get(2).and_then(|l| num32(l.as_str())).unwrap_or(first);
        let last = if last > first && last - first < SPAN_MAX {
            last
        } else {
            first
        };
        out.push((m.start(), (first..=last).collect()));
    }
    for c in ORDINAL.captures_iter(rest) {
        let m = c.get(0).unwrap();
        if out.iter().any(|(at, _)| *at >= m.start() && *at < m.end()) {
            continue;
        }
        let n = match (c.get(1), c.get(2)) {
            (Some(w), _) => {
                let w = w.as_str().to_lowercase();
                ORDINALS
                    .iter()
                    .position(|o| *o == w)
                    .map(|i| i as u32 + 1)
                    .or_else(|| num32(w.trim_end_matches(char::is_alphabetic)))
            }
            (None, Some(w)) => {
                let w = w.as_str().to_lowercase();
                CARDINALS.iter().position(|o| *o == w).map(|i| i as u32 + 1)
            }
            _ => None,
        };
        if let Some(n) = n.filter(|n| *n > 0) {
            out.push((m.start(), vec![n]));
        }
    }
    out.sort_by_key(|m| m.0);
    out
}

fn num(s: &str) -> Option<u8> {
    s.trim().parse().ok()
}

fn num32(s: &str) -> Option<u32> {
    s.trim().parse().ok()
}

fn month_num(name: &str) -> Option<u8> {
    let name = name.to_lowercase();
    const PREFIXES: &[(&str, u8)] = &[
        ("jan", 1),
        ("ene", 1),
        ("feb", 2),
        ("fév", 2),
        ("fev", 2),
        ("mär", 3),
        ("mae", 3),
        ("mar", 3),
        ("apr", 4),
        ("abr", 4),
        ("avr", 4),
        ("mayo", 5),
        ("may", 5),
        ("mai", 5),
        ("juin", 6),
        ("jun", 6),
        ("juil", 7),
        ("jul", 7),
        ("aug", 8),
        ("ago", 8),
        ("aoû", 8),
        ("aou", 8),
        ("sep", 9),
        ("set", 9),
        ("oct", 10),
        ("okt", 10),
        ("nov", 11),
        ("dec", 12),
        ("déc", 12),
        ("dic", 12),
        ("dez", 12),
        ("gen", 1),
        ("mag", 5),
        ("giu", 6),
        ("lug", 7),
        ("ott", 10),
        ("out", 10),
    ];
    PREFIXES
        .iter()
        .find(|(p, _)| name.starts_with(p))
        .map(|(_, m)| *m)
}

/// Noon UTC on a day: a release list names the day, rarely the hour. A day
/// with no year takes the last year the text named above it, else the one
/// that puts it closest ahead of `now`.
fn dated(year: Option<i32>, ctx: Ctx, month: u8, d: u8) -> Option<When> {
    let at = |y: i32| {
        let date = Date::from_calendar_date(y, Month::try_from(month).ok()?, d).ok()?;
        Some(
            PrimitiveDateTime::new(date, Time::from_hms(12, 0, 0).ok()?)
                .assume_utc()
                .unix_timestamp(),
        )
    };
    if let Some(y) = year.or(ctx.line_year).or(ctx.year_seen) {
        return at(y).map(When::On);
    }
    let year_of = |t: i64| {
        OffsetDateTime::from_unix_timestamp(t)
            .ok()
            .map(|d| d.year())
    };
    // A list runs forward: the year that keeps this day at or after the
    // one before it, a month's slack for a list out of order
    if let Some(prev) = ctx.prev {
        let y = year_of(prev)?;
        let this = at(y)?;
        let month_back = 31 * 86_400;
        return if this < prev - month_back {
            at(y + 1)
        } else {
            Some(this)
        }
        .map(When::On);
    }
    let this = year_of(ctx.now)?;
    // A year named only further down places the day when it is in the
    // past: an older article. One ahead is about later episodes, not this.
    if let Some(y) = ctx.doc_year.filter(|y| *y < this) {
        return at(y).map(When::On);
    }
    let guess = at(this)?;
    let half_year = 183 * 86_400;
    if guess < ctx.now - half_year {
        at(this + 1).map(When::On)
    } else {
        Some(When::On(guess))
    }
}

/// Neighbouring notes landing at the same moment, as one note each; a later
/// line for an episode already read gives way to the first.
pub(crate) fn fold(rows: Vec<NoteRow>) -> Vec<NoteRow> {
    let mut out: Vec<NoteRow> = Vec::new();
    for mut r in rows {
        if let Some(prev) = out.last_mut() {
            if r.first <= prev.last {
                if r.last <= prev.last {
                    continue;
                }
                if r.when == prev.when {
                    prev.last = r.last;
                    continue;
                }
                // The overlap gives way to the first; the rest keeps its own date
                r.first = prev.last + 1;
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// 2026-11-04, the day this was written.
    pub const NOW: i64 = 1_793_793_600;

    /// A note as the catalogue reads it: "8-15 2027", "4 Wed 21 Oct 2026",
    /// "8 Jan 2027", "9 TBC".
    pub fn show(rows: &[NoteRow]) -> String {
        rows.iter()
            .map(|r| {
                let eps = if r.first == r.last {
                    r.first.to_string()
                } else {
                    format!("{}-{}", r.first, r.last)
                };
                let when = match r.when {
                    When::On(at) => {
                        let d = OffsetDateTime::from_unix_timestamp(at).unwrap().date();
                        format!(
                            "{} {} {}",
                            d.day(),
                            &format!("{:?}", d.month())[..3],
                            d.year()
                        )
                    }
                    When::Month(y, m) => {
                        format!("{} {y}", &format!("{:?}", Month::try_from(m).unwrap())[..3])
                    }
                    When::Year(y) => y.to_string(),
                    When::Unknown => "TBC".to_string(),
                };
                format!("{eps} {when}")
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// The catalogue: every shape the paste box reads, what it reads, and
    /// the season it names. `docs/schedule-formats.md` is made from it.
    pub const CASES: &[(&str, &str, &str, Option<u32>)] = &[
        (
            "A release list, dates written out",
            "Helluva Boss season 3 Part 1:\nEpisode 1 \u{2013} Wednesday 14 October 2026 (out now)\nEpisode 2 \u{2013} Wednesday 14 October 2026 (out now)\nEpisode 3 \u{2013} Wednesday 21 October 2026\nEpisode 8 - TBC in 2027\nEpisode 9 - TBC in 2027",
            "1-2 14 Oct 2026; 3 21 Oct 2026; 8-9 2027",
            Some(3),
        ),
        (
            "HTML copied from a web page",
            "<ul><li><strong>Episode 7 \u{2013} Wednesday 28 October 2026 (out now)</strong></li><li><strong>Episode 8 - TBC in 2027 </strong></li></ul>",
            "7 28 Oct 2026; 8 2027",
            None,
        ),
        (
            "A range on one line",
            "Episodes 8-15: TBC\nEps. 16 to 20 - 2028",
            "8-15 TBC; 16-20 2028",
            None,
        ),
        (
            "A list on one line",
            "Episodes 8, 9 and 10 arrive December 2, 2026",
            "8-10 2 Dec 2026",
            None,
        ),
        (
            "American dates",
            "Ep 4: December 8th, 2026\nEp 5 (Dec. 15)\nEp 6 airs 12/22/2026",
            "4 8 Dec 2026; 5 15 Dec 2026; 6 22 Dec 2026",
            None,
        ),
        (
            "European and ISO dates",
            "Folge 3 - 05.01.2027\nEpisode 4 \u{2014} 2027-01-12\n#5 19/01/2027",
            "3 5 Jan 2027; 4 12 Jan 2027; 5 19 Jan 2027",
            None,
        ),
        (
            "Season and episode codes",
            "S03E08 - Jan 7, 2027\nS03 E09 Jan 14 2027\n3x10 2027-01-21",
            "8 7 Jan 2027; 9 14 Jan 2027; 10 21 Jan 2027",
            Some(3),
        ),
        (
            "A day and month with no year",
            "Episode 5 - 12 Nov\nEpisode 6 - 19 Nov",
            "5 12 Nov 2026; 6 19 Nov 2026",
            None,
        ),
        (
            "A year above, days below",
            "Schedule for 2027\nEpisode 1 - 8 January\nEpisode 2 - 15 January",
            "1 8 Jan 2027; 2 15 Jan 2027",
            None,
        ),
        (
            "Only the month known",
            "Episode 8: January 2027\nEpisode 9: Feb 2027",
            "8 Jan 2027; 9 Feb 2027",
            None,
        ),
        (
            "A vague time of year",
            "Part 2 (episodes 8-15) is coming in early 2027",
            "8-15 2027",
            None,
        ),
        (
            "No date at all",
            "Episode 8: to be announced\nEpisodes 9 to 12 - not yet dated",
            "8-12 TBC",
            None,
        ),
        (
            "Weekly from a start date",
            "Episodes 8-11 release weekly from January 7, 2027",
            "8 7 Jan 2027; 9 14 Jan 2027; 10 21 Jan 2027; 11 28 Jan 2027",
            None,
        ),
        (
            "A table copied from Wikipedia",
            "No. overall\tNo. in season\tTitle\tDirected by\tOriginal release date\n27\t8\t\"The Return\"\tBrandon Blake\tJanuary 7, 2027\n28\t9\t\"Loose Ends\"\tBrandon Blake\tJanuary 14, 2027",
            "8 7 Jan 2027; 9 14 Jan 2027",
            None,
        ),
        (
            "An HTML table",
            "<table><tr><th>Ep</th><th>Air date</th></tr><tr><td>8</td><td>2027-01-07</td></tr><tr><td>9</td><td>TBA</td></tr></table>",
            "8 7 Jan 2027; 9 TBC",
            None,
        ),
        (
            "A streaming site's episode cards",
            "E8 - The Return\n24m\nJan 7, 2027\nE9 - Loose Ends\n23m\nJan 14, 2027",
            "8 7 Jan 2027; 9 14 Jan 2027",
            None,
        ),
        (
            "A date above the episode",
            "Wednesday, January 7, 2027\nEpisode 8: The Return\nWednesday, January 14, 2027\nEpisode 9: Loose Ends",
            "8 7 Jan 2027; 9 14 Jan 2027",
            None,
        ),
        (
            "Japanese",
            "第8話 2027年1月7日\n第9話 1月14日\n第10話 未定",
            "8 7 Jan 2027; 9 14 Jan 2027; 10 TBC",
            None,
        ),
        (
            "A day after a dash is not a range",
            "Episode 1 - 14 October 2026",
            "1 14 Oct 2026",
            None,
        ),
        (
            "A calendar: a day heading the episodes under it",
            "Jan 7 (Thu)\n  Ep 1: Gull Road\n  Ep 2: Lamp Street\nJan 14 (Thu)\n  Ep 3: Pale Anchor",
            "1-2 7 Jan 2027; 3 14 Jan 2027",
            None,
        ),
        (
            "Several episodes in one sentence",
            "Episode 10 premieres March 1st, 2027, followed by episode 11 on March 8th, 2027. Ep 12 date TBD.",
            "10 1 Mar 2027; 11 8 Mar 2027; 12 TBC",
            None,
        ),
        (
            "Spanish, French and German",
            "El episodio 3 se estrena el 8 de febrero de 2027\nÉpisode 4 : disponible le 3 mars 2027\nFolge 5 am 12.03.2027\nFolge 6: Termin folgt",
            "3 8 Feb 2027; 4 3 Mar 2027; 5 12 Mar 2027; 6 TBC",
            None,
        ),
        (
            "A calendar export",
            "ep=11,air=2027-01-14\nep=12,air=null",
            "11 14 Jan 2027; 12 TBC",
            None,
        ),
        (
            "A site's episode cards, the date four lines down",
            "Episode 3\nTheir Hands in the Dark\n52m\nRating: 16+\nAvailable Nov 20, 2026\nEpisode 4\nBrass Cathedral\n48m\nComing soon",
            "3 20 Nov 2026; 4 TBC",
            None,
        ),
        (
            "Korean, Italian and Portuguese",
            "13화 「안개 항구」 2027년 1월 10일 방송\n14화 방영 일정 미정\nEpisodio 15 - Il faro (17 gennaio 2027)\nEpisódio 16: Sal e Ferro, 24 de janeiro de 2027",
            "13 10 Jan 2027; 14 TBC; 15 17 Jan 2027; 16 24 Jan 2027",
            None,
        ),
        (
            "A Japanese TV listing, the day as a heading",
            "1月10日(日) 24:30〜 TOKYO MX\n「夜明けの鈴」第13話\n1月17日(日) 24:30〜 TOKYO MX\n「灰の橋」第14話",
            "13 10 Jan 2027; 14 17 Jan 2027",
            None,
        ),
        (
            "A correction and a delay",
            "Episode 9, originally set for Nov 12, now arrives on Nov 19.\nEp 10 was Nov 26 but is delayed by a week.",
            "9 19 Nov 2026; 10 3 Dec 2026",
            None,
        ),
        (
            "A later line updates an earlier one",
            "Ep 7 is Dec 3 and ep 8 is Dec 10.\nEdit: site got updated, ep 7 is Dec 4 now and ep 8 is Dec 11.",
            "7 4 Dec 2026; 8 11 Dec 2026",
            None,
        ),
        (
            "A run of episodes over a run of days",
            "第48話～第54話：1月8日～2月12日\n閑話：2月19日\nEpisodes 55-59: Feb 26 to Mar 26",
            "48-49 8 Jan 2027; 50 15 Jan 2027; 51 22 Jan 2027; 52 29 Jan 2027; 53 5 Feb 2027; 54 12 Feb 2027; 55 26 Feb 2027; 56 5 Mar 2027; 57 12 Mar 2027; 58 19 Mar 2027; 59 26 Mar 2027",
            None,
        ),
        (
            "Real articles: a date first, S.5 E.1, a month on its own",
            "November 9 - Season 2 episode 1 - 3\nS.5 E.4: \u{201c}Sorcerer\u{201d} \u{2013} Wednesday, Nov. 26\nOctober: Season 2, Episode 10: \u{201c}Ghostfuckers\u{201d}\nFrieren season 2's sixth episode has been pushed back to February 27.",
            "1-3 9 Nov 2026; 4 26 Nov 2026; 6 27 Feb 2027; 10 Oct 2027",
            Some(2),
        ),
        (
            "A day with no year, and a later year about other episodes",
            "Episode 7: YES NO - October 28\nPart 2 (Episodes 8-15) - 2027",
            "7 28 Oct 2026; 8-15 2027",
            None,
        ),
        (
            "A month after the episode number, in a table or a line",
            "<table><tr><td>8</td><td>March 2027</td></tr></table>\nEpisode 9 April 2027",
            "8 Mar 2027; 9 Apr 2027",
            None,
        ),
        (
            "A date heading that is really a note",
            "Updated November 3, 2026\nEpisode 9 is the finale",
            "",
            None,
        ),
        (
            "Lines with nothing to read are skipped",
            "Episode 3 is the best one\nWhat a season\nWe loved it",
            "",
            None,
        ),
    ];

    #[test]
    fn an_overlap_gives_way_to_the_first_and_keeps_the_rest() {
        let rows = vec![
            NoteRow {
                first: 5,
                last: 8,
                when: When::Unknown,
            },
            NoteRow {
                first: 7,
                last: 12,
                when: When::Year(2027),
            },
        ];
        assert_eq!(
            fold(rows),
            vec![
                NoteRow {
                    first: 5,
                    last: 8,
                    when: When::Unknown
                },
                NoteRow {
                    first: 9,
                    last: 12,
                    when: When::Year(2027)
                },
            ]
        );
    }

    #[test]
    fn the_catalogue_reads_as_written() {
        let mut wrong = Vec::new();
        for (name, text, want, season) in CASES {
            let (got_season, rows) = parse(text, NOW);
            let got = show(&rows);
            if got != *want || got_season != *season {
                wrong.push(format!(
                    "{name}:\n  want {want} {season:?}\n  got  {got} {got_season:?}"
                ));
            }
        }
        assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
    }

    /// Runs a JSON file of `{source, text, expect}` samples named by
    /// `SCHEDULE_SAMPLES` and prints the ones read differently.
    #[test]
    #[ignore]
    fn samples_from_a_file() {
        let path = std::env::var("SCHEDULE_SAMPLES").unwrap();
        let list: Vec<serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let mut misses = 0;
        for (i, s) in list.iter().enumerate() {
            let text = s["text"].as_str().unwrap_or("");
            let want = s["expect"].as_str().unwrap_or("");
            let (_, rows) = parse(text, NOW);
            let got = show(&rows);
            if got != want {
                misses += 1;
                println!(
                    "--- #{i} {}\n{text}\n  want {want}\n  got  {got}",
                    s["source"]
                );
            }
        }
        println!(
            "SAMPLES {} of {} read as expected",
            list.len() - misses,
            list.len()
        );
    }

    /// `cargo test -p anibeam-core -- --ignored print_catalogue --nocapture`
    /// prints the catalogue as JSON, for the format page.
    #[test]
    #[ignore]
    fn print_catalogue() {
        let out: Vec<serde_json::Value> = CASES
            .iter()
            .map(|(name, text, _, _)| {
                let (season, rows) = parse(text, NOW);
                serde_json::json!({ "name": name, "text": text, "read": show(&rows), "season": season })
            })
            .collect();
        println!("CATALOGUE {}", serde_json::to_string(&out).unwrap());
    }
}
