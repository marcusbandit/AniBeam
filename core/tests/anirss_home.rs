//! The core reads anirss's files where anirss keeps them (#610): anirss expands
//! `~/.config/anirss/config.toml` and `~/.local/state/anirss` from the home folder and never
//! reads the XDG variables, so an installed Anibeam does not either. One test in its own
//! binary: `HOME`, `USERPROFILE` and the XDG homes are process-wide.

use std::path::Path;

use anibeam_core::downloads::home::AnirssHome;
use anibeam_core::downloads::settings;

/// An anirss config naming qBittorrent at `url`, and the password anirss's CLI saved, under
/// `config` and `state`.
fn anirss_files(config: &Path, state: &Path, url: &str, password: &str) {
    std::fs::create_dir_all(config.join("anirss")).unwrap();
    std::fs::write(
        config.join("anirss/config.toml"),
        format!("[qbittorrent]\nurl = \"{url}\"\nusername = \"someone\"\n"),
    )
    .unwrap();
    std::fs::create_dir_all(state.join("anirss")).unwrap();
    std::fs::write(state.join("anirss/qbt.pass"), format!("{password}\n")).unwrap();
}

#[test]
fn the_persons_own_anirss_is_read_under_home_whatever_the_xdg_homes_say() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = home.path().join("elsewhere");
    anirss_files(
        &home.path().join(".config"),
        &home.path().join(".local/state"),
        "http://localhost:9091",
        "the-home-password",
    );
    // What a reader that followed the XDG variables would find instead
    anirss_files(
        &elsewhere.join("config"),
        &elsewhere.join("state"),
        "http://localhost:9092",
        "the-xdg-password",
    );
    unsafe {
        // The home folder comes from `HOME` on Unix and from `USERPROFILE` on Windows.
        std::env::set_var("HOME", home.path());
        std::env::set_var("USERPROFILE", home.path());
        std::env::set_var("XDG_CONFIG_HOME", elsewhere.join("config"));
        std::env::set_var("XDG_STATE_HOME", elsewhere.join("state"));
    }

    let own = AnirssHome::Shared;
    assert_eq!(
        own.config_path().unwrap(),
        home.path().join(".config/anirss/config.toml")
    );
    assert_eq!(
        own.state_dir().unwrap(),
        home.path().join(".local/state/anirss")
    );
    let fresh = settings::fresh(&own);
    assert_eq!(fresh.torrent_url, "http://localhost:9091");
    assert_eq!(fresh.torrent_username, "someone");
    assert_eq!(
        settings::anirss_saved_password(&own).as_deref(),
        Some("the-home-password")
    );
}
