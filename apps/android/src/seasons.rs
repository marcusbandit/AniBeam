//! Airing now's seasons: the quarter this moment falls in, the names of the four, and the
//! years the picker lists, newest first. The desktop's SeasonPicker reaches back to 1940
//! and a year ahead, where AniList's seasons start and end.

use anibeam_core::{SeasonName, SeasonPick};
use chrono::{Datelike, Utc};

/// The oldest year the picker lists.
pub const OLDEST: i32 = 1940;

/// The four quarters' message IDs, Winter first.
const NAMES: [&str; 4] = [
    "season-winter",
    "season-spring",
    "season-summer",
    "season-fall",
];
/// What the core and AniList call them, in the order of `NAMES`.
const CORE_NAMES: [&str; 4] = ["Winter", "Spring", "Summer", "Fall"];
const ORDER: [SeasonName; 4] = [
    SeasonName::Winter,
    SeasonName::Spring,
    SeasonName::Summer,
    SeasonName::Fall,
];

/// A year and a quarter, 0 for Winter to 3 for Fall.
pub type Moment = (i32, usize);

/// The quarter the clock is in now, by UTC as the desktop reads it.
pub fn now() -> Moment {
    let d = Utc::now();
    (d.year(), (d.month0() / 3) as usize)
}

/// "Spring 2027", in the app's language.
pub fn named(quarter: usize, year: i32) -> String {
    tr!(
        "season-named",
        season = tr!(NAMES[quarter.min(3)]),
        year = year.to_string()
    )
}

/// A season label the core wrote ("Fall 2012") in the app's language. Anything that is not
/// a season and a year stays as it came.
pub fn localized(label: &str) -> String {
    let parsed = label.split_once(' ').and_then(|(word, year)| {
        let quarter = CORE_NAMES.iter().position(|n| *n == word)?;
        Some(named(quarter, year.parse().ok()?))
    });
    parsed.unwrap_or_else(|| label.to_string())
}

/// The season after the one `at` is in, by AniList's quarters.
pub fn next(at: Moment) -> Moment {
    if at.1 == 3 {
        (at.0 + 1, 0)
    } else {
        (at.0, at.1 + 1)
    }
}

/// The years the picker lists: a year past this one, down to where AniList's seasons start.
pub fn years(at: Moment) -> Vec<i32> {
    (OLDEST..=at.0 + 1).rev().collect()
}

/// What a tap on a quarter of a year asks for. The next season by name is the next season.
pub fn pick(quarter: usize, year: i32, at: Moment) -> SeasonPick {
    if (year, quarter) == next(at) {
        SeasonPick::Next
    } else {
        SeasonPick::Season {
            name: ORDER[quarter.min(3)],
            year,
        }
    }
}

/// The season the page shows beside "This season": the next one, or the one picked, as the
/// desktop's seasonLabel. Empty for the current season.
pub fn label(showing: SeasonPick, at: Moment) -> String {
    match showing {
        SeasonPick::Now => String::new(),
        SeasonPick::Next => {
            let n = next(at);
            named(n.1, n.0)
        }
        SeasonPick::Season { name, year } => {
            named(ORDER.iter().position(|n| *n == name).unwrap_or(0), year)
        }
    }
}

/// The year the picker opens on: the showing season's, else this one's.
pub fn year_of(showing: SeasonPick, at: Moment) -> i32 {
    match showing {
        SeasonPick::Now => at.0,
        SeasonPick::Next => next(at).0,
        SeasonPick::Season { year, .. } => year,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_next_season_wraps_into_the_next_year() {
        assert_eq!(next((2026, 3)), (2027, 0));
        assert_eq!(next((2026, 0)), (2026, 1));
    }

    #[test]
    fn the_picker_lists_a_year_ahead_back_to_1940() {
        let y = years((2026, 3));
        assert_eq!(y.first(), Some(&2027));
        assert_eq!(y.last(), Some(&1940));
        assert_eq!(y.len(), 88);
    }

    #[test]
    fn the_next_season_by_name_is_the_next_season() {
        assert_eq!(pick(0, 2027, (2026, 3)), SeasonPick::Next);
        assert_eq!(
            pick(1, 2019, (2026, 3)),
            SeasonPick::Season {
                name: SeasonName::Spring,
                year: 2019
            }
        );
    }

    #[test]
    fn a_core_label_is_a_season_and_a_year() {
        assert_eq!(localized("Fall 2012"), "Fall 2012");
        assert_eq!(localized("Autumn"), "Autumn");
    }

    #[test]
    fn the_label_names_the_season_showing() {
        assert_eq!(label(SeasonPick::Now, (2026, 3)), "");
        assert_eq!(label(SeasonPick::Next, (2026, 3)), "Winter 2027");
        let p = pick(2, 1999, (2026, 3));
        assert_eq!(label(p, (2026, 3)), "Summer 1999");
    }
}
