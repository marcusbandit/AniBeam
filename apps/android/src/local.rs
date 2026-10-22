//! The phone's own AniBeam: the same core the desktop runs, opened over the app's files.
//!
//! The core keeps its database, cache and secrets under `files/core`, and the phone's
//! library is one folder the app owns, `files/Anime`. Events go to logcat under the tag
//! `AniBeamCore` until the screens read them.

use std::path::Path;
use std::sync::Arc;

use anibeam_core::trackers::Secrets;
use anibeam_core::{Call, Core, CorePaths, Event, EventListener, Reply};

/// Where downloads land and what the library scans.
pub const LIBRARY_DIR: &str = "Anime";

struct Logcat;

impl EventListener for Logcat {
    fn on_event(&self, event: Event) {
        log(&format!("{:?} {:?} {}", event.level, event.stage, event.message));
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

/// Opens and starts the core, with the library folder as its one source.
pub fn open(data_dir: &Path) -> Result<Arc<Core>, Box<dyn std::error::Error>> {
    let paths = CorePaths::under(&data_dir.join("core"));
    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_secrets(paths, secrets)?;
    // The subscription lives as long as the core; dropping it would stop the log.
    std::mem::forget(core.subscribe(Arc::new(Logcat)));

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
    Ok(core)
}

/// A test hook: each line of `/data/local/tmp/anibeam-seed` names a file to create, empty,
/// under the library folder, so a scan and a match can be tried without a download.
fn seed(library: &Path) {
    let Ok(list) = std::fs::read_to_string("/data/local/tmp/anibeam-seed") else {
        return;
    };
    for name in list.lines().map(str::trim).filter(|l| !l.is_empty() && !l.contains("..")) {
        let path = library.join(name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&path, b"").is_ok() {
            log(&format!("seeded {name}"));
        }
    }
}
