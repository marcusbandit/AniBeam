mod args;
mod bridge;
mod dbus;
mod format;
mod json;
mod nowplaying;
mod paths;
mod player_config;
mod runtime;
mod theme;
mod tracks;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

pub const APP_ID: &str = "com.marcusrosado.AniBeam";
pub const MAIN_QML: &str = "qrc:/qt/qml/com/marcusrosado/AniBeam/qml/Main.qml";

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

    // Both of these have to run before the core does: set_render_loop_env is a setenv, and
    // setenv is not safe once another thread exists, which the core's tokio runtime is.
    // The one setenv that cannot obey this rule is raise_window's, in cpp/helpers.cpp;
    // the comment there says why Qt leaves no other way and what the risk comes to.
    bridge::helpers::ffi::set_render_loop_env();
    bridge::helpers::ffi::use_opengl_scene_graph();

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
    runtime::install_core(core);

    let mut app = QGuiApplication::new();
    bridge::helpers::ffi::set_desktop_file_name(&QString::from(APP_ID));

    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(MAIN_QML));
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
    std::process::exit(code);
}
