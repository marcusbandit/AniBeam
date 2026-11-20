//! A show's page beyond playing it: the list status and score today, and the sheets for the
//! rest. `sync` copies the open series into `ShowData` every tick; the callbacks here run the
//! core's own calls, the ones the desktop's series page runs, and say how they went.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use anibeam_companion::format;
use anibeam_core::{Call, FranchiseLayout, ListStatus, Reply, SeriesDetail, TrackerAccount};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::remote::Art;
use crate::{AppData, AppWindow, ShowCard, ShowData, ShowPerson, Tile};

/// How long the line under the buttons keeps what a write said.
const NOTE_FOR: Duration = Duration::from_secs(6);

/// The art key of a character's portrait, set apart from the series ids and the AniList
/// ids the posters share (`app::AIRING_ART`).
const PERSON_ART: u64 = 1 << 49;

/// How often a franchise the crawl has not finished is read again.
const GRAPH_EVERY: Duration = Duration::from_secs(4);
/// And a finished one, so its rows follow what the library and the lists hold now.
const GRAPH_DONE_EVERY: Duration = Duration::from_secs(20);

/// The franchise graph of the open series, read off the UI thread.
#[derive(Default)]
struct Graph {
    /// Every related row is listed, not the first dozen.
    all: bool,
    series: Option<u64>,
    /// The AniList id the show is matched to: a new match is a new franchise.
    matched: Option<u64>,
    layout: Option<FranchiseLayout>,
    reading: bool,
    read_at: Option<Instant>,
}

/// The models the sections draw, and the cards behind their taps.
struct Models {
    related: Rc<VecModel<ShowCard>>,
    recs: Rc<VecModel<Tile>>,
    people: Rc<VecModel<ShowPerson>>,
    /// Every related and recommended card of the open series, for a tap to find.
    entries: Vec<ShowCard>,
}

/// How many related rows the page lists until Show all.
const RELATED_SHOWN: usize = 12;

thread_local! {
    /// What the sections were last built from, so a tick that changes nothing builds nothing.
    static SIG: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    static NOTE: RefCell<Option<(String, Instant)>> = const { RefCell::new(None) };
    static GRAPH: RefCell<Graph> = RefCell::new(Graph::default());
    static MODELS: RefCell<Option<Models>> = const { RefCell::new(None) };
}

/// Puts the line under the buttons up for a few seconds and asks the pages to read again.
pub(crate) fn say(line: String) {
    NOTE.with(|n| *n.borrow_mut() = Some((line, Instant::now())));
    crate::local::touch();
}

/// A status as the desktop's series page words it.
pub fn status_word(status: ListStatus) -> String {
    match status {
        ListStatus::Watching => tr!("show-status-watching"),
        ListStatus::Planning => tr!("show-status-planning"),
        ListStatus::Completed => tr!("show-status-completed"),
        ListStatus::Paused => tr!("show-status-paused"),
        ListStatus::Dropped => tr!("show-status-dropped"),
        ListStatus::Repeating => tr!("show-status-repeating"),
    }
}

/// The statuses in the order the sheet counts them: `ShowData.status-index`.
const STATUSES: [ListStatus; 6] = [
    ListStatus::Watching,
    ListStatus::Planning,
    ListStatus::Completed,
    ListStatus::Paused,
    ListStatus::Dropped,
    ListStatus::Repeating,
];

fn status_index(status: ListStatus) -> i32 {
    STATUSES
        .iter()
        .position(|s| *s == status)
        .map_or(-1, |i| i as i32)
}

fn parse_status(index: i32) -> Option<ListStatus> {
    usize::try_from(index)
        .ok()
        .and_then(|i| STATUSES.get(i).copied())
}

/// "8.5", the way the desktop writes a score: a whole one keeps its point.
pub fn score_text(tenths: i32) -> String {
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// Where a status or score of this show goes, in a line, the desktop's `listHome`.
fn list_home(detail: &SeriesDetail, anilist: bool, mal: bool) -> String {
    let ids = detail.card.match_info.as_ref();
    let al = ids.is_some_and(|m| m.anilist_id.is_some());
    let my = ids.is_some_and(|m| m.mal_id.is_some());
    if (anilist && al) || (mal && my) {
        tr!("show-home-each")
    } else if !al && !my {
        tr!("show-home-no-tracker")
    } else if anilist || mal {
        tr!("show-home-tracker-lacks")
    } else {
        tr!("show-home-until-connected")
    }
}

/// The connected trackers this show's writes reach, as a person names them.
fn tracker_sites(detail: &SeriesDetail, anilist: bool, mal: bool) -> String {
    let ids = detail.card.match_info.as_ref();
    let al = anilist && ids.is_some_and(|m| m.anilist_id.is_some());
    let my = mal && ids.is_some_and(|m| m.mal_id.is_some());
    match (al, my) {
        (true, true) => tr!("show-sites-both"),
        (true, false) => tr!("tracker-anilist"),
        (false, true) => tr!("tracker-mal"),
        (false, false) => String::new(),
    }
}

/// Copies the open series into the sheets' state, or empties it when this phone has none
/// open (a closed page, or the computer's show, whose lists the computer owns).
///
/// Answers the poster files the sections still need decoded, each with its art key.
pub fn sync(
    window: &AppWindow,
    detail: Option<&SeriesDetail>,
    trackers: Option<&(TrackerAccount, TrackerAccount)>,
    art: &Art,
) -> Vec<(u64, String)> {
    let show = window.global::<ShowData>();
    crate::show_match::track(detail.map(|d| &d.card));
    crate::schedule::track(window, detail);
    let Some(detail) = detail else {
        show.set_series_id(-1);
        show.set_sheet(0);
        GRAPH.with(|g| *g.borrow_mut() = Graph::default());
        SIG.with(|s| s.set(None));
        sections(&[], &[], &[]);
        show.set_related_total(0);
        show.set_related_filling(false);
        return Vec::new();
    };
    let card = &detail.card;
    show.set_series_id(i32::try_from(card.id).unwrap_or(-1));
    show.set_series_title(card.title.clone().into());
    // A hidden show is off every list: its tracker sync is off.
    show.set_lists(!card.hidden);
    show.set_matched(card.match_info.is_some());
    show.set_status_index(card.list_status.map_or(-1, status_index));
    show.set_status_word(
        card.list_status
            .map(status_word)
            .unwrap_or_else(|| tr!("phone-show-add-to-list"))
            .into(),
    );
    let score = card
        .my_score
        .map(|s| (s * 10.0).round().clamp(0.0, 100.0) as i32);
    show.set_score(score.unwrap_or(-1));
    show.set_score_word(
        match score {
            Some(s) => tr!("phone-show-your-score", score = score_text(s)),
            None => tr!("show-rate"),
        }
        .into(),
    );
    let (anilist, mal) = trackers.map_or((false, false), |(a, m)| (a.connected, m.connected));
    show.set_home(list_home(detail, anilist, mal).into());
    show.set_notes(detail.notes.clone().unwrap_or_default().into());
    show.set_sites(tracker_sites(detail, anilist, mal).into());
    let note = NOTE.with(|n| {
        let mut n = n.borrow_mut();
        if n.as_ref().is_some_and(|(_, at)| at.elapsed() > NOTE_FOR) {
            *n = None;
        }
        n.as_ref().map(|(line, _)| line.clone())
    });
    show.set_note(note.unwrap_or_default().into());
    graph(
        card.id,
        card.match_info.as_ref().and_then(|m| m.anilist_id),
        detail.has_graph,
    );
    let (layout, all) = GRAPH.with(|g| {
        let g = g.borrow();
        (g.layout.clone(), g.all)
    });
    // The page changes when what it lists, or a poster it draws, does; not once a second.
    let sig = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (card.id, art.posters.len(), all).hash(&mut h);
        if let Some(l) = &layout {
            l.complete.hash(&mut h);
            for n in &l.nodes {
                (
                    n.anilist_id,
                    n.owned,
                    n.list_status,
                    n.current,
                    n.released,
                    &n.title,
                    &n.relation,
                    &n.format,
                    n.year,
                    (n.x.to_bits(), n.y.to_bits()),
                )
                    .hash(&mut h);
            }
        }
        for r in &detail.recommendations {
            (r.anilist_id, r.owned, r.list_status, &r.title).hash(&mut h);
        }
        for p in &detail.characters {
            (p.anilist_id, &p.name, &p.role).hash(&mut h);
        }
        h.finish()
    };
    if SIG.with(|s| s.replace(Some(sig))) == Some(sig) {
        return Vec::new();
    }
    let mut asks = Vec::new();
    let mut want = |key: u64, path: &Option<String>| {
        if let Some(path) = path
            && !art.posters.contains_key(&key)
        {
            asks.push((key, path.clone()));
        }
    };
    // Related: the franchise as the crawl has it, in its own order, the page itself left out.
    let mut nodes: Vec<&anibeam_core::GraphNode> = layout
        .as_ref()
        .map(|l| l.nodes.iter().filter(|n| !n.current).collect())
        .unwrap_or_default();
    nodes.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    let related: Vec<ShowCard> = nodes
        .iter()
        .map(|n| {
            want(crate::app::AIRING_ART | n.anilist_id, &n.poster);
            let line = [
                relation_label(n.relation.as_deref()),
                // A node with no format says nothing here, not "Series".
                n.format
                    .as_deref()
                    .map(|f| format_label(Some(f)))
                    .unwrap_or_default(),
                n.year.map(|y| y.to_string()).unwrap_or_default(),
            ];
            let line: Vec<String> = line.into_iter().filter(|s| !s.is_empty()).collect();
            entry(
                art,
                n.anilist_id,
                n.owned,
                &n.title,
                line.join(", "),
                &if n.owned.is_some() {
                    tr!("show-card-owned")
                } else if !n.released {
                    tr!("show-card-unreleased")
                } else {
                    tr!("show-card-on-anilist")
                },
                n.list_status,
                n.site_url.as_deref(),
            )
        })
        .collect();
    let recs: Vec<ShowCard> = detail
        .recommendations
        .iter()
        .map(|r| {
            want(crate::app::AIRING_ART | r.anilist_id, &r.poster);
            entry(
                art,
                r.anilist_id,
                r.owned,
                &r.title,
                String::new(),
                &if r.owned.is_some() {
                    tr!("show-card-owned")
                } else {
                    tr!("show-card-on-anilist")
                },
                r.list_status,
                None,
            )
        })
        .collect();
    let people: Vec<ShowPerson> = detail
        .characters
        .iter()
        .map(|p| {
            let id = p.anilist_id;
            if let Some(id) = id {
                want(PERSON_ART | id, &p.image);
            }
            let (image, has_image) = id.map_or((slint::Image::default(), false), |id| {
                art.poster(Some(PERSON_ART | id))
            });
            ShowPerson {
                id: id.and_then(|id| i32::try_from(id).ok()).unwrap_or(-1),
                name: p.name.clone().into(),
                role: role_label(&p.role).into(),
                image,
                has_image,
                initials: format::initials(&p.name).into(),
            }
        })
        .collect();
    let filling = layout.as_ref().is_some_and(|l| !l.complete);
    show.set_related_filling(filling);
    show.set_related_total(i32::try_from(related.len()).unwrap_or(0));
    let related = if all {
        related
    } else {
        related.into_iter().take(RELATED_SHOWN).collect()
    };
    sections(&related, &recs, &people);
    asks
}

/// One related or recommended show as the page and its sheet carry it.
#[allow(clippy::too_many_arguments)]
fn entry(
    art: &Art,
    anilist_id: u64,
    owned: Option<u64>,
    title: &str,
    line: String,
    note: &str,
    status: Option<ListStatus>,
    site_url: Option<&str>,
) -> ShowCard {
    let (poster, has_poster) = art.poster(Some(crate::app::AIRING_ART | anilist_id));
    ShowCard {
        id: i32::try_from(anilist_id).unwrap_or(-1),
        owned: owned.and_then(|o| i32::try_from(o).ok()).unwrap_or(-1),
        title: title.into(),
        poster,
        has_poster,
        initials: format::initials(title).into(),
        line: line.into(),
        note: note.into(),
        status: status.map(status_word).unwrap_or_default().into(),
        url: site_url
            .map(str::to_string)
            .unwrap_or_else(|| format!("https://anilist.co/anime/{anilist_id}"))
            .into(),
    }
}

/// "SIDE_STORY" as "Side story".
fn relation_label(relation: Option<&str>) -> String {
    match relation.unwrap_or_default() {
        "PREQUEL" => tr!("show-relation-prequel"),
        "SEQUEL" => tr!("show-relation-sequel"),
        "SIDE_STORY" => tr!("show-relation-side-story"),
        "PARENT" => tr!("show-relation-parent"),
        "SPIN_OFF" => tr!("show-relation-spin-off"),
        "ADAPTATION" => tr!("show-relation-adaptation"),
        "ALTERNATIVE" => tr!("show-relation-alternative"),
        "SUMMARY" => tr!("show-relation-summary"),
        "CHARACTER" => tr!("show-relation-character"),
        "SOURCE" => tr!("show-relation-source"),
        "COMPILATION" => tr!("show-relation-compilation"),
        "CONTAINS" => tr!("show-relation-contains"),
        "OTHER" => tr!("show-relation-other"),
        // A relation the core adds later reads as its own name until the catalog has it.
        other => sentence(&other.replace('_', " ")),
    }
}

/// A character's role as AniList gives it ("MAIN"), worded for a person.
fn role_label(role: &str) -> String {
    match role {
        "MAIN" => tr!("show-role-main"),
        "SUPPORTING" => tr!("show-role-supporting"),
        "BACKGROUND" => tr!("show-role-background"),
        other => sentence(other),
    }
}

/// The first letter up, the rest down: "main" as "Main".
fn sentence(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// An AniList format as the desktop words it.
pub(crate) fn format_label(format: Option<&str>) -> String {
    match format {
        None => tr!("show-format-series"),
        Some("TV") => tr!("show-format-tv"),
        Some("TV_SHORT") => tr!("show-format-tv-short"),
        Some("OVA") => tr!("show-format-ova"),
        Some("ONA") => tr!("show-format-ona"),
        Some("SPECIAL") => tr!("show-format-special"),
        Some("MOVIE") => tr!("show-format-film"),
        Some(other) => other.replace('_', " "),
    }
}

/// Puts the sections' rows in their models, touching only what changed.
fn sections(related: &[ShowCard], recs: &[ShowCard], people: &[ShowPerson]) {
    MODELS.with(|m| {
        let mut m = m.borrow_mut();
        let Some(models) = m.as_mut() else { return };
        crate::app::update(&models.related, related.to_vec());
        let tiles = recs
            .iter()
            .map(|c| Tile {
                id: c.id,
                title: c.title.clone(),
                poster: c.poster.clone(),
                has_poster: c.has_poster,
                initials: c.initials.clone(),
                badge: if c.owned >= 0 {
                    tr!("show-rec-available")
                } else {
                    tr!("tracker-anilist")
                }
                .into(),
                flag: c.status.clone(),
                ..Default::default()
            })
            .collect();
        crate::app::update(&models.recs, tiles);
        crate::app::update(&models.people, people.to_vec());
        models.entries = related.iter().chain(recs).cloned().collect();
    });
}

/// Reads the open series' franchise when it has none yet, and again while the crawl is still
/// walking it. The page draws what the last read held.
fn graph(series: u64, matched: Option<u64>, has_graph: bool) {
    let start = GRAPH.with(|g| {
        let mut g = g.borrow_mut();
        if g.series != Some(series) || g.matched != matched {
            *g = Graph {
                series: Some(series),
                matched,
                ..Graph::default()
            };
        }
        if !has_graph || g.reading {
            return false;
        }
        let due = match (&g.layout, g.read_at) {
            (None, None) => true,
            (Some(l), Some(at)) => {
                at.elapsed()
                    > if l.complete {
                        GRAPH_DONE_EVERY
                    } else {
                        GRAPH_EVERY
                    }
            }
            (None, Some(at)) => at.elapsed() > GRAPH_EVERY,
            _ => false,
        };
        if due {
            g.reading = true;
            g.read_at = Some(Instant::now());
        }
        due
    });
    if !start {
        return;
    }
    let Some(core) = crate::local::core() else {
        GRAPH.with(|g| g.borrow_mut().reading = false);
        return;
    };
    let _ = std::thread::Builder::new()
        .name("franchise".into())
        .spawn(move || {
            let layout = match core.call(Call::GetFranchiseGraph { series }) {
                Ok(Reply::Graph { layout }) => layout,
                _ => None,
            };
            let _ = slint::invoke_from_event_loop(move || {
                GRAPH.with(|g| {
                    let mut g = g.borrow_mut();
                    if g.series == Some(series) && g.matched == matched {
                        g.reading = false;
                        if layout.is_some() {
                            g.layout = layout;
                        }
                    }
                });
                crate::local::touch();
            });
        });
}

/// The language changed: the related rows and recommendations carry words (what each is, the
/// list it is on) that are built once per change of the page, so build them again.
pub fn relocalize() {
    SIG.with(|s| s.set(None));
    crate::local::touch();
}

/// Hooks the sheets' callbacks to the core.
pub fn install(window: &AppWindow) {
    let show = window.global::<ShowData>();
    let weak = window.as_weak();
    let models = Models {
        related: Rc::new(VecModel::default()),
        recs: Rc::new(VecModel::default()),
        people: Rc::new(VecModel::default()),
        entries: Vec::new(),
    };
    show.set_related(ModelRc::from(models.related.clone()));
    show.set_recs(ModelRc::from(models.recs.clone()));
    show.set_people(ModelRc::from(models.people.clone()));
    MODELS.with(|m| *m.borrow_mut() = Some(models));
    // A related row or a recommendation: this library's own opens its page, the rest ask.
    let pick = {
        let weak = weak.clone();
        move |id: i32| {
            let Some(window) = weak.upgrade() else { return };
            let card = MODELS.with(|m| {
                m.borrow()
                    .as_ref()
                    .and_then(|m| m.entries.iter().find(|c| c.id == id).cloned())
            });
            let Some(card) = card else { return };
            if card.owned >= 0 {
                window.global::<AppData>().invoke_open_series(card.owned);
            } else {
                let show = window.global::<ShowData>();
                show.set_entry(card);
                show.set_sheet(3);
            }
        }
    };
    show.on_pick_related(pick.clone());
    show.on_pick_rec(pick);
    show.on_rescan({
        let weak = weak.clone();
        move || {
            let Some(window) = weak.upgrade() else { return };
            let Ok(series) = u64::try_from(window.global::<ShowData>().get_series_id()) else {
                return;
            };
            call(Call::RescanSeries { series }, |_| {
                tr!("show-rescan-started")
            });
        }
    });
    show.on_show_all_related(|| {
        GRAPH.with(|g| g.borrow_mut().all = true);
        SIG.with(|s| s.set(None));
        crate::local::touch();
    });
    show.on_pick_person(|id| {
        crate::platform::open_url(&format!("https://anilist.co/character/{id}"));
    });
    show.on_open_link(|url| {
        if url.starts_with("https://") {
            crate::platform::open_url(&url);
        }
    });
    show.on_set_status({
        let weak = weak.clone();
        move |index| {
            let (Some(window), Some(status)) = (weak.upgrade(), parse_status(index)) else {
                return;
            };
            let Ok(series) = u64::try_from(window.global::<ShowData>().get_series_id()) else {
                return;
            };
            call(Call::SetListStatus { series, status }, move |started| {
                // Worded when the core answers, in the language the app speaks by then.
                let label = status_word(status);
                if started {
                    tr!("phone-show-moving-to", label = label)
                } else {
                    tr!("phone-show-moved-to", label = label)
                }
            });
        }
    });
    show.on_save_notes({
        let weak = weak.clone();
        move |notes| {
            let Some(window) = weak.upgrade() else { return };
            let Ok(series) = u64::try_from(window.global::<ShowData>().get_series_id()) else {
                return;
            };
            let notes = notes.to_string();
            let clearing = notes.trim().is_empty();
            call(Call::SetSeriesNotes { series, notes }, move |_| {
                if clearing {
                    tr!("show-notes-clearing")
                } else {
                    tr!("show-notes-saving")
                }
            });
        }
    });
    show.on_set_score(move |tenths| {
        let Some(window) = weak.upgrade() else { return };
        let Ok(series) = u64::try_from(window.global::<ShowData>().get_series_id()) else {
            return;
        };
        let score = (tenths >= 0).then(|| f64::from(tenths) / 10.0);
        call(Call::SetScore { series, score }, move |started| {
            match (score.is_some(), started) {
                (true, true) => tr!("show-score-saving"),
                (true, false) => tr!("show-score-saved"),
                (false, true) => tr!("show-score-clearing"),
                (false, false) => tr!("show-score-cleared"),
            }
        });
    });
}

/// Runs a core call off the UI thread and puts a line under the buttons when it went through:
/// `working` when the core started a job that is still under way, `done` when it answered at
/// once. The core's own words when it refused.
pub fn run(call_: Call, working: String, done: String) {
    call(call_, move |started| if started { working } else { done });
}

/// Runs a core call off the UI thread and puts what it said under the buttons. `done` gets
/// whether the core started a job (a tracker write) rather than answering at once (the
/// list kept on this phone).
fn call(call: Call, done: impl FnOnce(bool) -> String + Send + 'static) {
    let Some(core) = crate::local::core() else {
        say(tr!("show-still-starting"));
        return;
    };
    let _ = std::thread::Builder::new()
        .name("show".into())
        .spawn(move || {
            let line = match core.call(call) {
                Ok(reply) => done(matches!(reply, Reply::Started { .. })),
                Err(e) => e.to_string(),
            };
            let _ = slint::invoke_from_event_loop(move || say(line));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_score_keeps_its_point() {
        assert_eq!(score_text(80), "8.0");
        assert_eq!(score_text(85), "8.5");
        assert_eq!(score_text(100), "10.0");
        assert_eq!(score_text(0), "0.0");
    }

    #[test]
    fn statuses_round_trip_and_on_hold_is_worded_like_the_desktop() {
        for status in [
            ListStatus::Watching,
            ListStatus::Planning,
            ListStatus::Completed,
            ListStatus::Paused,
            ListStatus::Dropped,
            ListStatus::Repeating,
        ] {
            assert_eq!(parse_status(status_index(status)), Some(status));
        }
        assert_eq!(status_word(ListStatus::Paused), "On hold");
        assert_eq!(parse_status(-1), None);
        assert_eq!(parse_status(99), None);
    }

    #[test]
    fn relations_and_formats_read_like_the_desktop() {
        assert_eq!(relation_label(Some("SIDE_STORY")), "Side story");
        assert_eq!(relation_label(Some("PREQUEL")), "Prequel");
        assert_eq!(relation_label(None), "");
        assert_eq!(format_label(Some("TV_SHORT")), "TV short");
        assert_eq!(format_label(Some("MOVIE")), "Film");
        assert_eq!(format_label(None), "Series");
        assert_eq!(sentence("SUPPORTING"), "Supporting");
    }
}
