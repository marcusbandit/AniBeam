//! A core under a root never reads, writes or logs in with the person's own anirss or
//! qBittorrent (#637). `HOME` and the XDG homes point at a stand-in for the person's own
//! home, holding an anirss config and the qBittorrent password anirss saved. The core takes
//! neither, the anirss it starts gets a home inside the root, a fresh install does not settle
//! on the qBittorrent the stand-in names, and nothing in the stand-in is created, changed or
//! removed. One test in its own binary: `HOME`, `PATH` and the XDG homes are process-wide.
//! The fakes are shell scripts, so the file is Unix only.
#![cfg(unix)]

mod common;

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use anibeam_core::net::FakeHttp;
use anibeam_core::*;
use serde_json::json;

/// The address and password only the stand-in's anirss knows.
const OWN_QBT: &str = "http://localhost:9090";
const OWN_PASSWORD: &str = "owners-own-password";

/// Every file and directory under `dir`, with its length and modification time.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, (u64, SystemTime)> {
    let mut out = BTreeMap::new();
    let mut todo = vec![dir.to_path_buf()];
    while let Some(d) = todo.pop() {
        for entry in std::fs::read_dir(&d).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            out.insert(entry.path(), (meta.len(), meta.modified().unwrap()));
            if meta.is_dir() {
                todo.push(entry.path());
            }
        }
    }
    out
}

/// What the fake wrote, once it has: FakeHttp answers the sidecar's health check
/// without the script having run.
fn captured(path: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(text) = std::fs::read_to_string(path)
            && !text.is_empty()
        {
            return text;
        }
        assert!(
            Instant::now() < deadline,
            "{} was never written",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn started(core: &Core, call: Call) -> u64 {
    match core.call(call).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    }
}

fn status(core: &Core) -> DownloadsStatus {
    match core.call(Call::GetDownloads).unwrap() {
        Reply::Downloads { status } => status,
        other => panic!("{other:?}"),
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

#[test]
fn a_core_under_a_root_keeps_anirss_and_qbittorrent_inside_it() {
    // The person's own home, as anirss and its CLI leave it
    let own = tempfile::tempdir().unwrap();
    let config = own.path().join(".config/anirss");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        config.join("config.toml"),
        format!("[qbittorrent]\nurl = \"{OWN_QBT}\"\nusername = \"owner\"\n"),
    )
    .unwrap();
    let state = own.path().join(".local/state/anirss");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join("qbt.pass"), format!("{OWN_PASSWORD}\n")).unwrap();
    for sub in [".local/share", ".cache"] {
        std::fs::create_dir_all(own.path().join(sub)).unwrap();
    }

    // A fake anirss that writes down the home it was given
    let bin = tempfile::tempdir().unwrap();
    let capture = bin.path().join("env");
    let anirss = bin.path().join("anirss");
    std::fs::write(
        &anirss,
        format!(
            "#!/bin/sh\nprintf '%s\\n%s\\n%s\\n%s\\n%s\\n%s' \"$HOME\" \"$XDG_CONFIG_HOME\" \
             \"$XDG_STATE_HOME\" \"$XDG_DATA_HOME\" \"$XDG_CACHE_HOME\" \"$ANIRSS_QBT_PASSWORD\" \
             > '{}'\nexec sleep 300\n",
            capture.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&anirss, std::fs::Permissions::from_mode(0o755)).unwrap();
    // A fake qbittorrent-nox, so a fresh install has AniBeam's own to fall back on
    let nox = bin.path().join("qbittorrent-nox");
    std::fs::write(&nox, "#!/bin/sh\nexec sleep 5\n").unwrap();
    std::fs::set_permissions(&nox, std::fs::Permissions::from_mode(0o755)).unwrap();

    let original_path = std::env::var("PATH").unwrap_or_default();
    unsafe {
        std::env::set_var("HOME", own.path());
        std::env::set_var("XDG_CONFIG_HOME", own.path().join(".config"));
        std::env::set_var("XDG_STATE_HOME", own.path().join(".local/state"));
        std::env::set_var("XDG_DATA_HOME", own.path().join(".local/share"));
        std::env::set_var("XDG_CACHE_HOME", own.path().join(".cache"));
        std::env::set_var(
            "PATH",
            format!("{}:{}", bin.path().display(), original_path),
        );
        std::env::set_var("ANIBEAM_QBITTORRENT_NOX", &nox);
    }
    let before = snapshot(own.path());

    let http = FakeHttp::new();
    let (dir, core, c) = common::open_core_with_http(http.clone());
    let home = dir.path().join("state/home");

    // Fresh settings: qBittorrent's stock address, not the one the person's anirss names,
    // and the anirss config the page names is the root's own.
    let fresh = status(&core);
    assert_eq!(fresh.settings.torrent_url, "http://localhost:8080");
    assert_eq!(fresh.settings.torrent_username, "admin");
    assert_eq!(
        PathBuf::from(&fresh.anirss.config_path),
        home.join(".config/anirss/config.toml")
    );

    // A fresh install settles on AniBeam's own qBittorrent without asking the address the
    // person's anirss names, and saves into the root's own home.
    let mut auto = fresh.settings.clone();
    auto.torrent_url = OWN_QBT.to_string();
    save(&core, auto);
    // Nothing on AniBeam's own port until it starts; then it answers
    http.fail_next("connection refused");
    for _ in 0..4 {
        http.push_for("app/version", 200, "v5.1.2");
    }
    let job = started(&core, Call::TestTorrentClient);
    common::wait_job(&c, job);
    let settled = status(&core).settings;
    assert_eq!(settled.torrent_mode, TorrentMode::Managed);
    assert!(
        http.requests().iter().all(|r| !r.url.starts_with(OWN_QBT)),
        "{:?}",
        http.requests().iter().map(|r| &r.url).collect::<Vec<_>>()
    );
    let profile = dir.path().join("data/qbittorrent/qBittorrent/config");
    let name = if cfg!(target_os = "macos") {
        "qBittorrent.ini"
    } else {
        "qBittorrent.conf"
    };
    let conf = std::fs::read_to_string(profile.join(name)).unwrap();
    assert!(
        conf.contains(&home.join("Downloads/Anime").display().to_string()),
        "{conf}"
    );

    // The person's own qBittorrent picked by hand: the anirss the core starts lives in the
    // root's home and never gets the password the person's anirss saved for it.
    let mut existing = settled.clone();
    existing.torrent_mode = TorrentMode::Existing;
    existing.torrent_url = OWN_QBT.to_string();
    existing.torrent_username = "owner".to_string();
    save(&core, existing);
    let health = json!({"ok": true, "version": "0.7.1", "features": ["qbt_env_session"]});
    http.push_for("/health", 200, health.to_string());
    http.push_for("/health", 200, health.to_string());
    http.push_for("/endpoints", 200, json!({"endpoints": []}).to_string());
    http.push_for(
        "/qbt/status",
        200,
        json!({"url": OWN_QBT, "reachable": true, "authenticated": true, "login_failed": false})
            .to_string(),
    );
    http.push_for("app/version", 200, "v5.1.2");
    let job = started(&core, Call::ConnectDownloads);
    common::wait_job(&c, job);
    let env = captured(&capture);
    let lines: Vec<&str> = env.split('\n').collect();
    let expect = |p: &str| home.join(p).display().to_string();
    assert_eq!(lines[0], home.display().to_string(), "{env}");
    assert_eq!(lines[1], expect(".config"), "{env}");
    assert_eq!(lines[2], expect(".local/state"), "{env}");
    assert_eq!(lines[3], expect(".local/share"), "{env}");
    assert_eq!(lines[4], expect(".cache"), "{env}");
    assert_eq!(lines.get(5).copied().unwrap_or(""), "", "{env}");
    for r in http.requests() {
        let sent = format!("{:?} {:?}", r.headers, r.body);
        assert!(!sent.contains(OWN_PASSWORD), "{} sent {sent}", r.url);
    }

    core.shutdown();
    unsafe {
        std::env::set_var("PATH", original_path);
        std::env::remove_var("ANIBEAM_QBITTORRENT_NOX");
    }
    assert_eq!(snapshot(own.path()), before, "the person's own home changed");
}
