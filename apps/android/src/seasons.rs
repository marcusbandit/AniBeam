//! Airing now's seasons: the quarter this moment falls in, the names of the four, and the
//! years the picker lists, newest first. The desktop's SeasonPicker reaches back to 1940
//! and a year ahead, where AniList's seasons start and end.

use anibeam_core::{SeasonName, SeasonPick};
use chrono::{Datelike, Utc};

/// The oldest year the picker lists.
const OLDEST: i32 = 1940;

/// The four quarters, Winter first: the core's name, the word the core and AniList write in
/// a label, and the message ID.
const QUARTERS: [(SeasonName, &str, &str); 4] = [
    (SeasonName::Winter, "Winter", "season-winter"),
    (SeasonName::Spring, "Spring", "season-spring"),
    (SeasonName::Summer, "Summer", "season-summer"),
    (SeasonName::Fall, "Fall", "season-fall"),
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
        season = tr!(QUARTERS[quarter.min(3)].2),
        year = year.to_string()
    )
}

/// A season label the core wrote ("Fall 2012") in the app's language. Anything that is not
/// a season and a year stays as it came.
pub fn localized(label: &str) -> String {
    let parsed = label.split_once(' ').and_then(|(word, year)| {
        let quarter = QUARTERS.iter().position(|(_, core, _)| *core == word)?;
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
            name: QUARTERS[quarter.min(3)].0,
            year,
        }
    }
}

/// The season showing as a year and quarter, for the picker to mark; none for this season.
pub fn moment_of(showing: SeasonPick, at: Moment) -> Option<Moment> {
    match showing {
        SeasonPick::Now => None,
        SeasonPick::Next => Some(next(at)),
        SeasonPick::Season { name, year } => {
            Some((year, QUARTERS.iter().position(|q| q.0 == name).unwrap_or(0)))
        }
    }
}

/// The season the page shows beside "This season": the next one, or the one picked, as the
/// desktop's seasonLabel. Empty for the current season.
pub fn label(showing: SeasonPick, at: Moment) -> String {
    moment_of(showing, at).map_or_else(String::new, |(year, quarter)| named(quarter, year))
}

/// The year the picker opens on: the showing season's, else this one's.
pub fn year_of(showing: SeasonPick, at: Moment) -> i32 {
    moment_of(showing, at).map_or(at.0, |(year, _)| year)
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
    fn the_picker_marks_and_opens_on_the_season_showing() {
        let at = (2026, 3);
        assert_eq!(moment_of(SeasonPick::Now, at), None);
        assert_eq!(moment_of(SeasonPick::Next, at), Some((2027, 0)));
        let spring = SeasonPick::Season {
            name: SeasonName::Spring,
            year: 2019,
        };
        assert_eq!(moment_of(spring, at), Some((2019, 1)));
        assert_eq!(year_of(SeasonPick::Now, at), 2026);
        assert_eq!(year_of(SeasonPick::Next, at), 2027);
        assert_eq!(year_of(spring, at), 2019);
    }

    #[test]
    fn the_label_names_the_season_showing() {
        assert_eq!(label(SeasonPick::Now, (2026, 3)), "");
        assert_eq!(label(SeasonPick::Next, (2026, 3)), "Winter 2027");
        let p = pick(2, 1999, (2026, 3));
        assert_eq!(label(p, (2026, 3)), "Summer 1999");
    }
}
