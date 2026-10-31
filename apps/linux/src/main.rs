// A release build on Windows is a GUI program: no console window opens beside it.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

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
mod read_cache;
mod runtime;
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

    // Spec 4.5: the flock is the single-instance guarantee and it is taken before
    // anything else opens. A second launch that loses it hands its activation token to the
    // running window and leaves; the bus is only how the raise travels, so a hand-off that
    // cannot reach the bus still exits rather than opening a second window.
    let lock_path = runtime::paths().lock_path();
    let lock = match dbus::instance::try_lock(&lock_path) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            let result = runtime::runtime()
                .block_on(dbus::instance::hand_off(runtime::args().action.as_deref()));
            match result {
                Ok(()) => std::process::exit(0),
                Err(e) => {
                    eprintln!("anibeam: another AniBeam is running and {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("anibeam: lock {}: {e}", lock_path.display());
            std::process::exit(2);
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
    // finds it already there. A run under --root keeps its secrets in that root's own
    // secrets.json and never probes the machine's keyring: a sandbox is asked for
    // precisely so it touches nothing outside itself.
    let core = {
        let core_paths = runtime::paths().core.clone();
        let opened = match runtime::args().root.as_deref() {
            Some(_) => {
                let secrets = anibeam_core::trackers::Secrets::file_only(core_paths.secrets_path());
                anibeam_core::Core::open_with_secrets(core_paths, secrets)
            }
            None => anibeam_core::Core::open(core_paths),
        };
        match opened {
            Ok(c) => c,
            Err(e) => {
                eprintln!("anibeam: {e}");
                std::process::exit(2);
            }
        }
    };
    // What's new: whether the app started here before and the version it last logged, read
    // now because the Door's start logs this run's.
    let starts = core.earlier_starts();
    runtime::install_earlier(runtime::Earlier {
        started: !starts.is_empty(),
        version: starts
            .iter()
            .find_map(|v| anibeam_core::changelog::version_of(v)),
    });
    // A drive, a shot or a sandbox root runs a copy of the library against the owner's real
    // anirss and qBittorrent, so nothing may change there on its own: no unsubscribing, no
    // replacing.
    let args = runtime::args();
    if args.drive.is_some() || args.shoot.is_some() || args.root.is_some() {
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
    // An update readied in this run finishes once this process is gone, by a helper that
    // waits for it; started last, so the new version finds the lock free.
    bridge::shell::finish_update();
    std::process::exit(code);
}
