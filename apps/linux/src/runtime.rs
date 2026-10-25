//! The process-wide singletons the QML engine cannot be handed through a constructor: the
//! tokio runtime, the core, the parsed arguments and what the core logged at its earlier
//! starts. `main` installs each once; the bridge objects, which the QML engine constructs,
//! find them here.

use std::sync::{Arc, OnceLock};

use anibeam_core::Core;

use crate::args::Args;
use crate::paths::ShellPaths;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static CORE: OnceLock<Arc<Core>> = OnceLock::new();
static ARGS: OnceLock<Args> = OnceLock::new();
static PATHS: OnceLock<ShellPaths> = OnceLock::new();
static EARLIER: OnceLock<Earlier> = OnceLock::new();

pub fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("anibeam-shell")
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

pub fn install_core(core: Arc<Core>) {
    CORE.set(core).ok();
}

pub fn core() -> &'static Arc<Core> {
    CORE.get()
        .expect("the core is installed before the QML engine loads")
}

pub fn install_paths(paths: ShellPaths) {
    PATHS.set(paths).ok();
}

pub fn paths() -> &'static ShellPaths {
    PATHS
        .get()
        .expect("the paths are installed before the QML engine loads")
}

pub fn install_args(args: Args) {
    ARGS.set(args).ok();
}

pub fn args() -> &'static Args {
    ARGS.get()
        .expect("the arguments are installed before the QML engine loads")
}

/// What the core logged at its starts before this run, read before this run's own: whether
/// the app ever started here, and the newest app version among them. What's new takes them
/// for an install that ran before it kept a record of its own. Never installed means never
/// started.
#[derive(Clone, Debug, Default)]
pub struct Earlier {
    pub started: bool,
    pub version: Option<String>,
}

pub fn install_earlier(earlier: Earlier) {
    EARLIER.set(earlier).ok();
}

pub fn earlier() -> Earlier {
    EARLIER.get().cloned().unwrap_or_default()
}
