//! `anibeam serve`: the core and its tailnet host with no window and no player, for a
//! machine other devices connect to (#381). It runs until SIGTERM or SIGINT, which is what
//! `systemctl --user stop anibeam-host` sends.
//!
//! The service and the window share one database, so only one of them has the core open at
//! a time, and the single-instance lock is how that is kept. The service takes the lock and
//! serves. A window launched while it does asks it to step aside over the service socket:
//! the service closes its core, lets the lock go and says `ok`, and the window opens the
//! library in its place, holding the host on as the service would. The window keeps that
//! connection for the life of its process, so the socket closing is the service's sign to
//! wait for the lock again; it serves once the window is gone. A service started while a
//! window is already open waits for the lock the same way.
//!
//! A window that was open before the service started never asks, so the service tells it
//! over the window's own org.freedesktop.Application: `hold-host` while it waits, so the
//! devices keep their host, and `release-host` when it stops waiting without taking over.
//!
//! The socket speaks one line each way. `status` answers `serving` or `waiting`; `yield`
//! answers `ok` once the lock is free, or `waiting` when the service does not hold it. A
//! second lock beside the socket keeps a second service from starting at all.

#[cfg(unix)]
pub use unix::{ask_to_yield, run, status};

/// The actions the service sends a running window: keep the host on in the service's
/// place, or hand it back to the preference.
pub const HOLD_HOST: &str = "hold-host";
pub const RELEASE_HOST: &str = "release-host";

/// The exit code for a second service, which the unit does not restart.
pub const ALREADY_RUNNING: i32 = 3;

/// What a running service is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// It holds the library and the host is up.
    Serving,
    /// A window has the library; the service takes it back when the window closes.
    Waiting,
}

impl State {
    pub fn word(self) -> &'static str {
        match self {
            State::Serving => "serving",
            State::Waiting => "waiting",
        }
    }
}

#[cfg(not(unix))]
pub fn run() -> i32 {
    eprintln!("anibeam: serve needs a Unix socket, and this platform has none");
    2
}

#[cfg(not(unix))]
pub fn status(_socket: &std::path::Path) -> Option<State> {
    None
}

#[cfg(not(unix))]
pub fn ask_to_yield(_socket: &std::path::Path) -> Option<()> {
    None
}

#[cfg(unix)]
mod unix {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::time::Duration;

    use anibeam_core::{Call, Core, CoreError, Reply, TorrentMode};

    use super::State;
    use crate::dbus::instance::{self, Lock};
    use crate::runtime;

    /// How long one side waits for the other's line. A yield waits longer: the service
    /// closes its core first, which gives its tasks a couple of seconds to finish.
    /// A status read runs on the window's GUI thread, so it gets far less than the yield.
    const ANSWER: Duration = Duration::from_millis(500);
    const YIELD: Duration = Duration::from_secs(15);
    /// The room in `sun_path`, terminator included: 108 bytes on Linux, 104 on a Mac.
    const SOCKET_PATH_MAX: usize = if cfg!(target_os = "macos") { 104 } else { 108 };

    /// What the service tells a window it is waiting on: `true` to hold the host, `false`
    /// to let it go.
    pub type TellWindow = Arc<dyn Fn(bool) + Send + Sync>;

    enum Event {
        /// The lock is ours: open the core and serve.
        Locked(Lock),
        /// A window wants the library.
        Yield(UnixStream),
        /// The window that took the library has closed its end.
        Gone,
        /// SIGTERM or SIGINT.
        Stop,
    }

    /// One question, one line back.
    fn ask(socket: &Path, question: &str, wait: Duration) -> Option<(String, UnixStream)> {
        let mut conn = UnixStream::connect(socket).ok()?;
        conn.set_read_timeout(Some(wait)).ok()?;
        conn.write_all(format!("{question}\n").as_bytes()).ok()?;
        let mut line = String::new();
        BufReader::new(&conn).read_line(&mut line).ok()?;
        // Only a yield's connection is read again. A Mac refuses this with EINVAL once the
        // service has closed its end, which a status answer always has.
        let _ = conn.set_read_timeout(None);
        Some((line.trim().to_string(), conn))
    }

    /// What the service on this socket is doing, or `None` when none answers.
    pub fn status(socket: &Path) -> Option<State> {
        match ask(socket, "status", ANSWER)?.0.as_str() {
            "serving" => Some(State::Serving),
            "waiting" => Some(State::Waiting),
            _ => None,
        }
    }

    /// Asks the service to let the library go. `Some` once it has: the lock is free, and the
    /// caller holds the connection until it is done with the library.
    pub fn ask_to_yield(socket: &Path) -> Option<UnixStream> {
        match ask(socket, "yield", YIELD)? {
            (answer, conn) if answer == "ok" => Some(conn),
            _ => None,
        }
    }

    /// The service's whole life. Returns the exit code.
    pub fn run() -> i32 {
        // A setenv, so before the core's runtime starts a thread, as in `main`.
        crate::paths::name_bundled_helpers();
        let paths = runtime::paths();
        let tell: TellWindow = Arc::new(|hold| {
            let action = if hold {
                super::HOLD_HOST
            } else {
                super::RELEASE_HOST
            };
            // No window, or no session bus: nobody to tell, which is fine.
            let _ = runtime::runtime().block_on(instance::hand_off(Some(action), None));
        });
        let service = match Service::bind(paths.lock_path(), paths.service_socket(), tell) {
            Ok(s) => s,
            Err((code, e)) => {
                eprintln!("anibeam: {e}");
                return code;
            }
        };
        watch_signals(service.stopper());
        eprintln!(
            "anibeam: host service on {}",
            paths.service_socket().display()
        );
        let sandbox = runtime::args().root.is_some();
        service.run(|| open(sandbox))
    }

    /// The socket bound and the loop's channel, before the lock is asked for.
    pub struct Service {
        lock_path: PathBuf,
        socket: PathBuf,
        tx: Sender<Event>,
        rx: Receiver<Event>,
        serving: Arc<AtomicBool>,
        tell: TellWindow,
        /// The service's own lock, held for its whole life.
        _own: Lock,
    }

    /// Ends a running service, from a signal handler or a test.
    pub struct Stopper(Sender<Event>);

    impl Stopper {
        pub fn stop(&self) {
            let _ = self.0.send(Event::Stop);
        }
    }

    impl Service {
        /// Takes the service's own lock, binds the socket and starts answering on it. Fails
        /// with `ALREADY_RUNNING` when another service holds that lock.
        pub fn bind(
            lock_path: PathBuf,
            socket: PathBuf,
            tell: TellWindow,
        ) -> Result<Service, (i32, String)> {
            let own_path = socket.with_file_name("anibeam-host.lock");
            let own = match instance::try_lock(&own_path) {
                Ok(Some(lock)) => lock,
                Ok(None) => {
                    return Err((
                        super::ALREADY_RUNNING,
                        format!("another anibeam serve is running on {}", socket.display()),
                    ));
                }
                Err(e) => return Err((1, format!("lock {}: {e}", own_path.display()))),
            };
            let listener = bind(&socket).map_err(|e| (1, e))?;
            let (tx, rx) = mpsc::channel();
            let serving = Arc::new(AtomicBool::new(false));
            {
                let (tx, serving) = (tx.clone(), serving.clone());
                std::thread::spawn(move || answer(listener, tx, serving));
            }
            Ok(Service {
                lock_path,
                socket,
                tx,
                rx,
                serving,
                tell,
                _own: own,
            })
        }

        pub fn stopper(&self) -> Stopper {
            Stopper(self.tx.clone())
        }

        /// Takes the lock, opens the core with `open` and serves until stopped, stepping
        /// aside for each window that asks. Returns the exit code.
        pub fn run(self, open: impl Fn() -> Result<Arc<Core>, CoreError>) -> i32 {
            let Service {
                lock_path,
                socket,
                tx,
                rx,
                serving,
                tell,
                _own,
            } = self;
            wait_for_lock(&lock_path, &tx, &tell);
            let mut held: Option<(Arc<Core>, Lock)> = None;
            let code = loop {
                let Ok(event) = rx.recv() else { break 0 };
                match event {
                    Event::Locked(lock) => match open() {
                        Ok(core) => {
                            held = Some((core, lock));
                            serving.store(true, Ordering::SeqCst);
                            eprintln!("anibeam: the library is open and the host is on");
                        }
                        Err(e) => {
                            eprintln!("anibeam: {e}");
                            break 2;
                        }
                    },
                    Event::Yield(mut conn) => match held.take() {
                        Some((core, lock)) => {
                            serving.store(false, Ordering::SeqCst);
                            core.shutdown();
                            drop(core);
                            drop(lock);
                            eprintln!(
                                "anibeam: a window took the library; waiting for it to close"
                            );
                            if conn.write_all(b"ok\n").is_ok() {
                                let tx = tx.clone();
                                std::thread::spawn(move || {
                                    // Any byte or the end: the window has nothing to send.
                                    let _ = conn.read(&mut [0u8; 1]);
                                    let _ = tx.send(Event::Gone);
                                });
                            } else {
                                wait_for_lock(&lock_path, &tx, &tell);
                            }
                        }
                        None => {
                            let _ = conn.write_all(b"waiting\n");
                        }
                    },
                    Event::Gone => wait_for_lock(&lock_path, &tx, &tell),
                    Event::Stop => break 0,
                }
            };
            match held.take() {
                Some((core, lock)) => {
                    core.shutdown();
                    drop(core);
                    drop(lock);
                }
                // A window may be holding the host on the service's behalf.
                None => tell(false),
            }
            let _ = std::fs::remove_file(&socket);
            eprintln!("anibeam: stopped");
            code
        }
    }

    /// Binds the socket. The caller holds the service's own lock, so a file already there was
    /// left by a service that died, and is cleared.
    fn bind(socket: &Path) -> Result<UnixListener, String> {
        if let Some(dir) = socket.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        // $XDG_RUNTIME_DIR always fits; a deep --root may not.
        if socket.as_os_str().len() >= SOCKET_PATH_MAX {
            return Err(format!(
                "{} is too long for a socket; pick a shorter --root",
                socket.display()
            ));
        }
        let _ = std::fs::remove_file(socket);
        UnixListener::bind(socket).map_err(|e| format!("{}: {e}", socket.display()))
    }

    /// Takes the lock. A window holding it is told to keep the host on, here on the loop's
    /// thread so a later `release-host` can never overtake it, and a thread of its own then
    /// blocks until that window is gone.
    fn wait_for_lock(path: &Path, tx: &Sender<Event>, tell: &TellWindow) {
        let (path, tx): (PathBuf, _) = (path.to_path_buf(), tx.clone());
        match instance::try_lock(&path) {
            Ok(Some(lock)) => {
                let _ = tx.send(Event::Locked(lock));
                return;
            }
            Ok(None) => tell(true),
            Err(e) => {
                eprintln!("anibeam: lock {}: {e}", path.display());
                let _ = tx.send(Event::Stop);
                return;
            }
        }
        std::thread::spawn(move || match instance::wait_lock(&path) {
            Ok(lock) => {
                let _ = tx.send(Event::Locked(lock));
            }
            Err(e) => {
                eprintln!("anibeam: lock {}: {e}", path.display());
                let _ = tx.send(Event::Stop);
            }
        });
    }

    /// Opens the core, holds the host on and starts it. Downloads that were set up come up
    /// with it, so AniBeam's own qBittorrent keeps fetching what the subscriptions find; a
    /// sandbox root leaves the owner's real clients alone, as the window does.
    fn open(sandbox: bool) -> Result<Arc<Core>, CoreError> {
        let core = runtime::open_core()?;
        core.hold_host(true);
        core.start()?;
        if !sandbox
            && let Ok(Reply::Downloads { status }) = core.call(Call::GetDownloads)
            && status.settings.torrent_mode != TorrentMode::Auto
        {
            let _ = core.call(Call::ConnectDownloads);
        }
        Ok(core)
    }

    /// One thread per connection: read the question, answer it or hand it to the loop.
    fn answer(listener: UnixListener, tx: Sender<Event>, serving: Arc<AtomicBool>) {
        for conn in listener.incoming() {
            let mut conn = match conn {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("anibeam: the service socket could not accept: {e}");
                    // Out of descriptors, most likely: give some back before the next try.
                    std::thread::sleep(ANSWER);
                    continue;
                }
            };
            let (tx, serving) = (tx.clone(), serving.clone());
            std::thread::spawn(move || {
                let _ = conn.set_read_timeout(Some(ANSWER));
                let mut line = String::new();
                let Ok(reader) = conn.try_clone() else { return };
                if BufReader::new(reader).read_line(&mut line).is_err() {
                    return;
                }
                let _ = conn.set_read_timeout(None);
                match line.trim() {
                    "status" => {
                        let state = match serving.load(Ordering::SeqCst) {
                            true => State::Serving,
                            false => State::Waiting,
                        };
                        let _ = conn.write_all(format!("{}\n", state.word()).as_bytes());
                    }
                    "yield" => {
                        let _ = tx.send(Event::Yield(conn));
                    }
                    _ => {}
                }
            });
        }
    }

    fn watch_signals(stopper: Stopper) {
        use tokio::signal::unix::{SignalKind, signal};
        runtime::runtime().spawn(async move {
            let (Ok(mut term), Ok(mut int)) = (
                signal(SignalKind::terminate()),
                signal(SignalKind::interrupt()),
            ) else {
                eprintln!("anibeam: no signal handler; stop the service with SIGKILL");
                return;
            };
            tokio::select! {
                _ = term.recv() => {}
                _ = int.recv() => {}
            }
            stopper.stop();
        });
    }

    /// The service against a sandbox core, driven from the window's side by hand: the socket
    /// line and the flock the real window takes. The cores bind the host on a loopback port
    /// of their own, where the env var would be shared by every test in the process.
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs::{File, OpenOptions};
        use std::net::{SocketAddr, TcpListener, TcpStream};
        use std::time::Instant;

        use anibeam_core::CorePaths;
        use anibeam_core::trackers::Secrets;

        struct Running {
            stopper: Stopper,
            done: std::thread::JoinHandle<i32>,
        }

        fn free_addr() -> SocketAddr {
            TcpListener::bind("127.0.0.1:0")
                .unwrap()
                .local_addr()
                .unwrap()
        }

        fn lock_path(root: &Path) -> PathBuf {
            root.join("runtime").join("anibeam.lock")
        }

        fn socket(root: &Path) -> PathBuf {
            root.join("runtime").join("anibeam-host.sock")
        }

        /// What the service told the window, in order.
        type Told = Arc<std::sync::Mutex<Vec<bool>>>;

        fn bind(root: &Path, told: &Told) -> Result<Service, (i32, String)> {
            let told = told.clone();
            let tell: TellWindow = Arc::new(move |hold| told.lock().unwrap().push(hold));
            Service::bind(lock_path(root), socket(root), tell)
        }

        fn start(root: &Path, addr: SocketAddr) -> (Running, Told) {
            let told = Told::default();
            let service = bind(root, &told).unwrap();
            let stopper = service.stopper();
            let root = root.to_path_buf();
            let done = std::thread::spawn(move || {
                service.run(move || {
                    let paths = CorePaths::under(&root);
                    let secrets = Secrets::file_only(paths.secrets_path());
                    let core = Core::open_with_secrets(paths, secrets)?;
                    core.set_host_bind(Some(addr.to_string()));
                    core.hold_host(true);
                    core.start()?;
                    Ok(core)
                })
            });
            (Running { stopper, done }, told)
        }

        fn stop(running: Running) -> i32 {
            running.stopper.stop();
            running.done.join().unwrap()
        }

        /// The hello route's body, or `None` while nothing answers on the port.
        fn hello(addr: SocketAddr) -> Option<String> {
            let mut conn = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).ok()?;
            conn.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
            let ask = format!("GET /api/v1/hello HTTP/1.0\r\nHost: {addr}\r\n\r\n");
            conn.write_all(ask.as_bytes()).ok()?;
            let mut reply = String::new();
            conn.read_to_string(&mut reply).ok()?;
            reply
                .lines()
                .next()
                .is_some_and(|l| l.starts_with("HTTP/1.") && l.contains(" 200 "))
                .then(|| reply.split("\r\n\r\n").nth(1).unwrap_or("").to_string())
        }

        fn state(root: &Path) -> Option<State> {
            let answer = status(&socket(root));
            if answer.is_none() {
                eprintln!("no status: {}", probe(&socket(root)));
            }
            answer
        }

        /// Why a status read came back empty, for a failure on a machine the test cannot
        /// be run on by hand.
        fn probe(socket: &Path) -> String {
            let mut conn = match UnixStream::connect(socket) {
                Ok(c) => c,
                Err(e) => return format!("connect {}: {e}", socket.display()),
            };
            let _ = conn.set_read_timeout(Some(Duration::from_secs(5)));
            if let Err(e) = conn.write_all(b"status\n") {
                return format!("write: {e}");
            }
            let mut line = String::new();
            match BufReader::new(&conn).read_line(&mut line) {
                Ok(_) => format!("answered {line:?} given 5 s"),
                Err(e) => format!("read: {e}"),
            }
        }

        /// Polls `check` until it holds: the service and its host work on threads of their
        /// own, a beat after they are asked.
        fn eventually(what: &str, check: impl Fn() -> bool) {
            let deadline = Instant::now() + Duration::from_secs(20);
            while !check() {
                assert!(Instant::now() < deadline, "never: {what}");
                std::thread::sleep(Duration::from_millis(50));
            }
        }

        fn open_lock(root: &Path) -> File {
            std::fs::create_dir_all(root.join("runtime")).unwrap();
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(lock_path(root))
                .unwrap()
        }

        #[test]
        fn the_service_serves_steps_aside_for_a_window_and_stops_cleanly() {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            let addr = free_addr();
            let (running, told) = start(root, addr);

            eventually("the host answers", || hello(addr).is_some());
            let body = hello(addr).unwrap();
            assert!(body.contains("\"app\":\"anibeam\""), "{body}");
            assert_eq!(state(root), Some(State::Serving));
            assert!(
                open_lock(root).try_lock().is_err(),
                "the service holds the lock"
            );

            // A second service on the same socket is refused and says why.
            let (code, second) = bind(root, &Told::default()).err().unwrap();
            assert_eq!(code, crate::serve::ALREADY_RUNNING);
            assert!(
                second.contains("another anibeam serve is running"),
                "{second}"
            );
            assert_eq!(state(root), Some(State::Serving), "the first is untouched");

            // A window asks for the library: the service closes its core and lets the lock
            // go before it answers. The lock can still read held for a beat after that, while
            // a process another test forks carries a copy of the service's open file (#577),
            // so the window's lock is waited for, not assumed.
            let window = ask_to_yield(&socket(root)).expect("the service steps aside");
            let window_lock = open_lock(root);
            eventually("the lock is free for the window", || {
                window_lock.try_lock().is_ok()
            });
            eventually("the service's host stops", || hello(addr).is_none());
            assert_eq!(state(root), Some(State::Waiting));
            assert!(
                ask_to_yield(&socket(root)).is_none(),
                "nothing left to give"
            );

            // The window closes, its lock and its connection with it: the service serves again.
            drop(window_lock);
            drop(window);
            eventually("the service serves again", || {
                state(root) == Some(State::Serving)
            });
            eventually("the host answers again", || hello(addr).is_some());

            assert_eq!(stop(running), 0);
            assert!(!socket(root).exists(), "the socket is cleared");
            assert!(hello(addr).is_none());
            let after = open_lock(root);
            eventually("the lock is free after stopping", || {
                after.try_lock().is_ok()
            });
            drop(after);
            assert_eq!(state(root), None);
            // The window it stepped aside for asked, so it was never told anything.
            assert!(told.lock().unwrap().is_empty());
            bind(root, &Told::default()).expect("a new service starts once this one is gone");
        }

        #[test]
        fn a_service_started_beside_an_open_window_waits_for_it() {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            let window = open_lock(root);
            window.try_lock().unwrap();

            let addr = free_addr();
            let (running, told) = start(root, addr);
            eventually("the service answers", || state(root).is_some());
            assert_eq!(state(root), Some(State::Waiting));
            assert!(hello(addr).is_none(), "the window has the library");
            eventually("the window is told to hold the host", || {
                *told.lock().unwrap() == [true]
            });

            drop(window);
            eventually("the service takes the library", || {
                state(root) == Some(State::Serving)
            });
            eventually("the host answers", || hello(addr).is_some());
            assert_eq!(stop(running), 0);
        }

        #[test]
        fn a_service_stopped_while_it_waits_exits_cleanly() {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            let window = open_lock(root);
            window.try_lock().unwrap();
            let (running, told) = start(root, free_addr());
            eventually("the service answers", || state(root).is_some());
            assert_eq!(stop(running), 0);
            assert!(!socket(root).exists());
            assert_eq!(
                *told.lock().unwrap(),
                [true, false],
                "held, then handed back"
            );
        }

        #[test]
        fn a_socket_left_by_a_dead_service_is_cleared() {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            std::fs::create_dir_all(root.join("runtime")).unwrap();
            drop(UnixListener::bind(socket(root)).unwrap());
            assert!(socket(root).exists());
            let service = bind(root, &Told::default()).expect("stale file cleared");
            drop(service);
        }
    }
}
