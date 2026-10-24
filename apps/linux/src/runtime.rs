//! The process-wide singletons the QML engine cannot be handed through a constructor: the
//! tokio runtime, the core, the parsed arguments and whether this install ran before. `main`
//! installs each once; the bridge objects, which the QML engine constructs, find them here.

use std::sync::{Arc, OnceLock};

use anibeam_core::Core;

use crate::args::Args;
use crate::paths::ShellPaths;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static CORE: OnceLock<Arc<Core>> = OnceLock::new();
static ARGS: OnceLock<Args> = OnceLock::new();
static PATHS: OnceLock<ShellPaths> = OnceLock::new();
static USED_BEFORE: OnceLock<bool> = OnceLock::new();

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

/// Whether the library database was there before this run opened it: an install in use
/// before What's new kept a record gets the notes after its update; a fresh one does not.
pub fn install_used_before(used: bool) {
    USED_BEFORE.set(used).ok();
}

pub fn used_before() -> bool {
    USED_BEFORE.get().copied().unwrap_or(false)
}
