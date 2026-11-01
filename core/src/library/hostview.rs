//! The library as the tailnet host serves it (`host`): each series reduced to
//! the wire record the phone app reads. Built off the same `Snapshot` the
//! cards are, so the phone and the desktop page can never disagree about
//! what is on disk, what is watched or when the next episode airs.

use std::collections::HashSet;

use anibeam_host_api as wire;

use crate::contract::*;
use crate::library::classifier::format_number;
use crate::library::snapshot::{FileRow, SeriesRow, Snapshot};
use crate::time;

/// How many future airings one series carries. A long-running show has
/// hundreds scheduled; a phone counting down needs the next few months.
pub(crate) const UPCOMING_CAP: usize = 26;

/// One series on the wire, and the poster file behind its `poster` tag.
pub(crate) struct HostSeries {
    pub series: wire::Series,
    /// The local file the card's poster is. `host` stats it for the tag and
    /// serves it; nothing about it reaches the wire but the tag.
    pub poster_path: Option<String>,
}

impl Snapshot {
    /// Every series that is in the library and not hidden, in id order.
    /// A missing series (its folder is gone) is out: nothing on disk to
    /// show for it.
    pub(crate) fn host_series(&self) -> Vec<HostSeries> {
        self.series
            .iter()
            .filter(|row| !row.hidden && !row.missing)
            .map(|row| self.host_row(row))
            .collect()
    }

    fn host_row(&self, row: &SeriesRow) -> HostSeries {
        let now = time::to_secs(self.now);
        let card = self.card_of(row);
        let m = card.match_info.as_ref();
        let upcoming = self.upcoming(row, now);
        let completed: HashSet<&str> = self
            .completed
            .get(&row.id)
            .map(|rows| rows.iter().map(|(k, _)| k.as_str()).collect())
            .unwrap_or_default();
        let episodes = self
            .episodes_on_disk(row.id)
            .into_iter()
            .map(|f| self.host_episode(row, f, &completed, card.watched))
            .collect();
        let series = wire::Series {
            id: row.id,
            kind: match row.kind {
                SeriesKind::Show => wire::Kind::Show,
                SeriesKind::Movie => wire::Kind::Movie,
            },
            title: card.title.clone(),
            titles: wire::Titles {
                romaji: card.titles.romaji.clone(),
                english: card.titles.english.clone(),
                native: card.titles.native.clone(),
            },
            status: card.status.map(|s| wire_status(s).to_string()),
            format: card.format.clone(),
            year: card.year,
            season: card.season.clone(),
            anilist_id: m.and_then(|m| m.anilist_id),
            mal_id: m.and_then(|m| m.mal_id),
            tvmaze_id: m.and_then(|m| m.tvmaze_id),
            total_episodes: card.total_episodes,
            episodes_on_disk: card.episodes_on_disk,
            progress: card.watched,
            next_airing: upcoming.first().cloned(),
            upcoming,
            poster: None,
            episodes,
            added_at: row.added_at,
            latest_activity_at: time::to_secs(card.latest_activity_at),
        };
        HostSeries {
            series,
            poster_path: row.poster_path.clone(),
        }
    }

    /// Every episode still to air, soonest first, capped. An AniList match
    /// has one schedule; a TVmaze match has a schedule per season, and an
    /// announced next season counts, the same as it does for the card's
    /// `next_airing`, so `upcoming[0]` is that card's next airing.
    fn upcoming(&self, row: &SeriesRow, now: i64) -> Vec<wire::Airing> {
        let mut out: Vec<wire::Airing> = if row.provider == Some(Provider::Tvmaze) {
            self.tv_seasons_of(row)
                .flat_map(|(season, eps)| eps.iter().map(move |a| (Some(season), a)))
                .filter_map(|(season, a)| {
                    a.aired_at.filter(|at| *at > now).map(|at| wire::Airing {
                        episode: a.number,
                        season,
                        at,
                    })
                })
                .collect()
        } else {
            self.airing_of(row)
                .iter()
                .filter_map(|a| {
                    a.aired_at.filter(|at| *at > now).map(|at| wire::Airing {
                        episode: a.number,
                        season: None,
                        at,
                    })
                })
                .collect()
        };
        out.sort_by_key(|a| a.at);
        out.truncate(UPCOMING_CAP);
        out
    }

    /// One file on disk. `watched` is the series page's rule: a completed
    /// view of the episode, or the tracker's progress reaching its number.
    fn host_episode(
        &self,
        row: &SeriesRow,
        f: &FileRow,
        completed: &HashSet<&str>,
        tracked: Option<u32>,
    ) -> wire::Episode {
        let air = self.episode_airing(row, f);
        wire::Episode {
            key: f.episode_key.clone(),
            number: f.number,
            season: f.season,
            code: episode_code(f.season, f.number),
            file: Some(f.id),
            title: air.and_then(|a| a.title.clone()),
            watched: completed.contains(f.episode_key.as_str())
                || tracked.is_some_and(|w| f.number <= f64::from(w)),
            aired_at: air.and_then(|a| a.aired_at),
        }
    }
}

/// `E07` without a season, `S02E01` with one, a half episode as `E07.5`.
fn episode_code(season: Option<u32>, number: f64) -> String {
    let n = if number.fract() == 0.0 {
        format!("{:02}", number as i64)
    } else {
        format_number(number)
    };
    match season {
        Some(s) => format!("S{s:02}E{n}"),
        None => format!("E{n}"),
    }
}

/// The status as the host contract spells it. Named here, not taken from
/// the enum's Debug, so renaming a variant cannot change the wire.
fn wire_status(s: crate::contract::AiringStatus) -> &'static str {
    use crate::contract::AiringStatus as A;
    match s {
        A::Releasing => "Releasing",
        A::Finished => "Finished",
        A::NotYetReleased => "NotYetReleased",
        A::Cancelled => "Cancelled",
        A::Hiatus => "Hiatus",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_follow_the_wire_contract() {
        assert_eq!(episode_code(None, 7.0), "E07");
        assert_eq!(episode_code(None, 12.0), "E12");
        assert_eq!(episode_code(Some(2), 1.0), "S02E01");
        assert_eq!(episode_code(None, 7.5), "E7.5");
    }
}
