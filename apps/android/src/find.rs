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
    BestFit, Call, CoreError, Direction, DownloadOptions, RefineOp, Refinement, Release,
    ReleaseFacts, ReleaseGroup, ReleaseSearch, Reply, SearchResult, SeriesCard, Sort,
    SubscriptionRule, Tab, Torrent, WatchingEntry,
};
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel, Weak};

use crate::prefs;
use crate::{AppWindow, FindData, GridRow, GroupChip, ReleaseRow, SourceChip, Suggestion, Tile};

const LENSES: [&str; 5] = ["Your list", "Gaps", "Airing", "Following", "Downloaded"];

/// A show a lens offers, enough to open the show view.
#[derive(Clone, Debug, Default)]
struct Offer {
    show: grid::Show,
    /// Its AniList id, when it is known: from AniList or from the list (#734). It finds
    /// the show on the computer and on the list.
    anilist: Option<u64>,
    /// The library series it is, for the list status of a show found in the library.
    series: Option<u64>,
    /// A library series, not one found only on AniList.
    in_library: bool,
    poster: Option<String>,
    ledger: String,
    /// The tile's tone for the ledger: 0 quiet, 1 accent, 2 warning, 3 good.
    ledger_tone: i32,
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
    /// The sources with no feed to follow, which answer JSON (#507): no subscription
    /// is offered on them.
    no_feed: Vec<String>,
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
        ListState {
            query: s.query,
            endpoint: s.endpoint,
            items: s.items,
            facts: s.facts,
            groups: s.groups,
            best_fit: s.best_fit,
        }
    }
}

impl From<Refinement> for ListState {
    fn from(r: Refinement) -> Self {
        ListState {
            query: r.query,
            endpoint: r.endpoint,
            items: r.items,
            facts: r.facts,
            groups: r.groups,
            best_fit: r.best_fit,
        }
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
    /// The person's list on AniList: Watching, then Planning (#734).
    entries: Vec<Entry>,
    /// AniList is being asked for the list; it is read again as the core reports.
    list_refreshing: bool,
    /// Why the list is empty when it is not just empty: no tracker signed in.
    list_note: Option<String>,
    /// Shows put on Planning from here that the list has not caught up with yet.
    planned: Vec<(u64, std::time::Instant)>,
    /// Shows Plan to watch found already on a list, with whatever status they have there.
    listed: Vec<u64>,
    /// The connected computer, when it answered that it takes downloads (#733).
    desk: Option<crate::desk::Desk>,
    /// A hello to the computer is out. One at a time, so an older answer never lands
    /// over a newer one.
    desk_asking: bool,
    /// When the computer last answered, or did not, so browsing shows asks it at most
    /// once a minute.
    desk_asked: Option<std::time::Instant>,
    /// Get goes to the computer: the pick in Download to, kept in prefs.
    to_desk: bool,
}

/// One show on the person's list.
#[derive(Clone, Debug)]
struct Entry {
    planning: bool,
    show: WatchingEntry,
}

thread_local! {
    static FIND: RefCell<Option<Find>> = const { RefCell::new(None) };
}

/// The active source's releases for the show, their facts, how many were dropped, the
/// query that found them and the source's name.
type Current = (Vec<Release>, Vec<ReleaseFacts>, usize, String, String);

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
    if lower.contains("timed out")
        || lower.contains("error sending request")
        || lower.contains("unreachable")
        || lower.contains("connect")
    {
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
    let Some(core) = crate::local::core() else {
        return;
    };
    let _ = std::thread::Builder::new()
        .name("find".into())
        .spawn(move || {
            let answer = core.call(call);
            let _ = slint::invoke_from_event_loop(move || with(|f| then(f, answer)));
        });
}

/// The app's tick, with whether the core reported a change since the last one.
pub fn tick(changed: bool) {
    with(|f| f.list_tick(changed));
}

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

pub fn install(window: &AppWindow, data_dir: PathBuf) {
    let to_desk = prefs::load(&data_dir).download_to_desk;
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
            posters: BTreeMap::new(),
            suggestions: Vec::new(),
            suggest_for: String::new(),
            answered_for: String::new(),
            answered: Vec::new(),
            entries: Vec::new(),
            list_refreshing: false,
            list_note: None,
            planned: Vec::new(),
            listed: Vec::new(),
            desk: None,
            desk_asking: false,
            desk_asked: None,
            to_desk,
        })
    });
    let d = window.global::<FindData>();
    d.set_lenses(model(
        LENSES.iter().map(|l| SharedString::from(*l)).collect(),
    ));
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
            let Some(offer) = usize::try_from(i)
                .ok()
                .and_then(|i| f.suggestions.get(i))
                .cloned()
            else {
                return;
            };
            f.clear_suggestions();
            f.open_show(offer);
        })
    });
    d.on_pick_lens(|i| {
        with(|f| {
            f.lens = usize::try_from(i).unwrap_or(0).min(LENSES.len() - 1);
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
            let tokens = f
                .list
                .as_ref()
                .and_then(|l| l.groups.get(i as usize))
                .map(|g| g.tokens.clone());
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
                f.refine(RefineOp::Exclude {
                    term: term.to_string(),
                });
            } else if !t.is_empty() {
                f.refine(RefineOp::Term { term: t });
            }
        })
    });
    d.on_undo(|| with(Find::undo));
    d.on_download_release(|i| with(|f| f.download_release(i)));
    d.on_download_all(|| with(Find::download_all));
    d.on_subscribe(|| with(Find::subscribe));
    d.on_unfollow(|files| with(|f| f.unfollow(files)));
    d.on_plan_to_watch(|| with(Find::plan));
    d.on_set_target(|i| with(|f| f.set_target(i)));
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
        // AniList is asked only here, and the core skips it when the list is fresh.
        self.load_list(true);
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
        let view = self
            .data()
            .map(|w| w.global::<FindData>().get_view())
            .unwrap_or(0);
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
                let typed = self
                    .data()
                    .map(|w| w.global::<FindData>().get_text().to_string())
                    .unwrap_or_default();
                self.suggest(typed);
            }
            _ => self.close(),
        }
    }

    /// Reads the library, the torrents and the subscriptions again, for the lenses.
    fn refresh(&mut self) {
        let Some(core) = crate::local::core() else {
            return;
        };
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
                // The show view's Following card reads the rules too.
                f.draw_show();
            }
        });
        self.show_lens();
    }

    fn offer_of(card: &SeriesCard) -> Offer {
        let folder = last_folder(&card.path);
        Offer {
            show: grid::Show {
                title: card.title.clone(),
                queries: names_of(card),
                total: card.total_episodes.unwrap_or(0),
                airing: card.next_airing.is_some()
                    || matches!(card.status, Some(anibeam_core::AiringStatus::Releasing)),
                cubes: card.cubes.clone(),
                here: Some(card.episodes_here.clone()),
                folder,
            },
            anilist: None,
            series: Some(card.id),
            in_library: true,
            poster: card.poster.clone(),
            ledger: String::new(),
            ledger_tone: 0,
        }
    }

    /// A show known only by a name and a folder: a subscription or a download not in the
    /// library yet.
    fn offer_named(&self, name: &str, folder: &str) -> Offer {
        if let Some(card) = self
            .cards
            .iter()
            .find(|c| c.path.ends_with(&format!("/{folder}")))
        {
            return Self::offer_of(card);
        }
        let title = anibeam_core::releases::titles::show_name(name);
        let title = if title.is_empty() {
            name.to_string()
        } else {
            title
        };
        Offer {
            show: grid::Show {
                title: title.clone(),
                queries: vec![title],
                total: 0,
                airing: true,
                cubes: String::new(),
                here: None,
                folder: folder.to_string(),
            },
            anilist: None,
            series: None,
            in_library: false,
            poster: None,
            ledger: String::new(),
            ledger_tone: 0,
        }
    }

    fn show_lens(&mut self) {
        let now = anibeam_companion::now();
        let offers: Vec<Offer> = match self.lens {
            0 => self.list_offers(),
            1 => self
                .cards
                .iter()
                .filter_map(|c| {
                    gap_ledger(&c.cubes).map(|ledger| Offer {
                        ledger,
                        ..Self::offer_of(c)
                    })
                })
                .collect(),
            2 => {
                let mut airing: Vec<(&SeriesCard, i64)> = self
                    .cards
                    .iter()
                    .filter_map(|c| {
                        c.next_airing
                            .as_ref()
                            .map(|a| (c, crate::phone::secs(a.at)))
                    })
                    .collect();
                airing.sort_by_key(|(_, at)| *at);
                airing
                    .into_iter()
                    .map(|(c, at)| Offer {
                        ledger: if at > now {
                            format::countdown(at - now)
                        } else {
                            "aired".into()
                        },
                        ..Self::offer_of(c)
                    })
                    .collect()
            }
            3 => self
                .rules
                .iter()
                .map(|r| {
                    let folder = last_folder(&r.save_path);
                    Offer {
                        // A rule on a source that answers JSON reads no feed (#536)
                        ledger: if !r.followable {
                            "no new episodes: its source has no feed".into()
                        } else if r.torrent_count == 1 {
                            // Short, for a tile; the show view says the rest.
                            "1 download".into()
                        } else {
                            format!("{} downloads", r.torrent_count)
                        },
                        ledger_tone: if r.followable { 0 } else { 2 },
                        ..self.offer_named(&r.name, &folder)
                    }
                })
                .collect(),
            _ => {
                let mut folders: Vec<(String, String, f64)> = Vec::new();
                for t in &self.torrents {
                    let folder = last_folder(&t.save_path);
                    match folders.iter_mut().find(|(f, _, _)| *f == folder) {
                        Some(entry) => entry.2 = entry.2.min(t.progress),
                        None => folders.push((folder, t.name.clone(), t.progress)),
                    }
                }
                folders
                    .into_iter()
                    .map(|(folder, name, progress)| Offer {
                        ledger: if progress >= 1.0 {
                            "downloaded".into()
                        } else {
                            format!("{}%", (progress * 100.0) as i64)
                        },
                        ..self.offer_named(&name, &folder)
                    })
                    .collect()
            }
        };
        let lines = [
            "Watching and Planning on your list. Open one to get its episodes.",
            "Aired episodes that are not on this phone. Open a show to see who has them.",
            "New episodes still to come, the soonest first.",
            "The subscriptions: new episodes download by themselves. Open one to stop it.",
            "What this phone downloaded, a show at a time.",
        ];
        let list_empty = self.list_note.clone().unwrap_or_else(|| {
            if self.list_refreshing {
                "Reading your list...".into()
            } else {
                "Nothing on your list yet. Search a show above and add it.".into()
            }
        });
        let empty = [
            list_empty.as_str(),
            "No gaps: every show you have started has its aired episodes here.",
            "Nothing on this phone is airing.",
            "No subscriptions yet. Open a show and turn on Follow.",
            "Nothing downloaded yet.",
        ];
        let counts = [
            self.entries.len(),
            self.cards
                .iter()
                .filter(|c| gap_ledger(&c.cubes).is_some())
                .count(),
            self.cards
                .iter()
                .filter(|c| c.next_airing.is_some())
                .count(),
            self.rules.len(),
            offers
                .len()
                .max(if self.lens == 4 { offers.len() } else { 0 }),
        ];
        let tiles: Vec<Tile> = offers
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let (poster, has_poster) = match o.poster.as_ref().and_then(|p| self.posters.get(p))
                {
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
                    ledger_tone: o.ledger_tone,
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
            .map(|(i, l)| {
                if i == 4 {
                    SharedString::from(*l)
                } else {
                    format!("{l} {}", counts[i]).into()
                }
            })
            .collect();
        let line = if offers.is_empty() {
            empty[lens]
        } else {
            lines[lens]
        };
        self.with_data(|d| {
            d.set_lens(lens as i32);
            d.set_lenses(model(names));
            d.set_lens_line(line.into());
            d.set_lens_tiles(model(tiles));
        });
        self.offers = offers;
    }

    /// The list as tiles: a show the library has opens as the library's, with its
    /// cubes; the rest open as AniList knows them, every episode still to get.
    fn list_offers(&self) -> Vec<Offer> {
        self.entries
            .iter()
            .map(|e| {
                let w = &e.show;
                let ledger = if e.planning {
                    "Planning".to_string()
                } else {
                    match w.total {
                        Some(t) => format!("Watching, {} of {t}", w.progress),
                        None => format!("Watching, {} seen", w.progress),
                    }
                };
                let card = w
                    .owned
                    .and_then(|id| self.cards.iter().find(|c| c.id == id));
                let mut offer = match card {
                    Some(c) => Self::offer_of(c),
                    None => list_offer(w),
                };
                offer.anilist = Some(w.anilist_id);
                offer.ledger = ledger;
                offer.ledger_tone = i32::from(!e.planning);
                offer
            })
            .collect()
    }

    /// Reads the list off the core's cache; with `ask`, has the core ask AniList first
    /// when its copy is older than five minutes, and reads again as that lands.
    fn load_list(&mut self, ask: bool) {
        let Some(core) = crate::local::core() else {
            return;
        };
        let _ = std::thread::Builder::new()
            .name("list".into())
            .spawn(move || {
                // The list is AniList's; the cache answers even signed out, so ask first.
                let signed_in = matches!(
                    core.call(Call::GetTrackers),
                    Ok(Reply::Trackers { state }) if state.anilist.connected
                );
                if ask && signed_in {
                    let _ = core.call(Call::RefreshWatching { force: false });
                }
                let watching = core.call(Call::ListWatching);
                let planning = core.call(Call::ListPlanning);
                let _ = slint::invoke_from_event_loop(move || {
                    with(|f| f.list_read(signed_in, watching, planning))
                });
            });
    }

    fn list_read(
        &mut self,
        signed_in: bool,
        watching: Result<Reply, CoreError>,
        planning: Result<Reply, CoreError>,
    ) {
        // A refresh is under way while the core names its job; each change reads again.
        self.list_refreshing = matches!(
            &watching,
            Ok(Reply::Watching {
                refreshing: Some(_),
                ..
            })
        );
        let mut entries = Vec::new();
        self.list_note = None;
        if !signed_in {
            self.list_note = Some(
                "Sign in to AniList under More, Accounts, and your list shows up here.".into(),
            );
            self.list_refreshing = false;
            self.entries.clear();
            self.show_lens();
            self.draw_show();
            return;
        }
        for (answer, planning) in [(watching, false), (planning, true)] {
            match answer {
                Ok(Reply::Watching { list, .. } | Reply::Planning { list }) => entries.extend(
                    list.entries
                        .into_iter()
                        .map(|show| Entry { planning, show }),
                ),
                Ok(_) => {}
                Err(e) => self.list_note = Some(format!("Your list could not be read: {e}")),
            }
        }
        // A show put on Planning here stays until the list has it, or a minute goes by:
        // the plan job writes the cache itself, so a show that never turns up stops asking.
        self.planned.retain(|(id, at)| {
            at.elapsed() < std::time::Duration::from_secs(60)
                && !entries.iter().any(|e| e.show.anilist_id == *id)
        });
        self.entries = entries;
        self.show_lens();
        self.draw_show();
    }

    /// Called on the app's tick: while AniList is being asked, or a show was just put
    /// on Planning, each change the core reports reads the cache again. Reads only; the
    /// refresh and the plan job write the cache themselves.
    fn list_tick(&mut self, changed: bool) {
        if changed && (self.list_refreshing || !self.planned.is_empty()) {
            self.load_list(false);
        }
    }

    /// Where the open show is on the list, as the show view says it, or empty.
    fn list_status(&self, offer: &Offer) -> String {
        let found = self.entries.iter().find(|e| {
            offer.anilist == Some(e.show.anilist_id)
                || (offer.series.is_some() && offer.series == e.show.owned)
        });
        let id = offer.anilist;
        match found {
            Some(e) if e.planning => "On your list: Planning".into(),
            Some(_) => "On your list: Watching".into(),
            None if id.is_some_and(|id| self.planned.iter().any(|(p, _)| *p == id)) => {
                "On your list: Planning".into()
            }
            None if id.is_some_and(|id| self.listed.contains(&id)) => "Already on your list".into(),
            None => String::new(),
        }
    }

    /// Puts the open show on Planning, on every tracker signed in.
    fn plan(&mut self) {
        let Some((id, title)) = self
            .show
            .as_ref()
            .and_then(|s| Some((s.offer.anilist?, s.offer.show.title.clone())))
        else {
            return;
        };
        ask(Call::PlanToWatch { anilist_id: id }, move |f, answer| {
            match answer {
                // A job: Planning on every tracker that lacked it.
                Ok(Reply::Started { .. }) => {
                    f.planned.push((id, std::time::Instant::now()));
                    f.line(&format!("{title} is going on Planning."), "");
                }
                // Every tracker lists it already, with its own status; nothing moves.
                Ok(_) => {
                    f.listed.push(id);
                    f.line(&format!("{title} is already on your list."), "");
                }
                Err(CoreError::NotConnected { .. }) => f.line(
                    "",
                    "Adding to your list needs AniList or MAL. Sign in under More, Accounts.",
                ),
                Err(e) => f.line("", &format!("Not added to your list: {e}")),
            }
            f.show_lens();
            f.draw_show();
        });
    }

    fn load_posters(&self, offers: &[Offer]) {
        let wanted: Vec<String> = offers
            .iter()
            .filter_map(|o| o.poster.clone())
            .filter(|p| !self.posters.contains_key(p))
            .collect();
        if wanted.is_empty() {
            return;
        }
        let _ = std::thread::Builder::new()
            .name("find-posters".into())
            .spawn(move || {
                for path in wanted {
                    let Some(decoded) = std::fs::read(&path)
                        .ok()
                        .and_then(|b| crate::posters::decode(&b))
                    else {
                        continue;
                    };
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
        let shown: Vec<&SeriesCard> = self
            .cards
            .iter()
            .filter(|c| {
                names_of(c)
                    .iter()
                    .any(|n| n.to_lowercase().contains(&needle))
            })
            .take(5)
            .collect();
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
        ask(
            Call::PhoneFindShows {
                query: text.clone(),
                limit: 8,
            },
            move |f, answer| {
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
                            let names: Vec<String> = primary_names(hit)
                                .iter()
                                .map(|n| n.to_lowercase())
                                .collect();
                            let card = f.cards.iter().find(|c| {
                                names_of(c)
                                    .iter()
                                    .any(|n| names.contains(&n.to_lowercase()))
                            });
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
                        let why = SharedString::from(format!(
                            "AniList could not be asked: {}",
                            words(&e)
                        ));
                        f.with_data(|d| {
                            d.set_suggesting(false);
                            d.set_suggest_done(true);
                            d.set_suggest_error(why);
                        });
                    }
                }
            },
        );
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
                let (poster, has_poster) = match o.poster.as_ref().and_then(|p| self.posters.get(p))
                {
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
        let Some(offer) = usize::try_from(i)
            .ok()
            .and_then(|i| self.offers.get(i))
            .cloned()
        else {
            return;
        };
        self.open_show(offer);
    }

    fn open_show(&mut self, offer: Offer) {
        // A confirm left open on another show never carries over.
        self.with_data(|d| d.set_unfollow_ask(false));
        let listed: Vec<_> = crate::local::core()
            .and_then(|c| match c.call(Call::PhoneSearchSources) {
                Ok(Reply::PhoneSources { sources }) => Some(sources),
                _ => None,
            })
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.enabled)
            .collect();
        let no_feed = listed
            .iter()
            .filter(|s| !anibeam_core::releases::endpoints::answers_rss(&s.kind))
            .map(|s| s.name.clone())
            .collect();
        let sources = listed.into_iter().map(|s| (s.name, Answer::Idle)).collect();
        let follow = offer.show.airing;
        self.show = Some(ShowState {
            offer,
            sources,
            no_feed,
            active: None,
            picks: grid::Picks::default(),
            follow,
            replace: false,
        });
        self.list = None;
        self.history.clear();
        self.set_view(1);
        self.line("", "");
        self.draw_show();
        // The first source is asked at once, as a tap on it would.
        self.fetch(0);
        self.probe_desk(false);
        self.draw_targets();
    }

    /// Asks the connected computer, off the UI thread, whether it takes downloads, and
    /// offers it in Download to once it says so. At most once a minute unless `again`.
    fn probe_desk(&mut self, again: bool) {
        let fresh = self
            .desk_asked
            .is_some_and(|at| at.elapsed() < std::time::Duration::from_secs(60));
        if self.desk_asking || (fresh && !again) {
            return;
        }
        self.desk_asking = true;
        let dir = self.data_dir.clone();
        let _ = std::thread::Builder::new()
            .name("desk".into())
            .spawn(move || {
                let desk = crate::desk::probe(&dir);
                let _ = slint::invoke_from_event_loop(move || with(|f| f.desk_answered(desk)));
            });
    }

    fn desk_answered(&mut self, desk: Option<crate::desk::Desk>) {
        let (before, _) = self.aimed();
        self.desk_asking = false;
        self.desk_asked = Some(std::time::Instant::now());
        if let Some(d) = &desk {
            let mut p = prefs::load(&self.data_dir);
            if p.desk_name != d.name {
                p.desk_name = d.name.clone();
                let _ = prefs::save(&self.data_dir, &p);
            }
        }
        self.desk = desk;
        // Picks made against what the computer had before it answered are picked again.
        let (after, _) = self.aimed();
        if (after.cubes, after.here) != (before.cubes, before.here) {
            self.repick();
        }
        self.draw_targets();
        self.draw_show();
    }

    /// Download to: this phone, and the computer once it answered or while it is the pick.
    fn draw_targets(&self) {
        let targets: Vec<SharedString> = if self.desk.is_some() || self.to_desk {
            vec!["This phone".into(), self.desk_name().into()]
        } else {
            Vec::new()
        };
        let target = i32::from(self.to_desk);
        self.with_data(|d| {
            d.set_targets(model(targets));
            d.set_target(target);
        });
    }

    fn set_target(&mut self, i: i32) {
        self.to_desk = i == 1;
        let mut p = prefs::load(&self.data_dir);
        p.download_to_desk = self.to_desk;
        let _ = prefs::save(&self.data_dir, &p);
        if self.to_desk && self.desk.is_none() {
            self.probe_desk(true);
        }
        self.repick();
        self.draw_targets();
        self.draw_show();
    }

    /// The picks were made against the other target's episodes: start again from the
    /// group that fills the most gaps there.
    fn repick(&mut self) {
        if let Some(s) = self.show.as_mut() {
            s.picks = grid::Picks::default();
        }
        self.auto_pick();
    }

    fn fetch(&mut self, i: i32) {
        let Some(state) = self.show.as_mut() else {
            return;
        };
        let Some(i) = usize::try_from(i).ok().filter(|i| *i < state.sources.len()) else {
            return;
        };
        state.active = Some(i);
        state.picks = grid::Picks::default();
        state.replace = false;
        if matches!(
            state.sources[i].1,
            Answer::Done(_) | Answer::Searching { .. }
        ) {
            self.auto_pick();
            self.draw_show();
            return;
        }
        self.search_as(i, 0);
    }

    /// One source's search by the show's `at`th name; an empty answer moves on to the next.
    fn search_as(&mut self, i: usize, at: usize) {
        let Some(state) = self.show.as_mut() else {
            return;
        };
        let Some(query) = state.offer.show.queries.get(at).cloned() else {
            return;
        };
        let name = state.sources[i].0.clone();
        state.sources[i].1 = Answer::Searching { at };
        let title = state.offer.show.title.clone();
        self.draw_show();
        ask(
            Call::PhoneSearch {
                query,
                endpoint: Some(name),
                fallback: false,
            },
            move |f, answer| {
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
                    let next = f.show.as_ref().and_then(|s| {
                        s.sources
                            .iter()
                            .skip(i + 1)
                            .position(|(_, a)| matches!(a, Answer::Idle))
                            .map(|p| i + 1 + p)
                    });
                    if nothing && let Some(next) = next {
                        f.fetch(next as i32);
                        return;
                    }
                    f.auto_pick();
                }
                f.draw_show();
            },
        );
    }

    /// The active source's releases that name the show, and the grid of them.
    fn current(&self) -> Option<Current> {
        let state = self.show.as_ref()?;
        let i = state.active?;
        let Answer::Done(search) = &state.sources[i].1 else {
            return None;
        };
        let (items, facts, dropped) =
            grid::of_show(&state.offer.show, &search.items, &search.facts);
        Some((
            items,
            facts,
            dropped,
            search.query.clone(),
            state.sources[i].0.clone(),
        ))
    }

    /// The computer downloads go to, when it is the pick and it answered.
    fn on_desk(&self) -> Option<&crate::desk::Desk> {
        self.desk.as_ref().filter(|_| self.to_desk)
    }

    /// Why Get cannot go yet: the computer is the pick but has not answered. Get never
    /// falls back to the phone on its own, since that would spend the phone's storage
    /// and maybe its mobile data on a download meant for the computer.
    fn desk_waiting(&self) -> Option<String> {
        if !self.to_desk || self.desk.is_some() {
            return None;
        }
        let name = self.desk_name();
        Some(if self.desk_asking {
            format!("Asking {name}...")
        } else {
            format!("{name} does not answer. Pick This phone, or try again when it is on.")
        })
    }

    /// The computer's name as it last answered.
    fn desk_name(&self) -> String {
        self.desk
            .as_ref()
            .map(|d| d.name.clone())
            .or_else(|| Some(prefs::load(&self.data_dir).desk_name))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Your computer".into())
    }

    /// The computer's own series and options for the open show.
    fn desk_options(&self, desk: &crate::desk::Desk, provider: String) -> DownloadOptions {
        DownloadOptions {
            provider,
            series: self
                .show
                .as_ref()
                .and_then(|s| desk.series_for(s.offer.anilist, &s.offer.show.queries)),
            ..Default::default()
        }
    }

    /// The open show as the picked target sees it. On this phone: what the phone has and
    /// is getting. On the computer: what its last snapshot says it has a file for.
    fn aimed(&self) -> (grid::Show, BTreeMap<u32, f64>) {
        let Some(state) = self.show.as_ref() else {
            return Default::default();
        };
        if !self.to_desk {
            return (state.offer.show.clone(), self.flight());
        }
        let had = self
            .desk
            .as_ref()
            .map(|d| d.had(state.offer.anilist, &state.offer.show.queries))
            .unwrap_or_default();
        (desk_show(&state.offer.show, had), BTreeMap::new())
    }

    fn flight(&self) -> BTreeMap<u32, f64> {
        let Some(state) = self.show.as_ref() else {
            return BTreeMap::new();
        };
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
        let Some((items, facts, _, _, provider)) = self.current() else {
            return;
        };
        let (aimed, flight) = self.aimed();
        let Some(state) = self.show.as_mut() else {
            return;
        };
        if !state.picks.chosen.is_empty() {
            return;
        }
        let total = grid::total(&aimed, &facts, &flight);
        let library = grid::library(&aimed, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        if let Some(best) = grid::best_row(&rows, &library, total) {
            state.picks = grid::pick_row(&rows[best], &library, total, false);
        }
    }

    fn pick_row(&mut self, r: i32) {
        let Some((items, facts, _, _, provider)) = self.current() else {
            return;
        };
        let (aimed, flight) = self.aimed();
        let Some(state) = self.show.as_mut() else {
            return;
        };
        let total = grid::total(&aimed, &facts, &flight);
        let library = grid::library(&aimed, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        if let Some(row) = usize::try_from(r).ok().and_then(|r| rows.get(r)) {
            state.picks = grid::pick_row(row, &library, total, true);
        }
        self.draw_show();
    }

    fn toggle_cube(&mut self, r: i32, n: i32) {
        let Some((items, facts, _, _, provider)) = self.current() else {
            return;
        };
        let (aimed, flight) = self.aimed();
        let Some(state) = self.show.as_mut() else {
            return;
        };
        let total = grid::total(&aimed, &facts, &flight);
        let library = grid::library(&aimed, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        if let (Some(row), Ok(n)) = (
            usize::try_from(r).ok().and_then(|r| rows.get(r)),
            u32::try_from(n + 1),
        ) {
            grid::toggle_cube(&mut state.picks, row, n, &library, total);
        }
        self.draw_show();
    }

    fn library_code(c: char) -> i32 {
        crate::phone::cube_codes(&c.to_string())[0]
    }

    fn draw_show(&self) {
        let Some(state) = self.show.as_ref() else {
            return;
        };
        let show = &state.offer.show;
        let chips: Vec<SourceChip> = state
            .sources
            .iter()
            .map(|(name, answer)| {
                let (st, line) = match answer {
                    Answer::Idle => ("idle", "Tap to ask".to_string()),
                    Answer::Searching { at } => (
                        "searching",
                        if *at > 0 {
                            format!(
                                "Trying {}",
                                show.queries.get(*at).cloned().unwrap_or_default()
                            )
                        } else {
                            "Searching".into()
                        },
                    ),
                    Answer::Done(s) => (
                        "done",
                        if s.items.len() == 1 {
                            "1 release".into()
                        } else {
                            format!("{} releases", s.items.len())
                        },
                    ),
                    Answer::Failed(_) => ("error", "Not answering".into()),
                };
                SourceChip {
                    name: name.clone().into(),
                    state: st.into(),
                    line: line.into(),
                }
            })
            .collect();
        let (poster, has_poster) = match state
            .offer
            .poster
            .as_ref()
            .and_then(|p| self.posters.get(p))
        {
            Some(img) => (img.clone(), true),
            None => (Image::default(), false),
        };
        let mut facts_line = Vec::new();
        if show.total > 0 {
            facts_line.push(if show.total == 1 {
                "1 episode".to_string()
            } else {
                format!("{} episodes", show.total)
            });
        }
        if show.airing {
            facts_line.push("Airing".into());
        }
        let on_phone = match &show.here {
            Some(here) => here.len(),
            None => show
                .cubes
                .chars()
                .filter(|c| matches!(c, 'd' | 'l' | 'w'))
                .count(),
        };
        facts_line.push(if on_phone == 1 {
            "1 on this phone".into()
        } else {
            format!("{on_phone} on this phone")
        });

        let mut library_codes = Vec::new();
        let mut grid_rows = Vec::new();
        let mut dropped_line = String::new();
        let (mut plan_line, mut plan_detail, mut go_text, mut can_go, mut can_follow, mut has_had) = (
            String::new(),
            String::new(),
            "Get".to_string(),
            false,
            false,
            false,
        );
        let mut error = String::new();
        if let Some(Answer::Failed(e)) = state.active.map(|i| &state.sources[i].1) {
            error = e.clone();
        }
        if let Some((items, facts, dropped, query, provider)) = self.current() {
            let (aimed, flight) = self.aimed();
            let total = grid::total(&aimed, &facts, &flight);
            let library = grid::library(&aimed, total, &flight);
            let rows = grid::rows(&items, &facts, total, &provider);
            library_codes = library.iter().map(|c| Self::library_code(*c)).collect();
            if dropped > 0 {
                dropped_line = if dropped == 1 {
                    "1 release that does not name this show is left out.".to_string()
                } else {
                    format!("{dropped} releases that do not name this show are left out.")
                };
            }
            grid_rows = rows
                .iter()
                .map(|row| {
                    // A batch says so in its tag, with the episodes it covers, rather than in
                    // its name.
                    let label = format!("{} {}", row.group, row.resolution);
                    let batch = if row.batch {
                        format!("Batch {}", row.label).trim().to_string()
                    } else {
                        String::new()
                    };
                    let meta = format!(
                        "{}  {} seeds  {}",
                        if row.count == 1 {
                            "1 episode".to_string()
                        } else {
                            format!("{} episodes", row.count)
                        },
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
                    GridRow {
                        label: label.trim().into(),
                        batch: batch.into(),
                        meta: meta.into(),
                        cubes: model(cubes),
                        chosen: row.key == state.picks.chosen,
                    }
                })
                .collect();
            let plan = grid::plan(
                &aimed,
                &query,
                &items,
                &facts,
                &rows,
                &state.picks,
                &library,
                state.replace,
                &provider,
            );
            has_had = state.picks.map.keys().any(|n| grid::have(&library, *n));
            can_follow =
                !plan.follow_query.is_empty() && show.airing && !state.no_feed.contains(&provider);
            let following = state.follow && can_follow;
            let n = plan.episodes.len();
            can_go = n > 0 || following;
            go_text = match (n, following) {
                (0, true) => "Subscribe".into(),
                (0, false) => "Pick episodes to get".into(),
                (n, f) => format!(
                    "Get {}{}{}",
                    if n == 1 {
                        "1 episode".to_string()
                    } else {
                        format!("{n} episodes")
                    },
                    if state.replace && has_had {
                        " and replace"
                    } else {
                        ""
                    },
                    if f { " and subscribe" } else { "" }
                ),
            };
            plan_line = if n == 0 {
                if rows.is_empty() {
                    "This source has no releases of this show.".into()
                } else {
                    "Nothing picked: tap a row or its cubes.".into()
                }
            } else {
                format!(
                    "{} from {}, {}",
                    grid::episodes_phrase(&plan.episodes),
                    plan.groups.join(" and "),
                    format::bytes(plan.bytes)
                )
            };
            let skipped = state
                .picks
                .map
                .keys()
                .filter(|n| grid::have(&library, **n))
                .count();
            if skipped > 0 && !state.replace {
                plan_detail = format!(
                    "{} you already have {} left out. Turn on Replace to get {} again.",
                    if skipped == 1 {
                        "1 episode"
                    } else {
                        "Episodes"
                    },
                    if skipped == 1 { "is" } else { "are" },
                    if skipped == 1 { "it" } else { "them" }
                );
            }
            if following {
                let more = format!(
                    "New episodes from {} will download by themselves.",
                    plan.follow_query
                );
                plan_detail = if plan_detail.is_empty() {
                    more
                } else {
                    format!("{plan_detail} {more}")
                };
            }
        }
        let desk = self.on_desk().map(|d| d.name.clone());
        if let Some(why) = self.desk_waiting() {
            can_go = false;
            plan_detail = why;
        } else if let Some(name) = &desk
            && can_go
        {
            let there = format!("They download on {name}, into its library.");
            plan_detail = if plan_detail.is_empty() {
                there
            } else {
                format!("{plan_detail} {there}")
            };
        }
        let library_label = if self.to_desk {
            format!("On {}", self.desk_name())
        } else {
            "On this phone".to_string()
        };
        let list_status = self.list_status(&state.offer);
        let can_plan = state.offer.anilist.is_some() && list_status.is_empty();
        let active = state.active.map(|i| i as i32).unwrap_or(-1);
        let follow = state.follow;
        let replace = state.replace;
        let rules = rules_for(&self.rules, &show.folder);
        let rule_name = rules.first().map(|r| r.name.clone()).unwrap_or_default();
        let rule_said = rules.first().map(|r| rule_line(r)).unwrap_or_default();
        self.with_data(|d| {
            if rule_name.is_empty() {
                d.set_unfollow_ask(false);
            }
            d.set_rule_name(rule_name.into());
            d.set_list_status(list_status.into());
            d.set_can_plan(can_plan);
            d.set_rule_line(rule_said.into());
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
            d.set_library_label(library_label.into());
            d.set_error(error.into());
        });
    }

    fn limit(&self) -> Option<u64> {
        let p = prefs::load(&self.data_dir);
        p.limit_on.then_some((p.limit_gb * 1e9) as u64)
    }

    /// Get, and Follow: the picked releases download into the show's folder, then the
    /// subscription is made when Follow is on.
    fn go(&mut self) {
        let Some((items, facts, _, query, provider)) = self.current() else {
            return;
        };
        let (aimed, flight) = self.aimed();
        let Some(state) = self.show.as_ref() else {
            return;
        };
        let total = grid::total(&aimed, &facts, &flight);
        let library = grid::library(&aimed, total, &flight);
        let rows = grid::rows(&items, &facts, total, &provider);
        let plan = grid::plan(
            &aimed,
            &query,
            &items,
            &facts,
            &rows,
            &state.picks,
            &library,
            state.replace,
            &provider,
        );
        let follow = state.follow
            && !plan.follow_query.is_empty()
            && state.offer.show.airing
            && !state.no_feed.contains(&provider);
        if let Some(why) = self.desk_waiting() {
            self.line("", &why);
            return;
        }
        let what = count_word(plan.episodes.len(), "episode", "episodes");
        let name = Some(plan.name.clone());
        let download = !plan.items.is_empty();
        let calls = match self.on_desk().cloned() {
            Some(desk) => {
                let options = self.desk_options(&desk, provider.clone());
                let series = options.series;
                Sent {
                    download: download.then(|| Call::DownloadReleasesWith {
                        items: plan.items,
                        name: name.clone(),
                        options,
                    }),
                    subscribe: follow.then_some(Call::Subscribe {
                        query: plan.follow_query,
                        endpoint: provider,
                        items: plan.follow_items,
                        name,
                        wire_query: None,
                        series,
                    }),
                    what,
                    to: To::Desk(desk),
                }
            }
            None => Sent {
                download: download.then(|| Call::PhoneDownload {
                    items: plan.items,
                    name: name.clone(),
                    limit_bytes: self.limit(),
                }),
                subscribe: follow.then_some(Call::PhoneSubscribe {
                    query: plan.follow_query,
                    endpoint: provider,
                    items: plan.follow_items,
                    name,
                }),
                what,
                to: To::Phone,
            },
        };
        self.send(calls);
    }

    // ---- The release list

    fn list_all(&mut self) {
        let Some(state) = self.show.as_ref() else {
            return;
        };
        let Some(Answer::Done(search)) = state.active.map(|i| &state.sources[i].1) else {
            return;
        };
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
        self.probe_desk(false);
        self.draw_targets();
        self.with_data(|d| d.set_busy(true));
        self.line(&format!("Searching for {text}..."), "");
        ask(
            Call::PhoneSearch {
                query: text,
                endpoint: None,
                fallback: true,
            },
            |f, answer| {
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
            },
        );
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
        let Some(list) = self.list.clone() else {
            return;
        };
        self.with_data(|d| d.set_busy(true));
        ask(
            Call::PhoneRefine {
                query: list.query.clone(),
                endpoint: list.endpoint.clone(),
                items: list.items.clone(),
                op,
            },
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
        let Some(list) = self.list.as_ref() else {
            return;
        };
        let best_link = list.best_fit.item.as_ref().map(|i| i.link.clone());
        // On the computer, on disk means a file there, by its last snapshot.
        let desk_had = self.to_desk.then(|| {
            let (show, _) = self.aimed();
            show.here.unwrap_or_default()
        });
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
                    on_disk: match &desk_had {
                        Some(had) => f.is_some_and(|f| {
                            !f.batch && f.episode.is_some_and(|e| had.contains(&(e as u32)))
                        }),
                        None => f.is_some_and(|f| f.on_disk),
                    },
                    tone: if r.seeders == Some(0) { 2 } else { 0 },
                }
            })
            .collect();
        let groups: Vec<GroupChip> = list
            .groups
            .iter()
            .map(|g| GroupChip {
                label: g.label.clone().into(),
                count: g.member_count as i32,
            })
            .collect();
        let best = list.best_fit.available;
        let best_line = if best {
            format!("Best fit searches: {}", list.best_fit.query)
        } else {
            String::new()
        };
        let can_undo = !self.history.is_empty();
        crate::inspect::track(
            list.items
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let batch = list
                        .facts
                        .get(i)
                        .is_some_and(|f| f.batch && f.first_episode.is_none());
                    (r.link.clone(), batch)
                })
                .collect(),
        );
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
        let count = items.len();
        let name = self
            .show
            .as_ref()
            .map(|s| s.offer.show.folder.clone())
            .filter(|f| !f.is_empty());
        if let Some(why) = self.desk_waiting() {
            self.line("", &why);
            return;
        }
        if let Some(desk) = self.on_desk().cloned() {
            let provider = self
                .list
                .as_ref()
                .map(|l| l.endpoint.clone())
                .unwrap_or_default();
            let options = self.desk_options(&desk, provider);
            self.send(Sent {
                download: Some(Call::DownloadReleasesWith {
                    items,
                    name,
                    options,
                }),
                subscribe: None,
                what: count_word(count, "release", "releases"),
                to: To::Desk(desk),
            });
            return;
        }
        ask(
            Call::PhoneDownload {
                items,
                name,
                limit_bytes: self.limit(),
            },
            move |f, answer| match answer {
                Ok(_) => {
                    f.line(
                        &if count == 1 {
                            "Downloading 1 release.".into()
                        } else {
                            format!("Downloading {count} releases.")
                        },
                        "",
                    );
                    crate::app::download_started();
                }
                Err(e) => f.line("", &format!("Not added: {}", words(&e))),
            },
        );
    }

    fn download_release(&mut self, i: i32) {
        let item = self
            .list
            .as_ref()
            .and_then(|l| usize::try_from(i).ok().and_then(|i| l.items.get(i)))
            .cloned();
        if let Some(item) = item {
            self.download(vec![item]);
        }
    }

    fn download_all(&mut self) {
        let items = self
            .list
            .as_ref()
            .map(|l| l.items.clone())
            .unwrap_or_default();
        self.download(items);
    }

    fn subscribe(&mut self) {
        let Some(list) = self.list.clone() else {
            return;
        };
        let name = self
            .show
            .as_ref()
            .map(|s| s.offer.show.folder.clone())
            .filter(|f| !f.is_empty());
        if let Some(why) = self.desk_waiting() {
            self.line("", &why);
            return;
        }
        if let Some(desk) = self.on_desk().cloned() {
            let series = self.desk_options(&desk, String::new()).series;
            self.send(Sent {
                download: None,
                subscribe: Some(Call::Subscribe {
                    query: list.query,
                    endpoint: list.endpoint,
                    items: list.items,
                    name,
                    wire_query: None,
                    series,
                }),
                what: String::new(),
                to: To::Desk(desk),
            });
            return;
        }
        ask(
            Call::PhoneSubscribe {
                query: list.query,
                endpoint: list.endpoint,
                items: list.items,
                name,
            },
            |f, answer| match answer {
                Ok(Reply::PhoneSubscribed { subscribed }) => {
                    f.line(
                        &format!(
                            "Subscribed as {}: new episodes download by themselves.",
                            subscribed.name
                        ),
                        "",
                    );
                    f.refresh();
                    crate::app::download_started();
                }
                Ok(_) => {}
                Err(e) => f.line("", &format!("Not subscribed: {}", words(&e))),
            },
        );
    }
}

impl Find {
    /// Stops following the open show: every subscription that fills its folder goes, and
    /// with `files` what downloaded into that folder too, episodes included. The desktop's
    /// Unsubscribe, after the same confirm.
    fn unfollow(&mut self, files: bool) {
        let busy = self
            .data()
            .map(|w| w.global::<FindData>().get_busy())
            .unwrap_or(false);
        if busy {
            return;
        }
        let names: Vec<String> = self
            .show
            .as_ref()
            .map(|s| rules_for(&self.rules, &s.offer.show.folder))
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.name.clone())
            .collect();
        let Some(name) = names.first().cloned() else {
            return;
        };
        self.with_data(|d| {
            d.set_unfollow_ask(false);
            d.set_busy(true);
        });
        ask(
            Call::PhoneUnsubscribe {
                names: names.clone(),
                files,
            },
            move |f, answer| {
                f.with_data(|d| d.set_busy(false));
                match answer {
                    Ok(_) => {
                        f.rules.retain(|r| !names.contains(&r.name));
                        f.line(&unfollowed_line(&name, files), "");
                    }
                    // The core takes the subscription out before it deletes, so a failure
                    // can leave it gone with its downloads still there: the rules are read
                    // again and the page says what is left.
                    Err(e) => f.line("", &format!("Stopping did not finish: {}", words(&e))),
                }
                f.draw_show();
                f.refresh();
            },
        );
    }
}

/// Where Get sends its calls.
enum To {
    Phone,
    Desk(crate::desk::Desk),
}

/// What Get sends: the download, the subscription once the download went in, and what
/// it covers in words ("3 episodes").
struct Sent {
    download: Option<Call>,
    subscribe: Option<Call>,
    what: String,
    to: To,
}

/// "1 episode", "3 episodes".
fn count_word(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// What the Gaps lens says about a show, the desktop's rule (`hasGap`): a hole before an
/// episode that is here is missing, and an aired one past the last is not downloaded, but
/// only once the show has something on this phone. A show with nothing here has no gap.
fn gap_ledger(cubes: &str) -> Option<String> {
    let count = |c: char| cubes.chars().filter(|x| *x == c).count();
    let (missing, not_got) = (count('m'), count('g'));
    // On disk, or watched here; 'w' is watched somewhere else, nothing on this phone.
    let has_some = cubes.chars().any(|x| matches!(x, 'd' | 'l'));
    let not_got = if has_some { not_got } else { 0 };
    let parts: Vec<String> = [(missing, "missing"), (not_got, "not downloaded")]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, word)| format!("{n} {word}"))
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// The show as the computer has it: an episode with a file there is had, a hole before
/// the last one it has is missing, an episode not aired yet stays so.
fn desk_show(show: &grid::Show, had: Vec<u32>) -> grid::Show {
    let codes: Vec<char> = show.cubes.chars().collect();
    let last = had.iter().copied().max().unwrap_or(0);
    let len = (codes.len() as u32).max(last);
    let cubes = (1..=len)
        .map(|n| {
            if had.contains(&n) {
                'd'
            } else if codes.get(n as usize - 1) == Some(&'f') {
                'f'
            } else if n < last {
                'm'
            } else {
                'g'
            }
        })
        .collect();
    grid::Show {
        cubes,
        here: Some(had),
        ..show.clone()
    }
}

impl Find {
    /// Sends Get off the UI thread, to this phone's core or to the computer. Only the
    /// phone's own downloads are held for Wi-Fi, capped and announced in a notification;
    /// nothing sent to the computer lands here.
    fn send(&mut self, sent: Sent) {
        let core = crate::local::core();
        if matches!(sent.to, To::Phone) && core.is_none() {
            return;
        }
        self.with_data(|d| d.set_busy(true));
        self.line(
            &match &sent.to {
                To::Phone => "Working...".to_string(),
                To::Desk(d) => format!("Sending to {}...", d.name),
            },
            "",
        );
        let _ = std::thread::Builder::new()
            .name("get".into())
            .spawn(move || {
                let call = |c: &Call| match (&sent.to, &core) {
                    (To::Desk(d), _) => d.call(c),
                    (To::Phone, Some(core)) => core.call(c.clone()),
                    (To::Phone, None) => Err(CoreError::internal("the app is still starting")),
                };
                let mut line = Vec::new();
                let mut error = String::new();
                if let Some(c) = &sent.download {
                    match (call(c), &sent.to) {
                        (Ok(_), To::Phone) => line.push(format!("Getting {}.", sent.what)),
                        (Ok(r), To::Desk(d)) => {
                            line.push(crate::desk::sent_line(&r, &sent.what, &d.name))
                        }
                        (Err(e), To::Phone) => error = words(&e),
                        (Err(e), To::Desk(d)) => {
                            error = format!("{} did not take it: {}", d.name, words(&e))
                        }
                    }
                }
                if error.is_empty()
                    && let Some(c) = &sent.subscribe
                {
                    match (call(c), &sent.to) {
                        (Ok(_), To::Phone) => {
                            line.push("Subscribed: new episodes download by themselves.".into())
                        }
                        (Ok(_), To::Desk(d)) => line.push(format!(
                            "Subscribed on {}: new episodes download there by themselves.",
                            d.name
                        )),
                        (Err(e), To::Phone) => error = words(&e),
                        (Err(e), To::Desk(d)) => {
                            error = format!("Not subscribed on {}: {}", d.name, words(&e))
                        }
                    }
                }
                let here = matches!(sent.to, To::Phone);
                let _ = slint::invoke_from_event_loop(move || {
                    with(|f| {
                        f.with_data(|d| d.set_busy(false));
                        f.line(&line.join(" "), &error);
                        // Something was taken here: the first time, the notification is
                        // explained before Android asks for it.
                        if here && !line.is_empty() {
                            crate::app::download_started();
                        }
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
}

/// A folder's own name, the last part of `path`.
fn last_folder(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// A folder name as matching reads it: lower case letters and digits only, so the same show
/// matches whatever case or punctuation its folder has on either side.
fn plain_folder(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Every subscription that downloads into a show's folder, matched by the folder's name the
/// way the Following lens names its shows.
fn rules_for<'a>(rules: &'a [SubscriptionRule], folder: &str) -> Vec<&'a SubscriptionRule> {
    let want = plain_folder(folder);
    if want.is_empty() {
        return Vec::new();
    }
    rules
        .iter()
        .filter(|r| plain_folder(&last_folder(&r.save_path)) == want)
        .collect()
}

/// What is in a subscription's folder: "3 downloads, 6 files in its folder". The core counts
/// the folder, so downloads got by hand into it count too.
fn taken(r: &SubscriptionRule) -> String {
    let count = |n: u64, one: &str, many: &str| {
        if n == 1 {
            format!("1 {one}")
        } else {
            format!("{n} {many}")
        }
    };
    format!(
        "{}, {} in its folder",
        count(r.torrent_count, "download", "downloads"),
        count(r.file_count, "file", "files")
    )
}

/// The show view's line about its subscription: whether new episodes still come, and what
/// its folder holds.
fn rule_line(r: &SubscriptionRule) -> String {
    let state = if !r.rule_enabled {
        "It is paused, so no new episodes come"
    } else if !r.followable {
        "Its source has no feed, so no new episodes come"
    } else {
        "New episodes download by themselves"
    };
    format!("{state}. {}.", taken(r))
}

/// What the page says once a show is no longer followed.
fn unfollowed_line(name: &str, files: bool) -> String {
    if files {
        format!("Stopped following {name} and deleted the downloads in its folder.")
    } else {
        format!("Stopped following {name}. Its episodes stay on this phone.")
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
    let here = c.episodes_here.len();
    // Files numbered past the record's count (absolute numbering) still count as here.
    let total = c.cubes.chars().filter(|k| *k != 'f').count().max(here);
    match (here, total) {
        (_, 0) => "In your library".to_string(),
        (0, t) => format!("In your library, none of {t} on this phone"),
        (h, t) => format!("In your library, {h} of {t} on this phone"),
    }
}

/// The names an AniList hit is known by first: romaji, English, its title.
fn primary_names(h: &SearchResult) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for n in [h.romaji.as_ref(), h.english.as_ref(), Some(&h.title)]
        .into_iter()
        .flatten()
    {
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
    let latin = |n: &str| {
        n.chars()
            .all(|c| c.is_ascii() || (c.is_alphabetic() && (c as u32) < 0x0250))
    };
    for n in h
        .synonyms
        .iter()
        .filter(|n| n.chars().count() >= 5 && latin(n))
        .take(2)
    {
        if !names.contains(n) {
            names.push(n.clone());
        }
    }
    names
}

/// A title as a folder name: a slash or backslash would make it a path, so it becomes a
/// space (Fate/Zero goes into "Fate Zero"), and control characters go.
fn folder_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c == '/' || c == '\\' { ' ' } else { c })
        .filter(|c| !c.is_control())
        .collect();
    let folded = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    match folded.as_str() {
        "" | "." | ".." => "Show".to_string(),
        _ => folded,
    }
}

/// A list entry the library does not have, as AniList knows it.
fn list_offer(w: &WatchingEntry) -> Offer {
    anilist_offer(
        w.anilist_id,
        &w.title,
        [w.romaji.as_ref(), w.english.as_ref()],
        w.total,
        w.next_airing.is_some(),
        w.poster.clone(),
    )
}

/// A show known only from AniList, every episode still to get. Searched by its latin
/// titles first, whatever the page shows.
fn anilist_offer(
    id: u64,
    title: &str,
    latin: [Option<&String>; 2],
    total: Option<u32>,
    airing: bool,
    poster: Option<String>,
) -> Offer {
    let mut queries: Vec<String> = Vec::new();
    for name in latin.into_iter().flatten().chain([&title.to_string()]) {
        if !name.is_empty() && !queries.contains(name) {
            queries.push(name.clone());
        }
    }
    let total = total.unwrap_or(0);
    // Saved under the romaji name, else the English, as a search hit is.
    let title = latin
        .into_iter()
        .flatten()
        .next()
        .cloned()
        .unwrap_or_else(|| title.to_string());
    Offer {
        show: fresh_show(title, queries, total, airing),
        anilist: Some(id),
        series: None,
        in_library: false,
        poster,
        ledger: String::new(),
        ledger_tone: 0,
    }
}

/// A show the library does not have: every episode still to get, in a folder of its own.
fn fresh_show(title: String, queries: Vec<String>, total: u32, airing: bool) -> grid::Show {
    grid::Show {
        folder: folder_name(&title),
        title,
        queries,
        total,
        airing,
        cubes: "g".repeat(total.min(400) as usize),
        here: None,
    }
}

/// A search hit's names, the same search that typing the title runs, for a show Airing
/// now only knows from its Out now strip.
pub fn search_for(title: &str) {
    let title = title.to_string();
    with(|f| {
        f.open(String::new());
        f.with_data(|d| d.set_text(title.clone().into()));
        f.suggest(title);
    });
}

/// Get episodes for a show on Airing now that the library does not have: its show view,
/// asking the first source at once, as a search hit opens.
pub fn open_airing(a: &anibeam_core::AiringShow) {
    let offer = anilist_offer(
        a.anilist_id,
        &a.title,
        [a.romaji.as_ref(), a.english.as_ref()],
        a.episodes,
        a.next_airing.is_some(),
        a.poster.clone(),
    );
    with(|f| {
        f.with_data(|d| d.set_shown(true));
        f.refresh();
        // Opened from Airing now, Downloads may not have read the list yet.
        if f.entries.is_empty() {
            f.load_list(false);
        }
        f.with_data(|d| d.set_text(SharedString::new()));
        f.clear_suggestions();
        f.open_show(offer);
    });
}

fn offer_of_hit(h: &SearchResult, poster: String) -> Offer {
    let queries = hit_names(h);
    let title = h
        .romaji
        .clone()
        .or_else(|| h.english.clone())
        .unwrap_or_else(|| h.title.clone());
    let total = h.episodes.unwrap_or(0);
    let mut facts = Vec::new();
    if let Some(f) = &h.format {
        facts.push(f.replace('_', " "));
    }
    if let Some(y) = h.year {
        facts.push(y.to_string());
    }
    if total > 0 {
        facts.push(if total == 1 {
            "1 episode".to_string()
        } else {
            format!("{total} episodes")
        });
    }
    Offer {
        show: fresh_show(title, queries, total, h.airing),
        anilist: matches!(h.provider, anibeam_core::Provider::Anilist).then_some(h.id),
        series: None,
        in_library: false,
        poster: (!poster.is_empty()).then_some(poster),
        ledger: facts.join(", "),
        ledger_tone: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_show_with_nothing_here_has_no_gap() {
        assert_eq!(gap_ledger("gggg"), None);
        assert_eq!(gap_ledger("ggff"), None);
        assert_eq!(gap_ledger("ddgg").as_deref(), Some("2 not downloaded"));
        assert_eq!(
            gap_ledger("dmdg").as_deref(),
            Some("1 missing, 1 not downloaded")
        );
        assert_eq!(gap_ledger("wwdd"), None);
        assert_eq!(gap_ledger("wwwwwggggggg"), None);
        assert_eq!(gap_ledger("llgg").as_deref(), Some("2 not downloaded"));
    }

    fn rule(name: &str, save_path: &str, torrents: u64, files: u64) -> SubscriptionRule {
        serde_json::from_value(serde_json::json!({
            "name": name, "feed_url": "https://feed.test/rss?q=x", "save_path": save_path,
            "torrent_count": torrents, "file_count": files,
        }))
        .unwrap()
    }

    fn names(rules: Vec<&SubscriptionRule>) -> Vec<&str> {
        rules.iter().map(|r| r.name.as_str()).collect()
    }

    #[test]
    fn a_show_finds_every_subscription_that_fills_its_folder() {
        let rules = [
            rule("Frieren", "/data/files/Anime/Sousou no Frieren", 3, 3),
            rule("Dandadan", "/data/files/Anime/Dandadan", 1, 1),
            rule("Frieren 1080p", "/data/files/Anime/Sousou no Frieren", 3, 3),
        ];
        assert_eq!(
            names(rules_for(&rules, "Sousou no Frieren")),
            ["Frieren", "Frieren 1080p"]
        );
        assert_eq!(
            names(rules_for(&rules, "dandadan!")),
            ["Dandadan"],
            "case and punctuation"
        );
        assert!(
            rules_for(&rules, "Sousou").is_empty(),
            "the whole name, not a part"
        );
        assert!(
            rules_for(&rules, "").is_empty(),
            "a show with no folder has none"
        );
    }

    #[test]
    fn a_subscription_says_what_its_folder_holds() {
        assert_eq!(
            taken(&rule("A", "/a", 3, 4)),
            "3 downloads, 4 files in its folder"
        );
        assert_eq!(
            taken(&rule("A", "/a", 1, 1)),
            "1 download, 1 file in its folder"
        );
        let mut stuck = rule("A", "/a", 0, 0);
        stuck.followable = false;
        assert!(rule_line(&stuck).starts_with("Its source has no feed"));
        let mut paused = rule("A", "/a", 0, 0);
        paused.rule_enabled = false;
        assert!(rule_line(&paused).starts_with("It is paused"));
        assert!(rule_line(&rule("A", "/a", 0, 0)).starts_with("New episodes download"));
    }

    #[test]
    fn stopping_says_what_happened_to_the_episodes() {
        assert!(unfollowed_line("Frieren", false).contains("stay on this phone"));
        assert!(unfollowed_line("Frieren", true).contains("deleted the downloads in its folder"));
    }
}
