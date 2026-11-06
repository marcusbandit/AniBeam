//! Stats (#216): a year of watching, added up off the history the core keeps.

mod common;
mod fixtures;

use anibeam_core::*;
use rusqlite::params;

fn stats(core: &Core, year: Option<i32>) -> Stats {
    stats_with(core, year, 0, Vec::new())
}

fn stats_with(core: &Core, year: Option<i32>, utc_offset: i32, month_starts: Vec<i64>) -> Stats {
    match core
        .call(Call::GetStats {
            year,
            utc_offset,
            month_starts,
        })
        .unwrap()
    {
        Reply::Stats { stats } => stats,
        other => panic!("{other:?}"),
    }
}

/// Unix seconds for a day of 2026 at noon UTC.
fn day(month: u32, d: u32) -> i64 {
    let days_before = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][(month - 1) as usize];
    let jan1_2026 = 1_767_225_600; // 2026-01-01T00:00:00Z
    jan1_2026 + i64::from(days_before + d - 1) * 86_400 + 12 * 3600
}

fn genres(
    core: &Core,
    media: u64,
    genres: &[&str],
    studio: &str,
    season: &str,
    year: u32,
    duration: u32,
) {
    let g = serde_json::to_string(genres).unwrap();
    let (studio, season) = (studio.to_string(), season.to_string());
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE anilist_media SET genres = ?2, studio = ?3, season = ?4, year = ?5, duration = ?6 WHERE id = ?1",
                params![media as i64, g, studio, season, year, duration],
            )?;
            Ok(())
        })
        .unwrap();
}

#[test]
fn a_year_adds_up_by_month_genre_studio_and_season() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    // Bebop: 3 of 3 episodes, finished in March. 24 minutes each.
    let bebop = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Bebop", "Cowboy Bebop");
    fixtures::insert_media(
        &core,
        1,
        Some("Cowboy Bebop"),
        None,
        Some(3),
        "FINISHED",
        "TV",
        Some(86),
    );
    fixtures::match_series(&core, bebop, Some(1), None);
    genres(
        &core,
        1,
        &["Action", "Sci-Fi"],
        "Sunrise",
        "SPRING",
        1998,
        24,
    );
    fixtures::insert_completed(&core, bebop, "1", day(2, 10));
    fixtures::insert_completed(&core, bebop, "2", day(3, 1));
    fixtures::insert_completed(&core, bebop, "3", day(3, 2));
    // Mushishi: 2 of 26 in March, one last December, 30 minutes each.
    let mushishi =
        fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Mushishi", "Mushishi");
    fixtures::insert_media(
        &core,
        2,
        Some("Mushishi"),
        None,
        Some(26),
        "FINISHED",
        "TV",
        Some(85),
    );
    fixtures::match_series(&core, mushishi, Some(2), None);
    genres(
        &core,
        2,
        &["Mystery", "Sci-Fi"],
        "Artland",
        "FALL",
        2005,
        30,
    );
    fixtures::insert_completed(&core, mushishi, "1", day(1, 1) - 30 * 86_400);
    fixtures::insert_completed(&core, mushishi, "2", day(3, 20));
    fixtures::insert_completed(&core, mushishi, "3", day(3, 21));

    let s = stats(&core, Some(2026));
    assert_eq!(s.year, 2026);
    assert_eq!(s.years, vec![2026, 2025]);
    assert_eq!(s.episodes, 5);
    assert_eq!(s.minutes, 3 * 24 + 2 * 30);
    assert_eq!(s.series_completed, 1);
    assert_eq!(s.months.len(), 12);
    assert_eq!((s.months[1].episodes, s.months[1].minutes), (1, 24));
    assert_eq!((s.months[2].episodes, s.months[2].series_completed), (4, 1));
    let names: Vec<(&str, u32)> = s
        .genres
        .iter()
        .map(|g| (g.name.as_str(), g.episodes))
        .collect();
    assert_eq!(names, vec![("Sci-Fi", 5), ("Action", 3), ("Mystery", 2)]);
    assert_eq!(s.studios[0].name, "Sunrise");
    let seasons: Vec<&str> = s.seasons.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(seasons, vec!["Spring 1998", "Fall 2005"]);
    assert_eq!(s.top_series[0].series, bebop, "most minutes first");
    assert_eq!(s.top_series[0].episodes, 3);

    // Last year holds the one December episode alone.
    let last = stats(&core, Some(2025));
    assert_eq!(
        (last.episodes, last.minutes, last.series_completed),
        (1, 30, 0)
    );
    assert_eq!(last.months[11].episodes, 1);
}

#[test]
fn a_record_with_no_length_counts_a_plain_episode() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Loose", "Loose");
    fixtures::insert_completed(&core, loose, "1", day(5, 5));
    let s = stats(&core, Some(2026));
    assert_eq!((s.episodes, s.minutes), (1, 24));
    assert!(s.genres.is_empty());
    assert_eq!(s.top_series[0].title, "Loose");
}

#[test]
fn an_offset_moves_a_late_night_into_the_next_month() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Loose", "Loose");
    // 23:30 UTC on 31 January is 00:30 on 1 February an hour east.
    fixtures::insert_completed(&core, loose, "1", day(1, 31) + 11 * 3600 + 1800);
    let utc = stats_with(&core, Some(2026), 0, Vec::new());
    assert_eq!(utc.months[0].episodes, 1);
    let east = stats_with(&core, Some(2026), 3600, Vec::new());
    assert_eq!(east.months[1].episodes, 1);
}

#[test]
fn a_stale_count_or_an_extra_episode_does_not_finish_a_series() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    // AniList says 24; a hand-set 12 left over must not finish it at 12.
    let long = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Long", "Long");
    fixtures::insert_media(&core, 1, Some("Long"), None, Some(24), "FINISHED", "TV", None);
    fixtures::match_series(&core, long, Some(1), None);
    core.store()
        .write(move |c| {
            c.execute(
                "UPDATE series SET episodes_set = 12 WHERE id = ?1",
                params![long as i64],
            )?;
            Ok(())
        })
        .unwrap();
    for n in 1..=12 {
        fixtures::insert_completed(&core, long, &n.to_string(), day(4, n));
    }
    // Two of three, plus an OVA and an episode 0: still not finished.
    let short = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Short", "Short");
    fixtures::insert_media(&core, 2, Some("Short"), None, Some(3), "FINISHED", "TV", None);
    fixtures::match_series(&core, short, Some(2), None);
    for key in ["1", "2", "0", "S00E01"] {
        fixtures::insert_completed(&core, short, key, day(5, 1));
    }
    let s = stats(&core, Some(2026));
    assert_eq!(s.series_completed, 0);
    assert_eq!(s.episodes, 16, "every finished episode still counts its time");
}

#[test]
fn the_month_edges_the_shell_sends_hold_across_a_clock_change() {
    let (_dir, core, _c) = common::open_core();
    let src = fixtures::insert_source(&core, "/lib");
    let loose = fixtures::insert_series(&core, src, SeriesKind::Show, "/lib/Loose", "Loose");
    // 22:30 UTC on 31 July is 00:30 on 1 August in summer time two hours
    // east, though the page is opened in winter, at one hour east.
    let at = day(7, 31) + 10 * 3600 + 1800;
    fixtures::insert_completed(&core, loose, "1", at);
    // Local midnights opening each month of 2026: one hour east in winter,
    // two from the end of March to the end of October.
    let starts: Vec<i64> = (1..=12)
        .map(|m| {
            let east = if (4..=10).contains(&m) { 7200 } else { 3600 };
            day(m, 1) - 12 * 3600 - east
        })
        .chain(std::iter::once(1_798_761_600 - 3600))
        .collect();
    let s = stats_with(&core, Some(2026), 3600, starts);
    assert_eq!(s.months[7].episodes, 1, "August, not July");
    assert_eq!(s.months[6].episodes, 0);
}
