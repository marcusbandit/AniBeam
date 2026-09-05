mod common;
use anibeam_core::*;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

/// A listener runs inline on whatever thread emitted the event, which for
/// anything a job says is a tokio worker. A shell is free to call back into
/// the core from there, so every call has to survive it, writing calls
/// included: they block on the writer thread, and blocking a worker the
/// naive way panics the runtime.
struct CallsBack {
    core: Weak<Core>,
    fired: AtomicBool,
    outcome: Mutex<Sender<Result<Reply, CoreError>>>,
}

impl CallsBack {
    fn new(core: &Arc<Core>) -> (Arc<CallsBack>, Receiver<Result<Reply, CoreError>>) {
        let (tx, rx) = mpsc::channel();
        let listener = Arc::new(CallsBack {
            core: Arc::downgrade(core),
            fired: AtomicBool::new(false),
            outcome: Mutex::new(tx),
        });
        (listener, rx)
    }

    fn answer(&self, outcome: Result<Reply, CoreError>) {
        let _ = self.outcome.lock().unwrap().send(outcome);
    }
}

impl EventListener for CallsBack {
    fn on_event(&self, event: Event) {
        // Only the scan's terminal, and only once: SetHidden emits its own
        // SeriesChanged, so a listener that answered every event would
        // recurse for ever.
        if !matches!(event.body, EventBody::ScanFinished { .. }) {
            return;
        }
        if self.fired.swap(true, Ordering::SeqCst) {
            return;
        }
        let Some(core) = self.core.upgrade() else {
            return;
        };
        let series = match core.call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        }) {
            Ok(Reply::Series { series }) => series,
            other => {
                self.answer(Err(CoreError::internal(format!("list: {other:?}"))));
                return;
            }
        };
        let id = series.first().map(|s| s.id).unwrap_or(0);
        let hidden = core.call(Call::SetHidden {
            series: id,
            hidden: true,
        });
        // A read of the log goes through the writer channel too, so it is
        // the same wait from the same thread.
        let _ = core.call(Call::RecentEvents { limit: 5 });
        self.answer(hidden);
    }
}

/// The whole point: a shell that writes from inside `on_event` gets its
/// write, and the worker that ran the listener carries on. Before the
/// `block_in_place` in `Store::write` this took the worker down and the
/// scan's own terminal never reached the collector.
#[test]
fn a_listener_that_writes_from_on_event_is_served_not_a_panic() {
    let (dir, core, c) = common::open_core();
    let lib = dir.path().join("lib");
    fs::create_dir_all(lib.join("Show A")).unwrap();
    fs::write(lib.join("Show A").join("Show A - 01.mkv"), b"x").unwrap();

    let (listener, outcomes) = CallsBack::new(&core);
    let _sub = core.subscribe(listener);

    core.call(Call::AddSource {
        path: lib.to_string_lossy().into_owned(),
    })
    .unwrap();

    let scan = common::wait_for(
        &c,
        |e| {
            matches!(
                e.body,
                EventBody::JobStarted {
                    kind: JobKind::Scan
                }
            )
        },
        Duration::from_secs(5),
    )
    .job
    .unwrap()
    .id;
    common::wait_job(&c, scan);

    // The listener ran on the worker that emitted ScanFinished, and its
    // write went through.
    let outcome = outcomes
        .recv_timeout(Duration::from_secs(5))
        .expect("the listener never answered");
    assert!(matches!(outcome, Ok(Reply::Ok)), "{outcome:?}");

    // The runtime is still whole: another job runs to its terminal after
    // the listener blocked one of its workers.
    let again = match core
        .call(Call::Scan { source: None })
        .expect("rescan refused")
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    common::wait_job(&c, again);

    // And the hide really landed.
    let visible = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Series { series } => series.len(),
        other => panic!("{other:?}"),
    };
    assert_eq!(visible, 0);
    core.shutdown();
}
