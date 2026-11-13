//! The words and numbers the screens show.

use chrono::{DateTime, Datelike, Local, TimeZone};

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

/// A moment written by the catalog's own date pattern (`fmt-day-month = %-d %b` in English),
/// with month and weekday names in the language the app speaks. A pattern chrono cannot
/// read falls back to `fallback` rather than panicking.
fn dated<Tz: TimeZone>(t: &DateTime<Tz>, id: &str, fallback: &str) -> String
where
    Tz::Offset: std::fmt::Display,
{
    use std::fmt::Write;
    let locale = crate::i18n::chrono_locale(crate::i18n::lang());
    let mut out = String::new();
    if write!(out, "{}", t.format_localized(&tr!(id), locale)).is_err() {
        out.clear();
        let _ = write!(out, "{}", t.format_localized(fallback, locale));
    }
    out
}

/// A calendar day written by a catalog pattern, with weekday and month names in the language
/// the app speaks: `cal-weekday = %A` gives "Thursday", "torsdag", "木曜日".
pub fn day(date: chrono::NaiveDate, id: &str, fallback: &str) -> String {
    use std::fmt::Write;
    let locale = crate::i18n::chrono_locale(crate::i18n::lang());
    let mut out = String::new();
    if write!(out, "{}", date.format_localized(&tr!(id), locale)).is_err() {
        out.clear();
        let _ = write!(out, "{}", date.format_localized(fallback, locale));
    }
    out
}

/// "2d 4h", "4h 12m", "12m 05s", "42s": how long is left, with no "in". Seconds show only
/// inside the last hour, where they are worth watching tick.
pub fn span(seconds: i64) -> String {
    let s = seconds.max(0);
    if s >= DAY {
        tr!("fmt-span-days", d = s / DAY, h = (s % DAY) / HOUR)
    } else if s >= HOUR {
        tr!("fmt-span-hours", h = s / HOUR, m = (s % HOUR) / MINUTE)
    } else if s >= MINUTE {
        tr!(
            "fmt-span-minutes",
            m = s / MINUTE,
            s = format!("{:02}", s % MINUTE)
        )
    } else {
        tr!("fmt-span-seconds", s = s)
    }
}

/// "in 2d 4h", "in 4h 12m", "in 12m 05s", "in 42s": [`span`] after the catalog's "in".
pub fn countdown(seconds: i64) -> String {
    tr!("fmt-in", span = span(seconds))
}

/// The desktop series page's ticking form: "3d 02h 15m 07s", "2h 05m 07s", "15m 07s".
pub fn countdown_full(seconds: i64) -> String {
    let s = seconds.max(0);
    let (d, h, m, sec) = (s / DAY, (s % DAY) / HOUR, (s % HOUR) / MINUTE, s % MINUTE);
    let two = |n: i64| format!("{n:02}");
    if d > 0 {
        tr!("fmt-full-days", d = d, h = two(h), m = two(m), s = two(sec))
    } else if h > 0 {
        tr!("fmt-full-hours", h = h, m = two(m), s = two(sec))
    } else {
        tr!("fmt-full-minutes", m = m, s = two(sec))
    }
}

/// "17:56" in the phone's zone.
pub fn clock(at: i64) -> String {
    Local
        .timestamp_opt(at, 0)
        .single()
        .map(|t| dated(&t, "fmt-clock", "%H:%M"))
        .unwrap_or_default()
}

/// "17 Oct" in the phone's zone.
pub fn day_month(at: i64) -> String {
    Local
        .timestamp_opt(at, 0)
        .single()
        .map(|t| dated(&t, "fmt-day-month", "%-d %b"))
        .unwrap_or_default()
}

/// "just now", "1 min ago", "3h ago", "2d ago".
pub fn ago(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < MINUTE {
        tr!("fmt-ago-now")
    } else if s < HOUR {
        tr!("fmt-ago-minutes", count = s / MINUTE)
    } else if s < DAY {
        tr!("fmt-ago-hours", count = s / HOUR)
    } else {
        tr!("fmt-ago-days", count = s / DAY)
    }
}

/// The day an air time falls on, written with its time: "Today 14:00", "Tomorrow 02:30".
fn day_time<Tz: TimeZone>(at: &DateTime<Tz>, now: &DateTime<Tz>) -> Option<String>
where
    Tz::Offset: std::fmt::Display,
{
    let days = at.date_naive().num_days_from_ce() - now.date_naive().num_days_from_ce();
    let id = match days {
        -1 => "fmt-yesterday-at",
        0 => "fmt-today-at",
        1 => "fmt-tomorrow-at",
        _ => return None,
    };
    Some(tr!(id, time = dated(at, "fmt-clock", "%H:%M")))
}

/// An air time in the phone's own zone: "Today 14:00", "Tomorrow 02:30", "Sat 25 Oct 14:00".
pub fn when_in<Tz: TimeZone>(at: i64, now: i64, zone: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let (Some(at), Some(now)) = (
        zone.timestamp_opt(at, 0).single(),
        zone.timestamp_opt(now, 0).single(),
    ) else {
        return String::new();
    };
    match day_time(&at, &now) {
        Some(text) => text,
        None if at.year() == now.year() => dated(&at, "fmt-weekday-date-time", "%a %-d %b %H:%M"),
        None => dated(&at, "fmt-weekday-date-year-time", "%a %-d %b %Y %H:%M"),
    }
}

pub fn when(at: i64, now: i64) -> String {
    when_in(at, now, &Local)
}

/// A moment for the status line: "Oct 17 14:02".
pub fn stamp_in<Tz: TimeZone>(at: i64, zone: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    zone.timestamp_opt(at, 0)
        .single()
        .map(|t| dated(&t, "fmt-stamp", "%b %-d %H:%M"))
        .unwrap_or_default()
}

pub fn stamp(at: i64) -> String {
    stamp_in(at, &Local)
}

/// "E08", or "S02E01" past the first season, the way the host writes episode codes.
pub fn episode_code(season: Option<u32>, episode: u32) -> String {
    match season {
        Some(s) if s > 1 => format!("S{s:02}E{episode:02}"),
        _ => format!("E{episode:02}"),
    }
}

/// Every distinct episode code of a list, one per line, for the series page to measure: a
/// text's width is its widest line, so the code column fits whichever code draws widest.
pub fn code_column<'a>(codes: impl IntoIterator<Item = &'a str>) -> String {
    let mut seen: Vec<&str> = Vec::new();
    for c in codes {
        if !seen.contains(&c) {
            seen.push(c);
        }
    }
    seen.join("\n")
}

/// "SN" for "Sousou no Frieren": what a tile without a poster shows.
pub fn initials(title: &str) -> String {
    title
        .split_whitespace()
        .filter_map(|w| w.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

/// A number written with its unit, "1.4 GB": `unit` indexes B, KB, MB, GB, TB. Whole from
/// 100 up and for plain bytes, one decimal below, with the language's decimal point.
pub fn sized(value: f64, unit: usize) -> String {
    const IDS: [&str; 5] = [
        "fmt-bytes-b",
        "fmt-bytes-kb",
        "fmt-bytes-mb",
        "fmt-bytes-gb",
        "fmt-bytes-tb",
    ];
    let unit = unit.min(IDS.len() - 1);
    let number = if unit == 0 || value >= 100.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}").replace('.', &tr!("fmt-decimal-point"))
    };
    tr!(IDS[unit], value = number)
}

/// "1.4 GB", "820 MB", "12 KB".
pub fn bytes(n: u64) -> String {
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < 4 {
        value /= 1000.0;
        unit += 1;
    }
    sized(value, unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{Lang, chrono_locale, set_lang};
    use chrono::FixedOffset;

    /// No test here switches the language: the catalog is shared by every test thread, and
    /// English is what it starts in. This only makes that explicit.
    fn english() {
        set_lang(Lang::En);
    }

    #[test]
    fn countdowns_lose_precision_with_distance() {
        english();
        assert_eq!(countdown(2 * DAY + 4 * HOUR + 59 * MINUTE), "in 2d 4h");
        assert_eq!(countdown(4 * HOUR + 12 * MINUTE + 30), "in 4h 12m");
        assert_eq!(countdown(12 * MINUTE + 5), "in 12m 05s");
        assert_eq!(countdown(42), "in 42s");
        assert_eq!(countdown(-5), "in 0s");
        assert_eq!(span(2 * HOUR + 5 * MINUTE + 7), "2h 5m");
        assert_eq!(span(2 * DAY + 4 * HOUR), "2d 4h");
        assert_eq!(span(42), "42s");
        assert_eq!(
            countdown_full(3 * DAY + 2 * HOUR + 15 * MINUTE + 7),
            "3d 02h 15m 07s"
        );
        assert_eq!(countdown_full(2 * HOUR + 5 * MINUTE + 7), "2h 05m 07s");
        assert_eq!(countdown_full(15 * MINUTE + 7), "15m 07s");
    }

    #[test]
    fn ago_reads_like_a_status_line() {
        english();
        assert_eq!(ago(20), "just now");
        assert_eq!(ago(61), "1 min ago");
        assert_eq!(ago(3 * HOUR + 5), "3h ago");
        assert_eq!(ago(2 * DAY), "2d ago");
    }

    #[test]
    fn air_times_name_the_day_when_it_is_close() {
        english();
        let cet = FixedOffset::east_opt(2 * 3600).unwrap();
        // 2026-10-17 15:40 CEST
        let now = 1_792_244_400;
        assert_eq!(when_in(now + 2 * HOUR, now, &cet), "Today 17:40");
        assert_eq!(when_in(now + DAY, now, &cet), "Tomorrow 15:40");
        assert_eq!(when_in(now - DAY, now, &cet), "Yesterday 15:40");
        assert_eq!(when_in(now + 3 * DAY, now, &cet), "Tue 20 Oct 15:40");
        assert_eq!(when_in(now + 90 * DAY, now, &cet), "Fri 15 Jan 2027 15:40");
        assert_eq!(stamp_in(now, &cet), "Oct 17 15:40");
    }

    #[test]
    fn codes_and_sizes() {
        english();
        assert_eq!(episode_code(None, 8), "E08");
        assert_eq!(episode_code(Some(1), 8), "E08");
        assert_eq!(episode_code(Some(2), 1), "S02E01");
        assert_eq!(initials("Sousou no Frieren"), "SN");
        assert_eq!(initials("Mushishi"), "M");
        assert_eq!(initials("[Oshi no Ko]"), "ON");
        assert_eq!(bytes(1_446_000_000), "1.4 GB");
        assert_eq!(bytes(820_000_000), "820 MB");
        assert_eq!(bytes(512), "512 B");
    }

    #[test]
    fn the_code_column_measures_every_distinct_code() {
        // Width, not length, decides the column: "EP 1.5" is as long as "S03E01" but narrower
        // in B612, so both go in, one per line, and the page measures the widest line.
        let codes = ["EP 1.5", "S03E01", "S03E02", "S03E01", "E100"];
        assert_eq!(code_column(codes), "EP 1.5\nS03E01\nS03E02\nE100");
        assert_eq!(code_column(std::iter::empty::<&str>()), "");
    }

    #[test]
    fn month_and_weekday_names_follow_the_language() {
        // 2026-10-17 15:40 CEST, a Saturday. The names come from the language's chrono
        // locale; the pattern around them is the catalog's.
        let cet = FixedOffset::east_opt(2 * 3600).unwrap();
        let t = cet.timestamp_opt(1_792_244_400, 0).unwrap();
        let name = |lang: Lang, pattern: &str| {
            t.format_localized(pattern, chrono_locale(lang)).to_string()
        };
        assert_eq!(name(Lang::En, "%-d %b"), "17 Oct");
        assert_eq!(name(Lang::Da, "%-d %b"), "17 okt");
        assert_eq!(name(Lang::Da, "%a"), "lør");
        assert_eq!(name(Lang::Ja, "%b"), "10月");
        assert_eq!(name(Lang::Ru, "%a"), "Сб");
    }
}
