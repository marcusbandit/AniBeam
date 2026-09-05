mod common;
use anibeam_core::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

/// Two shows and a film, the smallest library that is worth walking.
fn library(root: &Path) -> String {
    for show in ["Show A", "Show B"] {
        fs::create_dir_all(root.join(show)).unwrap();
        fs::write(root.join(show).join(format!("{show} - 01.mkv")), b"x").unwrap();
    }
    fs::create_dir_all(root.join("Movies")).unwrap();
    fs::write(root.join("Movies").join("A Film (2019).mkv"), b"x").unwrap();
    root.to_string_lossy().into_owned()
}

/// Every reply an auto-match search can get is an empty page, so every
/// series misses and is stamped, which is the quietest outcome the chain
/// has: nothing is fetched and nothing is written.
fn misses() -> Arc<anibeam_core::net::FakeHttp> {
    let http = anibeam_core::net::FakeHttp::new();
    for _ in 0..12 {
        http.push_for(
            "graphql.anilist.co",
            200,
            serde_json::json!({ "data": { "Page": { "media": [] } } }).to_string(),
        );
    }
    http
}

/// The five kinds the launch owes a session, in the order it starts them.
/// The stub backfill is a plain `Refresh`, which is not one at a time and so
/// is not something a test can wait on by kind alone.
const LAUNCH_JOBS: [JobKind; 5] = [
    JobKind::AutoMatch,
    JobKind::Crawl,
    JobKind::RefreshAiring,
    JobKind::RefreshProgress,
    JobKind::FillImages,
];

/// The whole launch: the catch-up scan walks the library, and the moment it
/// finishes every job the core owes a fresh start goes out at once. Nothing
/// here is on a timer; the scan's own terminal event is the trigger.
///
/// The network is canned, because the auto-match on that list ends in a
/// search. Every reply is an empty page, so every series misses and is
/// stamped, which is the quietest outcome the chain has.
#[test]
fn the_launch_queues_every_catch_up_job_once_the_scan_finishes() {
    let (dir, core, c) = common::open_core_with_http(misses());
    let path = library(&dir.path().join("lib"));
    core.call(Call::AddSource { path }).unwrap();

    core.start().unwrap();
    let ready = common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Ready),
        Duration::from_secs(5),
    );
    // The catch-up scan is the one that follows the launch, whatever the
    // AddSource above had already finished.
    let scan = common::wait_for(
        &c,
        |e| e.seq > ready.seq && matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    for kind in LAUNCH_JOBS {
        common::wait_for(
            &c,
            |e| {
                e.seq > scan.seq
                    && matches!(&e.body, EventBody::JobStarted { kind: k } if *k == kind)
            },
            Duration::from_secs(10),
        );
    }

    // The library the walk found: two shows and the film.
    let cards = match core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap()
    {
        Reply::Series { series } => series,
        other => panic!("{other:?}"),
    };
    assert_eq!(cards.len(), 3, "{cards:?}");

    // A second start says nothing and starts nothing.
    let readies = c
        .events()
        .iter()
        .filter(|e| matches!(e.body, EventBody::Ready))
        .count();
    core.start().unwrap();
    assert_eq!(
        c.events()
            .iter()
            .filter(|e| matches!(e.body, EventBody::Ready))
            .count(),
        readies
    );

    let began = Instant::now();
    core.shutdown();
    assert!(
        began.elapsed() < Duration::from_secs(3),
        "shutdown took {:?}",
        began.elapsed()
    );
    // Idempotent, and a call afterwards fails rather than hanging.
    core.shutdown();
    let err = core
        .call(Call::ListSeries {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        })
        .unwrap_err();
    assert!(matches!(err, CoreError::Internal { .. }), "{err:?}");
}

/// Cancels the first Scan it sees start, from inside the emit itself: the
/// `JobStarted` goes out on the thread that started the job and before the
/// body is spawned, so the job is registered and the cancel can never race
/// past a walk that finished first.
struct CancelFirstScan {
    core: Weak<Core>,
    done: AtomicBool,
}

impl EventListener for CancelFirstScan {
    fn on_event(&self, event: Event) {
        if !matches!(
            event.body,
            EventBody::JobStarted {
                kind: JobKind::Scan
            }
        ) {
            return;
        }
        if self.done.swap(true, Ordering::SeqCst) {
            return;
        }
        let (Some(core), Some(job)) = (self.core.upgrade(), event.job) else {
            return;
        };
        let _ = core.call(Call::CancelJob { job: job.id });
    }
}

/// Cancelling the catch-up scan does not cost the session its launch. The
/// scan ends in `JobCancelled` rather than `ScanFinished`, which is a
/// terminal event like any other, and everything the launch owes still goes
/// out behind it.
#[test]
fn a_cancelled_catch_up_scan_still_starts_the_rest_of_the_launch() {
    let (dir, core, c) = common::open_core_with_http(misses());
    let path = library(&dir.path().join("lib"));
    core.call(Call::AddSource { path }).unwrap();
    // AddSource's own scan is out of the way before the launch's starts, so
    // the listener below can only be looking at the catch-up scan.
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::ScanFinished { .. }),
        Duration::from_secs(10),
    );

    let killer = Arc::new(CancelFirstScan {
        core: Arc::downgrade(&core),
        done: AtomicBool::new(false),
    });
    let _sub = core.subscribe(killer);
    core.start().unwrap();

    let cancelled = common::wait_for(
        &c,
        |e| {
            matches!(e.body, EventBody::JobCancelled)
                && e.job
                    .as_ref()
                    .is_some_and(|j| j.kind == JobKind::Scan && j.phase == JobPhase::Finished)
        },
        Duration::from_secs(10),
    );
    for kind in LAUNCH_JOBS {
        common::wait_for(
            &c,
            |e| {
                e.seq > cancelled.seq
                    && matches!(&e.body, EventBody::JobStarted { kind: k } if *k == kind)
            },
            Duration::from_secs(10),
        );
    }
    core.shutdown();
}
