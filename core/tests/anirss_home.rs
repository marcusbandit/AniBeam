//! The core reads anirss's files where anirss keeps them (#610): anirss expands
//! `~/.config/anirss/config.toml` and `~/.local/state/anirss` from the home folder and never
//! reads the XDG variables, so an installed AniBeam does not either. One test in its own
//! binary: `HOME` and the XDG homes are process-wide.

use anibeam_core::downloads::home::AnirssHome;

#[test]
fn the_persons_own_anirss_is_read_under_home_whatever_the_xdg_homes_say() {
    let home = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("HOME", home.path());
        std::env::set_var("XDG_CONFIG_HOME", home.path().join("elsewhere/config"));
        std::env::set_var("XDG_STATE_HOME", home.path().join("elsewhere/state"));
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
}
