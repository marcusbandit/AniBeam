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
use slint::{ComponentHandle, Image, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};

use crate::worker::{self, Command, Commands, Event};
use crate::{AppWindow, DiskRow, NextUpRow, TorrentRow};

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

struct App {
    window: AppWindow,
    commands: Commands,
    cache: Option<Cache>,
    anilist: AniListCache,
    address: HostAddress,
    posters: HashMap<u64, Image>,
    reach: Reach,
    last_attempt: i64,
    next_up: Rc<VecModel<NextUpRow>>,
    disk: Rc<VecModel<DiskRow>>,
    torrents: Rc<VecModel<TorrentRow>>,
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

/// Puts a model's rows in place, touching only the rows that changed, so a once-a-second
/// countdown repaints the text that moved and not the whole list.
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
                self.refresh_disk();
            }
            Event::Poster(id, buffer) => {
                self.posters.insert(id, Image::from_rgba8(buffer));
                self.refresh_disk();
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
                match result {
                    Ok(text) => {
                        w.set_test_tone(0);
                        w.set_test_result(text.into());
                    }
                    Err(text) => {
                        w.set_test_tone(1);
                        w.set_test_result(text.into());
                    }
                }
            }
        }
        self.tick();
    }

    fn refresh_disk(&mut self) {
        update(&self.disk, crate::rows::disk(self.cache.as_ref(), &self.posters));
        let (torrents, known) = crate::rows::torrents(self.cache.as_ref());
        update(&self.torrents, torrents);
        self.window.set_torrents_known(known);
        let problems: Vec<SharedString> = self
            .cache
            .iter()
            .flat_map(|c| c.snapshot.problems.iter().map(|p| SharedString::from(p.as_str())))
            .collect();
        self.window.set_problems(ModelRc::new(VecModel::from(problems)));
        self.window.set_has_data(self.cache.is_some());
    }

    /// Everything that reads the clock: countdowns, "synced 1 min ago", freshness.
    fn tick(&mut self) {
        let now = anibeam_companion::now();
        update(&self.next_up, crate::rows::next_up(self.cache.as_ref(), &self.anilist, &self.posters, now));

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

        let live = matches!(self.reach, Reach::Online) && synced.is_some_and(|at| now - at < LIVE_FOR);
        w.set_live(live);
        w.set_freshness(
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
    let store = Store::open(&data_dir)?;
    let settings = store.settings();
    let address = HostAddress::parse(&settings.address)
        .unwrap_or_else(|_| HostAddress::parse(anibeam_companion::address::DEFAULT_ADDRESS).unwrap());

    let window = AppWindow::new()?;
    window.set_version(
        format!("AniBeam for Android {} (build {})", env!("CARGO_PKG_VERSION"), env!("ANIBEAM_VERSION_CODE")).into(),
    );
    window.set_address(address.display().into());

    let next_up = Rc::new(VecModel::default());
    let disk = Rc::new(VecModel::default());
    let torrents = Rc::new(VecModel::default());
    window.set_next_up(next_up.clone().into());
    window.set_disk(disk.clone().into());
    window.set_torrents(torrents.clone().into());

    window.on_refresh({
        let commands = commands.clone();
        move || {
            let _ = commands.send(Command::Sync);
        }
    });
    window.on_save_and_sync(|text| with_app(|app| app.save_and_sync(&text)));
    window.on_test_connection(|text| with_app(|app| app.test(&text)));

    let mut app = App {
        window: window.clone_strong(),
        commands: commands.clone(),
        cache: store.cache(),
        anilist: store.anilist(),
        address,
        posters: HashMap::new(),
        reach: Reach::NotYet,
        last_attempt: 0,
        next_up,
        disk,
        torrents,
    };
    app.refresh_disk();
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
