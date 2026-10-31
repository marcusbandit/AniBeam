//! The library sort and the search query, carried from HomePage.tsx's
//! comparator factory (`sortedItems`) and its search filter. One rule for
//! both lists: Electron had two slightly different query rules and the
//! union covers both.

use std::cmp::Ordering;

use crate::contract::*;
use crate::library::cards;
use crate::time;

/// The title a card sorts by. A native title sorts by its romaji: kanji
/// carry no reading, so code point order would scatter them.
fn title_of(c: &SeriesCard) -> String {
    let shown = c.title.as_str();
    let key = if c.titles.native.as_deref() == Some(shown) {
        c.titles
            .romaji
            .as_deref()
            .or(c.titles.english.as_deref())
            .unwrap_or(shown)
    } else {
        shown
    };
    key.to_lowercase()
}

/// Finished with it: completed on the tracker, watched at or above a
/// published total, or a film with any watch at all. An estimated total is
/// only how far the user has got, so reaching it is not finished.
fn watched_through(c: &SeriesCard) -> bool {
    if c.list_status == Some(ListStatus::Completed) {
        return true;
    }
    let Some(w) = c.watched.filter(|w| *w > 0) else {
        return false;
    };
    if cards::is_film(c) {
        return true;
    }
    if c.total_is_estimate {
        return false;
    }
    c.total_episodes.is_some_and(|t| w >= t)
}

/// Not mid-way through: finished with it, or never started.
fn progress_inactive(c: &SeriesCard) -> bool {
    watched_through(c) || c.watched.is_none_or(|w| w == 0)
}

fn value_of(c: &SeriesCard, sort: Sort) -> Option<f64> {
    match sort {
        Sort::LastViewed => c.last_viewed_at.map(|t| time::to_secs(t) as f64),
        Sort::Progress => match c.total_episodes {
            None => Some(if c.watched.unwrap_or(0) > 0 { 1.0 } else { 0.0 }),
            Some(0) => None,
            Some(t) => Some((f64::from(c.watched.unwrap_or(0)) / f64::from(t)).clamp(0.0, 1.0)),
        },
        Sort::CommunityScore => c.community_score,
        Sort::MyScore => c.my_score,
        Sort::Downloaded => c.downloaded_at.map(|t| time::to_secs(t) as f64),
        Sort::Alpha => Some(0.0),
    }
}

/// Nulls sink whatever the direction, ties break A to Z, and the progress
/// sort pins completed and not-started series to the bottom.
pub fn sort_cards(cards: &mut [SeriesCard], sort: Sort, direction: Direction) {
    let dir = match direction {
        Direction::Asc => 1.0,
        Direction::Desc => -1.0,
    };
    cards.sort_by(|a, b| {
        if sort == Sort::Progress {
            let (ia, ib) = (progress_inactive(a), progress_inactive(b));
            if ia != ib {
                return if ia {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }
            if ia && ib {
                return title_of(a).cmp(&title_of(b));
            }
        }
        if sort == Sort::Alpha {
            let o = title_of(a).cmp(&title_of(b));
            return if dir < 0.0 { o.reverse() } else { o };
        }
        match (value_of(a, sort), value_of(b, sort)) {
            (None, None) => title_of(a).cmp(&title_of(b)),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(va), Some(vb)) if va == vb => title_of(a).cmp(&title_of(b)),
            (Some(va), Some(vb)) => ((va - vb) * dir)
                .partial_cmp(&0.0)
                .unwrap_or(Ordering::Equal),
        }
    });
}

/// Trimmed and lower-cased; a card matches when any of romaji, english,
/// native, the folder name or the resolved title contains it.
pub fn matches_query(c: &SeriesCard, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    [
        Some(c.title.as_str()),
        c.titles.romaji.as_deref(),
        c.titles.english.as_deref(),
        c.titles.native.as_deref(),
        Some(c.titles.folder.as_str()),
    ]
    .into_iter()
    .flatten()
    .any(|t| t.to_lowercase().contains(&q))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[allow(clippy::too_many_arguments)]
    fn card(
        title: &str,
        watched: Option<u32>,
        total: Option<u32>,
        score: Option<f64>,
        my: Option<f64>,
        viewed: Option<u64>,
        status: Option<ListStatus>,
    ) -> SeriesCard {
        SeriesCard {
            id: 0,
            kind: SeriesKind::Show,
            path: String::new(),
            title: title.into(),
            titles: Titles {
                romaji: Some(title.into()),
                english: None,
                native: None,
                folder: title.into(),
            },
            poster: None,
            format: None,
            genres: Vec::new(),
            status: None,
            hidden: false,
            missing: false,
            match_info: None,
            episodes_on_disk: 1,
            extras_on_disk: 0,
            total_episodes: total,
            total_is_estimate: false,
            episodes_set: None,
            episodes_set_applied: false,
            count_season: None,
            bytes_on_disk: 0,
            watched_bytes: 0,
            watched_on_disk: 0,
            finished_watched: false,
            code: None,
            watched,
            watched_state: WatchedState::Unknown,
            strip: Strip {
                watched: 0.0,
                aired_unwatched: 0.0,
                unknown: 0.0,
            },
            cubes: String::new(),
            episodes_here: Vec::new(),
            community_score: score,
            my_score: my,
            list_status: status,
            next_airing: None,
            week_airings: Vec::new(),
            last_viewed_at: viewed.map(|s| UNIX_EPOCH + std::time::Duration::from_secs(s)),
            latest_activity_at: UNIX_EPOCH,
            added_at: UNIX_EPOCH,
            downloaded_at: None,
            downloading: Vec::new(),
            franchise: None,
            year: None,
            season: None,
            imdb_id: None,
            imdb_score: None,
            imdb_votes: None,
        }
    }

    fn titles(cards: &[SeriesCard]) -> Vec<&str> {
        cards.iter().map(|c| c.title.as_str()).collect()
    }

    #[test]
    fn alpha_follows_direction_and_ignores_case() {
        let mut cards = vec![
            card("b", None, None, None, None, None, None),
            card("A", None, None, None, None, None, None),
            card("c", None, None, None, None, None, None),
        ];
        sort_cards(&mut cards, Sort::Alpha, Direction::Asc);
        assert_eq!(titles(&cards), vec!["A", "b", "c"]);
        sort_cards(&mut cards, Sort::Alpha, Direction::Desc);
        assert_eq!(titles(&cards), vec!["c", "b", "A"]);
    }

    #[test]
    fn nulls_sink_whatever_the_direction_and_ties_break_a_to_z() {
        let mut cards = vec![
            card("x", None, None, Some(7.0), None, None, None),
            card("n2", None, None, None, None, None, None),
            card("y", None, None, Some(8.0), None, None, None),
            card("n1", None, None, None, None, None, None),
            card("w", None, None, Some(7.0), None, None, None),
        ];
        sort_cards(&mut cards, Sort::CommunityScore, Direction::Desc);
        assert_eq!(titles(&cards), vec!["y", "w", "x", "n1", "n2"]);
        sort_cards(&mut cards, Sort::CommunityScore, Direction::Asc);
        assert_eq!(titles(&cards), vec!["w", "x", "y", "n1", "n2"]);
    }

    #[test]
    fn progress_pins_completed_and_not_started_to_the_bottom() {
        let mut cards = vec![
            card("done", Some(12), Some(12), None, None, None, None),
            card("mid", Some(6), Some(12), None, None, None, None),
            card("never", None, Some(12), None, None, None, None),
            card("zero", Some(0), Some(12), None, None, None, None),
            card("early", Some(2), Some(12), None, None, None, None),
            card(
                "dropped-complete",
                Some(3),
                Some(12),
                None,
                None,
                None,
                Some(ListStatus::Completed),
            ),
        ];
        sort_cards(&mut cards, Sort::Progress, Direction::Desc);
        assert_eq!(
            titles(&cards),
            vec!["mid", "early", "done", "dropped-complete", "never", "zero"]
        );
        sort_cards(&mut cards, Sort::Progress, Direction::Asc);
        assert_eq!(
            titles(&cards),
            vec!["early", "mid", "done", "dropped-complete", "never", "zero"]
        );
    }

    #[test]
    fn progress_keeps_a_show_caught_up_to_an_estimate_in_play() {
        // No published total: the estimate is how far the user has got,
        // so a show caught up to it is still in progress, and so is one
        // with nothing knowable at all. A film seen once is finished.
        let mut caught_up = card("caught-up", Some(8), Some(8), None, None, None, None);
        caught_up.total_is_estimate = true;
        let unknown = card("unknown", Some(3), None, None, None, None, None);
        let mut film = card("film", Some(1), None, None, None, None, None);
        film.kind = SeriesKind::Movie;
        let mut cards = vec![
            film,
            card("done", Some(12), Some(12), None, None, None, None),
            caught_up,
            unknown,
            card("mid", Some(6), Some(12), None, None, None, None),
        ];
        sort_cards(&mut cards, Sort::Progress, Direction::Desc);
        assert_eq!(
            titles(&cards),
            vec!["caught-up", "unknown", "mid", "done", "film"]
        );
    }

    #[test]
    fn last_viewed_and_query() {
        let mut cards = vec![
            card("old", None, None, None, None, Some(10), None),
            card("new", None, None, None, None, Some(20), None),
            card("never", None, None, None, None, None, None),
        ];
        sort_cards(&mut cards, Sort::LastViewed, Direction::Desc);
        assert_eq!(titles(&cards), vec!["new", "old", "never"]);
        let c = card("Sousou no Frieren", None, None, None, None, None, None);
        assert!(matches_query(&c, " FRIEREN "));
        assert!(!matches_query(&c, "bleach"));
        assert!(matches_query(&c, ""));
    }
}
