//! The process-wide singletons the QML engine cannot be handed through a constructor: the
//! tokio runtime, the core and the parsed arguments. `main` installs each once; the bridge
//! objects, which the QML engine constructs, find them here.

use std::sync::{Arc, OnceLock};

use anibeam_core::Core;

use crate::args::Args;

// Task 6 opens the core on this runtime and installs it here; nothing calls `runtime()`,
// `install_core()` or `core()` until then, so a plain build sees them as dead code.
#[allow(dead_code)]
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
#[allow(dead_code)]
static CORE: OnceLock<Arc<Core>> = OnceLock::new();
static ARGS: OnceLock<Args> = OnceLock::new();

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn install_core(core: Arc<Core>) {
    CORE.set(core).ok();
}

#[allow(dead_code)]
pub fn core() -> &'static Arc<Core> {
    CORE.get()
        .expect("the core is installed before the QML engine loads")
}

pub fn install_args(args: Args) {
    ARGS.set(args).ok();
}

pub fn args() -> &'static Args {
    ARGS.get()
        .expect("the arguments are installed before the QML engine loads")
}
