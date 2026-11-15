//! A feed's size text as bytes. Every reader of a release's size goes
//! through here: the downloads list, the JSON kind, the best-fit pick and the
//! phone's planner.

use std::sync::LazyLock;

use regex::Regex;

/// A size anywhere in the text, so "Size: 1.4 GiB" reads, but never the tail of a longer
/// number: a size right after a digit, a sign or a mark ("1 234 MiB", "-1 GiB") reads as
/// none rather than as its last group.
static SIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:^|[^\d\s.,+-])\s*([\d.,]*\d)\s*([KMGT])?i?B\b").unwrap());

/// "1.4 GiB", "6,2 GiB", "700 MB", "1,234.5 MiB" as bytes, unrounded; a
/// decimal unit is read as the binary one, since every feed that writes "GB"
/// means GiB. The one reading of a feed's size: the downloads list, the JSON
/// kind and the best-fit pick all go through it.
pub fn size_in_bytes(text: &str) -> Option<f64> {
    let c = SIZE.captures(text)?;
    let n = size_number(&c[1])?;
    let scale: f64 = match c.get(2).map(|u| u.as_str().to_ascii_uppercase()) {
        None => 1.0,
        Some(u) => match u.as_str() {
            "K" => 1024.0,
            "M" => 1024.0 * 1024.0,
            "G" => 1024.0 * 1024.0 * 1024.0,
            "T" => 1024.0_f64.powi(4),
            _ => return None,
        },
    };
    Some(n * scale)
}

/// [`size_in_bytes`] rounded to a whole byte.
pub fn size_bytes(text: &str) -> Option<u64> {
    let b = size_in_bytes(text)?.round();
    (b.is_finite() && b < u64::MAX as f64).then_some(b as u64)
}

/// A size's number, with '.' or ',' as its decimal mark. With both, the last one is
/// the decimal mark and the other separates thousands: "1,024.5" and "1.024,5" are both
/// 1024.5. A lone ',' separates thousands only when three digits follow it: "1,024" is
/// 1024.0, "6,2" is 6.2.
fn size_number(raw: &str) -> Option<f64> {
    let text = raw.trim();
    let text = if text.contains('.') && text.contains(',') {
        if text.rfind(',') > text.rfind('.') {
            text.replace('.', "").replace(',', ".")
        } else {
            text.replace(',', "")
        }
    } else if text.contains(',') {
        let (head, tail) = text.rsplit_once(',').unwrap_or(("", text));
        let joiner = if tail.chars().count() == 3 { "" } else { "." };
        format!("{}{joiner}{tail}", head.replace(',', ""))
    } else {
        text.to_string()
    };
    text.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_in_any_unit() {
        assert_eq!(size_bytes("700 MiB"), Some(734_003_200));
        assert_eq!(size_bytes("700 MB"), Some(734_003_200));
        assert_eq!(size_bytes("1,024 KiB"), Some(1_048_576));
        assert_eq!(size_bytes("12 B"), Some(12));
        assert_eq!(size_bytes(""), None);
        assert_eq!(size_bytes("big"), None);
    }

    #[test]
    fn a_comma_is_a_decimal_mark_unless_three_digits_follow() {
        // "6,2 GiB" is six point two, never sixty-two (#853).
        assert_eq!(size_in_bytes("6,2 GiB"), size_in_bytes("6.2 GiB"));
        assert_eq!(size_bytes("6,2 GiB"), Some(6_657_199_309));
        assert_eq!(size_bytes("1,234 MiB"), Some(1234 * 1024 * 1024));
        assert_eq!(size_in_bytes("301.4 MiB"), Some(301.4 * 1024.0 * 1024.0));
        assert_eq!(size_number("1,024"), Some(1024.0));
        assert_eq!(size_number("6,2"), Some(6.2));
        assert_eq!(size_number("1,024.5"), Some(1024.5));
        assert_eq!(size_number("6.2"), Some(6.2));
        assert_eq!(size_number("1,234,567"), Some(1234567.0));
        assert_eq!(size_number("1.2.3"), None);
        assert_eq!(size_number("1.234,5"), Some(1234.5));
    }

    #[test]
    fn a_size_is_never_the_tail_of_a_longer_number() {
        assert_eq!(size_in_bytes(".5 GiB"), size_in_bytes("0.5 GiB"));
        assert_eq!(size_bytes("Size: 1.5 KiB"), Some(1536));
        assert_eq!(size_bytes("1 234 MiB"), None);
        assert_eq!(size_bytes("-1 GiB"), None);
        assert_eq!(size_bytes("1.234,5 MiB"), size_bytes("1234.5 MiB"));
    }
}
