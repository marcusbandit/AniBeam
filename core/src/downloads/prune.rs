//! Unsubscribing from finished shows. A show whose run ended long enough
//! ago has no episode left to download, so its RSS feed and rule are
//! removed from anirss. Only the subscription goes, exactly as the user's
//! own "keep the files and torrents" removal does it: the library series,
//! the files and the torrents are never touched.
//!
//! A show counts as finished for good when all of these hold:
//! - its rule is linked to a library series that is a show, not a film;
//! - the provider's status is Finished or Cancelled;
//! - the run ended at least `FINISHED_GRACE_DAYS` days ago, the 16 days
//!   being the chance for a special to follow. The end is the date of the
//!   last episode in the schedule, else the media's end date. With neither
//!   known the show is left alone.
//!
//! The check runs after each successful subscription read, at most once a
//! day (the time of the last one is kept in the settings table), only
//! while the setting is on and qBittorrent answers, and never in a shell
//! that disabled it (`Core::disable_feed_pruning`).

use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};
use time::macros::format_description;

use crate::contract::*;
use crate::core::Core;
use crate::store::settings as kv;

use super::anirss::AnirssClient;
use super::settings;

/// Days a show must have been finished before its feed goes.
pub const FINISHED_GRACE_DAYS: i64 = 16;

const DAY: i64 = 86_400;
/// The shortest gap between two checks.
const CHECK_EVERY: i64 = DAY;
/// The settings-table key of the last check, in Unix seconds.
const LAST_CHECK: &str = "downloads_pruned_at";

/// What the library knows about when a show's run ended.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunEnd {
    /// The latest air date in the show's episode schedule.
    pub last_aired: Option<i64>,
    /// The close of the day the media's own end date names, when it has one.
    pub ended_on: Option<i64>,
}

impl RunEnd {
    /// When the run ended: the last air date, else the end date.
    pub fn at(self) -> Option<i64> {
        self.last_aired.or(self.ended_on)
    }
}

/// A rule to unsubscribe, and the figures for its log line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Due {
    pub rule: String,
    pub title: String,
    pub days: i64,
}

/// Whole days the show has been finished, when that is at least the grace
/// period; `None` for anything that must keep its feed.
pub fn finished_days(show: &ShowLink, end: RunEnd, now: i64) -> Option<i64> {
    if show.kind != SeriesKind::Show {
        return None;
    }
    if !is_over(show) {
        return None;
    }
    let age = now - end.at()?;
    (age >= FINISHED_GRACE_DAYS * DAY).then_some(age / DAY)
}

fn is_over(show: &ShowLink) -> bool {
    matches!(
        show.status,
        Some(AiringStatus::Finished) | Some(AiringStatus::Cancelled)
    )
}

/// The rule when it is due: the setting is on, the rule is linked to a
/// library series and `finished_days` holds.
pub fn due(enabled: bool, rule: &SubscriptionRule, end: RunEnd, now: i64) -> Option<Due> {
    if !enabled {
        return None;
    }
    let show = rule.show.as_ref()?;
    let days = finished_days(show, end, now)?;
    Some(Due {
        rule: rule.name.clone(),
        title: show.title.clone(),
        days,
    })
}

/// When the series' run ended, off the stored schedule and end date of
/// whichever provider it is matched to. Nothing known is the default.
pub fn run_end(conn: &Connection, series: u64) -> Result<RunEnd, CoreError> {
    let ids: Option<(Option<String>, Option<i64>, Option<i64>)> = conn
        .query_row(
            "SELECT provider, anilist_id, tvmaze_id FROM series WHERE id = ?1",
            params![series as i64],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((provider, anilist, tvmaze)) = ids else {
        return Ok(RunEnd::default());
    };
    let (last_aired, ended_on): (Option<i64>, Option<String>) =
        match (provider.as_deref(), anilist, tvmaze) {
            (Some("tvmaze"), _, Some(show)) => (
                conn.query_row(
                    "SELECT MAX(airstamp) FROM tvmaze_episodes WHERE show_id = ?1",
                    params![show],
                    |r| r.get(0),
                )?,
                conn.query_row(
                    "SELECT ended FROM tvmaze_shows WHERE id = ?1",
                    params![show],
                    |r| r.get(0),
                )
                .optional()?
                .flatten(),
            ),
            (p, Some(media), _) if p != Some("tvmaze") => (
                conn.query_row(
                    "SELECT MAX(aired_at) FROM anilist_episodes WHERE anilist_id = ?1",
                    params![media],
                    |r| r.get(0),
                )?,
                conn.query_row(
                    "SELECT end_date FROM anilist_media WHERE id = ?1",
                    params![media],
                    |r| r.get(0),
                )
                .optional()?
                .flatten(),
            ),
            _ => (None, None),
        };
    Ok(RunEnd {
        last_aired,
        ended_on: ended_on.as_deref().and_then(end_of_day),
    })
}

/// The close of a `YYYY-MM-DD` day in Unix seconds, UTC. A date counts as
/// ended once the whole day is past.
fn end_of_day(date: &str) -> Option<i64> {
    let day = date.get(..10)?;
    let date = time::Date::parse(day, format_description!("[year]-[month]-[day]")).ok()?;
    Some(date.midnight().assume_utc().unix_timestamp() + DAY)
}

/// The rules to unsubscribe from, and how many were linked to a series at
/// all (a check that saw none had nothing to judge).
pub fn plan(
    conn: &Connection,
    enabled: bool,
    rules: &[SubscriptionRule],
    now: i64,
) -> Result<(Vec<Due>, usize), CoreError> {
    let mut out = Vec::new();
    let mut judged = 0;
    for rule in rules {
        let Some(show) = &rule.show else { continue };
        judged += 1;
        // The cheap tests first: only a finished show reads the schedule
        if show.kind != SeriesKind::Show || !is_over(show) {
            continue;
        }
        let end = run_end(conn, show.series_id)?;
        if let Some(d) = due(enabled, rule, end, now) {
            out.push(d);
        }
    }
    Ok((out, judged))
}

/// The hook `list_rules` calls after a successful read. Failures stay
/// here: a check that cannot run is not the page's problem.
pub(super) async fn after_read(
    core: &Arc<Core>,
    client: &AnirssClient,
    rules: &[SubscriptionRule],
) {
    if !core.downloads.pruning_allowed() {
        return;
    }
    if let Err(e) = check(core, client, rules, crate::time::now_secs()).await {
        tracing::warn!("finished feeds not pruned: {}", super::plain(&e));
    }
}

async fn check(
    core: &Arc<Core>,
    client: &AnirssClient,
    rules: &[SubscriptionRule],
    now: i64,
) -> Result<(), CoreError> {
    if rules.iter().all(|r| r.show.is_none()) {
        return Ok(());
    }
    let (enabled, last) = core.store.read(|c| {
        Ok((
            settings::load_stored(c)?.prune_finished_feeds,
            kv::get::<i64>(c, LAST_CHECK)?,
        ))
    })?;
    // A clock set back leaves `last` ahead of now: that is a check owed
    if !enabled || last.is_some_and(|t| t <= now && now - t < CHECK_EVERY) {
        return Ok(());
    }
    let (due, _) = core.store.read(|c| plan(c, true, rules, now))?;
    if !due.is_empty() {
        // Nothing is removed while the torrent client is down: the shell
        // would show a library it cannot act on
        if super::qbt_client(core).await?.version().await.is_err() {
            return Ok(());
        }
    }
    // Recorded before the removals, so the re-read each one prompts
    // finds the day already checked
    core.store.write(move |c| kv::set(c, LAST_CHECK, &now))?;
    for d in due {
        match client
            .remove(std::slice::from_ref(&d.rule), false, false, false)
            .await
        {
            Ok(_) => {
                core.bus.info(
                    Stage::Downloads,
                    format!(
                        "Unsubscribed from {}: finished airing {} ago",
                        d.title,
                        super::plural(d.days as u64, "day", "days")
                    ),
                    EventBody::SubscriptionRemoved { name: d.rule },
                );
            }
            Err(e) => {
                core.bus.warn(
                    Stage::Downloads,
                    format!(
                        "Could not unsubscribe from {}: {}",
                        d.title,
                        super::plain(&e)
                    ),
                    EventBody::DownloadsUnavailable {
                        what: "unsubscribe".to_string(),
                        error: e,
                    },
                );
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn show(status: Option<AiringStatus>) -> ShowLink {
        ShowLink {
            series_id: 1,
            kind: SeriesKind::Show,
            title: "Frieren".to_string(),
            path: "/lib/Frieren".to_string(),
            poster: None,
            status,
            episodes_on_disk: 12,
            total_episodes: Some(12),
            total_is_estimate: false,
            next_airing: None,
            complete: false,
            missing: Vec::new(),
        }
    }

    fn rule(show: Option<ShowLink>) -> SubscriptionRule {
        SubscriptionRule {
            name: "Frieren".to_string(),
            feed_url: "https://nyaa.si/?page=rss&q=frieren".to_string(),
            save_path: "/lib/Frieren".to_string(),
            rule_enabled: true,
            torrent_count: 0,
            file_count: 0,
            query: String::new(),
            show,
        }
    }

    fn aired(days_ago: i64) -> RunEnd {
        RunEnd {
            last_aired: Some(NOW - days_ago * DAY),
            ended_on: None,
        }
    }

    #[test]
    fn a_finished_show_waits_out_the_sixteen_days() {
        let s = show(Some(AiringStatus::Finished));
        assert_eq!(finished_days(&s, aired(15), NOW), None);
        assert_eq!(
            finished_days(
                &s,
                RunEnd {
                    last_aired: Some(NOW - 16 * DAY + 1),
                    ended_on: None
                },
                NOW
            ),
            None,
            "a second short of 16 days"
        );
        assert_eq!(finished_days(&s, aired(16), NOW), Some(16));
        assert_eq!(finished_days(&s, aired(40), NOW), Some(40));
    }

    #[test]
    fn a_show_that_is_not_over_keeps_its_feed() {
        for status in [
            Some(AiringStatus::Releasing),
            Some(AiringStatus::NotYetReleased),
            Some(AiringStatus::Hiatus),
            None,
        ] {
            assert_eq!(
                finished_days(&show(status), aired(100), NOW),
                None,
                "{status:?}"
            );
        }
    }

    #[test]
    fn a_cancelled_show_counts_as_finished() {
        let s = show(Some(AiringStatus::Cancelled));
        assert_eq!(finished_days(&s, aired(15), NOW), None);
        assert_eq!(finished_days(&s, aired(16), NOW), Some(16));
    }

    #[test]
    fn a_film_is_left_alone() {
        let mut s = show(Some(AiringStatus::Finished));
        s.kind = SeriesKind::Movie;
        assert_eq!(finished_days(&s, aired(100), NOW), None);
    }

    #[test]
    fn the_end_date_stands_in_when_no_air_date_is_known() {
        let s = show(Some(AiringStatus::Finished));
        let ended = |days_ago: i64| RunEnd {
            last_aired: None,
            ended_on: Some(NOW - days_ago * DAY),
        };
        assert_eq!(finished_days(&s, ended(15), NOW), None);
        assert_eq!(finished_days(&s, ended(16), NOW), Some(16));
        // An air date wins over a stale end date
        let both = RunEnd {
            last_aired: Some(NOW - 3 * DAY),
            ended_on: Some(NOW - 90 * DAY),
        };
        assert_eq!(finished_days(&s, both, NOW), None);
    }

    #[test]
    fn a_show_with_no_date_at_all_is_never_pruned() {
        let s = show(Some(AiringStatus::Finished));
        assert_eq!(finished_days(&s, RunEnd::default(), NOW), None);
    }

    #[test]
    fn a_rule_needs_the_setting_and_a_linked_series() {
        let linked = rule(Some(show(Some(AiringStatus::Finished))));
        let d = due(true, &linked, aired(20), NOW).expect("due");
        assert_eq!(
            d,
            Due {
                rule: "Frieren".to_string(),
                title: "Frieren".to_string(),
                days: 20
            }
        );
        assert_eq!(due(false, &linked, aired(20), NOW), None, "setting off");
        assert_eq!(due(true, &rule(None), aired(20), NOW), None, "unlinked");
    }

    #[test]
    fn an_end_date_reads_as_the_close_of_its_day() {
        // 2026-01-01 00:00 UTC is 1767225600
        assert_eq!(end_of_day("2026-01-01"), Some(1_767_225_600 + DAY));
        assert_eq!(end_of_day("2026-01-01T10:00"), Some(1_767_225_600 + DAY));
        assert_eq!(end_of_day("soon"), None);
        assert_eq!(end_of_day("2026-13-40"), None);
    }
}
