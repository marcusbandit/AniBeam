//! The UI thread: the window, what it shows, and the clock that keeps it current.
//!
//! Two sources feed the pages. This phone's own core is the default for everything; another
//! AniBeam on the tailnet, synced by the worker thread, only fills Library when it is picked
//! there, and its episodes stream from it.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use anibeam_companion::address::HostAddress;
use anibeam_companion::format;
use anibeam_companion::store::{AniListCache, Cache, Store};
use anibeam_core::{
    Call, Direction, PhoneAction, PhoneSource, Reply, SeriesCard, SeriesDetail, Sort, Tab, Torrent, Tracker, TrackerAccount,
};
use chrono::Datelike;
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};

use crate::prefs::{self, Prefs};
use crate::remote::Art;
use crate::worker::{self, Command, Commands, Event};
use crate::{AppData, AppWindow, CheckRow, Day, DownloadRow, EpisodeLine, Shelf, SourceRow, Tile};

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
    // The phone had saved its settings before this run: it was in use before What's new kept
    // a record, so an update opens the notes rather than counting them as read.
    used_before: bool,
    // This phone.
    cards: Vec<SeriesCard>,
    detail: Option<SeriesDetail>,
    art: Art,
    asked: HashSet<u64>,
    posters: Sender<(u64, String)>,
    trackers: Option<(TrackerAccount, TrackerAccount)>,
    /// The tracker a sign-in was last started for (0 AniList, 1 MAL), and why it failed,
    /// which More shows in place of "Not signed in" until it is tried again.
    /// The sign-in jobs started, by job id: which tracker each is for.
    signing_in: HashMap<u64, i32>,
    sign_in_error: Option<(i32, String)>,
    // The other device.
    cache: Option<Cache>,
    anilist: AniListCache,
    remote_art: Art,
    // What is picked.
    day: usize,
    open: Option<Open>,
    ticks: u32,
    devices: Vec<SharedString>,
    /// The last connection check as plain text, for Copy.
    check_text: String,
    home_shelves: Shelves,
    days: Rc<VecModel<Day>>,
    day_tiles: Rc<VecModel<Tile>>,
    library: Rc<VecModel<Tile>>,
    episodes: Rc<VecModel<EpisodeLine>>,
    /// Picking episodes to delete on the open series page, and the files picked.
    deleting: bool,
    picked: HashSet<i32>,
    /// A delete is on its way to the core.
    delete_busy: bool,
    // Downloads.
    sources: Vec<PhoneSource>,
    downloads: Rc<VecModel<DownloadRow>>,
    polling: bool,
    held: Option<bool>,
    service: Option<String>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

/// Endpoints were installed, removed or switched: the search page's chips follow.
pub fn endpoints_changed() {
    with_app(App::show_search_sources);
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
            Event::Snapshot(cache) => {
                crate::player::set_remote_numbers(remote_numbers(&cache));
                self.cache = Some(cache);
            }
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
            self.data().set_ready(true);
        }
        for card in &self.cards {
            if let Some(path) = &card.poster
                && self.asked.insert(card.id)
            {
                let _ = self.posters.send((card.id, path.clone()));
            }
        }
        if let Some(Open::Phone(id)) = self.open {
            // A failed read keeps the page as it was; only a series that is gone closes it.
            match core.call(Call::GetSeries { series: id }) {
                Ok(Reply::SeriesDetail { detail }) => self.detail = Some(*detail),
                Err(anibeam_core::CoreError::NotFound { .. }) => self.detail = None,
                _ => {}
            }
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
            let matches = |c: &&SeriesCard| {
                q.is_empty()
                    || [Some(&c.title), c.titles.english.as_ref(), c.titles.romaji.as_ref()]
                        .into_iter()
                        .flatten()
                        .any(|t| t.to_lowercase().contains(&q))
            };
            crate::phone::library(self.cards.iter().filter(matches), &self.art, now)
        } else {
            crate::remote::library(self.cache.as_ref(), &self.anilist, &self.remote_art, now, &query)
        };
        let shows = |n: usize| if n == 1 { "1 show".to_string() } else { format!("{n} shows") };
        data.set_library_note(
            if device == 0 {
                format!("This phone, {}", shows(self.cards.len()))
            } else {
                let at = self.cache.as_ref().map(|c| format::ago(now - c.synced_at)).unwrap_or_default();
                format!("{}, synced {at}", devices[device])
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
        if devices != self.devices {
            data.set_devices(ModelRc::new(VecModel::from(devices.clone())));
            self.devices = devices;
        }
        data.set_device(device as i32);

        // The series page.
        let page = match self.open {
            Some(Open::Phone(_)) => self.detail.as_ref().map(|d| crate::phone::series(d, &self.art, now)),
            Some(Open::Remote(id)) => crate::remote::series(self.cache.as_ref(), &self.anilist, &self.remote_art, id, now),
            None => None,
        };
        match page {
            Some((head, mut lines)) => {
                if data.get_series() != head {
                    data.set_series(head);
                }
                // A picked file that is gone (deleted, or the scan moved on) is dropped.
                let files: HashSet<i32> = lines.iter().map(|l| l.file).filter(|f| *f != 0).collect();
                self.picked.retain(|f| files.contains(f));
                for l in &mut lines {
                    l.picked = self.picked.contains(&l.file);
                }
                let local = matches!(self.open, Some(Open::Phone(_)));
                data.set_series_has_files(local && !files.is_empty());
                if self.deleting && files.is_empty() {
                    self.deleting = false;
                }
                data.set_deleting(self.deleting);
                data.set_delete_count(self.picked.len() as i32);
                update(&self.episodes, lines);
                data.set_series_open(true);
            }
            None => {
                self.open = None;
                data.set_series_open(false);
            }
        }

        // More.
        if let Some((job, why)) = crate::local::sign_in_failed()
            && let Some(which) = self.signing_in.remove(&job)
        {
            self.sign_in_error = Some((which, format!("Sign-in failed: {why}. Tap to try again.")));
        }
        if let Some((anilist, mal)) = &self.trackers {
            let line = |which: i32, a: &TrackerAccount| match &self.sign_in_error {
                Some((w, why)) if *w == which && !a.connected => why.clone(),
                _ => account_line(a),
            };
            data.set_anilist_state(line(0, anilist).into());
            data.set_mal_state(line(1, mal).into());
            let signed: Vec<&str> =
                [("AniList", anilist), ("MAL", mal)].iter().filter(|(_, a)| a.connected).map(|(n, _)| *n).collect();
            data.set_trackers_line(if signed.is_empty() { "Not signed in".into() } else { signed.join(" and ").into() });
        }
        if self.ticks.is_multiple_of(2) {
            self.poll_torrents();
        }
        if self.ticks.is_multiple_of(5) {
            self.hold_for_network();
        }
        if self.ticks.is_multiple_of(STORAGE_EVERY) {
            self.storage();
        }
        self.ticks = self.ticks.wrapping_add(1);
    }

    /// Reads the sources once the core is open, for the search page and More.
    fn load_sources(&mut self) {
        let Some(core) = crate::local::core() else { return };
        if let Ok(Reply::PhoneSources { sources }) = core.call(Call::PhoneSources) {
            self.show_sources(sources);
        }
    }

    fn show_sources(&mut self, sources: Vec<PhoneSource>) {
        let data = self.data();
        let rows: Vec<SourceRow> = sources
            .iter()
            .map(|s| SourceRow { name: s.name.clone().into(), url: anibeam_core::releases::native::mask(&s.url).into(), enabled: s.enabled })
            .collect();
        data.set_sources(ModelRc::new(VecModel::from(rows)));
        self.sources = sources;
        self.show_search_sources();
    }

    /// The Downloads chips: the sources and the installed endpoints, in search order.
    fn show_search_sources(&mut self) {
        let Some(core) = crate::local::core() else { return };
        let Ok(Reply::PhoneSources { sources }) = core.call(Call::PhoneSearchSources) else { return };
        let rows: Vec<SourceRow> = sources
            .iter()
            .map(|s| SourceRow { name: s.name.clone().into(), url: s.url.clone().into(), enabled: s.enabled })
            .collect();
        self.data().set_search_sources(ModelRc::new(VecModel::from(rows)));
    }

    fn torrent_action(&mut self, hash: String, action: i32) {
        let Some(core) = crate::local::core() else { return };
        let action = match action {
            0 => PhoneAction::Pause,
            1 => PhoneAction::Resume,
            3 => PhoneAction::Forget,
            _ => PhoneAction::Remove,
        };
        let _ = std::thread::Builder::new().name("torrent".into()).spawn(move || {
            if let Err(e) = core.call(Call::PhoneControl { hashes: vec![hash], action }) {
                crate::local::log(&format!("torrent action failed: {e}"));
            }
            let _ = slint::invoke_from_event_loop(|| with_app(App::poll_torrents));
        });
    }

    fn remove_source(&mut self, index: i32) {
        let Some(core) = crate::local::core() else { return };
        let Ok(index) = usize::try_from(index) else { return };
        if index >= self.sources.len() {
            return;
        }
        let mut sources = self.sources.clone();
        sources.remove(index);
        if let Ok(Reply::PhoneSources { sources }) = core.call(Call::SetPhoneSources { sources }) {
            self.show_sources(sources);
        }
    }

    /// Runs the connection check on a thread and lists what it found, failures first.
    fn run_check(&mut self) {
        let Some(core) = crate::local::core() else { return };
        self.data().set_checking(true);
        self.data().set_check_summary("".into());
        let _ = std::thread::Builder::new().name("check".into()).spawn(move || {
            let answer = core.call(Call::PhoneNetCheck);
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|app| {
                    let window = app.window.clone_strong();
                    let data = window.global::<AppData>();
                    data.set_checking(false);
                    let Ok(Reply::PhoneNetCheck { checks }) = answer else { return };
                    app.check_text = std::iter::once(format!("AniBeam {} connection check", env!("CARGO_PKG_VERSION")))
                        .chain(checks.iter().map(|c| {
                            format!("{} {}: {} ({} ms)", if c.ok { "OK  " } else { "FAIL" }, c.name, c.detail, c.ms)
                        }))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let failed = checks.iter().filter(|c| !c.ok).count();
                    data.set_check_summary(
                        if failed == 0 { "Every step went through.".to_string() } else { format!("{failed} of {} steps failed. A screenshot of this list says where.", checks.len()) }
                            .into(),
                    );
                    let rows: Vec<CheckRow> = checks
                        .iter()
                        .map(|c| CheckRow {
                            name: format!("{}  {} ms", c.name, c.ms).into(),
                            detail: c.detail.clone().into(),
                            ok: c.ok,
                        })
                        .collect();
                    data.set_checks(ModelRc::new(VecModel::from(rows)));
                })
            });
        });
    }

    /// Asks the engine for its torrents on a thread, one ask at a time.
    fn poll_torrents(&mut self) {
        if self.polling {
            return;
        }
        let Some(core) = crate::local::core() else { return };
        if self.sources.is_empty() {
            self.load_sources();
        }
        crate::repos::opened();
        self.polling = true;
        let _ = std::thread::Builder::new().name("torrents".into()).spawn(move || {
            let answer = core.call(Call::PhoneTorrents);
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|app| {
                    app.polling = false;
                    if let Ok(Reply::PhoneTorrents { torrents, held }) = answer {
                        app.show_torrents(torrents, held);
                    }
                })
            });
        });
    }

    fn show_torrents(&mut self, torrents: Vec<Torrent>, held: bool) {
        // The service is for torrents that are moving: one in error, stuck with no peers or
        // paused does not keep a notification up.
        let active: Vec<&Torrent> =
            torrents.iter().filter(|t| t.progress < 1.0 && matches!(t.state.as_str(), "downloading" | "checkingDL")).collect();
        update(&self.downloads, torrents.iter().map(|t| download_row(t, held)).collect());
        let note = if held && torrents.iter().any(|t| t.progress < 1.0) {
            "Waiting for Wi-Fi. Downloads go on by themselves when the phone is on Wi-Fi."
        } else if torrents.is_empty() {
            "Nothing downloading."
        } else {
            ""
        };
        self.data().set_downloads_note(note.into());
        let (down, up) = torrents.iter().fold((0u64, 0u64), |(d, u), t| (d + t.dlspeed, u + t.upspeed));
        self.data().set_rate_down(format!("{}/s", format::bytes(down)).into());
        self.data().set_rate_up(format!("{}/s", format::bytes(up)).into());
        let finished = torrents.iter().filter(|t| t.progress >= 1.0).count();
        self.data().set_finished_count(finished as i32);
        self.data().set_downloading_count((torrents.len() - finished) as i32);
        self.data().set_moving_count(active.len() as i32);

        // The service runs while something downloads, with how many in its notification.
        let text = match active.len() {
            0 => None,
            1 => Some(format!("Downloading {}", active[0].name)),
            n => Some(format!("Downloading {n} episodes")),
        };
        if text != self.service {
            let done = match &text {
                Some(t) => crate::android::start_service(t),
                None => {
                    crate::android::stop_service();
                    true
                }
            };
            if done {
                self.service = text;
            }
        }
    }

    /// Holds downloads off Wi-Fi when the person wants Wi-Fi only. The state is noted once
    /// the core took it, so a refusal is tried again on the next look.
    fn hold_for_network(&mut self) {
        let Some(core) = crate::local::core() else { return };
        let held = self.prefs.wifi_only && crate::android::unmetered() != Some(true);
        if self.held == Some(held) {
            return;
        }
        let _ = std::thread::Builder::new().name("hold".into()).spawn(move || match core.call(Call::SetPhoneHeld { held }) {
            Ok(_) => {
                let _ = slint::invoke_from_event_loop(move || with_app(|app| app.held = Some(held)));
            }
            Err(e) => crate::local::log(&format!("could not hold downloads: {e}")),
        });
    }

    /// Walks the library folder on a thread of its own and sets the storage lines when done,
    /// so a big library never holds up a frame.
    fn storage(&self) {
        let library = self.data_dir.join(crate::local::LIBRARY_DIR);
        let data_dir = self.data_dir.clone();
        let _ = std::thread::Builder::new().name("storage".into()).spawn(move || {
            let used = crate::android::used(&library);
            let space = crate::android::space(&data_dir);
            let _ = slint::invoke_from_event_loop(move || with_app(|app| app.show_storage(used, space)));
        });
    }

    fn show_storage(&self, used: u64, space: Option<(u64, u64)>) {
        let data = self.data();
        let free = space.map(|(free, _)| free);
        if let Some((_, total)) = space {
            let disk = (total as f64 / 1e9).floor().max(1.0);
            data.set_disk_gb(disk as f32);
            // A phone smaller than the suggestion starts at its whole size; once the person
            // has been through onboarding the cap is theirs, whatever it is.
            if !self.prefs.onboarded && self.prefs.limit_gb > disk {
                let mut prefs = self.prefs.clone();
                prefs.limit_gb = disk;
                let _ = slint::invoke_from_event_loop(move || {
                    with_app(|app| {
                        app.prefs.limit_gb = prefs.limit_gb;
                        app.save_prefs();
                    })
                });
            }
        }
        let free_text = free.map(|f| format!("{} free", format::bytes(f))).unwrap_or_default();
        data.set_used_figure(format::bytes(used).into());
        data.set_free_figure(free.map(format::bytes).unwrap_or_default().into());
        data.set_storage_line(
            format!(
                "{} used{}{}",
                format::bytes(used),
                if free_text.is_empty() { "" } else { ", " },
                free_text
            )
            .into(),
        );
        let limit = (self.prefs.limit_gb * 1e9) as u64;
        let below = "less than the cap: the phone fills up before AniBeam stops";
        data.set_free_line(
            free.map(|f| {
                if f < limit {
                    format!("This phone has {} free, {below}.", format::bytes(f))
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
        let quality = crate::stream::Quality::ALL.iter().position(|q| *q == self.prefs.stream_quality).unwrap_or(0);
        data.set_stream_quality(quality as i32);
        data.set_limit_on(self.prefs.limit_on);
        data.set_limit_gb(self.prefs.limit_gb as f32);
        data.set_limit_label(prefs::limit_label(self.prefs.limit_gb).into());
    }

    fn save_prefs(&mut self) {
        if let Err(e) = prefs::save(&self.data_dir, &self.prefs) {
            crate::local::log(&format!("could not save the phone's settings: {e}"));
        }
        self.show_prefs();
        self.storage();
    }

    /// What's new as the app starts: a fresh install counts this version's notes as read,
    /// and the first start after an update opens them.
    fn start_notes(&mut self) {
        if let Some(seen) = self.prefs.notes.started(crate::notes::VERSION, self.used_before) {
            self.prefs.notes = seen;
            self.save_prefs();
        } else if self.prefs.notes.due(crate::notes::VERSION, self.used_before) {
            self.show_notes();
        }
    }

    fn show_notes(&self) {
        let view = crate::notes::view(&self.prefs.notes, self.used_before);
        let data = self.data();
        data.set_notes_note(view.note.as_str().into());
        data.set_notes(ModelRc::new(VecModel::from(crate::notes::lines(&view))));
        data.set_notes_open(true);
    }

    /// Closed by Got it or Back: from now on the line stays at the version this one replaced.
    fn close_notes(&mut self) {
        self.data().set_notes_open(false);
        let read = self.prefs.notes.closed(crate::notes::VERSION);
        if read != self.prefs.notes {
            self.prefs.notes = read;
            self.save_prefs();
        }
    }

    fn open_series(&mut self, id: i32, remote: bool) {
        let Ok(id) = u64::try_from(id) else { return };
        // Picking to delete belongs to the page it started on.
        self.deleting = false;
        self.picked.clear();
        self.open = Some(if remote { Open::Remote(id) } else { Open::Phone(id) });
        self.detail = None;
        self.reload_open();
        self.tick();
    }

    fn confirm_delete(&mut self) {
        let (Some(Open::Phone(series)), Some(core)) = (self.open, crate::local::core()) else { return };
        let files: Vec<u64> = self.picked.iter().filter_map(|f| u64::try_from(*f).ok()).collect();
        if files.is_empty() || self.delete_busy {
            return;
        }
        self.delete_busy = true;
        self.data().set_delete_busy(true);
        self.data().set_delete_error("".into());
        let _ = std::thread::Builder::new().name("delete".into()).spawn(move || {
            let answer = core.call(Call::DeleteEpisodes { series, files });
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|app| {
                    app.delete_busy = false;
                    app.data().set_delete_busy(false);
                    match answer {
                        Ok(_) => {
                            app.deleting = false;
                            app.picked.clear();
                        }
                        Err(e) => app.data().set_delete_error(e.to_string().into()),
                    }
                    app.reload_open();
                    app.tick();
                })
            });
        });
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
        self.deleting = false;
        self.picked.clear();
        self.open = None;
        self.detail = None;
        self.data().set_deleting(false);
        self.data().set_series_open(false);
    }

    fn connect_tracker(&mut self, which: i32) {
        let tracker = if which == 0 { Tracker::Anilist } else { Tracker::Mal };
        self.sign_in_error = None;
        let Some(core) = crate::local::core() else {
            let line: SharedString = "AniBeam is still starting. Try again in a moment.".into();
            if which == 0 {
                self.data().set_anilist_state(line);
            } else {
                self.data().set_mal_state(line);
            }
            return;
        };
        let started = core.call(Call::ConnectTracker { tracker });
        if let Ok(Reply::Started { job }) = &started {
            self.signing_in.insert(*job, which);
        }
        if let Err(e) = started {
            crate::local::log(&format!("sign-in did not start: {e}"));
            self.sign_in_error = Some((which, format!("Sign-in did not start: {e}. Tap to try again.")));
            self.tick();
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

fn download_row(t: &Torrent, held: bool) -> DownloadRow {
    let done = t.progress >= 1.0;
    let paused = t.state.starts_with("paused") && !done;
    let state = match t.state.as_str() {
        _ if done => "Done",
        // Held off mobile data, nothing restarts it, a retry included, until Wi-Fi.
        "pausedDL" | "error" if held => "Waiting for Wi-Fi",
        "pausedDL" => "Paused",
        "stalledDL" => "Looking for peers",
        "checkingDL" => "Checking",
        "error" => "Stopped by an error",
        _ => "Downloading",
    };
    let amount = if done {
        format::bytes(t.size)
    } else {
        format!("{} of {}", format::bytes((t.progress * t.size as f64) as u64), format::bytes(t.size))
    };
    let mut speed = Vec::new();
    if !done && t.dlspeed > 0 {
        speed.push(format!("{}/s", format::bytes(t.dlspeed)));
    }
    if !done && t.eta > 0 && t.eta < 8_640_000 {
        speed.push(format!("{} left", format::countdown(t.eta).trim_start_matches("in ")));
    }
    let speed = speed.join(", ");
    let facts = if speed.is_empty() { amount.clone() } else { format!("{amount}, {speed}") };
    // Only the files picked to download: a skipped one has no cube and is not coming.
    let mut episodes: Vec<f64> = t.files.iter().filter(|f| f.priority != 0).filter_map(|f| f.episode).collect();
    episodes.sort_by(f64::total_cmp);
    let what = match (t.episode, episodes.first(), episodes.last()) {
        (Some(e), _, _) if !t.batch => format!("Episode {e}"),
        (_, Some(a), Some(b)) if a != b => format!("Episodes {a} to {b}"),
        (_, Some(a), _) => format!("Episode {a}"),
        _ if t.batch => "A batch".to_string(),
        _ => String::new(),
    };
    let release = [t.group.as_str(), t.resolution.as_str()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(", ");
    let folder = std::path::Path::new(&t.save_path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let cubes: Vec<i32> = if t.files.len() > 1 {
        t.files.iter().filter(|f| f.priority != 0).map(|f| if f.progress >= 1.0 { 2 } else if f.progress > 0.0 { 6 } else { 4 }).collect()
    } else {
        Vec::new()
    };
    let tone = match state {
        "Done" => 4,
        "Waiting for Wi-Fi" => 2,
        "Stopped by an error" => 3,
        _ if paused => 1,
        _ => 0,
    };
    DownloadRow {
        hash: t.hash.clone().into(),
        name: t.name.clone().into(),
        state: state.into(),
        facts: facts.into(),
        progress: t.progress as f32,
        paused,
        done,
        percent: format!("{}%", (t.progress * 100.0).floor() as i64).into(),
        what: what.into(),
        amount: amount.into(),
        speed: speed.into(),
        release: release.into(),
        folder: folder.into(),
        cubes: ModelRc::new(VecModel::from(cubes)),
        tone,
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

/// Each streamable file of the other device's, with its episode number.
fn remote_numbers(cache: &Cache) -> HashMap<u64, f64> {
    cache
        .snapshot
        .series
        .iter()
        .flat_map(|s| s.episodes.iter())
        .filter_map(|e| Some((e.file?, e.number)))
        .collect()
}

/// Builds the window, starts the core, the sync thread and the clocks, and runs until the
/// activity goes away.
pub fn run(data_dir: PathBuf, commands: Commands, inbox: Receiver<Command>) -> Result<(), Box<dyn std::error::Error>> {
    crate::local::start(data_dir.clone());
    crate::stream::init(data_dir.clone());
    let store = Store::open(&data_dir)?;
    let address = HostAddress::parse(&store.settings().address)
        .unwrap_or_else(|_| HostAddress::parse(anibeam_companion::address::DEFAULT_ADDRESS).unwrap());

    let window = AppWindow::new()?;
    crate::player::install(&window);
    let data = window.global::<AppData>();
    data.set_version(
        format!("AniBeam for Android {} (build {})", env!("CARGO_PKG_VERSION"), env!("ANIBEAM_VERSION_CODE")).into(),
    );
    data.set_host_address(address.display().into());

    let home_shelves = Shelves::new();
    let (days, day_tiles, library, episodes) = (list(), list(), list(), list());
    let downloads = list();
    data.set_downloads(downloads.clone().into());
    data.set_home_shelves(home_shelves.shelves.clone().into());
    data.set_days(days.clone().into());
    data.set_day_tiles(day_tiles.clone().into());
    data.set_library(library.clone().into());
    data.set_episodes(episodes.clone().into());

    data.on_open_series(|id| with_app(|app| app.open_series(id, false)));
    data.on_open_remote(|id| with_app(|app| app.open_series(id, true)));
    data.on_close_series(|| with_app(App::close_series));
    data.on_play(|file| crate::player::open(file as u64));
    data.on_play_remote(|file| crate::player::open_remote(file as u64));
    let qualities: Vec<slint::SharedString> = crate::stream::Quality::ALL.iter().map(|q| q.label().into()).collect();
    data.set_stream_qualities(ModelRc::new(VecModel::from(qualities)));
    data.on_set_stream_quality(|i| {
        with_app(|app| {
            if let Some(q) = usize::try_from(i).ok().and_then(|i| crate::stream::Quality::ALL.get(i)) {
                app.prefs.stream_quality = *q;
                app.save_prefs();
            }
        })
    });
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
            app.prefs.limit_gb = prefs::snap_limit(f64::from(gb));
            app.save_prefs();
        })
    });
    data.on_connect_tracker(|which| with_app(|app| app.connect_tracker(which)));
    data.on_start_delete(|| {
        with_app(|app| {
            app.deleting = true;
            app.picked.clear();
            app.data().set_delete_error("".into());
            app.tick();
        })
    });
    data.on_toggle_delete(|file| {
        with_app(|app| {
            if !app.picked.remove(&file) {
                app.picked.insert(file);
            }
            app.tick();
        })
    });
    data.on_cancel_delete(|| {
        with_app(|app| {
            app.deleting = false;
            app.picked.clear();
            app.tick();
        })
    });
    data.on_confirm_delete(|| with_app(App::confirm_delete));
    data.on_finish_onboarding(|| {
        with_app(|app| {
            app.prefs.onboarded = true;
            app.save_prefs();
        })
    });
    data.on_open_notes(|| with_app(|app| app.show_notes()));
    data.on_close_notes(|| with_app(App::close_notes));
    data.on_restart_onboarding(|| {
        with_app(|app| {
            app.prefs.onboarded = false;
            app.data().set_step(0);
            app.save_prefs();
        })
    });
    data.on_test_host(|text| with_app(|app| app.test_host(&text)));
    data.on_save_host(|text| with_app(|app| app.save_host(&text)));
    data.on_torrent_action(|hash, action| with_app(|app| app.torrent_action(hash.to_string(), action)));
    data.on_remove_source(|i| with_app(|app| app.remove_source(i)));
    data.on_run_check(|| with_app(App::run_check));
    data.on_copy_check(|| {
        with_app(|app| {
            let copied = crate::android::copy_text("AniBeam connection check", &app.check_text);
            app.data().set_check_summary(if copied { "Copied. Paste it in a message.".into() } else { "Could not copy.".into() });
        })
    });
    data.on_refresh({
        let commands = commands.clone();
        move || {
            let _ = commands.send(Command::Sync);
        }
    });

    crate::find::install(&window, data_dir.clone());
    crate::repos::install(&window);

    let now = anibeam_companion::now();
    let mut app = App {
        window: window.clone_strong(),
        commands: commands.clone(),
        data_dir: data_dir.clone(),
        // Read before anything here writes the file.
        used_before: prefs::exists(&data_dir),
        prefs: prefs::load(&data_dir),
        cards: Vec::new(),
        detail: None,
        art: Art::default(),
        asked: HashSet::new(),
        posters: start_poster_loader(),
        trackers: None,
        signing_in: HashMap::new(),
        sign_in_error: None,
        cache: {
            let cache = store.cache();
            if let Some(c) = &cache {
                crate::player::set_remote_numbers(remote_numbers(c));
            }
            cache
        },
        anilist: store.anilist(),
        remote_art: Art::default(),
        day: today_index(now),
        open: None,
        ticks: 0,
        devices: Vec::new(),
        check_text: String::new(),
        home_shelves,
        days,
        day_tiles,
        library,
        episodes,
        deleting: false,
        picked: HashSet::new(),
        delete_busy: false,
        sources: Vec::new(),
        downloads,
        polling: false,
        held: None,
        service: None,
    };
    app.show_prefs();
    app.start_notes();
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
