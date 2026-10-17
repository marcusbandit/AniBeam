//! The words and numbers the screens show.

use chrono::{DateTime, Datelike, Local, TimeZone};

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

/// "in 2d 4h", "in 4h 12m", "in 12m 05s", "in 42s". Seconds show only inside the last
/// hour, where they are worth watching tick.
pub fn countdown(seconds: i64) -> String {
    let s = seconds.max(0);
    if s >= DAY {
        format!("in {}d {}h", s / DAY, (s % DAY) / HOUR)
    } else if s >= HOUR {
        format!("in {}h {}m", s / HOUR, (s % HOUR) / MINUTE)
    } else if s >= MINUTE {
        format!("in {}m {:02}s", s / MINUTE, s % MINUTE)
    } else {
        format!("in {s}s")
    }
}

/// The desktop series page's ticking form: "3d 02h 15m 07s", "2h 05m 07s", "15m 07s".
pub fn countdown_full(seconds: i64) -> String {
    let s = seconds.max(0);
    let (d, h, m, sec) = (s / DAY, (s % DAY) / HOUR, (s % HOUR) / MINUTE, s % MINUTE);
    if d > 0 {
        format!("{d}d {h:02}h {m:02}m {sec:02}s")
    } else if h > 0 {
        format!("{h}h {m:02}m {sec:02}s")
    } else {
        format!("{m}m {sec:02}s")
    }
}

/// "17:56" in the phone's zone.
pub fn clock(at: i64) -> String {
    Local.timestamp_opt(at, 0).single().map(|t| t.format("%H:%M").to_string()).unwrap_or_default()
}

/// "17 Oct" in the phone's zone.
pub fn day_month(at: i64) -> String {
    Local.timestamp_opt(at, 0).single().map(|t| t.format("%-d %b").to_string()).unwrap_or_default()
}

/// "just now", "1 min ago", "3h ago", "2d ago".
pub fn ago(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < MINUTE {
        "just now".into()
    } else if s < HOUR {
        format!("{} min ago", s / MINUTE)
    } else if s < DAY {
        format!("{}h ago", s / HOUR)
    } else {
        format!("{}d ago", s / DAY)
    }
}

fn day_word<Tz: TimeZone>(at: &DateTime<Tz>, now: &DateTime<Tz>) -> Option<&'static str> {
    let days = at.date_naive().num_days_from_ce() - now.date_naive().num_days_from_ce();
    match days {
        -1 => Some("Yesterday"),
        0 => Some("Today"),
        1 => Some("Tomorrow"),
        _ => None,
    }
}

/// An air time in the phone's own zone: "Today 14:00", "Tomorrow 02:30", "Sat 25 Oct 14:00".
pub fn when_in<Tz: TimeZone>(at: i64, now: i64, zone: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let (Some(at), Some(now)) = (zone.timestamp_opt(at, 0).single(), zone.timestamp_opt(now, 0).single()) else {
        return String::new();
    };
    match day_word(&at, &now) {
        Some(word) => format!("{word} {}", at.format("%H:%M")),
        None if at.year() == now.year() => at.format("%a %-d %b %H:%M").to_string(),
        None => at.format("%a %-d %b %Y %H:%M").to_string(),
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
        .map(|t| t.format("%b %-d %H:%M").to_string())
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

/// "SN" for "Sousou no Frieren": what a tile without a poster shows.
pub fn initials(title: &str) -> String {
    title
        .split_whitespace()
        .filter_map(|w| w.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

/// "1.4 GB", "820 MB", "12 KB".
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 || value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// How a torrent row reads and how loud it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Moving: the accent.
    Active,
    /// Finished or idle: plain text.
    Quiet,
    /// Stuck or broken: the warning hue.
    Trouble,
}

/// qBittorrent's state word in the desktop's words.
pub fn torrent_state(state: &str) -> (&'static str, Tone) {
    match state {
        "downloading" | "forcedDL" => ("Downloading", Tone::Active),
        "metaDL" | "forcedMetaDL" => ("Fetching metadata", Tone::Active),
        "stalledDL" => ("Stalled", Tone::Trouble),
        "queuedDL" | "queuedUP" => ("Queued", Tone::Quiet),
        "pausedDL" | "stoppedDL" => ("Stopped", Tone::Quiet),
        "pausedUP" | "stoppedUP" => ("Done", Tone::Quiet),
        "uploading" | "stalledUP" | "forcedUP" => ("Seeding", Tone::Quiet),
        "checkingDL" | "checkingUP" | "checkingResumeData" | "moving" => ("Checking", Tone::Quiet),
        "error" => ("Error", Tone::Trouble),
        "missingFiles" => ("Files missing", Tone::Trouble),
        _ => ("Unknown", Tone::Quiet),
    }
}

/// "1.2 GB of 4.5 GB, 3.4 MB/s, 12m left", or just the size once it is done.
pub fn torrent_facts(progress: f64, size: u64, speed: u64, eta: Option<i64>) -> String {
    let progress = progress.clamp(0.0, 1.0);
    if progress >= 1.0 {
        return bytes(size);
    }
    let mut facts = format!("{} of {}", bytes((size as f64 * progress) as u64), bytes(size));
    if speed > 0 {
        facts.push_str(&format!(", {}/s", bytes(speed)));
    }
    // qBittorrent says 8640000 (100 days) for "no idea".
    if let Some(eta) = eta.filter(|e| *e > 0 && *e < 8_640_000) {
        facts.push_str(&format!(", {} left", countdown(eta).trim_start_matches("in ")));
    }
    facts
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn countdowns_lose_precision_with_distance() {
        assert_eq!(countdown(2 * DAY + 4 * HOUR + 59 * MINUTE), "in 2d 4h");
        assert_eq!(countdown(4 * HOUR + 12 * MINUTE + 30), "in 4h 12m");
        assert_eq!(countdown(12 * MINUTE + 5), "in 12m 05s");
        assert_eq!(countdown(42), "in 42s");
        assert_eq!(countdown(-5), "in 0s");
        assert_eq!(countdown_full(3 * DAY + 2 * HOUR + 15 * MINUTE + 7), "3d 02h 15m 07s");
        assert_eq!(countdown_full(2 * HOUR + 5 * MINUTE + 7), "2h 05m 07s");
        assert_eq!(countdown_full(15 * MINUTE + 7), "15m 07s");
    }

    #[test]
    fn ago_reads_like_a_status_line() {
        assert_eq!(ago(20), "just now");
        assert_eq!(ago(61), "1 min ago");
        assert_eq!(ago(3 * HOUR + 5), "3h ago");
        assert_eq!(ago(2 * DAY), "2d ago");
    }

    #[test]
    fn air_times_name_the_day_when_it_is_close() {
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
    fn codes_sizes_and_torrent_words() {
        assert_eq!(episode_code(None, 8), "E08");
        assert_eq!(episode_code(Some(1), 8), "E08");
        assert_eq!(episode_code(Some(2), 1), "S02E01");
        assert_eq!(initials("Sousou no Frieren"), "SN");
        assert_eq!(initials("Mushishi"), "M");
        assert_eq!(initials("[Oshi no Ko]"), "ON");
        assert_eq!(bytes(1_446_000_000), "1.4 GB");
        assert_eq!(bytes(820_000_000), "820 MB");
        assert_eq!(bytes(512), "512 B");
        assert_eq!(torrent_state("stalledDL"), ("Stalled", Tone::Trouble));
        assert_eq!(torrent_state("pausedUP").0, "Done");
        assert_eq!(torrent_facts(0.5, 1_400_000_000, 3_400_000, Some(240)), "700 MB of 1.4 GB, 3.4 MB/s, 4m 00s left");
        assert_eq!(torrent_facts(1.0, 1_446_000_000, 0, None), "1.4 GB");
        assert_eq!(torrent_facts(0.1, 1_000_000_000, 0, Some(8_640_000)), "100 MB of 1.0 GB");
    }
}
