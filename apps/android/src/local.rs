//! The phone's own AniBeam: the same core the desktop runs, opened over the app's files.
//!
//! The core keeps its database, cache and secrets under `files/core`, and the phone's
//! library is one folder the app owns, `files/Anime`. Events go to logcat under the tag
//! `AniBeamCore` until the screens read them.

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use anibeam_core::trackers::Secrets;
use anibeam_core::events::Subscription;
use anibeam_core::{Call, Core, CorePaths, Event, EventBody, EventListener, Level, Reply};

/// Where downloads land and what the library scans.
pub const LIBRARY_DIR: &str = "Anime";

/// Set by any event that can change what a page shows; the UI's clock reads and clears it.
static CHANGED: AtomicBool = AtomicBool::new(true);

/// Whether the core said something since the last call.
pub fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::AcqRel)
}

/// Asks the pages to read the core again, after a call that changed what they show.
pub fn touch() {
    CHANGED.store(true, Ordering::Release);
}

/// The process's core, once it is open.
pub fn core() -> Option<Arc<Core>> {
    CORE.get().map(|(core, _)| core.clone())
}

struct Logcat;

impl EventListener for Logcat {
    fn on_event(&self, event: Event) {
        if let EventBody::AuthUrlReady { open_url, .. } = &event.body {
            crate::android::open_url(open_url);
        }
        crate::player::core_event(&event.body);
        // Debug events are progress chatter; the state changes the pages show come as Info
        // and above, so only those make the UI read the core again.
        if !matches!(event.level, Level::Debug) {
            log(&format!("{:?} {:?} {}", event.level, event.stage, event.message));
            CHANGED.store(true, Ordering::Release);
        }
    }
}

pub fn log(line: &str) {
    #[cfg(target_os = "android")]
    {
        use std::ffi::CString;
        unsafe extern "C" {
            fn __android_log_write(prio: i32, tag: *const std::ffi::c_char, text: *const std::ffi::c_char) -> i32;
        }
        let tag = CString::new("AniBeamCore").unwrap();
        let text = CString::new(line.replace('\0', " ")).unwrap();
        // Priority 4 is INFO.
        unsafe { __android_log_write(4, tag.as_ptr(), text.as_ptr()) };
    }
    #[cfg(not(target_os = "android"))]
    eprintln!("AniBeamCore: {line}");
}

/// The process's one core and the logcat listener on it. Android may destroy and recreate
/// the activity in the same process; the core outlives that, and later the download
/// service, so it is opened once and kept for the life of the process.
static CORE: OnceLock<(Arc<Core>, Arc<Subscription>)> = OnceLock::new();

/// Opens and starts the core on its own thread, so migrations and the first scan never hold
/// up the first frame. A second call in the same process does nothing.
pub fn start(data_dir: PathBuf) {
    let spawned = std::thread::Builder::new().name("core-open".into()).spawn(move || {
        if CORE.get().is_some() {
            return;
        }
        match open(&data_dir) {
            Ok(opened) => {
                let _ = CORE.set(opened);
                CHANGED.store(true, Ordering::Release);
            }
            Err(e) => log(&format!("core did not open: {e}")),
        }
    });
    if let Err(e) = spawned {
        log(&format!("core thread did not start: {e}"));
    }
}

fn open(data_dir: &Path) -> Result<(Arc<Core>, Arc<Subscription>), Box<dyn std::error::Error>> {
    // `/data/data/<pkg>` and `/data/user/0/<pkg>` are the same folder; one spelling keeps
    // the library from being added twice under two names.
    let data_dir = data_dir.canonicalize().unwrap_or_else(|_| data_dir.to_path_buf());
    let paths = CorePaths::under(&data_dir.join("core"));
    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_secrets(paths, secrets)?;
    let listener = core.subscribe(Arc::new(Logcat));

    let library = data_dir.join(LIBRARY_DIR);
    std::fs::create_dir_all(&library)?;
    seed(&library);
    let library = library.to_string_lossy().into_owned();
    let known = match core.call(Call::ListSources)? {
        Reply::Sources { sources } => sources.iter().any(|s| s.path == library),
        _ => false,
    };
    if !known {
        core.call(Call::AddSource { path: library })?;
    }
    core.start()?;
    log("core started");
    Ok((core, listener))
}

/// A test hook: each line of `/data/local/tmp/anibeam-seed` names a file to create, empty,
/// under the library folder, so a scan and a match can be tried without a download. A line
/// `name < /data/local/tmp/file.mkv` copies that file there instead, for the player.
fn seed(library: &Path) {
    let Ok(list) = std::fs::read_to_string("/data/local/tmp/anibeam-seed") else {
        return;
    };
    for line in list.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let (name, source) = match line.split_once(" < ") {
            Some((name, source)) => (name.trim(), Some(source.trim())),
            None => (line, None),
        };
        // Only a plain relative path stays inside the library: an absolute one would
        // replace it in `join`, and `..` would climb out of it.
        if !Path::new(name).components().all(|c| matches!(c, Component::Normal(_))) {
            log(&format!("seed refused {name}"));
            continue;
        }
        let path = library.join(name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let done = match source {
            // Copied once: a second launch finds it there and leaves it.
            Some(_) if path.exists() => continue,
            Some(source) => std::fs::copy(source, &path).is_ok(),
            None => std::fs::write(&path, b"").is_ok(),
        };
        if done {
            log(&format!("seeded {name}"));
        }
    }
}
