//! Every call answers whatever thread makes it (#1368). A listener runs inline on the
//! runtime worker that emitted the event and may call back into the core from there, and a
//! test may call from a current-thread runtime. A writing call blocks on the store's writer
//! and a fetching call blocks on the core's runtime; neither may panic in either place.

mod common;

use std::sync::Arc;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

/// A writing call: it waits on the store's writer thread.
fn write(core: &Core) -> Result<Reply, CoreError> {
    core.call(Call::SetAutoSkip {
        intro: true,
        outro: false,
    })
}

#[test]
fn a_writing_call_answers_from_a_runtime_worker() {
    let (_dir, core, _c) = common::open_core_with_http(FakeHttp::new());
    let shell = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    // A clone goes to the worker, so the core is never dropped inside the runtime
    let on_worker = Arc::clone(&core);
    let task = shell.spawn(async move { write(&on_worker) });
    assert!(shell.block_on(task).unwrap().is_ok());
}

#[test]
fn a_writing_call_answers_on_a_current_thread_runtime() {
    let (_dir, core, _c) = common::open_core_with_http(FakeHttp::new());
    let caller = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    assert!(caller.block_on(async { write(&core) }).is_ok());
}

/// A fetching call: it waits on the core's runtime with `block_on`.
#[cfg(feature = "phone")]
fn fetch(core: &Core) -> Result<Reply, CoreError> {
    core.call(Call::PhoneFindShows {
        query: "Example".into(),
        limit: 5,
    })
}

#[cfg(feature = "phone")]
#[test]
fn a_fetching_call_answers_from_a_runtime_worker_and_a_current_thread_runtime() {
    let http = FakeHttp::new();
    let (_dir, core, _c) = common::open_core_with_http(http.clone());
    let empty = serde_json::json!({ "data": { "Page": { "media": [] } } });
    http.push_json(200, empty.clone());
    http.push_json(200, empty);

    let shell = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    let on_worker = Arc::clone(&core);
    // Whatever AniList answers, the call answers: a panic would fail the join
    let task = shell.spawn(async move { fetch(&on_worker) });
    let _ = shell.block_on(task).unwrap();

    let caller = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let _ = caller.block_on(async { fetch(&core) });
}
