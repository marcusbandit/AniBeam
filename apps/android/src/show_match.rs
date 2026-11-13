//! Fixing a wrong match from a show's page: the search, the pasted link, Apply, Refresh and
//! Clear, the desktop's match panel on the same core calls. A search and a link look-up are
//! jobs whose answers come back as events; `core_event` catches them and the UI thread
//! picks them up by job id.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use anibeam_core::{
    Call, Event, EventBody, JobKind, MatchInfo, MatchTarget, Provider, Reply, SearchResult,
    SeriesCard,
};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, MatchData, MatchRow, ShowData};

/// How many hits a search asks for.
const LIMIT: u32 = 12;

/// What a job's event said, kept until the UI thread asks for it.
enum Outcome {
    Search(Vec<SearchResult>),
    Link(MatchTarget),
    Failed(String),
}

/// An event can land before the call that started its job has returned, so the answer waits
/// here by job id.
static STASH: Mutex<Option<HashMap<u64, Outcome>>> = Mutex::new(None);

/// The open show, as the match needs it.
struct Open {
    series: u64,
    title: String,
    folder: String,
    info: Option<MatchInfo>,
}

#[derive(Default)]
struct State {
    open: Option<Open>,
    search_job: Option<u64>,
    resolve_job: Option<u64>,
    results: Vec<SearchResult>,
    found: Option<MatchTarget>,
    /// The search that was asked last, so a stale answer is dropped.
    asked: u64,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static ROWS: RefCell<Option<std::rc::Rc<VecModel<MatchRow>>>> = const { RefCell::new(None) };
}

/// Called for every core event, on the core's thread.
pub fn core_event(event: &Event) {
    let Some(job) = &event.job else { return };
    if !matches!(job.kind, JobKind::Search | JobKind::ResolveLink) {
        return;
    }
    let outcome = match &event.body {
        EventBody::SearchFinished { results } => Outcome::Search(results.clone()),
        EventBody::LinkResolved { target } => Outcome::Link(target.clone()),
        EventBody::JobFailed { error } => Outcome::Failed(error.to_string()),
        _ => return,
    };
    {
        let mut stash = STASH.lock().unwrap_or_else(|e| e.into_inner());
        let map = stash.get_or_insert_with(HashMap::new);
        map.insert(job.id, outcome);
        // Job ids only grow, so the oldest answers are the ones nobody is waiting for.
        while map.len() > 32 {
            let Some(oldest) = map.keys().min().copied() else {
                break;
            };
            map.remove(&oldest);
        }
    }
    let _ = slint::invoke_from_event_loop(deliver);
}

/// Hands the open search or look-up its answer, if it has come.
fn deliver() {
    let (search, resolve) = STATE.with(|s| {
        let s = s.borrow();
        (s.search_job, s.resolve_job)
    });
    let take = |job: Option<u64>| {
        job.and_then(|j| {
            STASH
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_mut()
                .and_then(|m| m.remove(&j))
        })
    };
    let search = take(search);
    let resolve = take(resolve);
    if search.is_none() && resolve.is_none() {
        return;
    }
    with_window(|window| {
        let data = window.global::<MatchData>();
        match search {
            Some(Outcome::Search(results)) => {
                STATE.with(|s| {
                    let mut s = s.borrow_mut();
                    s.search_job = None;
                    s.results = results;
                });
                data.set_searching(false);
                data.set_note("".into());
                show_results(&data);
            }
            Some(Outcome::Failed(why)) => {
                STATE.with(|s| s.borrow_mut().search_job = None);
                data.set_searching(false);
                data.set_note(why.into());
                show_results(&data);
            }
            _ => {}
        }
        match resolve {
            Some(Outcome::Link(target)) => {
                data.set_found(target_text(&target).into());
                data.set_note("".into());
                STATE.with(|s| {
                    let mut s = s.borrow_mut();
                    s.resolve_job = None;
                    s.found = Some(target);
                });
            }
            Some(Outcome::Failed(why)) => {
                STATE.with(|s| s.borrow_mut().resolve_job = None);
                data.set_note(why.into());
            }
            _ => {}
        }
    });
}

thread_local! {
    static WINDOW: RefCell<Option<slint::Weak<AppWindow>>> = const { RefCell::new(None) };
}

fn with_window(f: impl FnOnce(&AppWindow)) {
    if let Some(window) = WINDOW.with(|w| w.borrow().as_ref().and_then(|w| w.upgrade())) {
        f(&window);
    }
}

fn provider_name(provider: i32) -> String {
    if provider == 1 {
        tr!("match-tvmaze")
    } else {
        tr!("tracker-anilist")
    }
}

fn provider_of(provider: i32) -> Provider {
    if provider == 1 {
        Provider::Tvmaze
    } else {
        Provider::Anilist
    }
}

/// Where a looked-up link points, in words: the ids are the provider's business.
fn target_text(target: &MatchTarget) -> String {
    match target {
        MatchTarget::Tvmaze { .. } => tr!("phone-match-found-tvmaze"),
        MatchTarget::Anilist { season, .. } => match season {
            Some(s) => tr!("phone-match-found-anilist-season", season = *s),
            None => tr!("phone-match-found-anilist"),
        },
        MatchTarget::Mal { .. } => tr!("phone-match-found-mal"),
    }
}

/// What the show is matched to, in words.
fn match_text(info: &MatchInfo) -> String {
    let mut bits: Vec<String> = Vec::new();
    if info.anilist_id.is_some() {
        bits.push(tr!("tracker-anilist"));
    }
    if info.mal_id.is_some() {
        bits.push(tr!("tracker-mal"));
    }
    if info.tvmaze_id.is_some() {
        bits.push(tr!("match-tvmaze"));
    }
    let mut bits = bits.into_iter();
    let Some(first) = bits.next() else {
        return String::new();
    };
    let sites = bits.fold(first, |all, next| {
        tr!("match-join-and", first = all, second = next)
    });
    tr!("phone-match-matched-on", sites = sites)
}

/// An AniList format as the match panel words it.
fn format_label(format: &str) -> String {
    match format {
        "TV" => tr!("show-format-tv"),
        "TV_SHORT" => tr!("show-format-tv-short"),
        "OVA" => tr!("show-format-ova"),
        "ONA" => tr!("show-format-ona"),
        "SPECIAL" => tr!("show-format-special"),
        "MOVIE" => tr!("show-format-film"),
        "MUSIC" => tr!("match-format-music"),
        other => other.replace('_', " "),
    }
}

fn row(r: &SearchResult) -> MatchRow {
    let format = match (&r.format, r.provider) {
        (Some(f), Provider::Tvmaze) => f.clone(),
        (Some(f), _) => format_label(f),
        (None, _) => String::new(),
    };
    let meta: Vec<String> = [
        r.year.map(|y| y.to_string()).unwrap_or_default(),
        format,
        match r.episodes {
            Some(n) => tr!("common-episodes", count = n),
            None => String::new(),
        },
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect();
    MatchRow {
        title: r.title.clone().into(),
        alt: r.alt_title.clone().unwrap_or_default().into(),
        meta: meta.join(", ").into(),
    }
}

/// The results and the line that stands in when there are none.
fn show_results(data: &MatchData) {
    let rows: Vec<MatchRow> = STATE.with(|s| s.borrow().results.iter().map(row).collect());
    let searching = data.get_searching();
    let name = provider_name(data.get_provider());
    let query = data.get_query();
    data.set_empty(
        if searching {
            tr!("phone-match-searching", name = name)
        } else if query.trim().chars().count() < 2 {
            tr!("phone-match-type-more", name = name)
        } else {
            tr!("match-none", name = name)
        }
        .into(),
    );
    ROWS.with(|r| {
        if let Some(model) = r.borrow().as_ref() {
            crate::app::update(model, rows);
        }
    });
}

/// "[Group] Show Name (2024) [1080p]" as "Show Name": a folder's name without its release
/// brackets, the start of the search for a show nothing matched.
fn folder_title(folder: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in folder.chars() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let out = out.replace(['_', '.'], " ");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tells the match which show is open; called every tick while a show of this phone's is.
pub fn track(card: Option<&SeriesCard>) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.open = card.map(|c| Open {
            series: c.id,
            title: c.title.clone(),
            folder: c.titles.folder.clone(),
            info: c.match_info.clone(),
        });
    });
}

/// The language changed: the lines pushed once, what the show is matched to, what a link
/// pointed at and the stand-in under the results, are worded again. A note under the buttons
/// is a few seconds old and stays as it was.
pub fn relocalize() {
    with_window(|window| {
        let data = window.global::<MatchData>();
        let (current, found) = STATE.with(|s| {
            let s = s.borrow();
            (
                s.open
                    .as_ref()
                    .and_then(|o| o.info.as_ref())
                    .map(match_text)
                    .unwrap_or_default(),
                s.found.as_ref().map(target_text),
            )
        });
        data.set_current(current.into());
        if let Some(found) = found {
            data.set_found(found.into());
        }
        show_results(&data);
    });
}

/// Hooks the sheet's callbacks to the core.
pub fn install(window: &AppWindow) {
    WINDOW.with(|w| *w.borrow_mut() = Some(window.as_weak()));
    let model = std::rc::Rc::new(VecModel::<MatchRow>::default());
    ROWS.with(|r| *r.borrow_mut() = Some(model.clone()));
    let data = window.global::<MatchData>();
    data.set_results(ModelRc::from(model));

    data.on_begin(|| {
        with_window(|window| {
            let data = window.global::<MatchData>();
            let (provider, query, current) = STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.results.clear();
                s.found = None;
                s.search_job = None;
                s.resolve_job = None;
                match &s.open {
                    Some(o) => {
                        let tvmaze = o
                            .info
                            .as_ref()
                            .is_some_and(|m| m.provider == Provider::Tvmaze);
                        let query = if o.info.is_some() {
                            o.title.clone()
                        } else {
                            folder_title(&o.folder)
                        };
                        let current = o.info.as_ref().map(match_text).unwrap_or_default();
                        (i32::from(tvmaze), query, current)
                    }
                    None => (0, String::new(), String::new()),
                }
            });
            data.set_provider(provider);
            data.set_query(query.clone().into());
            data.set_link("".into());
            data.set_found("".into());
            data.set_note("".into());
            data.set_armed(false);
            data.set_current(current.into());
            data.set_searching(false);
            show_results(&data);
            search(query);
        });
    });
    data.on_search(|q| search(q.to_string()));
    data.on_pick_provider(|p| {
        with_window(|window| {
            let data = window.global::<MatchData>();
            if data.get_provider() == p {
                return;
            }
            data.set_provider(p);
            data.set_found("".into());
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.results.clear();
                s.found = None;
            });
            search(data.get_query().to_string());
        });
    });
    data.on_apply(|i| {
        let target = with_state(|s| {
            let o = s.open.as_ref()?;
            let r = s.results.get(usize::try_from(i).ok()?)?;
            let target = match r.provider {
                Provider::Tvmaze => MatchTarget::Tvmaze { id: r.id },
                Provider::Mal => MatchTarget::Mal { id: r.id },
                _ => MatchTarget::Anilist {
                    id: r.id,
                    season: None,
                },
            };
            Some((o.series, target, r.title.clone()))
        });
        if let Some((series, target, title)) = target {
            apply(series, target, tr!("phone-match-matched-to", title = title));
        }
    });
    data.on_look_up(|url| {
        let url = url.trim().to_string();
        if url.is_empty() {
            return;
        }
        let Some(core) = crate::local::core() else {
            with_window(|w| {
                w.global::<MatchData>()
                    .set_note(tr!("show-still-starting").into())
            });
            return;
        };
        with_window(|w| {
            let data = w.global::<MatchData>();
            data.set_found("".into());
            data.set_note(tr!("match-looking-up").into());
        });
        let _ = std::thread::Builder::new()
            .name("match-link".into())
            .spawn(move || {
                let answer = core.call(Call::ResolveLink { url });
                let _ = slint::invoke_from_event_loop(move || match answer {
                    Ok(Reply::Started { job }) => {
                        STATE.with(|s| s.borrow_mut().resolve_job = Some(job));
                        deliver();
                    }
                    Ok(_) => {}
                    Err(e) => {
                        with_window(|w| w.global::<MatchData>().set_note(e.to_string().into()))
                    }
                });
            });
    });
    data.on_apply_found(|| {
        let found = with_state(|s| Some((s.open.as_ref()?.series, s.found.clone()?)));
        if let Some((series, target)) = found {
            apply(series, target, tr!("phone-match-applied"));
        }
    });
    data.on_refresh(|| {
        if let Some((series, title)) =
            with_state(|s| s.open.as_ref().map(|o| (o.series, o.title.clone())))
        {
            crate::show::run(
                Call::RefreshSeries { series },
                tr!("phone-match-refreshing", title = title.as_str()),
                tr!("phone-match-refreshing", title = title),
            );
            with_window(|w| w.global::<ShowData>().invoke_close());
        }
    });
    data.on_clear(|| {
        if let Some(series) = with_state(|s| s.open.as_ref().map(|o| o.series)) {
            crate::show::run(
                Call::ClearMatch { series },
                tr!("match-clearing"),
                tr!("phone-match-cleared"),
            );
            with_window(|w| w.global::<ShowData>().invoke_close());
        }
    });
    data.on_auto_match(|| {
        let said = tr!("match-auto-looking");
        crate::show::run(Call::AutoMatch, said.clone(), said);
        with_window(|w| w.global::<ShowData>().invoke_close());
    });
}

fn with_state<T>(f: impl FnOnce(&State) -> Option<T>) -> Option<T> {
    STATE.with(|s| f(&s.borrow()))
}

fn apply(series: u64, target: MatchTarget, said: String) {
    crate::show::run(
        Call::ApplyMatch { series, target },
        tr!("match-applying"),
        said,
    );
    with_window(|w| w.global::<ShowData>().invoke_close());
}

/// Starts a search of the picked provider for `query`.
fn search(query: String) {
    let query = query.trim().to_string();
    let (provider, asked) = with_window_value(|w| {
        let data = w.global::<MatchData>();
        (data.get_provider(), query.clone())
    })
    .unwrap_or((0, query.clone()));
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.asked += 1;
        s.results.clear();
        s.search_job = None;
    });
    if asked.chars().count() < 2 {
        with_window(|w| {
            let data = w.global::<MatchData>();
            data.set_searching(false);
            show_results(&data);
        });
        return;
    }
    let Some(core) = crate::local::core() else {
        return;
    };
    let ticket = STATE.with(|s| s.borrow().asked);
    with_window(|w| {
        let data = w.global::<MatchData>();
        data.set_searching(true);
        data.set_note("".into());
        data.set_empty(tr!("phone-match-searching", name = provider_name(provider)).into());
        show_results(&data);
    });
    let _ = std::thread::Builder::new()
        .name("match-search".into())
        .spawn(move || {
            let answer = core.call(Call::SearchProvider {
                provider: provider_of(provider),
                query: asked,
                limit: LIMIT,
            });
            let _ = slint::invoke_from_event_loop(move || {
                if STATE.with(|s| s.borrow().asked) != ticket {
                    return;
                }
                match answer {
                    Ok(Reply::Started { job }) => {
                        STATE.with(|s| s.borrow_mut().search_job = Some(job));
                        deliver();
                    }
                    Ok(_) => {}
                    Err(e) => with_window(|w| {
                        let data = w.global::<MatchData>();
                        data.set_searching(false);
                        data.set_note(e.to_string().into());
                    }),
                }
            });
        });
}

fn with_window_value<T>(f: impl FnOnce(&AppWindow) -> T) -> Option<T> {
    WINDOW
        .with(|w| w.borrow().as_ref().and_then(|w| w.upgrade()))
        .map(|w| f(&w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_name_loses_its_release_brackets() {
        assert_eq!(
            folder_title("[Group] Show Name (2024) [1080p]"),
            "Show Name"
        );
        assert_eq!(folder_title("Some.Show.S02"), "Some Show S02");
        assert_eq!(folder_title("Plain"), "Plain");
    }

    #[test]
    fn a_match_is_named_by_its_providers() {
        let info = MatchInfo {
            provider: Provider::Anilist,
            anilist_id: Some(1),
            mal_id: Some(2),
            tmdb_id: None,
            tmdb_kind: None,
            tvmaze_id: None,
            confirmed: true,
        };
        assert_eq!(match_text(&info), "Matched on AniList and MyAnimeList");
    }

    #[test]
    fn a_looked_up_link_says_where_it_points() {
        assert_eq!(
            target_text(&MatchTarget::Anilist {
                id: 1,
                season: Some(2)
            }),
            "Found it on AniList, season 2"
        );
        assert_eq!(
            target_text(&MatchTarget::Tvmaze { id: 1 }),
            "Found the show on TVmaze"
        );
    }
}
