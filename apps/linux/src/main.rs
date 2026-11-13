// A release build on Windows is a GUI program: no console window opens beside it.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app_icon;
mod args;
mod bridge;
mod dbus;
mod devstats;
mod format;
mod inhibit;
#[cfg(target_os = "macos")]
mod iokit;
mod json;
mod library_view;
#[cfg(target_os = "macos")]
mod macnp;
mod media;
mod nowplaying;
mod paths;
mod player_config;
mod problem;
mod read_cache;
mod runtime;
mod scan_summary;
mod serve;
mod theme;
#[cfg(windows)]
mod winpower;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

pub const APP_ID: &str = "com.marcusrosado.AniBeam";
pub const MAIN_QML: &str = "qrc:/qt/qml/com/marcusrosado/AniBeam/qml/Main.qml";

/// Debug builds only: ANIBEAM_QML_DIR=qml (relative to where it runs; shoot.sh and drive.sh make
/// it absolute) loads the QML from disk, so a QML-only edit shows up on the next run with no
/// cargo build. A directory without Main.qml is almost certainly a mistyped or wrongly
/// relative path, so it warns and keeps the compiled QML instead of claiming a disk load.
#[cfg(debug_assertions)]
fn install_dev_qml_dir(
    mut engine: std::pin::Pin<&mut QQmlApplicationEngine>,
) -> std::pin::Pin<&mut QQmlApplicationEngine> {
    let Some(dir) = std::env::var_os("ANIBEAM_QML_DIR") else {
        return engine;
    };
    let dir = std::path::Path::new(&dir);
    if !dir.join("Main.qml").is_file() {
        eprintln!(
            "anibeam: ANIBEAM_QML_DIR={} has no Main.qml; using the compiled QML",
            dir.display()
        );
        return engine;
    }
    eprintln!("anibeam: loading QML from {}", dir.display());
    bridge::helpers::ffi::install_qml_dir(engine.as_mut(), &QString::from(&*dir.to_string_lossy()));
    engine
}

fn main() {
    // Before anything formats a word: the core, the bridges and QML all read it.
    bridge::tr::language_from_env();
    let argv: Vec<String> = std::env::args().collect();
    let args = match args::parse(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("anibeam: {e}");
            std::process::exit(2);
        }
    };
    if args.version {
        println!("anibeam {}", anibeam_core::VERSION);
        return;
    }
    // A drive script that is not one fails here, before a window or the core exists, with
    // the same marker every later drive failure prints.
    if let Some(script) = args.drive.as_deref() {
        if let Err(e) = args::load_drive(std::path::Path::new(script)) {
            eprintln!("DRIVE FAIL: {e}");
            std::process::exit(1);
        }
        let out = args::drive_out(args.out.as_deref(), std::env::var("ANIBEAM_CAPTURES").ok());
        if let Err(e) = std::fs::create_dir_all(&out) {
            eprintln!("DRIVE FAIL: cannot create {}: {e}", out.display());
            std::process::exit(1);
        }
    }
    let serving = args.serve;
    let paths = match paths::ShellPaths::resolve(args.root.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("anibeam: {e}");
            std::process::exit(2);
        }
    };
    // Installed before the lock, because the names the hand-off targets are the sandbox's
    // under --root and the arguments are where that is read from. Nothing else looks at
    // either until the engine loads.
    devstats::startup_mark("main");
    runtime::install_paths(paths);
    runtime::install_args(args);
    // The saved language before the engine loads, so the first frame is already in it
    bridge::tr::language_from_settings(&runtime::paths().config_dir().join("shell.conf"));
    if serving {
        std::process::exit(serve::run());
    }

    // Spec 4.5: the flock is the single-instance guarantee and it is taken before
    // anything else opens. A second launch that loses it hands its activation token to the
    // running window and leaves; the bus is only how the raise travels, so a hand-off that
    // cannot reach the bus still exits rather than opening a second window.
    let lock_path = runtime::paths().lock_path();
    // A headless `anibeam serve` holding the lock steps aside when asked: it closes its
    // core, lets the lock go and takes it back once this window is gone. The connection
    // that asked is held for the life of the process, which is how the service knows.
    let mut service = None;
    let lock = match dbus::instance::try_lock(&lock_path) {
        Ok(Some(lock)) => Some(lock),
        Ok(None) => {
            let socket = runtime::paths().service_socket();
            if let Some(conn) = serve::ask_to_yield(&socket) {
                service = Some(conn);
                match dbus::instance::try_lock(&lock_path) {
                    Ok(Some(lock)) => Some(lock),
                    _ => {
                        eprintln!(
                            "anibeam: the host service let the library go, but another AniBeam took it first"
                        );
                        std::process::exit(1);
                    }
                }
            } else if serve::status(&socket) == Some(serve::State::Serving) {
                // The service holds the library and did not let it go in time; there is no
                // window to raise.
                eprintln!("anibeam: the host service did not hand over the library; try again");
                std::process::exit(1);
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("anibeam: lock {}: {e}", lock_path.display());
            std::process::exit(2);
        }
    };
    let Some(lock) = lock else {
        let result = runtime::runtime()
            .block_on(dbus::instance::hand_off(runtime::args().action.as_deref()));
        match result {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("anibeam: another AniBeam is running and {e}");
                std::process::exit(1);
            }
        }
    };

    // These have to run before the core does: set_render_loop_env is a setenv, as is naming
    // the bundled helpers, and
    // setenv is not safe once another thread exists, which the core's tokio runtime is.
    // The one setenv that cannot obey this rule is raise_window's, in cpp/helpers.cpp;
    // the comment there says why Qt leaves no other way and what the risk comes to.
    bridge::helpers::ffi::set_render_loop_env();
    bridge::helpers::ffi::use_opengl_scene_graph();
    paths::name_bundled_helpers();

    // The core is open before the engine, so the Door singleton the first QML file names
    // finds it already there.
    let core = match runtime::open_core() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("anibeam: {e}");
            std::process::exit(2);
        }
    };
    // While `anibeam serve` exists on this machine, other devices expect the host whether
    // or not the window is open, so the window keeps it on in the service's place.
    if service.is_some() || serve::status(&runtime::paths().service_socket()).is_some() {
        core.hold_host(true);
    }
    // What's new: whether the app started here before and the version it last logged, read
    // now because the Door's start logs this run's.
    let starts = core.earlier_starts();
    runtime::install_earlier(runtime::Earlier {
        started: !starts.is_empty(),
        version: starts
            .iter()
            .find_map(|v| anibeam_core::changelog::version_of(v)),
    });
    // A drive or a shot runs against the owner's real anirss and qBittorrent, so nothing may
    // change there on its own: no unsubscribing, no replacing. A sandbox root is covered by
    // `open_core`.
    let args = runtime::args();
    if args.drive.is_some() || args.shoot.is_some() {
        core.disable_unattended_changes();
    }
    devstats::startup_mark("core open");
    runtime::install_core(core);

    let mut app = QGuiApplication::new();
    devstats::startup_mark("application");
    bridge::helpers::ffi::set_desktop_file_name(&QString::from(APP_ID));

    bridge::helpers::ffi::dev_enable_qml_profiling();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        bridge::helpers::ffi::install_dev_tools(engine.as_mut());
        #[cfg(debug_assertions)]
        let engine = install_dev_qml_dir(engine);
        engine.load(&QUrl::from(MAIN_QML));
        devstats::startup_mark("Main.qml loaded");
    }
    // The engine goes first: dropping it destroys the Door, which drops the event
    // subscription, so nothing is still listening when the core closes its store.
    let code = match app.as_mut() {
        Some(app) => app.exec(),
        None => 1,
    };
    drop(engine);
    runtime::core().shutdown();
    // Held until here on purpose: the lock is released by closing the file, so this is the
    // line that says the process is done being the one instance.
    drop(lock);
    drop(service);
    // An update readied in this run finishes once this process is gone, by a helper that
    // waits for it; started last, so the new version finds the lock free.
    bridge::shell::finish_update();
    std::process::exit(code);
}
