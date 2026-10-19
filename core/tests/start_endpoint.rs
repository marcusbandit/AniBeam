//! The confirmed click on an on-demand source: the core runs the endpoint's
//! start command (anirss's own, or the setting that overrides it), waits for
//! the ready URL and reports the status. A command the shell cannot find says
//! Jackett is not installed, and an endpoint that is not on demand is refused
//! up front. One test in its own binary because it points `XDG_CONFIG_HOME`
//! at an anirss config of its own, which is process-wide.

mod common;

use std::path::Path;

use anibeam_core::net::FakeHttp;
use anibeam_core::*;

fn remote(jackett_command: &str) -> DownloadSettings {
    DownloadSettings {
        anirss_mode: AnirssMode::Remote,
        anirss_command: "anirss".to_string(),
        prune_finished_feeds: false,
        jackett_command: jackett_command.to_string(),
        anirss_url: "http://anirss.test".to_string(),
        has_anirss_token: false,
        torrent_client: TorrentClient::Qbittorrent,
        torrent_url: "http://localhost:9090".to_string(),
        torrent_username: "me".to_string(),
        has_torrent_password: false,
    }
}

fn save(core: &Core, settings: DownloadSettings) {
    let reply = core
        .call(Call::SetDownloadSettings {
            settings,
            anirss_token: None,
            torrent_password: None,
        })
        .unwrap();
    assert_eq!(reply, Reply::Ok);
}

fn start(core: &Core, name: &str) -> u64 {
    match core
        .call(Call::StartEndpoint {
            name: name.to_string(),
        })
        .unwrap()
    {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

/// An anirss config with a plain nyaa endpoint and a 1337x one that has a
/// `start`, and the XDG config home pointed at it.
fn on_demand_config(dir: &Path, start: &str) {
    let anirss = dir.join("anirss");
    std::fs::create_dir_all(&anirss).unwrap();
    std::fs::write(
        anirss.join("config.toml"),
        format!(
            "[[endpoint]]\nname = \"nyaa\"\nkind = \"nyaa\"\nurl = \"https://nyaa.si/\"\n\n[[endpoint]]\nname = \"1337x\"\nkind = \"rss\"\nurl = \"http://127.0.0.1:9117/x?q={{query}}\"\nstart = \"{start}\"\nready = \"http://127.0.0.1:9117/caps\"\ntimeout = 5\n"
        ),
    )
    .unwrap();
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", dir);
        std::env::set_var("XDG_STATE_HOME", dir);
    }
}

/// What the start command wrote, once it has: the job can finish the moment the
/// ready URL answers, before the shell has got to its redirect.
fn written(path: &Path) -> String {
    for _ in 0..250 {
        if let Ok(text) = std::fs::read_to_string(path)
            && !text.is_empty()
        {
            return text.trim().to_string();
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    panic!("{} was never written", path.display())
}

#[test]
fn starting_an_endpoint_runs_its_command_and_names_why_it_cannot() {
    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let marker = dir.path().join("started");
    on_demand_config(dir.path(), &format!("echo anirss > {}", marker.display()));
    save(&core, remote(""));

    let refused = core.call(Call::StartEndpoint {
        name: "nyaa".to_string(),
    });
    assert!(
        matches!(refused, Err(CoreError::Invalid { ref field, .. }) if field == "name"),
        "{refused:?}"
    );

    // Down at the first look, answering on the next
    http.fail_next("refused");
    http.push(200, "ok");
    let event = common::wait_job(&c, start(&core, "1337x"));
    assert!(
        matches!(event.body, EventBody::DownloadsStatusChanged { .. }),
        "{:?}",
        event.body
    );
    assert_eq!(written(&marker), "anirss");

    // The setting wins over anirss's start
    let custom = dir.path().join("custom");
    save(
        &core,
        remote(&format!("echo setting > {}", custom.display())),
    );
    http.fail_next("refused");
    http.push(200, "ok");
    common::wait_job(&c, start(&core, "1337x"));
    assert_eq!(written(&custom), "setting");

    // Not installed: the shell's exit 127 for a command it cannot find
    save(&core, remote("anibeam-no-such-jackett"));
    http.fail_next("refused");
    let event = common::wait_job(&c, start(&core, "1337x"));
    let EventBody::JobFailed {
        error: CoreError::Service { code, message, .. },
    } = &event.body
    else {
        panic!("{:?}", event.body)
    };
    assert_eq!(code, "missing");
    assert!(message.starts_with("Jackett is not installed"), "{message}");
}
