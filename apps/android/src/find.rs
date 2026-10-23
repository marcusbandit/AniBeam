//! Find: the desktop's Downloads page on the phone, driven by the same core logic (the core's
//! port of anirss and the desktop's show view grid). Four lenses with nothing searched; a
//! show opens the show view; typed text opens the release list with the refine steps.
//! Every core call runs on a thread of its own and answers through the event loop.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

use anibeam_companion::format;
use anibeam_core::releases::grid;
use anibeam_core::{
    Call, CoreError, Direction, Refinement, RefineOp, Release, ReleaseFacts, ReleaseGroup, ReleaseSearch, Reply,
    SearchResult, SeriesCard, Sort, SubscriptionRule, Tab, Torrent, BestFit,
};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel, Weak};

use crate::prefs;
use crate::{AppWindow, FindData, GridRow, GroupChip, ReleaseRow, SourceChip, Suggestion, Tile};

const LENSES: [&str; 4] = ["Gaps", "Airing", "Following", "Downloaded"];

/// A show a lens offers, enough to open the show view.
#[derive(Clone, Debug, Default)]
struct Offer {
    show: grid::Show,
    /// A library series, not one found only on AniList.
    in_library: bool,
    poster: Option<String>,
    ledger: String,
}

/// What one source answered for the open show.
#[derive(Clone, Debug)]
enum Answer {
    Idle,
    Searching { at: usize },
    Done(Box<ReleaseSearch>),
    Failed(String),
}

struct ShowState {
    offer: Offer,
    sources: Vec<(String, Answer)>,
    active: Option<usize>,
    picks: grid::Picks,
    follow: bool,
    replace: bool,
}

/// The release list: anirss's search state, which the caller holds.
#[derive(Clone, Debug)]
struct ListState {
    query: String,
    endpoint: String,
    items: Vec<Release>,
    facts: Vec<ReleaseFacts>,
    groups: Vec<ReleaseGroup>,
    best_fit: BestFit,
}

impl From<ReleaseSearch> for ListState {
    fn from(s: ReleaseSearch) -> Self {
        ListState { query: s.query, endpoint: s.endpoint, items: s.items, facts: s.facts, groups: s.groups, best_fit: s.best_fit }
    }
}

impl From<Refinement> for ListState {
    fn from(r: Refinement) -> Self {
        ListState { query: r.query, endpoint: r.endpoint, items: r.items, facts: r.facts, groups: r.groups, best_fit: r.best_fit }
    }
}

struct Find {
    window: Weak<AppWindow>,
    data_dir: PathBuf,
    lens: usize,
    offers: Vec<Offer>,
    cards: Vec<SeriesCard>,
    torrents: Vec<Torrent>,
    rules: Vec<SubscriptionRule>,
    show: Option<ShowState>,
    list: Option<ListState>,
    history: Vec<ListState>,
    /// Whether the list was opened from the show view, so Back returns there.
    list_from_show: bool,
    asked_notifications: bool,
    posters: BTreeMap<String, Image>,
    /// The shows under the field as you type, the desktop's suggestions: the library's
    /// first, then AniList's.
    suggestions: Vec<Offer>,
    /// The typed text the AniList answer is for; an answer for older text is dropped.
    suggest_for: String,
    /// The last answer and the text it was for, so going back to the same words lists
    /// them again without asking AniList twice.
    answered_for: String,
    answered: Vec<Offer>,
}

thread_local! {
    static FIND: RefCell<Option<Find>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Find)) {
    FIND.with(|cell| {
        if let Some(find) = cell.borrow_mut().as_mut() {
            f(find);
        }
    });
}

/// A core error as a person reads it. A source that cannot be reached gets the reason it
/// most likely is: the network blocks it. AniBeam already looks names up privately, so
/// what is left is a block on the address, or the site being down.
fn words(e: &CoreError) -> String {
    let text = e.to_string();
    let lower = text.to_lowercase();
    if lower.contains("timed out") || lower.contains("error sending request") || lower.contains("unreachable") || lower.contains("connect") {
        let host = match e {
            CoreError::Service { service, .. } => service.clone(),
            _ => "The source".to_string(),
        };
        return format!(
            "{host} could not be reached. Some networks and countries block release sites; AniBeam already gets past blocks on names, so this one blocks the site itself, or the site is down. A VPN gets past it, or try another source."
        );
    }
    text
}

/// Runs `call` on a thread and hands its answer to `then` on the UI thread.
fn ask(call: Call, then: impl FnOnce(&mut Find, Result<Reply, CoreError>) + Send + 'static) {
    let Some(core) = crate::local::core() else { return };
    let _ = std::thread::Builder::new().name("find".into()).spawn(move || {
        let answer = core.call(call);
        let _ = slint::invoke_from_event_loop(move || with(|f| then(f, answer)));
    });
}

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

pub fn install(window: &AppWindow, data_dir: PathBuf) {
    FIND.with(|cell| {
        *cell.borrow_mut() = Some(Find {
            window: window.as_weak(),
            data_dir,
            lens: 0,
            offers: Vec::new(),
            cards: Vec::new(),
            torrents: Vec::new(),
            rules: Vec::new(),
            show: None,
            list: None,
            history: Vec::new(),
            list_from_show: false,
            asked_notifications: false,
            posters: BTreeMap::new(),
            suggestions: Vec::new(),
            suggest_for: String::new(),
            answered_for: String::new(),
            answered: Vec::new(),
        })
    });
    let d = window.global::<FindData>();
    d.set_lenses(model(LENSES.iter().map(|l| SharedString::from(*l)).collect()));
    d.on_open(|text| with(|f| f.open(text.to_string())));
    d.on_close(|| with(Find::close));
    d.on_back(|| with(Find::back));
    d.on_search(|text| with(|f| f.search(text.to_string())));
    d.on_find_series(|id| {
        with(|f| {
            f.with_data(|d| d.set_shown(true));
            f.refresh();
            f.with_data(|d| d.set_text(SharedString::new()));
            f.clear_suggestions();
            let card = f.cards.iter().find(|c| c.id as i32 == id).cloned();
            match card {
                Some(card) => f.open_show(Find::offer_of(&card)),
                None => {
                    f.show = None;
                    f.list = None;
                    f.set_view(0);
                    f.show_lens();
                    f.line("", "That show is no longer in the library on this phone.");
                }
            }
        })
    });
    d.on_suggest(|text| with(|f| f.suggest(text.to_string())));
    d.on_pick_suggestion(|i| {
        with(|f| {
            let Some(offer) = usize::try_from(i).ok().and_then(|i| f.suggestions.get(i)).cloned() else { return };
            f.clear_suggestions();
            f.open_show(offer);
        })
    });
    d.on_pick_lens(|i| {
        with(|f| {
            f.lens = usize::try_from(i).unwrap_or(0).min(3);
            f.show_lens();
        })
    });
    d.on_open_tile(|i| with(|f| f.open_offer(i)));
    d.on_fetch(|i| with(|f| f.fetch(i)));
    d.on_pick_row(|r| with(|f| f.pick_row(r)));
    d.on_toggle_cube(|r, n| with(|f| f.toggle_cube(r, n)));
    d.on_set_follow(|on| {
        with(|f| {
            if let Some(s) = f.show.as_mut() {
                s.follow = on;
            }
            f.draw_show();
        })
    });
    d.on_set_replace(|on| {
        with(|f| {
            if let Some(s) = f.show.as_mut() {
                s.replace = on;
            }
            f.draw_show();
        })
    });
    d.on_go(|| with(Find::go));
    d.on_list_all(|| with(Find::list_all));
    d.on_apply_group(|i| {
        with(|f| {
            let tokens = f.list.as_ref().and_then(|l| l.groups.get(i as usize)).map(|g| g.tokens.clone());
            if let Some(tokens) = tokens {
                f.refine(RefineOp::Tokens { tokens });
            }
        })
    });
    d.on_apply_best_fit(|| with(|f| f.refine(RefineOp::BestFit)));
    d.on_apply_narrow(|text| {
        with(|f| {
            let t = text.trim().to_string();
            if let Some(term) = t.strip_prefix('-') {
                f.refine(RefineOp::Exclude { term: term.to_string() });
            } else if !t.is_empty() {
                f.refine(RefineOp::Term { term: t });
            }
        })
    });
    d.on_undo(|| with(Find::undo));
    d.on_download_release(|i| with(|f| f.download_release(i)));
    d.on_download_all(|| with(Find::download_all));
    d.on_subscribe(|| with(Find::subscribe));
}

impl Find {
    fn data(&self) -> Option<AppWindow> {
        self.window.upgrade()
    }

    fn with_data(&self, f: impl FnOnce(FindData<'_>)) {
        if let Some(w) = self.data() {
            f(w.global::<FindData>());
        }
    }

    fn line(&self, text: &str, error: &str) {
        let (text, error) = (SharedString::from(text), SharedString::from(error));
        self.with_data(|d| {
            d.set_line(text);
            d.set_error(error);
        });
    }

    fn set_view(&self, view: i32) {
        self.with_data(|d| d.set_view(view));
    }

    // ---- Opening, closing, the lenses

    fn open(&mut self, text: String) {
        self.with_data(|d| d.set_shown(true));
        self.refresh();
        if text.trim().is_empty() {
            self.show = None;
            self.list = None;
            self.set_view(0);
            self.line("", "");
        } else {
            self.with_data(|d| d.set_text(text.clone().into()));
            self.search(text);
        }
    }

    fn close(&mut self) {
        self.with_data(|d| d.set_shown(false));
    }

    fn back(&mut self) {
        let view = self.data().map(|w| w.global::<FindData>().get_view()).unwrap_or(0);
        match view {
            2 if self.list_from_show && self.show.is_some() => {
                self.list = None;
                self.set_view(1);
                self.draw_show();
            }
            1 | 2 => {
                self.show = None;
                self.list = None;
                self.history.clear();
                self.set_view(0);
                self.line("", "");
                self.show_lens();
                // Back to the field as it was left: the typed words list their shows again.
                let typed = self.data().map(|w| w.global::<FindData>().get_text().to_string()).unwrap_or_default();
                self.suggest(typed);
            }
            _ => self.close(),
        }
    }

    /// Reads the library, the torrents and the subscriptions again, for the lenses.
    fn refresh(&mut self) {
        let Some(core) = crate::local::core() else { return };
        let library = Call::ListLibrary {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        };
        if let Ok(Reply::Library { series, .. }) = core.call(library) {
            self.cards = series;
        }
        ask(Call::PhoneTorrents, |f, answer| {
            if let Ok(Reply::PhoneTorrents { torrents, .. }) = answer {
                f.torrents = torrents;
                f.show_lens();
                f.draw_show();
            }
        });
        ask(Call::PhoneRules, |f, answer| {
            if let Ok(Reply::PhoneRules { rules }) = answer {
                f.rules = rules;
                f.show_lens();
            }
        });
        self.show_lens();
    }

    fn offer_of(card: &SeriesCard) -> Offer {
        let folder = std::path::Path::new(&card.path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        Offer {
            show: grid::Show {
                title: card.title.clone(),
                queries: names_of(card),
                total: card.total_episodes.unwrap_or(0),
                airing: card.next_airing.is_some()
                    || matches!(card.status, Some(anibeam_core::AiringStatus::Releasing)),
                cubes: card.cubes.clone(),
                folder,
            },
            in_library: true,
            poster: card.poster.clone(),
            ledger: String::new(),
        }
    }

    /// A show known only by a name and a folder: a subscription or a download not in the
    /// library yet.
    fn offer_named(&self, name: &str, folder: &str) -> Offer {
        if let Some(card) = self.cards.iter().find(|c| c.path.ends_with(&format!("/{folder}"))) {
            return Self::offer_of(card);
        }
        let title = anibeam_core::releases::titles::show_name(name);
        let title = if title.is_empty() { name.to_string() } else { title };
        Offer {
            show: grid::Show {
                title: title.clone(),
                queries: vec![title],
                total: 0,
                airing: true,
                cubes: String::new(),
                folder: folder.to_string(),
            },
            in_library: false,
            poster: None,
            ledger: String::new(),
        }
    }

    fn show_lens(&mut self) {
        let now = anibeam_companion::now();
        let offers: Vec<Offer> = match self.lens {
            0 => self
                .cards
                .iter()
                .filter_map(|c| {
                    let missing = c.cubes.chars().filter(|x| matches!(x, 'm' | 'g')).count();
                    (missing > 0).then(|| Offer { ledger: format!("{missing} missing"), ..Self::offer_of(c) })
                })
                .collect(),
            1 => {
                let mut airing: Vec<(&SeriesCard, i64)> = self
                    .cards
                    .iter()
                    .filter_map(|c| c.next_airing.as_ref().map(|a| (c, crate::phone::secs(a.at))))
                    .collect();
                airing.sort_by_key(|(_, at)| *at);
                airing
                    .into_iter()
                    .map(|(c, at)| Offer {
                        ledger: if at > now { format!("in {}", format::countdown(at - now)) } else { "aired".into() },
                        ..Self::offer_of(c)
                    })
                    .collect()
            }
            2 => self
                .rules
                .iter()
                .map(|r| {
                    let folder = std::path::Path::new(&r.save_path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    Offer {
                        ledger: if r.file_count == 1 { "1 file".into() } else { format!("{} files", r.file_count) },
                        ..self.offer_named(&r.name, &folder)
                    }
                })
                .collect(),
            _ => {
                let mut folders: Vec<(String, String, f64)> = Vec::new();
                for t in &self.torrents {
                    let folder = std::path::Path::new(&t.save_path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    match folders.iter_mut().find(|(f, _, _)| *f == folder) {
                        Some(entry) => entry.2 = entry.2.min(t.progress),
                        None => folders.push((folder, t.name.clone(), t.progress)),
                    }
                }
                folders
                    .into_iter()
                    .map(|(folder, name, progress)| Offer {
                        ledger: if progress >= 1.0 { "downloaded".into() } else { format!("{}%", (progress * 100.0) as i64) },
                        ..self.offer_named(&name, &folder)
                    })
                    .collect()
            }
        };
        let lines = [
            "Aired episodes that are not on this phone. Open a show to see who has them.",
            "New episodes still to come, the soonest first.",
            "The subscriptions: new episodes download by themselves.",
            "What this phone downloaded, a show at a time.",
        ];
        let empty = [
            "Nothing is missing: every aired episode of every show is on this phone.",
            "Nothing on this phone is airing.",
            "No subscriptions yet. Open a show and turn on Follow.",
            "Nothing downloaded yet.",
        ];
        let counts = [
            self.cards.iter().filter(|c| c.cubes.contains(['m', 'g'])).count(),
            self.cards.iter().filter(|c| c.next_airing.is_some()).count(),
            self.rules.len(),
            offers.len().max(if self.lens == 3 { offers.len() } else { 0 }),
        ];
        let tiles: Vec<Tile> = offers
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let (poster, has_poster) = match o.poster.as_ref().and_then(|p| self.posters.get(p)) {
                    Some(img) => (img.clone(), true),
                    None => (Image::default(), false),
                };
                Tile {
                    id: i as i32,
                    title: o.show.title.clone().into(),
                    poster,
                    has_poster,
                    initials: format::initials(&o.show.title).into(),
                    ledger: o.ledger.clone().into(),
                    cubes: model(crate::phone::cube_codes(&o.show.cubes)),
                    ..Default::default()
                }
            })
            .collect();
        self.load_posters(&offers);
        let lens = self.lens;
        let names: Vec<SharedString> = LENSES
            .iter()
            .enumerate()
            .map(|(i, l)| if i == 3 { SharedString::from(*l) } else { format!("{l} {}", counts[i]).into() })
            .collect();
        let line = if offers.is_empty() { empty[lens] } else { lines[lens] };
        self.with_data(|d| {
            d.set_lens(lens as i32);
            d.set_lenses(model(names));
            d.set_lens_line(line.into());
            d.set_lens_tiles(model(tiles));
        });
        self.offers = offers;
    }

    fn load_posters(&self, offers: &[Offer]) {
        let wanted: Vec<String> =
            offers.iter().filter_map(|o| o.poster.clone()).filter(|p| !self.posters.contains_key(p)).collect();
        if wanted.is_empty() {
            return;
        }
        let _ = std::thread::Builder::new().name("find-posters".into()).spawn(move || {
            for path in wanted {
                let Some(decoded) = std::fs::read(&path).ok().and_then(|b| crate::posters::decode(&b)) else { continue };
                let _ = slint::invoke_from_event_loop(move || {
                    with(|f| {
                        f.posters.insert(path, Image::from_rgba8(decoded.buffer));
                        f.show_lens();
                        f.draw_show();
                        f.draw_suggestions();
                    })
                });
            }
        });
    }

    // ---- The suggestions: shows first, the way the desktop's field works

    fn suggest(&mut self, text: String) {
        let text = text.trim().to_string();
        if text.chars().count() < 2 {
            self.clear_suggestions();
            return;
        }
        // The same words as the last answer: list it again, AniList already said.
        if text == self.answered_for {
            self.suggestions = self.answered.clone();
            self.suggest_for = text;
            self.with_data(|d| {
                d.set_suggesting(false);
                d.set_suggest_done(true);
                d.set_suggest_error(SharedString::new());
            });
            self.draw_suggestions();
            return;
        }
        let needle = text.to_lowercase();
        let shown: Vec<&SeriesCard> =
            self.cards.iter().filter(|c| names_of(c).iter().any(|n| n.to_lowercase().contains(&needle))).take(5).collect();
        let mut ids: Vec<u64> = shown.iter().map(|c| c.id).collect();
        let mut offers: Vec<Offer> = shown.into_iter().map(library_offer).collect();
        self.suggestions = offers.clone();
        self.suggest_for = text.clone();
        self.with_data(|d| {
            d.set_suggesting(true);
            d.set_suggest_done(false);
            d.set_suggest_error(SharedString::new());
        });
        self.draw_suggestions();
        ask(Call::PhoneFindShows { query: text.clone(), limit: 8 }, move |f, answer| {
            if f.suggest_for != text {
                return;
            }
            match answer {
                Ok(Reply::PhoneShows { shows, posters }) => {
                    for (hit, poster) in shows.iter().zip(posters) {
                        if offers.len() >= 10 {
                            break;
                        }
                        // A hit that is a library show is that show, from the library: listed
                        // once, with what the phone has, even past the first five matches.
                        let names: Vec<String> = primary_names(hit).iter().map(|n| n.to_lowercase()).collect();
                        let card = f.cards.iter().find(|c| names_of(c).iter().any(|n| names.contains(&n.to_lowercase())));
                        match card {
                            Some(c) if ids.contains(&c.id) => {}
                            Some(c) => {
                                ids.push(c.id);
                                offers.push(library_offer(c));
                            }
                            None => offers.push(offer_of_hit(hit, poster)),
                        }
                    }
                    f.answered_for = text.clone();
                    f.answered = offers.clone();
                    f.suggestions = offers;
                    f.with_data(|d| {
                        d.set_suggesting(false);
                        d.set_suggest_done(true);
                    });
                    f.load_posters(&f.suggestions.clone());
                    f.draw_suggestions();
                }
                Ok(_) => f.with_data(|d| d.set_suggesting(false)),
                Err(e) => {
                    let why = SharedString::from(format!("AniList could not be asked: {}", words(&e)));
                    f.with_data(|d| {
                        d.set_suggesting(false);
                        d.set_suggest_done(true);
                        d.set_suggest_error(why);
                    });
                }
            }
        });
    }

    fn clear_suggestions(&mut self) {
        self.suggestions.clear();
        self.suggest_for.clear();
        self.with_data(|d| {
            d.set_suggesting(false);
            d.set_suggest_done(false);
            d.set_suggest_error(SharedString::new());
        });
        self.draw_suggestions();
    }

    fn draw_suggestions(&self) {
        let rows: Vec<Suggestion> = self
            .suggestions
            .iter()
            .map(|o| {
                let (poster, has_poster) = match o.poster.as_ref().and_then(|p| self.posters.get(p)) {
                    Some(img) => (img.clone(), true),
                    None => (Image::default(), false),
                };
                Suggestion {
                    title: o.show.title.clone().into(),
                    line: o.ledger.clone().into(),
                    poster,
                    has_poster,
                    initials: format::initials(&o.show.title).into(),
                    library: o.in_library,
                    cubes: model(crate::phone::cube_codes(&o.show.cubes)),
                }
            })
            .collect();
        self.with_data(|d| d.set_suggestions(model(rows)));
    }

    // ---- The show view

    fn open_offer(&mut self, i: i32) {
        let Some(offer) = usize::try_from(i).ok().and_then(|i| self.offers.get(i)).cloned() else { return };
        self.open_show(offer);
    }

    fn open_show(&mut self, offer: Offer) {
        let sources = crate::local::core()
            .and_then(|c| match c.call(Call::PhoneSources) {
                Ok(Reply::PhoneSources { sources }) => Some(sources),
                _ => None,
            })
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.enabled)
            .map(|s| (s.name, Answer::Idle))
            .collect();
        let follow = offer.show.airing;
        self.show = Some(ShowState { offer, sources, active: None, picks: grid::Picks::default(), follow, replace: false });
        self.list = None;
        self.history.clear();
        self.set_view(1);
        self.line("", "");
        self.draw_show();
        // The first source is asked at once, as a tap on it would.
        self.fetch(0);
    }

    fn fetch(&mut self, i: i32) {
        let Some(state) = self.show.as_mut() else { return };
        let Some(i) = usize::try_from(i).ok().filter(|i| *i < state.sources.len()) else { return };
        state.active = Some(i);
        state.picks = grid::Picks::default();
        state.replace = false;
        if matches!(state.sources[i].1, Answer::Done(_) | Answer::Searching { .. }) {
            self.auto_pick();
            self.draw_show();
            return;
        }
        self.search_as(i, 0);
    }

    /// One source's search by the show's `at`th name; an empty answer moves on to the next.
    fn search_as(&mut self, i: usize, at: usize) {
        let Some(state) = self.show.as_mut() else { return };
        let Some(query) = state.offer.show.queries.get(at).cloned() else { return };
        let name = state.sources[i].0.clone();
        state.sources[i].1 = Answer::Searching { at };
        let title = state.offer.show.title.clone();
        self.draw_show();
        ask(Call::PhoneSearch { query, endpoint: Some(name), fallback: false }, move |f, answer| {
            let Some(state) = f.show.as_mut() else { return };
            if state.offer.show.title != title || i >= state.sources.len() {
                return;
            }
            match answer {
                Ok(Reply::PhoneSearch { search }) => {
                    if search.items.is_empty() && at + 1 < state.offer.show.queries.len() {
                        f.search_as(i, at + 1);
                        return;
                    }
                    state.sources[i].1 = Answer::Done(Box::new(search));
                }
                Ok(_) => {}
                Err(e) => state.sources[i].1 = Answer::Failed(words(&e)),
            }
            if f.show.as_ref().and_then(|s| s.active) == Some(i) {
                // A source that failed, or had nothing of this show, hands over to the next
                // one nobody has asked yet, as a person would tap it.
                let nothing = match f.current() {
                    Some((items, ..)) => items.is_empty(),
                    None => true,
                };
                let next = f
                    .show
                    .as_ref()
                    .and_then(|s| s.sources.iter().skip(i + 1).position(|(_, a)| matches!(a, Answer::Idle)).map(|p| i + 1 + p));
                if nothing && let Some(next) = next {
                    f.fetch(next as i32);
                    return;
                }
                f.auto_pick();
            }
            f.draw_show();
        });
    }

    /// The active source's releases that name the show, and the grid of them.
    fn current(&self) -> Option<(Vec<Release>, Vec<ReleaseFacts>, usize, String, String)> {
        let state = self.show.as_ref()?;
        let i = state.active?;
        let Answer::Done(search) = &state.sources[i].1 else { return None };
        let (items, facts, dropped) = grid::of_show(&state.offer.show, &search.items, &search.facts);
        Some((items, facts, dropped, search.query.clone(), state.sources[i].0.clone()))
    }

    fn flight(&self) -> BTreeMap<u32, f64> {
        let Some(state) = self.show.as_ref() else { return BTreeMap::new() };
        let folder = &state.offer.show.folder;
        let mine: Vec<Torrent> = self
            .torrents
            .iter()
            .filter(|t| !folder.is_empty() && t.save_path.ends_with(&format!("/{folder}")))
            .cloned()
            .collect();
        grid::in_flight(&mine)
    }

    /// When a source's releases arrive, the group that fills the most gaps is picked.
    fn auto_pick(&mut self) {
        let Some((items, facts, _, _, provider)) = self.current() else { return };
        let flight = self.flight();
        let Some(state) = self.show.as_mut() else { return };
        if !state.picks.chosen.is_empty() {
            return;
        }
        let total = grid::total(&state.offer.show, &facts, &flight);
        let library = grid::library(&state.offer.show, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        if let Some(best) = grid::best_row(&rows, &library, total) {
            state.picks = grid::pick_row(&rows[best], &library, total, false);
        }
    }

    fn pick_row(&mut self, r: i32) {
        let Some((items, facts, _, _, provider)) = self.current() else { return };
        let flight = self.flight();
        let Some(state) = self.show.as_mut() else { return };
        let total = grid::total(&state.offer.show, &facts, &flight);
        let library = grid::library(&state.offer.show, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        if let Some(row) = usize::try_from(r).ok().and_then(|r| rows.get(r)) {
            state.picks = grid::pick_row(row, &library, total, true);
        }
        self.draw_show();
    }

    fn toggle_cube(&mut self, r: i32, n: i32) {
        let Some((items, facts, _, _, provider)) = self.current() else { return };
        let flight = self.flight();
        let Some(state) = self.show.as_mut() else { return };
        let total = grid::total(&state.offer.show, &facts, &flight);
        let library = grid::library(&state.offer.show, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        if let (Some(row), Ok(n)) = (usize::try_from(r).ok().and_then(|r| rows.get(r)), u32::try_from(n + 1)) {
            grid::toggle_cube(&mut state.picks, row, n, &library, total);
        }
        self.draw_show();
    }

    fn library_code(c: char) -> i32 {
        crate::phone::cube_codes(&c.to_string())[0]
    }

    fn draw_show(&self) {
        let Some(state) = self.show.as_ref() else { return };
        let show = &state.offer.show;
        let chips: Vec<SourceChip> = state
            .sources
            .iter()
            .map(|(name, answer)| {
                let (st, line) = match answer {
                    Answer::Idle => ("idle", "Tap to ask".to_string()),
                    Answer::Searching { at } => (
                        "searching",
                        if *at > 0 { format!("Trying {}", show.queries.get(*at).cloned().unwrap_or_default()) } else { "Searching".into() },
                    ),
                    Answer::Done(s) => ("done", if s.items.len() == 1 { "1 release".into() } else { format!("{} releases", s.items.len()) }),
                    Answer::Failed(_) => ("error", "Not answering".into()),
                };
                SourceChip { name: name.clone().into(), state: st.into(), line: line.into() }
            })
            .collect();
        let (poster, has_poster) = match state.offer.poster.as_ref().and_then(|p| self.posters.get(p)) {
            Some(img) => (img.clone(), true),
            None => (Image::default(), false),
        };
        let mut facts_line = Vec::new();
        if show.total > 0 {
            facts_line.push(format!("{} episodes", show.total));
        }
        if show.airing {
            facts_line.push("Airing".into());
        }
        let on_phone = show.cubes.chars().filter(|c| matches!(c, 'd' | 'l' | 'w')).count();
        facts_line.push(if on_phone == 1 { "1 on this phone".into() } else { format!("{on_phone} on this phone") });

        let mut library_codes = Vec::new();
        let mut grid_rows = Vec::new();
        let mut dropped_line = String::new();
        let (mut plan_line, mut plan_detail, mut go_text, mut can_go, mut can_follow, mut has_had) =
            (String::new(), String::new(), "Get".to_string(), false, false, false);
        let mut error = String::new();
        if let Some(Answer::Failed(e)) = state.active.map(|i| &state.sources[i].1) {
            error = e.clone();
        }
        if let Some((items, facts, dropped, query, provider)) = self.current() {
            let flight = self.flight();
            let total = grid::total(show, &facts, &flight);
            let library = grid::library(show, total, &flight);
            let rows = grid::rows(&items, &facts, total, &provider);
            library_codes = library.iter().map(|c| Self::library_code(*c)).collect();
            if dropped > 0 {
                dropped_line = format!("{dropped} releases that do not name this show are left out.");
            }
            grid_rows = rows
                .iter()
                .map(|row| {
                    let label = if row.batch {
                        format!("{} {} batch {}", row.group, row.resolution, row.label)
                    } else {
                        format!("{} {}", row.group, row.resolution)
                    };
                    let meta = format!(
                        "{}  {} seeds  {}",
                        if row.count == 1 { "1 episode".to_string() } else { format!("{} episodes", row.count) },
                        row.seeds,
                        format::bytes(row.size)
                    );
                    let cubes = grid::codes_of(row, &state.picks, &library, total)
                        .into_iter()
                        .map(|c| match c {
                            'a' => 1,
                            'p' => 2,
                            'o' => 3,
                            't' => 4,
                            _ => 0,
                        })
                        .collect();
                    GridRow { label: label.trim().into(), meta: meta.into(), cubes: model(cubes), chosen: row.key == state.picks.chosen }
                })
                .collect();
            let plan = grid::plan(show, &query, &items, &facts, &rows, &state.picks, &library, state.replace, &provider);
            has_had = state.picks.map.keys().any(|n| grid::have(&library, *n));
            can_follow = !plan.follow_query.is_empty() && show.airing;
            let following = state.follow && can_follow;
            let n = plan.episodes.len();
            can_go = n > 0 || following;
            go_text = match (n, following) {
                (0, true) => "Subscribe".into(),
                (0, false) => "Pick episodes to get".into(),
                (n, f) => format!(
                    "Get {}{}{}",
                    if n == 1 { "1 episode".to_string() } else { format!("{n} episodes") },
                    if state.replace && has_had { " and replace" } else { "" },
                    if f { " and subscribe" } else { "" }
                ),
            };
            plan_line = if n == 0 {
                if rows.is_empty() { "This source has no releases of this show.".into() } else { "Nothing picked: tap a row or its cubes.".into() }
            } else {
                format!("{} from {}, {}", grid::episodes_phrase(&plan.episodes), plan.groups.join(" and "), format::bytes(plan.bytes))
            };
            let skipped = state.picks.map.keys().filter(|n| grid::have(&library, **n)).count();
            if skipped > 0 && !state.replace {
                plan_detail = format!(
                    "{} you already have {} left out. Turn on Replace to get {} again.",
                    if skipped == 1 { "1 episode" } else { "Episodes" },
                    if skipped == 1 { "is" } else { "are" },
                    if skipped == 1 { "it" } else { "them" }
                );
            }
            if following {
                let more = format!("New episodes from {} will download by themselves.", plan.follow_query);
                plan_detail = if plan_detail.is_empty() { more } else { format!("{plan_detail} {more}") };
            }
        }
        let active = state.active.map(|i| i as i32).unwrap_or(-1);
        let follow = state.follow;
        let replace = state.replace;
        self.with_data(|d| {
            d.set_show_title(show.title.clone().into());
            d.set_show_facts(facts_line.join("  ").into());
            d.set_show_poster(poster);
            d.set_show_has_poster(has_poster);
            d.set_sources(model(chips));
            d.set_source(active);
            d.set_library(model(library_codes));
            d.set_rows(model(grid_rows));
            d.set_dropped(dropped_line.into());
            d.set_plan(plan_line.into());
            d.set_plan_detail(plan_detail.into());
            d.set_go_text(go_text.into());
            d.set_can_go(can_go);
            d.set_can_follow(can_follow);
            d.set_follow(follow);
            d.set_replace(replace);
            d.set_has_had(has_had);
            d.set_error(error.into());
        });
    }

    fn limit(&self) -> Option<u64> {
        let p = prefs::load(&self.data_dir);
        p.limit_on.then(|| (p.limit_gb * 1e9) as u64)
    }

    fn ask_notifications(&mut self) {
        if !self.asked_notifications {
            self.asked_notifications = true;
            crate::android::ask_notifications();
        }
    }

    /// Get, and Follow: the picked releases download into the show's folder, then the
    /// subscription is made when Follow is on.
    fn go(&mut self) {
        let Some((items, facts, _, query, provider)) = self.current() else { return };
        let flight = self.flight();
        let Some(state) = self.show.as_ref() else { return };
        let total = grid::total(&state.offer.show, &facts, &flight);
        let library = grid::library(&state.offer.show, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        let plan = grid::plan(&state.offer.show, &query, &items, &facts, &rows, &state.picks, &library, state.replace, &provider);
        let follow = state.follow && !plan.follow_query.is_empty() && state.offer.show.airing;
        let limit = self.limit();
        self.ask_notifications();
        self.with_data(|d| d.set_busy(true));
        self.line("Working...", "");
        let name = plan.name.clone();
        let subscribe = follow.then(|| Call::PhoneSubscribe {
            query: plan.follow_query.clone(),
            endpoint: provider.clone(),
            items: plan.follow_items.clone(),
            name: Some(name.clone()),
        });
        let count = plan.episodes.len();
        let download = (!plan.items.is_empty()).then(|| Call::PhoneDownload { items: plan.items, name: Some(name), limit_bytes: limit });
        let Some(core) = crate::local::core() else { return };
        let _ = std::thread::Builder::new().name("get".into()).spawn(move || {
            let mut line = Vec::new();
            let mut error = String::new();
            if let Some(call) = download {
                match core.call(call) {
                    Ok(_) => line.push(if count == 1 { "Getting 1 episode.".to_string() } else { format!("Getting {count} episodes.") }),
                    Err(e) => error = words(&e),
                }
            }
            if error.is_empty()
                && let Some(call) = subscribe
            {
                match core.call(call) {
                    Ok(_) => line.push("Subscribed: new episodes download by themselves.".into()),
                    Err(e) => error = words(&e),
                }
            }
            let _ = slint::invoke_from_event_loop(move || {
                with(|f| {
                    f.with_data(|d| d.set_busy(false));
                    f.line(&line.join(" "), &error);
                    if error.is_empty()
                        && let Some(s) = f.show.as_mut()
                    {
                        s.picks = grid::Picks::default();
                    }
                    f.refresh();
                    f.draw_show();
                })
            });
        });
    }

    // ---- The release list

    fn list_all(&mut self) {
        let Some(state) = self.show.as_ref() else { return };
        let Some(Answer::Done(search)) = state.active.map(|i| &state.sources[i].1) else { return };
        self.list = Some(ListState::from((**search).clone()));
        self.history.clear();
        self.list_from_show = true;
        self.set_view(2);
        self.draw_list();
    }

    /// Typed text: the desktop's release search, every source with fallback.
    fn search(&mut self, text: String) {
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.show = None;
        self.list_from_show = false;
        self.history.clear();
        self.with_data(|d| d.set_busy(true));
        self.line(&format!("Searching for {text}..."), "");
        ask(Call::PhoneSearch { query: text, endpoint: None, fallback: true }, |f, answer| {
            f.with_data(|d| d.set_busy(false));
            match answer {
                Ok(Reply::PhoneSearch { search }) => {
                    let line = Self::search_line(&search);
                    f.list = Some(ListState::from(search));
                    f.set_view(2);
                    f.line(&line, "");
                    f.draw_list();
                }
                Ok(_) => {}
                Err(e) => f.line("", &words(&e)),
            }
        });
    }

    fn search_line(s: &ReleaseSearch) -> String {
        let mut line = match s.items.len() {
            0 => format!("Nothing on {} for that.", s.endpoint),
            1 => format!("1 release on {}", s.endpoint),
            n => format!("{n} releases on {}", s.endpoint),
        };
        if s.fell_back {
            line.push_str(", after the first source had nothing");
        }
        if let Some(token) = &s.auto_resolution {
            line.push_str(&format!(", narrowed to {token}"));
        }
        line.push('.');
        line
    }

    fn refine(&mut self, op: RefineOp) {
        let Some(list) = self.list.clone() else { return };
        self.with_data(|d| d.set_busy(true));
        ask(
            Call::PhoneRefine { query: list.query.clone(), endpoint: list.endpoint.clone(), items: list.items.clone(), op },
            move |f, answer| {
                f.with_data(|d| d.set_busy(false));
                match answer {
                    Ok(Reply::PhoneRefined { refinement }) if refinement.applied => {
                        f.history.push(list);
                        let line = format!(
                            "{} releases, {} {}.",
                            refinement.items.len(),
                            if refinement.delta < 0 { "down" } else { "up" },
                            refinement.delta.abs()
                        );
                        f.list = Some(ListState::from(refinement));
                        f.with_data(|d| d.set_narrow("".into()));
                        f.line(&line, "");
                        f.draw_list();
                    }
                    Ok(Reply::PhoneRefined { refinement }) => {
                        let why = match refinement.reason.as_deref() {
                            Some("zero_results") => "That leaves nothing.",
                            Some("no_match") => "No release has that.",
                            Some("empty_term") => "Type something first.",
                            _ => "That changes nothing.",
                        };
                        f.line("", why);
                    }
                    Ok(_) => {}
                    Err(e) => f.line("", &words(&e)),
                }
            },
        );
    }

    fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.list = Some(prev);
            self.line("", "");
            self.draw_list();
        }
    }

    fn draw_list(&self) {
        let Some(list) = self.list.as_ref() else { return };
        let best_link = list.best_fit.item.as_ref().map(|i| i.link.clone());
        let rows: Vec<ReleaseRow> = list
            .items
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let f = list.facts.get(i);
                let mut facts = Vec::new();
                if let Some(f) = f {
                    if !f.group.is_empty() {
                        facts.push(f.group.clone());
                    }
                    if !f.resolution.is_empty() {
                        facts.push(f.resolution.clone());
                    }
                    if f.batch {
                        facts.push(match (f.first_episode, f.last_episode) {
                            (Some(a), Some(b)) => format!("E{a:02}-{b:02}"),
                            _ => "batch".into(),
                        });
                    } else if let Some(e) = f.episode {
                        facts.push(format!("E{:02}", e as i64));
                    }
                    if let Some(b) = f.size_bytes {
                        facts.push(format::bytes(b));
                    }
                }
                if let Some(s) = r.seeders {
                    facts.push(format!("{s} seeds"));
                }
                if !r.source.is_empty() {
                    facts.push(format!("via {}", r.source));
                }
                ReleaseRow {
                    title: r.title.clone().into(),
                    facts: facts.join("  ").into(),
                    best: best_link.as_deref() == Some(r.link.as_str()),
                    on_disk: f.is_some_and(|f| f.on_disk),
                    tone: if r.seeders == Some(0) { 2 } else { 0 },
                }
            })
            .collect();
        let groups: Vec<GroupChip> =
            list.groups.iter().map(|g| GroupChip { label: g.label.clone().into(), count: g.member_count as i32 }).collect();
        let best = list.best_fit.available;
        let best_line = if best { format!("Best fit searches: {}", list.best_fit.query) } else { String::new() };
        let can_undo = !self.history.is_empty();
        self.with_data(|d| {
            d.set_releases(model(rows));
            d.set_groups(model(groups));
            d.set_best_fit(best);
            d.set_best_fit_line(best_line.into());
            d.set_can_undo(can_undo);
        });
    }

    fn download(&mut self, items: Vec<Release>) {
        if items.is_empty() {
            return;
        }
        self.ask_notifications();
        let count = items.len();
        let name = self.show.as_ref().map(|s| s.offer.show.folder.clone()).filter(|f| !f.is_empty());
        ask(Call::PhoneDownload { items, name, limit_bytes: self.limit() }, move |f, answer| match answer {
            Ok(_) => f.line(&if count == 1 { "Downloading 1 release.".into() } else { format!("Downloading {count} releases.") }, ""),
            Err(e) => f.line("", &format!("Not added: {}", words(&e))),
        });
    }

    fn download_release(&mut self, i: i32) {
        let item = self.list.as_ref().and_then(|l| usize::try_from(i).ok().and_then(|i| l.items.get(i))).cloned();
        if let Some(item) = item {
            self.download(vec![item]);
        }
    }

    fn download_all(&mut self) {
        let items = self.list.as_ref().map(|l| l.items.clone()).unwrap_or_default();
        self.download(items);
    }

    fn subscribe(&mut self) {
        let Some(list) = self.list.clone() else { return };
        let name = self.show.as_ref().map(|s| s.offer.show.folder.clone()).filter(|f| !f.is_empty());
        self.ask_notifications();
        ask(
            Call::PhoneSubscribe { query: list.query, endpoint: list.endpoint, items: list.items, name },
            |f, answer| match answer {
                Ok(Reply::PhoneSubscribed { subscribed }) => {
                    f.line(&format!("Subscribed as {}: new episodes download by themselves.", subscribed.name), "");
                    f.refresh();
                }
                Ok(_) => {}
                Err(e) => f.line("", &format!("Not subscribed: {}", words(&e))),
            },
        );
    }
}

/// Every name a library card goes by, the shown title first, each once. A show's
/// releases are searched by these in order.
fn names_of(c: &SeriesCard) -> Vec<String> {
    let mut names = vec![c.title.clone()];
    for t in [&c.titles.romaji, &c.titles.english].into_iter().flatten() {
        if !names.contains(t) {
            names.push(t.clone());
        }
    }
    names
}

/// A library show as a suggestion, with what the phone has of it.
fn library_offer(c: &SeriesCard) -> Offer {
    let mut offer = Find::offer_of(c);
    offer.ledger = library_line(c);
    offer
}

/// What a library show has on this phone, for its suggestion row.
fn library_line(c: &SeriesCard) -> String {
    let here = c.cubes.chars().filter(|k| matches!(k, 'w' | 'l' | 'd')).count();
    let total = c.cubes.chars().filter(|k| *k != 'f').count();
    match (here, total) {
        (_, 0) => "In your library".to_string(),
        (0, t) => format!("In your library, none of {t} on this phone"),
        (h, t) => format!("In your library, {h} of {t} on this phone"),
    }
}

/// The names an AniList hit is known by first: romaji, English, its title.
fn primary_names(h: &SearchResult) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for n in [h.romaji.as_ref(), h.english.as_ref(), Some(&h.title)].into_iter().flatten() {
        if !names.contains(n) {
            names.push(n.clone());
        }
    }
    names
}

/// The names to search releases by: the primary ones, then at most two synonyms that
/// release files could carry (latin script, long enough not to match everything).
fn hit_names(h: &SearchResult) -> Vec<String> {
    let mut names = primary_names(h);
    let latin = |n: &str| n.chars().all(|c| c.is_ascii() || (c.is_alphabetic() && (c as u32) < 0x0250));
    for n in h.synonyms.iter().filter(|n| n.chars().count() >= 5 && latin(n)).take(2) {
        if !names.contains(n) {
            names.push(n.clone());
        }
    }
    names
}

/// A title as a folder name: a slash or backslash would make it a path, so it becomes a
/// space (Fate/Zero goes into "Fate Zero"), and control characters go.
fn folder_name(title: &str) -> String {
    let cleaned: String = title.chars().map(|c| if c == '/' || c == '\\' { ' ' } else { c }).filter(|c| !c.is_control()).collect();
    let folded = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    match folded.as_str() {
        "" | "." | ".." => "Show".to_string(),
        _ => folded,
    }
}

/// A show found on AniList, not in the library: searched by its latin names, saved under
/// its romaji one.
fn offer_of_hit(h: &SearchResult, poster: String) -> Offer {
    let queries = hit_names(h);
    let title = h.romaji.clone().or_else(|| h.english.clone()).unwrap_or_else(|| h.title.clone());
    let total = h.episodes.unwrap_or(0);
    let mut facts = Vec::new();
    if let Some(f) = &h.format {
        facts.push(f.replace('_', " "));
    }
    if let Some(y) = h.year {
        facts.push(y.to_string());
    }
    if total > 0 {
        facts.push(if total == 1 { "1 episode".to_string() } else { format!("{total} episodes") });
    }
    Offer {
        show: grid::Show {
            title: title.clone(),
            queries,
            total,
            airing: h.airing,
            cubes: "g".repeat(total.min(400) as usize),
            folder: folder_name(&title),
        },
        in_library: false,
        poster: (!poster.is_empty()).then_some(poster),
        ledger: facts.join(", "),
    }
}
