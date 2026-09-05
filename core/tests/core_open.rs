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
    common::wait_for(&c, |e| matches!(e.body, EventBody::Ready), std::time::Duration::from_secs(2));
    match core.call(Call::RecentEvents { limit: 10 }).unwrap() {
        Reply::Events { events } => assert!(events.iter().any(|e| matches!(e.body, EventBody::Ready))),
        other => panic!("{other:?}"),
    }
    assert!(matches!(core.call(Call::ClearEvents), Ok(Reply::Ok)));
    assert!(matches!(core.call(Call::ListJobs), Ok(Reply::Jobs { .. })));
    assert!(matches!(core.call(Call::CancelJob { job: 999 }), Err(CoreError::NotFound { what: Entity::Job, id: 999 })));
    core.shutdown();
    core.shutdown();
}
