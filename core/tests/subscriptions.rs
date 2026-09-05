//! Subscriptions end to end, against a fake `anirss` on `PATH`. All four
//! cases live in one test: the process-wide `PATH` and `HOME` edits below
//! would otherwise race any other test that happens to run at the same
//! time, and this crate has no `serial_test` dependency to stop that.
//!
//! `HOME` is pointed at a throwaway directory for the whole test, not just
//! `PATH`: the machine this runs on may have a real anirss installed under
//! its actual `~/.local/bin`, and the core prepends that path whenever it
//! is missing from `PATH`. Redirecting `HOME` keeps that prepend pointing
//! at a directory that does not exist, so the fake is what gets found, and
//! the "missing" case stays missing regardless of what is installed on
//! the machine running the test.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use anibeam_core::*;

const OK_SCRIPT: &str = r#"#!/bin/sh
cat <<'JSON'
[{"name": "Frieren", "feed_url": "https://nyaa.si/?page=rss&q=Frieren+1080p&c=1_2", "save_path": "/downloads/frieren", "rule_enabled": true, "torrent_count": 3}]
JSON
"#;

const SLEEP_SCRIPT: &str = "#!/bin/sh\nsleep 20\n";

fn write_fake(path: &std::path::Path, script: &str) {
    write_with_mode(path, script, 0o755);
}

/// A fake with no execute bit at all, for the case where anirss is on
/// `PATH` but the OS refuses to run it: a permission error, not a missing
/// one.
fn write_unreadable_fake(path: &std::path::Path, script: &str) {
    write_with_mode(path, script, 0o644);
}

fn write_with_mode(path: &std::path::Path, script: &str, mode: u32) {
    std::fs::write(path, script).unwrap();
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(mode);
    std::fs::set_permissions(path, perms).unwrap();
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

#[test]
fn subscriptions_through_a_fake_anirss() {
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    let anirss_path = bin.path().join("anirss");
    let original_path = std::env::var("PATH").unwrap_or_default();
    let original_home = std::env::var("HOME").ok();

    // Both edits are process-wide for the rest of this test binary's one
    // test, which is the point of keeping every case in this one function.
    unsafe {
        std::env::set_var("HOME", home.path());
        // The fake's directory goes first. `HOME` is already redirected by
        // the line above, so `child_path()` never finds the redirected
        // `~/.local/bin` on this constructed `PATH` and always prepends
        // it; that directory does not exist, so the OS skips straight
        // past it on its way to the fake, wherever the fake sits behind it.
        std::env::set_var("PATH", format!("{}:{}", bin.path().display(), original_path));
    }

    let (_dir, core, c) = common::open_core();

    // Case 1: a clean exit with a JSON array is `Ok`, with the query
    // decoded off the feed's own url.
    write_fake(&anirss_path, OK_SCRIPT);
    let job = started(&core, Call::ListSubscriptions);
    let done = common::wait_job(&c, job);
    let EventBody::SubscriptionsListed { result } = &done.body else { panic!("{:?}", done.body) };
    let SubscriptionsResult::Ok { feeds } = result else { panic!("{result:?}") };
    assert_eq!(feeds.len(), 1);
    assert_eq!(feeds[0].name, "Frieren");
    assert_eq!(feeds[0].query, "Frieren 1080p");
    assert!(feeds[0].active);
    assert_eq!(feeds[0].torrents, 3);
    assert_eq!(done.level, Level::Debug);

    // Case 2: a fake that outlives the 15 s budget ends in `Timeout`,
    // inside the 16 s the brief allows for it.
    write_fake(&anirss_path, SLEEP_SCRIPT);
    let job = started(&core, Call::ListSubscriptions);
    let done = common::wait_for(
        &c,
        |e| e.job.as_ref().is_some_and(|j| j.id == job && j.phase == JobPhase::Finished),
        Duration::from_secs(16),
    );
    let EventBody::SubscriptionsListed { result } = &done.body else { panic!("{:?}", done.body) };
    assert_eq!(*result, SubscriptionsResult::Timeout);

    // Case 3: no fake on `PATH` at all gives `Missing`.
    std::fs::remove_file(&anirss_path).unwrap();
    unsafe {
        std::env::set_var("PATH", bin.path());
    }
    let job = started(&core, Call::ListSubscriptions);
    let done = common::wait_job(&c, job);
    let EventBody::SubscriptionsListed { result } = &done.body else { panic!("{:?}", done.body) };
    assert_eq!(*result, SubscriptionsResult::Missing);

    // Case 4: present but not executable is a permission error, which is
    // not the "go install it" story `Missing` tells, so the job fails
    // instead.
    write_unreadable_fake(&anirss_path, OK_SCRIPT);
    unsafe {
        std::env::set_var("PATH", bin.path());
    }
    let job = started(&core, Call::ListSubscriptions);
    let done = common::wait_job(&c, job);
    match &done.body {
        EventBody::JobFailed { error: CoreError::Io { .. } } => {}
        other => panic!("{other:?}"),
    }

    unsafe {
        match &original_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        std::env::set_var("PATH", &original_path);
    }
}
