//! Serves a library as the tailnet host on one address until killed, for trying the phone
//! app against it: the emulator reaches the machine it runs on at 10.0.2.2.
//! cargo run --example host_serve -- <root dir> <library dir> [ip:port]
//!
//! The root keeps the database between runs, so a second run serves at once.

use anibeam_core::{Call, Core, CorePaths, Reply};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(root), Some(library)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: host_serve <root dir> <library dir> [ip:port]");
        std::process::exit(2);
    };
    let bind = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| "0.0.0.0:8790".to_string());
    let core = Core::open(CorePaths::under(std::path::Path::new(root))).expect("core");
    core.set_host_bind(Some(bind));
    let has_source = matches!(core.call(Call::ListSources), Ok(Reply::Sources { sources }) if !sources.is_empty());
    if !has_source {
        core.call(Call::AddSource {
            path: library.clone(),
        })
        .expect("source");
    }
    let mut preferences = match core.call(Call::GetPreferences) {
        Ok(Reply::Preferences { preferences }) => preferences,
        other => panic!("{other:?}"),
    };
    preferences.share_over_tailscale = true;
    core.call(Call::SetPreferences { preferences })
        .expect("share");
    core.start().expect("start");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        if let Ok(Reply::Host { status }) = core.call(Call::GetHost) {
            println!("{}: {}", status.message, status.urls.join(" "));
        }
    }
}
