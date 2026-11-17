//! Readouts. Every rule here is Electron's, from `relativeTime.ts`, `airingUtils.ts` and
//! `VideoPlayer.tsx`: no weeks bucket, zero-padded lower units on a countdown, `m:ss` under
//! an hour, base 1024 bytes with one decimal past bytes.

use anibeam_core::i18n::{Arg, tr};

const MIN: f64 = 60.0;
const HOUR: f64 = 3600.0;
const DAY: f64 = 86400.0;
const MONTH: f64 = 30.0 * DAY;

/// "just now", "5m ago", "3h ago", "29d ago", "1mo ago", "1y 1mo ago", or "in 2h".
pub fn relative(ts_secs: f64, now_secs: f64) -> String {
    let diff = now_secs - ts_secs;
    let abs = diff.abs();
    if abs < MIN {
        return tr("fmt-just-now", &[]);
    }
    let unit = |id: &str, n: u64| tr(id, &[("n", n.into())]);
    let label = if abs < HOUR {
        unit("fmt-minutes-short", (abs / MIN).floor() as u64)
    } else if abs < DAY {
        unit("fmt-hours-short", (abs / HOUR).floor() as u64)
    } else if abs < MONTH {
        unit("fmt-days-short", (abs / DAY).floor() as u64)
    } else {
        let total_mo = (abs / MONTH).floor() as u64;
        let y = total_mo / 12;
        let mo = total_mo % 12;
        if y > 0 {
            if mo > 0 {
                tr(
                    "fmt-years-months-short",
                    &[("years", y.into()), ("months", mo.into())],
                )
            } else {
                unit("fmt-years-short", y)
            }
        } else {
            unit("fmt-months-short", total_mo)
        }
    };
    let id = if diff < 0.0 { "fmt-in" } else { "fmt-ago" };
    tr(id, &[("span", label.into())])
}

/// Two digits, as the lower units of a countdown show them: "04".
fn pad(n: u64) -> Arg<'static> {
    Arg::from(format!("{n:02}"))
}

/// "2d 04h 05m", "4h 12m", "12m", or "now".
pub fn countdown(secs_left: f64) -> String {
    if secs_left.is_nan() || secs_left <= 0.0 {
        return tr("fmt-now", &[]);
    }
    let total = secs_left.floor() as u64;
    let (d, h, m) = (total / 86400, (total % 86400) / 3600, (total % 3600) / 60);
    if d > 0 {
        tr(
            "fmt-countdown-days",
            &[("d", d.into()), ("h", pad(h)), ("m", pad(m))],
        )
    } else if h > 0 {
        tr("fmt-countdown-hours", &[("h", h.into()), ("m", pad(m))])
    } else {
        tr("fmt-minutes-short", &[("n", m.into())])
    }
}

/// The series hero's countdown, with seconds: "1d 01h 01m 01s", "1h 01m 01s", "1m 01s", "59s".
pub fn countdown_seconds(secs_left: f64) -> String {
    if secs_left.is_nan() || secs_left <= 0.0 {
        return tr("fmt-now", &[]);
    }
    let total = secs_left.floor() as u64;
    let (d, h, m, s) = (
        total / 86400,
        (total % 86400) / 3600,
        (total % 3600) / 60,
        total % 60,
    );
    if d > 0 {
        tr(
            "fmt-countdown-days-seconds",
            &[("d", d.into()), ("h", pad(h)), ("m", pad(m)), ("s", pad(s))],
        )
    } else if h > 0 {
        tr(
            "fmt-countdown-hours-seconds",
            &[("h", h.into()), ("m", pad(m)), ("s", pad(s))],
        )
    } else if m > 0 {
        tr(
            "fmt-countdown-minutes-seconds",
            &[("m", m.into()), ("s", pad(s))],
        )
    } else {
        tr("fmt-seconds-short", &[("n", s.into())])
    }
}

/// "m:ss", or "h:mm:ss" once there is an hour.
pub fn clock(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00".to_string();
    }
    let total = secs.floor() as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// "m:ss.mmm", or "h:mm:ss.mmm" once there is an hour: the frame step HUD.
pub fn clock_ms(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00.000".to_string();
    }
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total = total_ms / 1000;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}.{ms:03}")
    } else {
        format!("{m}:{s:02}.{ms:03}")
    }
}

/// How long an episode or a film runs: "24 min", "1 h 52 min", or "2 h" on the hour. An
/// unknown length is the caller's to leave out, never a 0 passed here.
pub fn runtime(minutes: u32) -> String {
    let (h, m) = (u64::from(minutes / 60), u64::from(minutes % 60));
    match (h, m) {
        (0, m) => tr("fmt-runtime-minutes", &[("m", m.into())]),
        (h, 0) => tr("fmt-runtime-hours", &[("h", h.into())]),
        (h, m) => tr(
            "fmt-runtime-hours-minutes",
            &[("h", h.into()), ("m", m.into())],
        ),
    }
}

/// A size in the language's own units and decimal mark: "1.5 GB", "1,5 Go" in French.
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 5] = [
        "fmt-size-b",
        "fmt-size-kb",
        "fmt-size-mb",
        "fmt-size-gb",
        "fmt-size-tb",
    ];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    let size = if i == 0 {
        n.to_string()
    } else {
        anibeam_core::i18n::decimal(&format!("{v:.1}"))
    };
    tr(UNITS[i], &[("size", size.into())])
}

/// An episode's name wherever one is shown (the series ledger, the player's header, the
/// feed): its own title, else "Episode 13" ("Episode 12.5" for a half-numbered one). A file
/// name is never a title: AniList often lists fewer titled episodes than are on disk.
pub fn episode_name(title: Option<&str>, number: f64) -> String {
    match title.map(str::trim) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ if number.is_finite() && number.fract() == 0.0 => {
            tr("episode-number", &[("number", (number as i64).into())])
        }
        _ if number.is_finite() => tr(
            "episode-number",
            &[("number", ((number * 100.0).round() / 100.0).into())],
        ),
        _ => tr("fmt-episode", &[]),
    }
}

/// The episodes one airing drop releases: "Episode 7", or "Episodes 7 to 9"
/// when `through` names a later one landing at the same moment.
pub fn episode_span(first: u32, through: u32) -> String {
    if through > first {
        tr(
            "fmt-episode-span",
            &[("first", first.into()), ("through", through.into())],
        )
    } else {
        tr("episode-number", &[("number", first.into())])
    }
}

pub fn plural(n: u64, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}

/// `{x:.1}` alone ties to even at an exact half (7.25 -> "7.2"); Electron's `toFixed` ties
/// away from zero instead. Scaling by 10 or 100 to find the tie is unreliable: `0.15 * 100.0`
/// lands on the exact integer 15 too, even though 0.15 isn't a real binary tie, so that test
/// misclassifies ordinary values `{x:.1}` already gets right. A real tie is instead an exact
/// quarter with an odd numerator: `x * 4.0` is exact (a pure exponent shift, no rounding), and
/// `.25`/`.75` are ties while `.0`/`.5` are not.
pub fn score(x: f64) -> String {
    let quarters = x * 4.0;
    let is_tie = quarters.fract() == 0.0 && (quarters as i64).rem_euclid(2) != 0;
    let text = if is_tie {
        format!("{:.1}", (x * 10.0).round() / 10.0)
    } else {
        format!("{x:.1}")
    };
    anibeam_core::i18n::decimal(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_time_has_no_weeks_bucket_and_says_in_or_ago() {
        let now = 1_000_000.0;
        assert_eq!(relative(now - 30.0, now), "just now");
        assert_eq!(relative(now - 5.0 * 60.0, now), "5m ago");
        assert_eq!(relative(now - 3.0 * 3600.0, now), "3h ago");
        assert_eq!(relative(now - 29.0 * 86400.0, now), "29d ago");
        assert_eq!(relative(now - 45.0 * 86400.0, now), "1mo ago");
        assert_eq!(relative(now - 400.0 * 86400.0, now), "1y 1mo ago");
        assert_eq!(relative(now - 730.0 * 86400.0, now), "2y ago");
        assert_eq!(relative(now + 2.0 * 3600.0, now), "in 2h");
    }

    #[test]
    fn episode_spans_name_one_or_several() {
        assert_eq!(episode_span(7, 0), "Episode 7");
        assert_eq!(episode_span(7, 7), "Episode 7");
        assert_eq!(episode_span(7, 9), "Episodes 7 to 9");
    }

    #[test]
    fn countdowns_pad_the_lower_units() {
        assert_eq!(countdown(0.0), "now");
        assert_eq!(countdown(12.0 * 60.0), "12m");
        assert_eq!(countdown(4.0 * 3600.0 + 12.0 * 60.0), "4h 12m");
        assert_eq!(
            countdown(2.0 * 86400.0 + 4.0 * 3600.0 + 5.0 * 60.0),
            "2d 04h 05m"
        );
        assert_eq!(countdown_seconds(59.0), "59s");
        assert_eq!(countdown_seconds(61.0), "1m 01s");
        assert_eq!(countdown_seconds(3661.0), "1h 01m 01s");
        assert_eq!(countdown_seconds(90061.0), "1d 01h 01m 01s");
    }

    #[test]
    fn clocks_switch_to_hours_and_keep_milliseconds() {
        assert_eq!(clock(-1.0), "0:00");
        assert_eq!(clock(65.0), "1:05");
        assert_eq!(clock(3665.0), "1:01:05");
        assert_eq!(clock_ms(95.9705), "1:35.971");
        assert_eq!(clock_ms(3600.5), "1:00:00.500");
        assert_eq!(clock_ms(f64::NAN), "0:00.000");
    }

    #[test]
    fn runtimes_read_in_hours_and_minutes() {
        assert_eq!(runtime(24), "24 min");
        assert_eq!(runtime(59), "59 min");
        assert_eq!(runtime(60), "1 h");
        assert_eq!(runtime(112), "1 h 52 min");
        assert_eq!(runtime(120), "2 h");
    }

    #[test]
    fn bytes_use_base_1024_with_one_decimal_above_bytes() {
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(1536), "1.5 KB");
        assert_eq!(bytes(312 * 1024 * 1024), "312.0 MB");
        assert_eq!(bytes(5 * 1024 * 1024 * 1024), "5.0 GB");
        assert_eq!(bytes(3 * 1024 * 1024 * 1024 * 1024 / 2), "1.5 TB");
    }

    #[test]
    fn plural_picks_the_word_by_count() {
        assert_eq!(plural(1, "file", "files"), "1 file");
        assert_eq!(plural(3, "file", "files"), "3 files");
    }

    #[test]
    fn an_untitled_episode_is_named_by_its_number() {
        assert_eq!(
            episode_name(Some("Hitagi Crab, Part 1"), 1.0),
            "Hitagi Crab, Part 1"
        );
        assert_eq!(episode_name(None, 13.0), "Episode 13");
        assert_eq!(episode_name(Some("  "), 13.0), "Episode 13");
        assert_eq!(episode_name(None, 12.5), "Episode 12.5");
    }

    #[test]
    fn score_rounds_only_a_genuine_tie_away_from_zero() {
        // Adjacent non-tie hundredths: `x * 100.0` lands on an exact integer for several of
        // these even though the value isn't a real binary tie, so `{x:.1}` alone must be
        // trusted here.
        assert_eq!(score(0.15), "0.1");
        assert_eq!(score(0.35), "0.3");
        assert_eq!(score(6.85), "6.8");
        assert_eq!(score(7.05), "7.0");
        assert_eq!(score(7.35), "7.3");
        assert_eq!(score(7.85), "7.8");
        // Genuine ties: exact quarters, where Rust's default formatting ties to even.
        assert_eq!(score(0.25), "0.3");
        assert_eq!(score(1.25), "1.3");
        assert_eq!(score(7.25), "7.3");
        // Not a tie, and not exactly n.95 either; the surrounding non-tie cases.
        assert_eq!(score(9.95), "9.9");
        assert_eq!(score(10.0), "10.0");
    }
}
