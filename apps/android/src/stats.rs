//! Stats (#743): a year of watching off the core's `GetStats`, laid out as the rows More's
//! Stats section draws. The core adds it up; this reads it again whenever the page opens or a
//! year is picked, on its own thread, and hands the result back to the UI thread.

use std::cell::RefCell;

use anibeam_core::{Call, Reply, StatCount, StatSeries, Stats};
use chrono::{Datelike, Local, NaiveDate, Offset, TimeZone};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};

use crate::{AppWindow, StatBar, StatRank, StatsData};

/// The newest years the picker offers; older ones would crowd the pill.
const YEARS_SHOWN: usize = 4;
/// Rows a ranked list keeps. The core already trims; this only bounds what a phone draws.
const ROWS: usize = 6;

struct Page {
    window: Weak<AppWindow>,
    /// The years the picker shows, newest first, in the order of its cells.
    years: Vec<i32>,
    /// A read is on its way; a second tap waits for it.
    busy: bool,
    /// The last year read, kept so the page can word itself again when the language changes.
    last: Option<Stats>,
}

thread_local! {
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Page)) {
    PAGE.with(|cell| {
        if let Some(p) = cell.borrow_mut().as_mut() {
            f(p);
        }
    });
}

pub fn install(window: &AppWindow) {
    PAGE.with(|cell| {
        *cell.borrow_mut() = Some(Page {
            window: window.as_weak(),
            years: Vec::new(),
            busy: false,
            last: None,
        })
    });
    window
        .global::<StatsData>()
        .on_read(|pick| with(|p| p.read(pick)));
}

/// The language changed: the page's lines are worded again from the last year read.
pub fn relocalize() {
    with(|p| {
        if let Some(stats) = p.last.clone() {
            p.show(&stats);
        }
    });
}

impl Page {
    /// Reads the year in cell `pick` of the picker, this year when it is -1 or the picker
    /// has not been drawn yet.
    fn read(&mut self, pick: i32) {
        // A test hook for captures, like the seed list: a year of watching as JSON.
        if let Some(stats) = std::fs::read("/data/local/tmp/anibeam-demo/stats.json")
            .ok()
            .and_then(|b| serde_json::from_slice::<Stats>(&b).ok())
        {
            return self.show(&stats);
        }
        let Some(core) = crate::local::core() else {
            return;
        };
        if self.busy {
            return;
        }
        let year = usize::try_from(pick)
            .ok()
            .and_then(|i| self.years.get(i).copied())
            .unwrap_or_else(|| Local::now().year());
        self.busy = true;
        let (offset, starts) = month_edges(year);
        let _ = std::thread::Builder::new()
            .name("stats".into())
            .spawn(move || {
                let answer = core.call(Call::GetStats {
                    year: Some(year),
                    utc_offset: offset,
                    month_starts: starts,
                });
                let _ = slint::invoke_from_event_loop(move || {
                    with(|p| {
                        p.busy = false;
                        match answer {
                            Ok(Reply::Stats { stats }) => p.show(&stats),
                            Ok(_) => {}
                            Err(e) => crate::local::log(&format!("stats: {e}")),
                        }
                    })
                });
            });
    }

    fn show(&mut self, stats: &Stats) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        self.last = Some(stats.clone());
        self.years = year_options(stats, Local::now().year());
        let d = window.global::<StatsData>();
        d.set_loaded(true);
        d.set_empty(stats.years.is_empty());
        d.set_year_line(tr!("stats-year-line", year = stats.year.to_string()).into());
        d.set_years(ModelRc::new(VecModel::from(
            self.years
                .iter()
                .map(|y| SharedString::from(y.to_string()))
                .collect::<Vec<_>>(),
        )));
        d.set_year_index(
            self.years
                .iter()
                .position(|&y| y == stats.year)
                .unwrap_or(0) as i32,
        );
        d.set_hours(hours(stats.minutes).into());
        d.set_episodes(stats.episodes.to_string().into());
        d.set_episodes_caption(tr!("stats-episodes-caption", count = stats.episodes).into());
        d.set_finished(stats.series_completed.to_string().into());
        d.set_busiest(busiest(stats).into());
        d.set_months(ModelRc::new(VecModel::from(bars(stats))));
        d.set_genres(ModelRc::new(VecModel::from(ranks(&stats.genres))));
        d.set_studios(ModelRc::new(VecModel::from(ranks(&stats.studios))));
        d.set_seasons(ModelRc::new(VecModel::from(ranks(&stats.seasons))));
        d.set_shows(ModelRc::new(VecModel::from(show_ranks(&stats.top_series))));
    }
}

/// The offset of `year`'s January, and the thirteen local midnights that open each month
/// and the next January, so a month keeps its edges across a clock change.
fn month_edges(year: i32) -> (i32, Vec<i64>) {
    // A midnight that a clock change skipped is the first hour that exists after it.
    let midnight = |y: i32, m: u32| {
        let day = NaiveDate::from_ymd_opt(y, m, 1)?;
        (0..3).find_map(|h| {
            let t = day.and_hms_opt(h, 0, 0)?;
            Local.from_local_datetime(&t).earliest()
        })
    };
    let offset = midnight(year, 1)
        .map(|t| t.offset().fix().local_minus_utc())
        .unwrap_or(0);
    let starts: Vec<i64> = (1..=12)
        .map(|m| midnight(year, m))
        .chain(std::iter::once(midnight(year + 1, 1)))
        .map(|t| t.map(|t| t.timestamp()))
        .collect::<Option<_>>()
        .unwrap_or_default();
    (offset, starts)
}

/// The years the picker offers: this one, the year shown and every year with any
/// watching, newest first, cut to what fits.
fn year_options(stats: &Stats, now: i32) -> Vec<i32> {
    let mut years = stats.years.clone();
    for y in [now, stats.year] {
        if !years.contains(&y) {
            years.push(y);
        }
    }
    years.sort_unstable_by(|a, b| b.cmp(a));
    years.truncate(YEARS_SHOWN);
    if !years.contains(&stats.year) {
        // A year picked from further back stays reachable.
        years.pop();
        years.push(stats.year);
        years.sort_unstable_by(|a, b| b.cmp(a));
    }
    years
}

/// Hours as the desktop writes them: whole from ten up, one decimal below.
fn hours(minutes: u64) -> String {
    let h = minutes as f64 / 60.0;
    if h >= 10.0 {
        format!("{}", h.round() as u64)
    } else {
        let r = (h * 10.0).round() / 10.0;
        if r.fract() == 0.0 {
            format!("{r:.0}")
        } else {
            format!("{r:.1}").replace('.', &tr!("fmt-decimal-point"))
        }
    }
}

/// The month with the most minutes, "None" when no month has any.
fn busiest(stats: &Stats) -> String {
    stats
        .months
        .iter()
        .filter(|m| m.minutes > 0)
        .max_by_key(|m| m.minutes)
        .filter(|m| (1..=12).contains(&m.month))
        .map(|m| tr!(&format!("stats-month-{}", m.month)))
        .unwrap_or_else(|| tr!("common-none"))
}

fn bars(stats: &Stats) -> Vec<StatBar> {
    let max = stats.months.iter().map(|m| m.minutes).max().unwrap_or(0);
    let top = stats
        .months
        .iter()
        .filter(|m| m.minutes > 0)
        .max_by_key(|m| m.minutes)
        .map(|m| m.month);
    (1..=12u32)
        .map(|n| {
            let minutes = stats
                .months
                .iter()
                .find(|m| m.month == n)
                .map_or(0, |m| m.minutes);
            StatBar {
                letter: tr!(&format!("stats-month-letter-{n}")).into(),
                hours: if minutes > 0 {
                    hours(minutes).into()
                } else {
                    "".into()
                },
                share: if max == 0 {
                    0.0
                } else {
                    minutes as f32 / max as f32
                },
                top: top == Some(n),
            }
        })
        .collect()
}

fn eps(n: u32) -> String {
    tr!("stats-eps", count = n)
}

fn ranks(rows: &[StatCount]) -> Vec<StatRank> {
    let lead = rows.first().map_or(1, |r| r.episodes.max(1));
    rows.iter()
        .take(ROWS)
        .enumerate()
        .map(|(i, r)| StatRank {
            name: r.name.as_str().into(),
            count: eps(r.episodes).into(),
            share: r.episodes as f32 / lead as f32,
            top: i == 0,
        })
        .collect()
}

fn show_ranks(rows: &[StatSeries]) -> Vec<StatRank> {
    let lead = rows.first().map_or(1, |r| r.episodes.max(1));
    rows.iter()
        .take(ROWS)
        .enumerate()
        .map(|(i, r)| StatRank {
            name: r.title.as_str().into(),
            count: eps(r.episodes).into(),
            share: r.episodes as f32 / lead as f32,
            top: i == 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::StatMonth;

    fn stats(years: Vec<i32>, year: i32, minutes: &[(u32, u64)]) -> Stats {
        Stats {
            year,
            years,
            minutes: minutes.iter().map(|m| m.1).sum(),
            episodes: 0,
            series_completed: 0,
            months: (1..=12)
                .map(|n| StatMonth {
                    month: n,
                    episodes: 0,
                    minutes: minutes.iter().find(|m| m.0 == n).map_or(0, |m| m.1),
                    series_completed: 0,
                })
                .collect(),
            genres: Vec::new(),
            studios: Vec::new(),
            seasons: Vec::new(),
            top_series: Vec::new(),
        }
    }

    #[test]
    fn hours_are_whole_from_ten_and_one_decimal_below() {
        crate::tr::set_lang(crate::tr::Lang::En);
        assert_eq!(hours(0), "0");
        assert_eq!(hours(90), "1.5");
        assert_eq!(hours(120), "2");
        assert_eq!(hours(599), "10");
        assert_eq!(hours(6000), "100");
    }

    #[test]
    fn the_busiest_month_is_named_or_none() {
        crate::tr::set_lang(crate::tr::Lang::En);
        let s = stats(vec![2026], 2026, &[(3, 100), (10, 500), (11, 200)]);
        assert_eq!(busiest(&s), "Oct");
        assert_eq!(busiest(&stats(vec![], 2026, &[])), "None");
    }

    #[test]
    fn bars_scale_to_the_busiest_and_mark_it() {
        let s = stats(vec![2026], 2026, &[(3, 100), (10, 400)]);
        let b = bars(&s);
        assert_eq!(b.len(), 12);
        assert_eq!(b[9].share, 1.0);
        assert!(b[9].top && !b[2].top);
        assert_eq!(b[2].share, 0.25);
        assert_eq!(b[0].hours, "");
    }

    #[test]
    fn the_picker_keeps_this_year_and_the_one_shown() {
        let s = stats(vec![2025, 2023], 2025, &[]);
        assert_eq!(year_options(&s, 2026), vec![2026, 2025, 2023]);
        let many = stats(vec![2026, 2025, 2024, 2023, 2022, 2021], 2021, &[]);
        let years = year_options(&many, 2026);
        assert_eq!(years.len(), YEARS_SHOWN);
        assert!(years.contains(&2021) && years.contains(&2026));
    }

    #[test]
    fn a_month_has_twelve_edges_and_a_thirteenth() {
        let (_, starts) = month_edges(2026);
        assert_eq!(starts.len(), 13);
        assert!(starts.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn ranks_scale_to_the_leader() {
        crate::tr::set_lang(crate::tr::Lang::En);
        let rows = vec![
            StatCount {
                name: "Action".into(),
                episodes: 40,
                series: 3,
            },
            StatCount {
                name: "Drama".into(),
                episodes: 10,
                series: 1,
            },
        ];
        let r = ranks(&rows);
        assert_eq!(r[0].share, 1.0);
        assert_eq!(r[1].share, 0.25);
        assert_eq!(r[1].count, "10 eps");
        assert_eq!(eps(1), "1 ep");
    }
}
