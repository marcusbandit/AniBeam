//! The UI thread: the window, what it shows, and the clock that keeps it current.
//!
//! Two sources feed the pages. This phone's own core is the default for everything; another
//! AniBeam on the tailnet, synced by the worker thread, only fills Library when it is picked
//! there, and its pages are read only.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use anibeam_companion::address::HostAddress;
use anibeam_companion::format;
use anibeam_companion::store::{AniListCache, Cache, Store};
use anibeam_core::{Call, Direction, Reply, SeriesCard, SeriesDetail, Sort, Tab, Tracker, TrackerAccount};
use chrono::Datelike;
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};

use crate::prefs::{self, Prefs};
use crate::remote::Art;
use crate::worker::{self, Command, Commands, Event};
use crate::{AppData, AppWindow, Day, EpisodeLine, Shelf, Tile};

/// How often the app looks at the other device while it is open.
const SYNC_EVERY: Duration = Duration::from_secs(5 * 60);
/// How often the storage lines walk the library folder.
const STORAGE_EVERY: u32 = 30;

/// Puts a model's rows in place, touching only the rows that changed, so a once-a-second
/// countdown repaints the text that moved and not the whole grid.
fn update<T: Clone + PartialEq + 'static>(model: &VecModel<T>, rows: Vec<T>) {
    if model.row_count() != rows.len() {
        model.set_vec(rows);
        return;
    }
    for (i, row) in rows.into_iter().enumerate() {
        if model.row_data(i).as_ref() != Some(&row) {
            model.set_row_data(i, row);
        }
    }
}

fn list<T: Clone + 'static>() -> Rc<VecModel<T>> {
    Rc::new(VecModel::default())
}

/// Shelves whose tiles keep one model per shelf title for the life of the app.
struct Shelves {
    shelves: Rc<VecModel<Shelf>>,
    tiles: HashMap<&'static str, Rc<VecModel<Tile>>>,
}

impl Shelves {
    fn new() -> Self {
        Shelves { shelves: list(), tiles: HashMap::new() }
    }

    fn set(&mut self, rows: Vec<(&'static str, Vec<Tile>)>) {
        let shelves = rows
            .into_iter()
            .filter(|(_, tiles)| !tiles.is_empty())
            .map(|(title, tiles)| {
                let model = self.tiles.entry(title).or_insert_with(list).clone();
                let count = tiles.len();
                update(&model, tiles);
                Shelf { title: title.into(), count: count.to_string().into(), tiles: model.into() }
            })
            .collect();
        update(&self.shelves, shelves);
    }
}

/// Which series page is open: one of this phone's, or one of the other device's.
#[derive(Clone, Copy, PartialEq)]
enum Open {
    Phone(u64),
    Remote(u64),
}

struct App {
    window: AppWindow,
    commands: Commands,
    data_dir: PathBuf,
    prefs: Prefs,
    // This phone.
    cards: Vec<SeriesCard>,
    detail: Option<SeriesDetail>,
    art: Art,
    asked: HashSet<u64>,
    posters: Sender<(u64, String)>,
    trackers: Option<(TrackerAccount, TrackerAccount)>,
    // The other device.
    cache: Option<Cache>,
    anilist: AniListCache,
    remote_art: Art,
    // What is picked.
    day: usize,
    open: Option<Open>,
    ticks: u32,
    home_shelves: Shelves,
    days: Rc<VecModel<Day>>,
    day_tiles: Rc<VecModel<Tile>>,
    library: Rc<VecModel<Tile>>,
    episodes: Rc<VecModel<EpisodeLine>>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app(f: impl FnOnce(&mut App)) {
    APP.with(|app| {
        if let Some(app) = app.borrow_mut().as_mut() {
            f(app);
        }
    });
}

fn today_index(now: i64) -> usize {
    anibeam_companion::calendar::today(now).weekday().num_days_from_monday() as usize
}

fn account_line(a: &TrackerAccount) -> String {
    match (a.connected, &a.username) {
        (true, Some(name)) => format!("Signed in as {name}"),
        (true, None) => "Signed in".to_string(),
        (false, _) if a.client_id.is_empty() && !a.bundled_credentials => "Not available in this build".to_string(),
        (false, _) => "Not signed in. Tap to sign in.".to_string(),
    }
}

impl App {
    fn data(&self) -> AppData<'_> {
        self.window.global::<AppData>()
    }

    /// The other device's name, once it has answered.
    fn host_name(&self) -> Option<String> {
        self.cache.as_ref().map(|c| if c.host.is_empty() { c.address.clone() } else { c.host.clone() })
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::SyncStarted => {}
            Event::SyncDone { at, error } => {
                let (tone, text) = match error {
                    None => (0, format!("Synced {}.", format::stamp(at))),
                    Some(e) => (1, format!("Last try {}: {e}", format::stamp(at))),
                };
                self.data().set_host_tone(tone);
                self.data().set_host_result(text.into());
            }
            Event::Snapshot(cache) => self.cache = Some(cache),
            Event::Poster(id, decoded) => {
                self.remote_art.posters.insert(id, Image::from_rgba8(decoded.buffer));
                self.remote_art.tones.insert(id, decoded.tone);
            }
            Event::AniList(anilist) => self.anilist = anilist,
            Event::TestStarted => {
                self.data().set_host_tone(2);
                self.data().set_host_result("Asking...".into());
            }
            Event::TestDone(result) => {
                let (tone, text) = match result {
                    Ok(text) => (0, text),
                    Err(text) => (1, text),
                };
                self.data().set_host_tone(tone);
                self.data().set_host_result(text.into());
            }
        }
        self.tick();
    }

    /// Reads the phone's cards, the open detail and the trackers again from the core.
    fn reload(&mut self) {
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
        for card in &self.cards {
            if let Some(path) = &card.poster
                && self.asked.insert(card.id)
            {
                let _ = self.posters.send((card.id, path.clone()));
            }
        }
        if let Some(Open::Phone(id)) = self.open {
            self.detail = match core.call(Call::GetSeries { series: id }) {
                Ok(Reply::SeriesDetail { detail }) => Some(*detail),
                _ => None,
            };
        }
        if let Ok(Reply::Trackers { state }) = core.call(Call::GetTrackers) {
            self.trackers = Some((state.anilist, state.mal));
        }
    }

    /// Everything the pages show, rebuilt from what is held and the clock. Cheap enough to
    /// run every second; the models only repaint what changed.
    fn tick(&mut self) {
        if crate::local::take_changed() {
            self.reload();
        }
        let now = anibeam_companion::now();
        // A handle of its own, so `data` does not hold `self` while the models change.
        let window = self.window.clone_strong();
        let data = window.global::<AppData>();

        // Home: this phone only.
        let home = crate::phone::home(&self.cards, &self.art, now, self.day);
        if data.get_hero() != home.hero {
            data.set_hero(home.hero);
        }
        update(&self.days, home.days);
        data.set_day(self.day as i32);
        data.set_day_title(home.day_title.into());
        update(&self.day_tiles, home.day_tiles);
        self.home_shelves.set(vec![("Recently watched", home.watched), ("Recently downloaded", home.downloaded)]);
        data.set_home_note(
            match self.cards.len() {
                0 => "On this phone".to_string(),
                1 => "On this phone, 1 show".to_string(),
                n => format!("On this phone, {n} shows"),
            }
            .into(),
        );

        // Library: this phone, or the device picked above it.
        let mut devices: Vec<SharedString> = vec!["This phone".into()];
        if let Some(name) = self.host_name() {
            devices.push(name.into());
        }
        let device = (data.get_device().max(0) as usize).min(devices.len() - 1);
        let query = data.get_query().to_string();
        let tiles = if device == 0 {
            let q = query.trim().to_lowercase();
            let cards: Vec<SeriesCard> = self
                .cards
                .iter()
                .filter(|c| {
                    q.is_empty()
                        || [Some(&c.title), c.titles.english.as_ref(), c.titles.romaji.as_ref()]
                            .into_iter()
                            .flatten()
                            .any(|t| t.to_lowercase().contains(&q))
                })
                .cloned()
                .collect();
            crate::phone::library(&cards, &self.art, now)
        } else {
            crate::remote::library(self.cache.as_ref(), &self.anilist, &self.remote_art, now, &query)
        };
        let shows = |n: usize| if n == 1 { "1 show".to_string() } else { format!("{n} shows") };
        data.set_library_note(
            if device == 0 {
                format!("This phone, {}", shows(self.cards.len()))
            } else {
                let at = self.cache.as_ref().map(|c| format::ago(now - c.synced_at)).unwrap_or_default();
                format!("{}, read only, synced {at}", devices[device])
            }
            .into(),
        );
        data.set_library_empty(
            if !query.trim().is_empty() {
                format!("Nothing matches \"{}\".", query.trim())
            } else if device == 0 {
                "Nothing on this phone yet.".to_string()
            } else {
                "That device has nothing in its library.".to_string()
            }
            .into(),
        );
        update(&self.library, tiles);
        data.set_devices(ModelRc::new(VecModel::from(devices)));
        data.set_device(device as i32);

        // The series page.
        let page = match self.open {
            Some(Open::Phone(_)) => self.detail.as_ref().map(|d| crate::phone::series(d, &self.art, now)),
            Some(Open::Remote(id)) => crate::remote::series(self.cache.as_ref(), &self.anilist, &self.remote_art, id, now),
            None => None,
        };
        match page {
            Some((head, lines)) => {
                if data.get_series() != head {
                    data.set_series(head);
                }
                update(&self.episodes, lines);
                data.set_series_open(true);
            }
            None => {
                self.open = None;
                data.set_series_open(false);
            }
        }

        // More.
        if let Some((anilist, mal)) = &self.trackers {
            data.set_anilist_state(account_line(anilist).into());
            data.set_mal_state(account_line(mal).into());
            let signed: Vec<&str> =
                [("AniList", anilist), ("MAL", mal)].iter().filter(|(_, a)| a.connected).map(|(n, _)| *n).collect();
            data.set_trackers_line(if signed.is_empty() { "Not signed in".into() } else { signed.join(" and ").into() });
        }
        if self.ticks.is_multiple_of(STORAGE_EVERY) {
            self.storage();
        }
        self.ticks = self.ticks.wrapping_add(1);
    }

    fn storage(&self) {
        let data = self.data();
        let library = self.data_dir.join(crate::local::LIBRARY_DIR);
        let used = crate::android::used(&library);
        let free = crate::android::space(&self.data_dir).map(|(free, _)| free);
        let free_text = free.map(|f| format!("{} free", format::bytes(f))).unwrap_or_default();
        data.set_storage_line(
            format!(
                "{} used{}{}",
                format::bytes(used),
                if free_text.is_empty() { "" } else { ", " },
                free_text
            )
            .into(),
        );
        let limit = u64::from(self.prefs.limit_gb) * 1_000_000_000;
        data.set_free_line(
            free.map(|f| {
                if f < limit {
                    format!("This phone has {} free, less than the limit.", format::bytes(f))
                } else {
                    format!("This phone has {} free.", format::bytes(f))
                }
            })
            .unwrap_or_default()
            .into(),
        );
    }

    fn show_prefs(&self) {
        let data = self.data();
        data.set_onboarding(!self.prefs.onboarded);
        data.set_wifi_only(self.prefs.wifi_only);
        data.set_limit_on(self.prefs.limit_on);
        data.set_limit_gb(self.prefs.limit_gb as i32);
    }

    fn save_prefs(&mut self) {
        if let Err(e) = prefs::save(&self.data_dir, &self.prefs) {
            crate::local::log(&format!("could not save the phone's settings: {e}"));
        }
        self.show_prefs();
    }

    fn open_series(&mut self, id: i32, remote: bool) {
        let Ok(id) = u64::try_from(id) else { return };
        self.open = Some(if remote { Open::Remote(id) } else { Open::Phone(id) });
        self.detail = None;
        self.reload_open();
        self.tick();
    }

    fn reload_open(&mut self) {
        if let (Some(Open::Phone(id)), Some(core)) = (self.open, crate::local::core()) {
            self.detail = match core.call(Call::GetSeries { series: id }) {
                Ok(Reply::SeriesDetail { detail }) => Some(*detail),
                _ => None,
            };
        }
    }

    fn close_series(&mut self) {
        self.open = None;
        self.detail = None;
        self.data().set_series_open(false);
    }

    fn connect_tracker(&mut self, which: i32) {
        let tracker = if which == 0 { Tracker::Anilist } else { Tracker::Mal };
        let Some(core) = crate::local::core() else { return };
        if let Err(e) = core.call(Call::ConnectTracker { tracker }) {
            crate::local::log(&format!("sign-in did not start: {e}"));
        }
    }

    fn save_host(&mut self, text: &str) {
        match HostAddress::parse(text) {
            Ok(address) => {
                self.data().set_host_address(address.display().into());
                self.data().set_host_result("Saved. Syncing...".into());
                self.data().set_host_tone(2);
                let _ = self.commands.send(Command::SaveAndSync(address));
            }
            Err(why) => {
                self.data().set_host_tone(1);
                self.data().set_host_result(why.into());
            }
        }
    }

    fn test_host(&mut self, text: &str) {
        match HostAddress::parse(text) {
            Ok(address) => {
                let _ = self.commands.send(Command::Test(address));
            }
            Err(why) => {
                self.data().set_host_tone(1);
                self.data().set_host_result(why.into());
            }
        }
    }
}

/// Reads poster files off the UI thread and hands each decoded one back.
fn start_poster_loader() -> Sender<(u64, String)> {
    let (tx, rx) = std::sync::mpsc::channel::<(u64, String)>();
    let _ = std::thread::Builder::new().name("posters".into()).spawn(move || {
        for (id, path) in rx {
            let Some(decoded) = std::fs::read(&path).ok().and_then(|b| crate::posters::decode(&b)) else { continue };
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|app| {
                    app.art.posters.insert(id, Image::from_rgba8(decoded.buffer));
                    app.art.tones.insert(id, decoded.tone);
                });
            });
        }
    });
    tx
}

/// Builds the window, starts the core, the sync thread and the clocks, and runs until the
/// activity goes away.
pub fn run(data_dir: PathBuf, commands: Commands, inbox: Receiver<Command>) -> Result<(), Box<dyn std::error::Error>> {
    crate::local::start(data_dir.clone());
    let store = Store::open(&data_dir)?;
    let address = HostAddress::parse(&store.settings().address)
        .unwrap_or_else(|_| HostAddress::parse(anibeam_companion::address::DEFAULT_ADDRESS).unwrap());

    let window = AppWindow::new()?;
    let data = window.global::<AppData>();
    data.set_version(
        format!("AniBeam for Android {} (build {})", env!("CARGO_PKG_VERSION"), env!("ANIBEAM_VERSION_CODE")).into(),
    );
    data.set_host_address(address.display().into());

    let home_shelves = Shelves::new();
    let (days, day_tiles, library, episodes) = (list(), list(), list(), list());
    data.set_home_shelves(home_shelves.shelves.clone().into());
    data.set_days(days.clone().into());
    data.set_day_tiles(day_tiles.clone().into());
    data.set_library(library.clone().into());
    data.set_episodes(episodes.clone().into());

    data.on_open_series(|id| with_app(|app| app.open_series(id, false)));
    data.on_open_remote(|id| with_app(|app| app.open_series(id, true)));
    data.on_close_series(|| with_app(App::close_series));
    data.on_pick_day(|day| {
        with_app(|app| {
            app.day = usize::try_from(day).unwrap_or(0).min(6);
            app.tick();
        })
    });
    data.on_pick_device(|i| {
        with_app(|app| {
            app.data().set_device(i);
            app.tick();
        })
    });
    data.on_search(|_| with_app(App::tick));
    data.on_set_wifi_only(|on| {
        with_app(|app| {
            app.prefs.wifi_only = on;
            app.save_prefs();
        })
    });
    data.on_set_limit(|on, gb| {
        with_app(|app| {
            app.prefs.limit_on = on;
            app.prefs.limit_gb = u32::try_from(gb).unwrap_or(prefs::SUGGESTED_LIMIT_GB);
            app.save_prefs();
        })
    });
    data.on_connect_tracker(|which| with_app(|app| app.connect_tracker(which)));
    data.on_finish_onboarding(|| {
        with_app(|app| {
            app.prefs.onboarded = true;
            app.save_prefs();
        })
    });
    data.on_restart_onboarding(|| {
        with_app(|app| {
            app.prefs.onboarded = false;
            app.data().set_step(0);
            app.save_prefs();
        })
    });
    data.on_test_host(|text| with_app(|app| app.test_host(&text)));
    data.on_save_host(|text| with_app(|app| app.save_host(&text)));
    data.on_refresh({
        let commands = commands.clone();
        move || {
            let _ = commands.send(Command::Sync);
        }
    });

    let now = anibeam_companion::now();
    let mut app = App {
        window: window.clone_strong(),
        commands: commands.clone(),
        data_dir: data_dir.clone(),
        prefs: prefs::load(&data_dir),
        cards: Vec::new(),
        detail: None,
        art: Art::default(),
        asked: HashSet::new(),
        posters: start_poster_loader(),
        trackers: None,
        cache: store.cache(),
        anilist: store.anilist(),
        remote_art: Art::default(),
        day: today_index(now),
        open: None,
        ticks: 0,
        home_shelves,
        days,
        day_tiles,
        library,
        episodes,
    };
    app.show_prefs();
    app.tick();
    APP.with(|cell| *cell.borrow_mut() = Some(app));

    worker::start(Store::open(&data_dir)?, inbox, |event| {
        let _ = slint::invoke_from_event_loop(move || with_app(|app| app.handle(event)));
    });
    let _ = commands.send(Command::Sync);

    let clock = Timer::default();
    clock.start(TimerMode::Repeated, Duration::from_secs(1), || with_app(App::tick));
    let syncs = Timer::default();
    syncs.start(TimerMode::Repeated, SYNC_EVERY, move || {
        let _ = commands.send(Command::Sync);
    });

    window.run()?;
    APP.with(|cell| cell.borrow_mut().take());
    Ok(())
}
