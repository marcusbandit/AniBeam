//! `ANIBEAM_HOST_BIND` itself, which is process-wide and so gets a test
//! binary of its own: the other host tests use `Core::set_host_bind`.

mod common;

use std::time::{Duration, Instant};

use anibeam_core::*;

#[test]
fn the_env_var_names_where_the_host_listens() {
    unsafe { std::env::set_var(anibeam_core::host::BIND_ENV, "127.0.0.1:0") };
    let (_dir, core, _c) = common::open_core();
    let preferences = Preferences {
        share_over_tailscale: true,
        ..Preferences::default()
    };
    core.call(Call::SetPreferences { preferences }).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        let Reply::Host { status } = core.call(Call::GetHost).unwrap() else {
            panic!("not a host reply")
        };
        if !status.urls.is_empty() {
            break status;
        }
        assert!(Instant::now() < deadline, "{status:?}");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(status.urls[0].starts_with("http://127.0.0.1:"));
    core.shutdown();
}
