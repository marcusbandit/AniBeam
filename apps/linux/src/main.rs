mod args;
mod bridge;
mod format;
mod json;
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
    // Task 13 takes the lock first.
    runtime::install_paths(paths);
    runtime::install_args(args);

    // Both of these have to run before the core does: set_render_loop_env is a setenv, and
    // setenv is not safe once another thread exists, which the core's tokio runtime is.
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
    std::process::exit(code);
}
