mod common;
use anibeam_core::*;

#[test]
fn open_about_and_events_work_without_start() {
    let (dir, core, c) = common::open_core();
    match core.call(Call::About).unwrap() {
        Reply::About { about } => {
            assert_eq!(about.version, VERSION);
            assert!(about.db_path.starts_with(dir.path().to_str().unwrap()));
        }
        other => panic!("{other:?}"),
    }
    core.start().unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Ready),
        std::time::Duration::from_secs(2),
    );
    match core.call(Call::RecentEvents { limit: 10 }).unwrap() {
        Reply::Events { events } => {
            assert!(events.iter().any(|e| matches!(e.body, EventBody::Ready)))
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(core.call(Call::ClearEvents), Ok(Reply::Ok)));
    assert!(matches!(core.call(Call::ListJobs), Ok(Reply::Jobs { .. })));
    assert!(matches!(
        core.call(Call::CancelJob { job: 999 }),
        Err(CoreError::NotFound {
            what: Entity::Job,
            id: 999
        })
    ));
    core.shutdown();
    core.shutdown();
}

/// A limit past what the ring can hold is the ring. The shell is on the
/// other side of a bridge, so a number that would not fit an i64 has to be
/// clamped rather than cast: the query binds an i64, and u64::MAX as i64
/// is a negative limit.
#[test]
fn a_recent_limit_past_the_ring_is_the_ring() {
    let (_dir, core, c) = common::open_core();
    core.start().unwrap();
    common::wait_for(
        &c,
        |e| matches!(e.body, EventBody::Ready),
        std::time::Duration::from_secs(2),
    );
    let all = match core.call(Call::RecentEvents { limit: u64::MAX }).unwrap() {
        Reply::Events { events } => events,
        other => panic!("{other:?}"),
    };
    assert!(
        all.iter().any(|e| matches!(e.body, EventBody::Ready)),
        "a negative limit answers with nothing at all"
    );
    core.shutdown();
}
