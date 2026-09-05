#![allow(dead_code)]
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::events::{Collector, Subscription};
use anibeam_core::net::FakeHttp;
use anibeam_core::trackers::Secrets;
use anibeam_core::{Core, CorePaths, Event, JobPhase};

/// Owns the test's temp directory and the subscription collecting its
/// events, so the subscription lives exactly as long as the test's `dir`
/// binding: the caller never has to remember to keep it alive separately,
/// and it unsubscribes when `dir` drops at the end of the test.
pub struct Dir {
    dir: tempfile::TempDir,
    _sub: Arc<Subscription>,
}

impl Deref for Dir {
    type Target = tempfile::TempDir;

    fn deref(&self) -> &tempfile::TempDir {
        &self.dir
    }
}

/// Every core a test opens keeps its secrets in the temp directory's own
/// `secrets.json`: the file store is chosen here rather than probed for, so
/// no test ever reaches the machine's real keyring or puts a prompt on the
/// owner's screen.
pub fn open_core() -> (Dir, Arc<Core>, Arc<Collector>) {
    let dir = tempfile::tempdir().unwrap();
    let paths = CorePaths::under(dir.path());
    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_secrets(paths, secrets).unwrap();
    let collector = Arc::new(Collector::default());
    let sub = core.subscribe(collector.clone());
    (Dir { dir, _sub: sub }, core, collector)
}

/// The same core with the network swapped for canned replies. A test that
/// drives a provider job keeps its own handle on the `FakeHttp` to queue
/// replies and read back the requests.
pub fn open_core_with_http(http: Arc<FakeHttp>) -> (Dir, Arc<Core>, Arc<Collector>) {
    let dir = tempfile::tempdir().unwrap();
    let paths = CorePaths::under(dir.path());
    let secrets = Secrets::file_only(paths.secrets_path());
    let core = Core::open_with_http_and_secrets(paths, http, secrets).unwrap();
    let collector = Arc::new(Collector::default());
    let sub = core.subscribe(collector.clone());
    (Dir { dir, _sub: sub }, core, collector)
}

/// Blocks until some event collected so far satisfies `pred`, then returns
/// the first such event. Built on `Collector::wait_for`'s condvar, so this
/// never sleep-polls. Panics with every event seen on timeout.
pub fn wait_for(c: &Collector, pred: impl Fn(&Event) -> bool, timeout: Duration) -> Event {
    let arrived = c.wait_for(|events| events.iter().any(&pred), timeout);
    if !arrived {
        panic!("timed out waiting; saw {:#?}", c.events());
    }
    c.events()
        .into_iter()
        .find(|e| pred(e))
        .expect("just confirmed present")
}

/// The contract's "await job N": the first event of that job whose phase is
/// Finished.
pub fn wait_job(c: &Collector, job: u64) -> Event {
    wait_for(
        c,
        |e| {
            e.job
                .as_ref()
                .is_some_and(|j| j.id == job && j.phase == JobPhase::Finished)
        },
        Duration::from_secs(30),
    )
}
