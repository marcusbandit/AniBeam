//! The UI thread: the window, what it shows, and the clocks that keep it current.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use anibeam_companion::address::HostAddress;
use anibeam_companion::client::HostError;
use anibeam_companion::format;
use anibeam_companion::store::{AniListCache, Cache, Store};
use chrono::Datelike;
use slint::{ComponentHandle, Image, Model, ModelRc, Timer, TimerMode, VecModel};

use crate::rows::{Art, ShelfRows};
use crate::worker::{self, Command, Commands, Event};
use crate::{AppData, AppWindow, Day, EpisodeLine, RuleRow, Shelf, Tile, TorrentRow};

/// How often the app looks at the host while it is open.
const SYNC_EVERY: Duration = Duration::from_secs(5 * 60);
/// A snapshot younger than this from a host that answered reads as live.
const LIVE_FOR: i64 = 10 * 60;

/// How the last sync went, as far as the screens care.
enum Reach {
    /// Nothing tried since the app started.
    NotYet,
    Online,
    Offline(HostError),
}

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

    fn set(&mut self, rows: Vec<ShelfRows>) {
        let shelves = rows
            .into_iter()
            .map(|row| {
                let tiles = self.tiles.entry(row.title).or_insert_with(list).clone();
                let count = row.tiles.len();
                update(&tiles, row.tiles);
                Shelf { title: row.title.into(), count: count.to_string().into(), tone: row.tone, tiles: tiles.into() }
            })
            .collect();
        update(&self.shelves, shelves);
    }
}

struct App {
    window: AppWindow,
    commands: Commands,
    cache: Option<Cache>,
    anilist: AniListCache,
    address: HostAddress,
    art: Art,
    reach: Reach,
    last_attempt: i64,
    /// Weeks from this one, and the picked day in it (0 is Monday).
    week: i64,
    day: usize,
    /// The series whose page is open.
    series: Option<u64>,
    next_shelves: Shelves,
    library: Shelves,
    days: Rc<VecModel<Day>>,
    day_tiles: Rc<VecModel<Tile>>,
    episodes: Rc<VecModel<EpisodeLine>>,
    torrents: Rc<VecModel<TorrentRow>>,
    rules: Rc<VecModel<RuleRow>>,
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

impl App {
    fn host_name(&self) -> String {
        match &self.cache {
            Some(cache) if cache.address == self.address.display() && !cache.host.is_empty() => cache.host.clone(),
            _ => self.address.short_name(),
        }
    }

    fn handle(&mut self, event: Event) {
        let w = &self.window;
        match event {
            Event::SyncStarted => w.set_syncing(true),
            Event::Snapshot(cache) => {
                self.cache = Some(cache);
                self.reach = Reach::Online;
            }
            Event::Poster(id, decoded) => {
                self.art.posters.insert(id, Image::from_rgba8(decoded.buffer));
                self.art.tones.insert(id, decoded.tone);
            }
            Event::AniList(anilist) => self.anilist = anilist,
            Event::SyncDone { at, error } => {
                w.set_syncing(false);
                self.last_attempt = at;
                self.reach = match error {
                    None => Reach::Online,
                    Some(e) => Reach::Offline(e),
                };
            }
            Event::TestStarted => {
                w.set_testing(true);
                w.set_test_result("".into());
            }
            Event::TestDone(result) => {
                w.set_testing(false);
                let (tone, text) = match result {
                    Ok(text) => (0, text),
                    Err(text) => (1, text),
                };
                w.set_test_tone(tone);
                w.set_test_result(text.into());
            }
        }
        self.tick();
    }

    /// Everything the pages show, rebuilt from the cache and the clock. Cheap enough to run
    /// every second; the models only repaint what changed.
    fn tick(&mut self) {
        let now = anibeam_companion::now();
        let cache = self.cache.as_ref();
        let data = self.window.global::<AppData>();

        let (hero, shelves) = crate::rows::next_up(cache, &self.anilist, &self.art, now);
        if data.get_hero() != hero {
            data.set_hero(hero);
        }
        self.next_shelves.set(shelves);
        data.set_has_data(cache.is_some());

        let view = crate::rows::calendar(cache, &self.anilist, &self.art, now, self.week, self.day);
        data.set_week_label(view.label.into());
        data.set_week_note(view.note.into());
        data.set_week_is_current(view.current);
        update(&self.days, view.days);
        data.set_day(self.day as i32);
        data.set_day_title(view.day_title.into());
        data.set_day_note(view.day_note.into());
        update(&self.day_tiles, view.tiles);

        self.library.set(crate::rows::library(cache, &self.anilist, &self.art, now));

        let (torrents, known, rules) = crate::rows::downloads(cache, &self.anilist, now);
        update(&self.torrents, torrents);
        data.set_torrents_known(known);
        update(&self.rules, rules);
        data.set_problems(ModelRc::new(VecModel::from(crate::rows::problems(cache))));

        match self.series.and_then(|id| crate::rows::series(cache, &self.anilist, &self.art, id, now)) {
            Some((head, lines)) => {
                if data.get_series() != head {
                    data.set_series(head);
                }
                update(&self.episodes, lines);
                data.set_series_open(true);
            }
            None => {
                self.series = None;
                data.set_series_open(false);
            }
        }

        self.status(now);
    }

    /// The header line, the Downloads banner and the Settings facts.
    fn status(&self, now: i64) {
        let host = self.host_name();
        let synced = self.cache.as_ref().map(|c| c.synced_at);
        let (status, tone) = match (&self.reach, synced) {
            (Reach::Online, Some(at)) => (format!("{host} online, synced {}", format::ago(now - at)), 0),
            (Reach::Online, None) => (format!("{host} online"), 0),
            (Reach::Offline(_), Some(at)) => (format!("{host} offline, last synced {}", format::stamp(at)), 1),
            (Reach::Offline(_), None) => (format!("{host} offline, never synced"), 1),
            (Reach::NotYet, Some(at)) => (format!("{host}, last synced {}", format::stamp(at)), 2),
            (Reach::NotYet, None) => (format!("{host}, not synced yet"), 2),
        };
        let w = &self.window;
        w.set_status(status.into());
        w.set_status_tone(tone);

        let data = w.global::<AppData>();
        let live = matches!(self.reach, Reach::Online) && synced.is_some_and(|at| now - at < LIVE_FOR);
        data.set_live(live);
        data.set_freshness(
            match synced {
                None => "Nothing synced yet. Set the host in Settings.".to_string(),
                Some(at) if live => format!("Live from {host}, synced {}.", format::ago(now - at)),
                Some(at) => format!(
                    "Last synced {} ({}). {host} is not answering, so this is what it had then.",
                    format::stamp(at),
                    format::ago(now - at)
                ),
            }
            .into(),
        );

        let mut sync_info = match &self.cache {
            Some(c) => format!(
                "Last synced {} from {} ({}), AniBeam {}.",
                format::stamp(c.synced_at),
                c.host,
                c.address,
                c.host_version
            ),
            None => "Never synced.".to_string(),
        };
        if let Reach::Offline(e) = &self.reach {
            sync_info.push_str(&format!(" Last try {}: {e}", format::ago(now - self.last_attempt)));
        }
        w.set_sync_info(sync_info.into());
        w.set_anilist_info(anilist_line(&self.anilist, synced, now).into());
    }

    fn open_series(&mut self, id: i32) {
        self.series = u64::try_from(id).ok();
        self.tick();
    }

    fn close_series(&mut self) {
        self.series = None;
        self.window.global::<AppData>().set_series_open(false);
    }

    fn shift_week(&mut self, by: i32) {
        let now = anibeam_companion::now();
        self.week = if by == 0 { 0 } else { self.week + i64::from(by) };
        // This week opens on today; any other on its first day with something on.
        self.day = if self.week == 0 {
            today_index(now)
        } else {
            let view = crate::rows::calendar(self.cache.as_ref(), &self.anilist, &self.art, now, self.week, 0);
            view.days.iter().position(|d| d.count > 0).unwrap_or(0)
        };
        self.tick();
    }

    fn pick_day(&mut self, day: i32) {
        self.day = usize::try_from(day).unwrap_or(0).min(6);
        self.tick();
    }

    fn save_and_sync(&mut self, text: &str) {
        match HostAddress::parse(text) {
            Ok(address) => {
                self.window.set_address(address.display().into());
                self.window.set_test_result("".into());
                self.address = address.clone();
                let _ = self.commands.send(Command::SaveAndSync(address));
                self.reach = Reach::NotYet;
                self.tick();
            }
            Err(why) => {
                self.window.set_test_tone(1);
                self.window.set_test_result(why.into());
            }
        }
    }

    fn test(&mut self, text: &str) {
        match HostAddress::parse(text) {
            Ok(address) => {
                let _ = self.commands.send(Command::Test(address));
            }
            Err(why) => {
                self.window.set_test_tone(1);
                self.window.set_test_result(why.into());
            }
        }
    }
}

/// What Settings says about AniList: when it last answered, whether its times are in use,
/// and why the last ask failed.
fn anilist_line(a: &AniListCache, synced: Option<i64>, now: i64) -> String {
    let mut line = if a.fetched_at > 0 {
        let in_use = synced.is_none_or(|s| a.fetched_at > s);
        format!(
            "AniList gave airing times for {} shows {}; {}.",
            a.shows.len(),
            format::ago(now - a.fetched_at),
            if in_use { "they are in use" } else { "the host's later snapshot replaces them" }
        )
    } else if a.attempted_at == 0 {
        "AniList has not been needed yet.".to_string()
    } else {
        String::new()
    };
    if let Some(error) = &a.error {
        line.push_str(&format!(" The last ask failed: {error}"));
    }
    line.trim().to_string()
}

/// Builds the window from what is stored, starts the sync thread and the clocks, and runs
/// until the activity goes away.
pub fn run(data_dir: PathBuf, commands: Commands, inbox: Receiver<Command>) -> Result<(), Box<dyn std::error::Error>> {
    // The phone's own core runs beside the host view until the new screens replace it.
    let _core = match crate::local::open(&data_dir) {
        Ok(core) => Some(core),
        Err(e) => {
            crate::local::log(&format!("core did not open: {e}"));
            None
        }
    };
    let store = Store::open(&data_dir)?;
    let settings = store.settings();
    let address = HostAddress::parse(&settings.address)
        .unwrap_or_else(|_| HostAddress::parse(anibeam_companion::address::DEFAULT_ADDRESS).unwrap());

    let window = AppWindow::new()?;
    window.set_version(
        format!("AniBeam for Android {} (build {})", env!("CARGO_PKG_VERSION"), env!("ANIBEAM_VERSION_CODE")).into(),
    );
    window.set_address(address.display().into());

    let next_shelves = Shelves::new();
    let library = Shelves::new();
    let (days, day_tiles, episodes, torrents, rules) = (list(), list(), list(), list(), list());
    let data = window.global::<AppData>();
    data.set_next_shelves(next_shelves.shelves.clone().into());
    data.set_library(library.shelves.clone().into());
    data.set_days(days.clone().into());
    data.set_day_tiles(day_tiles.clone().into());
    data.set_episodes(episodes.clone().into());
    data.set_torrents(torrents.clone().into());
    data.set_rules(rules.clone().into());

    data.on_open_series(|id| with_app(|app| app.open_series(id)));
    data.on_close_series(|| with_app(App::close_series));
    data.on_shift_week(|by| with_app(|app| app.shift_week(by)));
    data.on_pick_day(|day| with_app(|app| app.pick_day(day)));
    window.on_refresh({
        let commands = commands.clone();
        move || {
            let _ = commands.send(Command::Sync);
        }
    });
    window.on_save_and_sync(|text| with_app(|app| app.save_and_sync(&text)));
    window.on_test_connection(|text| with_app(|app| app.test(&text)));

    let now = anibeam_companion::now();
    let mut app = App {
        window: window.clone_strong(),
        commands: commands.clone(),
        cache: store.cache(),
        anilist: store.anilist(),
        address,
        art: Art::default(),
        reach: Reach::NotYet,
        last_attempt: 0,
        week: 0,
        day: today_index(now),
        series: None,
        next_shelves,
        library,
        days,
        day_tiles,
        episodes,
        torrents,
        rules,
    };
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
