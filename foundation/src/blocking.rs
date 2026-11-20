//! Blocking work from whatever thread asked for it. The phone's update check blocks on the
//! core's runtime (#809). A shell calls from its own thread, where blocking is plain. A
//! listener runs inline on the runtime worker that emitted the event, where a plain
//! `block_on` or `blocking_recv` panics, so there the work goes through
//! `block_in_place`, which hands the worker's other tasks to a sibling before it blocks. A
//! caller on a current-thread runtime, a `#[tokio::test]` say, cannot give its thread up
//! that way, so the work runs on a thread of its own and the caller waits for it.

use tokio::runtime::{Handle, RuntimeFlavor};

/// Runs `work`, which may block, so that it is safe on the calling thread.
pub fn blocking<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    match Handle::try_current().map(|h| h.runtime_flavor()) {
        Ok(RuntimeFlavor::MultiThread) => tokio::task::block_in_place(work),
        Ok(_) => std::thread::scope(|s| {
            s.spawn(work)
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
        }),
        Err(_) => work(),
    }
}

#[cfg(test)]
mod tests {
    use super::blocking;

    fn block_on_its_own_runtime() -> u32 {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async { 7 })
    }

    #[test]
    fn a_plain_thread_just_runs_it() {
        assert_eq!(blocking(block_on_its_own_runtime), 7);
    }

    #[test]
    fn a_multi_thread_worker_blocks_in_place() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .unwrap();
        let got = rt.block_on(rt.spawn(async { blocking(block_on_its_own_runtime) }));
        assert_eq!(got.unwrap(), 7);
    }

    #[tokio::test]
    async fn a_current_thread_runtime_waits_on_a_thread_of_its_own() {
        assert_eq!(blocking(block_on_its_own_runtime), 7);
    }
}
