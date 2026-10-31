//! Shell: what QML needs to know about this run before anything else exists. The version,
//! the --shoot arguments and, from Task 7 on, the page a shoot opens, and from Task 9 on
//! the JSON props that page opens with. A --drive run also carries its script's text and
//! the directory its shots land in, and the input invokables the QML driver sends through.
//! The url of shell.conf, in the config directory, rides along for the few things QML keeps
//! for itself (the library's tracker nudge, once dismissed, and the versions What's new has
//! shown), so a --root run keeps them inside its sandbox too. What's new reads the built-in
//! CHANGELOG.md through here, and the check for a newer release starts here, as does
//! installing it.
//!
//! It is also where the session bus starts, because this object exists exactly once and
//! exists as soon as the engine loads Main.qml. Its thread handle is what the
//! org.freedesktop.Application interface and MPRIS's Raise and Quit queue their work on.

use core::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use anibeam_core::update::Update;
use anibeam_core::update::install::{self, Install, Prepared};

use cxx_qt::Threading;
use cxx_qt_lib::{QJsonObject, QString};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, version)]
        #[qproperty(QString, release)]
        #[qproperty(QString, build)]
        #[qproperty(QString, own_track)]
        #[qproperty(QString, shoot)]
        #[qproperty(QString, page)]
        #[qproperty(QString, props)]
        #[qproperty(i32, shoot_width)]
        #[qproperty(i32, shoot_height)]
        #[qproperty(QString, drive)]
        #[qproperty(QString, drive_out)]
        #[qproperty(QString, settings_url)]
        type Shell = super::ShellRust;

        /// A second launch, or MPRIS Raise, asked for the window; the token is the launcher's.
        #[qsignal]
        fn activate_requested(self: Pin<&mut Self>, token: QString);
        /// MPRIS Quit. The window closes itself; nothing here ends the process.
        #[qsignal]
        fn quit_requested(self: Pin<&mut Self>);
        /// A desktop build newer than this one is out on `track`: its version, its page and
        /// how this copy takes it. `how` is `install` (installUpdate does it), `command`
        /// (the package manager does it, with `command`), `source` (a build from a
        /// checkout) or `page` (the release has no file for this copy).
        #[qsignal]
        fn update_available(
            self: Pin<&mut Self>,
            version: QString,
            url: QString,
            track: QString,
            how: QString,
            command: QString,
        );
        /// How far installUpdate's download has come, in bytes.
        #[qsignal]
        fn update_progress(self: Pin<&mut Self>, got: f64, total: f64);
        /// installUpdate is done: a restart opens the new version, and a quit leaves it for
        /// the next start.
        #[qsignal]
        fn update_ready(self: Pin<&mut Self>);
        /// installUpdate failed, in a sentence for the person.
        #[qsignal]
        fn update_failed(self: Pin<&mut Self>, message: QString);
        /// Raises `window` (the QML Window) with the xdg-activation token.
        #[qinvokable]
        unsafe fn raise_window(self: &Self, window: *mut QObject, token: &QString);
        /// --drive: a mouse event at window coordinates (see cpp/helpers.h for `kind`).
        #[qinvokable]
        unsafe fn drive_mouse(
            self: &Self,
            window: *mut QObject,
            kind: i32,
            x: f64,
            y: f64,
            button: i32,
            modifiers: i32,
        );
        /// --drive: a wheel event; 120 is one notch.
        #[qinvokable]
        unsafe fn drive_wheel(
            self: &Self,
            window: *mut QObject,
            x: f64,
            y: f64,
            dx: i32,
            dy: i32,
            modifiers: i32,
        );
        /// --drive: a key press or release with the text it types.
        #[qinvokable]
        unsafe fn drive_key(
            self: &Self,
            window: *mut QObject,
            press: bool,
            key: i32,
            modifiers: i32,
            text: &QString,
        );
        /// --drive: one line to stderr, bare, with none of the QML logger's prefixes.
        #[qinvokable]
        fn drive_log(self: &Self, line: &QString);
        /// The pointer hidden over the whole application (the player with its chrome
        /// gone), or shown again.
        #[qinvokable]
        fn set_cursor_hidden(self: &Self, hidden: bool);
        /// A link opened in the desktop's browser (or a path in its file manager) through
        /// xdg-open, bypassing Qt's Wayland activation-token wait, which never returned on
        /// Hyprland. False when xdg-open could not start.
        #[qinvokable]
        fn open_url(self: &Self, url: &QString) -> bool;
        /// What's new for this build, from the two versions QML keeps in shell.conf: whether
        /// it opens by itself, what to keep now and once it closes, and what it shows.
        #[qinvokable]
        fn whats_new(self: &Self, seen: &QString, from: &QString) -> QJsonObject;
        /// Asks GitHub whether a newer desktop build is out on `track` (`stable` or
        /// `nightly`, empty for this build's own); updateAvailable says so. No answer, or no
        /// newer build, says nothing.
        #[qinvokable]
        fn check_for_update(self: &Self, track: &QString);
        /// Downloads and readies `version`, an update updateAvailable offered with
        /// `install`. updateFailed answers when it cannot.
        #[qinvokable]
        fn install_update(self: &Self, version: &QString);
        /// Stops installUpdate's download. Nothing answers; QML has already moved on.
        #[qinvokable]
        fn cancel_update(self: &Self);
        /// The next exit opens the readied update; QML quits the app right after.
        #[qinvokable]
        fn restart_for_update(self: &Self);
    }

    impl cxx_qt::Threading for Shell {}
    impl cxx_qt::Initialize for Shell {}
}

pub struct ShellRust {
    version: QString,
    release: QString,
    build: QString,
    own_track: QString,
    shoot: QString,
    page: QString,
    props: QString,
    shoot_width: i32,
    shoot_height: i32,
    drive: QString,
    drive_out: QString,
    settings_url: QString,
}

impl Default for ShellRust {
    fn default() -> Self {
        let a = crate::runtime::args();
        let shooting = a.shoot.is_some() || a.drive.is_some();
        // main has already read and checked the script and left on a bad one, so a failure
        // here is the file changing underfoot; the driver reports an empty script as one.
        let drive = a
            .drive
            .as_deref()
            .map(|p| crate::args::load_drive(std::path::Path::new(p)).unwrap_or_default())
            .unwrap_or_default();
        let drive_out = if a.drive.is_some() {
            crate::args::drive_out(a.out.as_deref(), std::env::var("ANIBEAM_CAPTURES").ok())
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };
        ShellRust {
            version: QString::from(anibeam_core::VERSION),
            // The rail and About show the version number alone, never the commit.
            release: QString::from(env!("CARGO_PKG_VERSION")),
            // The whole name a shipped build carries, 2.4.0-nightly.20261030.12, for About.
            build: QString::from(anibeam_core::RELEASE),
            own_track: QString::from(
                match anibeam_core::update::Track::of(anibeam_core::RELEASE) {
                    anibeam_core::update::Track::Nightly => "nightly",
                    anibeam_core::update::Track::Stable => "stable",
                },
            ),
            shoot: QString::from(a.shoot.as_deref().unwrap_or("")),
            page: QString::from(a.page.as_deref().unwrap_or("home")),
            props: QString::from(a.props.as_deref().unwrap_or("")),
            shoot_width: if shooting { a.width as i32 } else { 0 },
            shoot_height: if shooting { a.height as i32 } else { 0 },
            drive: QString::from(drive.as_str()),
            drive_out: QString::from(drive_out.as_str()),
            settings_url: settings_url(&crate::runtime::paths().config_dir()),
        }
    }
}

/// shell.conf as a file url QtCore's Settings can take as its `location`. Made absolute
/// first, since a relative --root is relative to where the shell was started, and encoded
/// by Qt rather than by string concatenation, so a `#`, `?` or `%` in the path is part of the
/// path and not the start of a fragment, a query or an escape.
fn settings_url(config_dir: &std::path::Path) -> QString {
    let file = config_dir.join("shell.conf");
    let file = std::path::absolute(&file).unwrap_or(file);
    let url = cxx_qt_lib::QUrl::from_local_file(&QString::from(file.to_string_lossy().as_ref()));
    QString::from(String::from_utf8_lossy(url.to_encoded().as_slice()).as_ref())
}

/// The desktop's notes, built in. The core's tests hold the newest heading to the workspace
/// version, which is this crate's.
const NOTES: &str = include_str!("../../CHANGELOG.md");

impl qobject::Shell {
    pub fn whats_new(&self, seen: &QString, from: &QString) -> QJsonObject {
        use anibeam_core::changelog::{self, Seen};
        let mut seen = Seen {
            version: seen.to_string(),
            from: from.to_string(),
        };
        let current = env!("CARGO_PKG_VERSION");
        // An install from before What's new kept a record: the version its core last logged
        // stands in for the notes it last read.
        let earlier = crate::runtime::earlier();
        if seen.version.is_empty()
            && let Some(version) = earlier.version
        {
            seen.version = version;
        }
        let used = earlier.started;
        let notes = changelog::parse(NOTES);
        let state = serde_json::json!({
            // A version main has only just moved to has no lines yet; opening it would mark
            // it read before its notes are written.
            "due": seen.due(current, used) && changelog::has_lines(&notes, current),
            "started": seen.started(current, used),
            "closed": seen.closed(current),
            "view": changelog::whats_new(&notes, current, &seen, used),
        });
        crate::json::to_qjson_object(&state)
    }
}

/// How this copy was installed, read once, off the async workers since it may ask pacman.
/// Then every update a check offered, by version, so installUpdate takes the one the
/// notice shows even when a check for a track since left answers later; what installUpdate
/// left ready; and whether the exit opens it.
static INSTALL: OnceLock<Install> = OnceLock::new();
static OFFERED: Mutex<Vec<Update>> = Mutex::new(Vec::new());
static PREPARED: Mutex<Option<Prepared>> = Mutex::new(None);
static INSTALLING: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);
static RESTART: AtomicBool = AtomicBool::new(false);

/// Called by main as the last thing before exit: finishes a readied update, and opens it
/// when the person asked for a restart.
pub fn finish_update() {
    let prepared = PREPARED.lock().ok().and_then(|mut p| p.take());
    if let Some(prepared) = prepared
        && let Err(e) = prepared.finish(RESTART.load(Ordering::SeqCst))
    {
        eprintln!("anibeam: update: {e}");
    }
}

impl qobject::Shell {
    pub fn check_for_update(&self, track: &QString) {
        use anibeam_core::update::Track;
        let track = Track::parse(&track.to_string()).unwrap_or(Track::of(anibeam_core::RELEASE));
        let asked = match track {
            Track::Nightly => "nightly",
            Track::Stable => "stable",
        };
        // A debug build can pretend to be an older version, to try an update against a
        // real release
        let current = match std::env::var("ANIBEAM_PRETEND_VERSION") {
            Ok(v) if cfg!(debug_assertions) => v,
            _ => anibeam_core::RELEASE.to_string(),
        };
        let shell = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            let http = match anibeam_core::net::ReqwestHttp::new(std::time::Duration::from_secs(15))
            {
                Ok(http) => http,
                Err(e) => return eprintln!("anibeam: update check: {e:?}"),
            };
            let install = match INSTALL.get() {
                Some(install) => install.clone(),
                None => {
                    let found = tokio::task::spawn_blocking(Install::detect)
                        .await
                        .unwrap_or(Install::Source);
                    INSTALL.get_or_init(|| found).clone()
                }
            };
            let offer = |version: &str, url: &str, how: &'static str, command: String| {
                let (version, url) = (QString::from(version), QString::from(url));
                let _ = shell.queue(move |shell| {
                    shell.update_available(
                        version,
                        url,
                        QString::from(asked),
                        QString::from(how),
                        QString::from(&command),
                    )
                });
            };
            // A build the system's package manager installed is updated by that package
            // manager. The AUR's is offered once the AUR has it, with the command that
            // fetches it; one pacman does not know is left alone.
            if let Install::Package { package } = &install {
                let Some(package) = package else { return };
                match anibeam_core::update::check_aur(&http, package, &current).await {
                    Ok(Some(version)) => {
                        let name = package.clone();
                        let command =
                            tokio::task::spawn_blocking(move || install::aur_command(&name))
                                .await
                                .unwrap_or_default();
                        offer(
                            &version,
                            &format!("https://aur.archlinux.org/packages/{package}"),
                            "command",
                            command,
                        )
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("anibeam: update check: {e:?}"),
                }
                return;
            }
            let update = match anibeam_core::update::check(&http, &current, track).await {
                Ok(Some(update)) => update,
                Ok(None) => return,
                Err(e) => return eprintln!("anibeam: update check: {e:?}"),
            };
            let how = if install.asset(&update.assets).is_some() {
                "install"
            } else if install == Install::Source {
                "source"
            } else {
                "page"
            };
            if let Ok(mut offered) = OFFERED.lock() {
                offered.retain(|u| u.version != update.version);
                offered.push(update.clone());
            }
            offer(&update.version, &update.url, how, String::new());
        });
    }

    pub fn install_update(&self, version: &QString) {
        let version = version.to_string();
        let found = OFFERED
            .lock()
            .ok()
            .and_then(|o| o.iter().find(|u| u.version == version).cloned());
        let (Some(install), Some(update)) = (INSTALL.get().cloned(), found) else {
            return self.fail(format!("AniBeam {version} is no longer on offer"));
        };
        // A click while one runs, even one just cancelled, carries on with that download
        CANCEL.store(false, Ordering::SeqCst);
        if INSTALLING.swap(true, Ordering::SeqCst) {
            return;
        }
        let shell = self.qt_thread();
        crate::runtime::runtime().spawn(async move {
            // This process's own, so another instance or a sandbox run never empties it
            let work = std::env::temp_dir().join(format!("anibeam-update-{}", std::process::id()));
            let tell = shell.clone();
            let progress = move |got: u64, total: u64| {
                let _ = tell.queue(move |shell| shell.update_progress(got as f64, total as f64));
            };
            let result = install::fetch(&install, &update, &work, &CANCEL, progress).await;
            INSTALLING.store(false, Ordering::SeqCst);
            match result {
                Ok(prepared) => {
                    if let Ok(mut p) = PREPARED.lock() {
                        *p = Some(prepared);
                    }
                    let _ = shell.queue(|shell| shell.update_ready());
                }
                Err(_) if CANCEL.load(Ordering::SeqCst) => {}
                Err(message) => {
                    eprintln!("anibeam: update: {message}");
                    let _ = shell.queue(move |shell| shell.update_failed(QString::from(&message)));
                }
            }
        });
    }

    fn fail(&self, message: String) {
        let _ = self
            .qt_thread()
            .queue(move |shell| shell.update_failed(QString::from(&message)));
    }

    pub fn cancel_update(&self) {
        CANCEL.store(true, Ordering::SeqCst);
    }

    pub fn restart_for_update(&self) {
        RESTART.store(true, Ordering::SeqCst);
    }
}

impl cxx_qt::Initialize for qobject::Shell {
    fn initialize(self: Pin<&mut Self>) {
        // The bus name and MPRIS start once the Shell exists, which is once the engine
        // loads. A run whose --root lets it start beside a real one, a --shoot, finds both
        // names taken and says so; it draws its page either way. The Mac starts the bus
        // name alone: its Now Playing starts with the page's first publish, on this thread,
        // which is the one MediaPlayer wants and never the runtime's (macnp.rs).
        let shell = self.qt_thread();
        #[cfg(not(target_os = "macos"))]
        crate::runtime::runtime().spawn(async move {
            if let Some(h) = crate::dbus::mpris::start(shell).await {
                crate::dbus::mpris::install(h)
            }
        });
        #[cfg(target_os = "macos")]
        crate::runtime::runtime().spawn(crate::dbus::instance::start(shell));
    }
}

impl qobject::Shell {
    /// # Safety
    /// `window` is a live QObject: QML hands its own Window here and the call returns
    /// before that Window can be destroyed.
    pub unsafe fn raise_window(&self, window: *mut cxx_qt::QObject, token: &QString) {
        // A panic crossing the FFI aborts the process, so it stops here instead.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            crate::bridge::helpers::ffi::raise_window(window, token);
        }));
        if result.is_err() {
            eprintln!("anibeam: shell: raiseWindow panicked");
        }
    }
}

impl qobject::Shell {
    /// # Safety
    /// `window` is the live QML Window, as for `raise_window`.
    pub unsafe fn drive_mouse(
        &self,
        window: *mut cxx_qt::QObject,
        kind: i32,
        x: f64,
        y: f64,
        button: i32,
        modifiers: i32,
    ) {
        unsafe { crate::bridge::helpers::ffi::drive_mouse(window, kind, x, y, button, modifiers) }
    }

    /// # Safety
    /// `window` is the live QML Window, as for `raise_window`.
    pub unsafe fn drive_wheel(
        &self,
        window: *mut cxx_qt::QObject,
        x: f64,
        y: f64,
        dx: i32,
        dy: i32,
        modifiers: i32,
    ) {
        unsafe { crate::bridge::helpers::ffi::drive_wheel(window, x, y, dx, dy, modifiers) }
    }

    /// # Safety
    /// `window` is the live QML Window, as for `raise_window`.
    pub unsafe fn drive_key(
        &self,
        window: *mut cxx_qt::QObject,
        press: bool,
        key: i32,
        modifiers: i32,
        text: &QString,
    ) {
        unsafe { crate::bridge::helpers::ffi::drive_key(window, press, key, modifiers, text) }
    }

    pub fn drive_log(&self, line: &QString) {
        eprintln!("{line}");
    }
    pub fn set_cursor_hidden(&self, hidden: bool) {
        crate::bridge::helpers::ffi::set_cursor_hidden(hidden);
    }
    pub fn open_url(&self, url: &QString) -> bool {
        crate::bridge::helpers::ffi::open_url(url)
    }
}

#[cfg(test)]
mod tests {
    /// The url is absolute whatever the root was given as, and the characters a url
    /// would read as delimiters are escaped into the path.
    #[test]
    fn the_settings_url_is_absolute_and_escaped() {
        let url = super::settings_url(std::path::Path::new("/tmp/a #b?c%d/anibeam")).to_string();
        // Windows puts the current drive in front: file:///D:/tmp/...
        let tail = "/tmp/a%20%23b%3Fc%25d/anibeam/shell.conf";
        if cfg!(windows) {
            assert!(url.starts_with("file:///") && url.ends_with(tail), "{url}");
        } else {
            assert_eq!(url, format!("file://{tail}"));
        }
        let relative = super::settings_url(std::path::Path::new("rel/anibeam")).to_string();
        let cwd = std::env::current_dir().unwrap();
        assert!(relative.starts_with("file:///"), "{relative}");
        assert!(relative.ends_with("/rel/anibeam/shell.conf"), "{relative}");
        assert!(
            relative.contains(cwd.file_name().unwrap().to_str().unwrap()),
            "{relative}"
        );
    }
}
