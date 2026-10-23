//! player.toml and the mpv option lists the shell owns. Spec 4.4: the layers load through
//! `include` after init, and the options the shell owns are set last so no config line
//! can take them back.

use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, value};

use crate::paths::ShellPaths;

// The options every AniBeam player sets, shared with the phone.
pub use anibeam_core::playback::options::{owned_options, subtitle_options};

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerSettings {
    pub volume: f64,
    pub mute: bool,
    pub use_my_mpv_conf: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        PlayerSettings {
            volume: 100.0,
            mute: false,
            use_my_mpv_conf: false,
        }
    }
}

pub fn load(path: &Path) -> PlayerSettings {
    let d = PlayerSettings::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        return d;
    };
    let Ok(doc) = text.parse::<DocumentMut>() else {
        return d;
    };
    // An integer literal is a volume too, and anything outside mpv's 0 to 100 range is not
    // a volume at all, so it falls back rather than being clamped into a value nobody wrote.
    let volume = doc
        .get("volume")
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
        .filter(|v| (0.0..=100.0).contains(v))
        .unwrap_or(d.volume);
    PlayerSettings {
        volume,
        mute: doc.get("mute").and_then(|v| v.as_bool()).unwrap_or(d.mute),
        use_my_mpv_conf: doc
            .get("use_my_mpv_conf")
            .and_then(|v| v.as_bool())
            .unwrap_or(d.use_my_mpv_conf),
    }
}

pub fn save(path: &Path, s: &PlayerSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // The file the user already has is edited, not replaced: toml_edit keeps their
    // comments and any key a later task adds beside these three.
    let mut doc = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| t.parse::<DocumentMut>().ok())
        .unwrap_or_default();
    doc["volume"] = value(s.volume);
    doc["mute"] = value(s.mute);
    doc["use_my_mpv_conf"] = value(s.use_my_mpv_conf);
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, doc.to_string())?;
    std::fs::rename(tmp, path)
}

/// Haruna's MpvPreview recipe: its own core, nothing audible, nothing drawn but the frame.
pub fn preview_options() -> Vec<(&'static str, &'static str)> {
    vec![
        ("vo", "libmpv"),
        ("mute", "yes"),
        ("pause", "yes"),
        ("really-quiet", "yes"),
        ("hwdec", "auto"),
        ("hr-seek", "yes"),
        ("aid", "no"),
        ("audio-file-auto", "no"),
        ("sid", "no"),
        ("sub-auto", "no"),
        ("osd-level", "0"),
        ("audio-pitch-correction", "no"),
        ("use-text-osd", "no"),
        ("audio-display", "no"),
        ("keep-open", "always"),
    ]
}

/// The bundled file, then the user's own while the toggle is on, then AniBeam's own.
/// A layer that is not on disk is not a layer: mpv's `include` of a missing file is an
/// error the player would have nothing useful to say about.
pub fn config_layers(paths: &ShellPaths, use_my_conf: bool) -> Vec<PathBuf> {
    let mut out = vec![paths.bundled_mpv_conf()];
    if use_my_conf {
        out.push(paths.user_mpv_conf());
    }
    out.push(paths.anibeam_mpv_conf());
    out.into_iter().filter(|p| p.is_file()).collect()
}

/// The subtitle preview's fallback source: a black lavfi picture with one sample line,
/// through a generated SRT under the cache directory. The file is written once and reused
/// after that, so a preview opened twice does not rewrite it.
pub fn sample_preview(paths: &ShellPaths) -> (String, PathBuf) {
    let dir = paths.cache_dir();
    let srt = dir.join("sample.srt");
    if !srt.exists() {
        std::fs::create_dir_all(&dir).ok();
        std::fs::write(
            &srt,
            "1\n00:00:00,000 --> 01:00:00,000\nSample subtitle line\n",
        )
        .ok();
    }
    (
        "av://lavfi:color=c=0x101216:s=1280x720:d=3600".to_string(),
        srt,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::ShellPaths;

    #[test]
    fn player_toml_round_trips_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("player.toml");
        let d = load(&p);
        assert_eq!((d.volume, d.mute, d.use_my_mpv_conf), (100.0, false, false));
        save(
            &p,
            &PlayerSettings {
                volume: 42.5,
                mute: true,
                use_my_mpv_conf: true,
            },
        )
        .unwrap();
        let s = load(&p);
        assert_eq!((s.volume, s.mute, s.use_my_mpv_conf), (42.5, true, true));
        std::fs::write(&p, "volume = 900\n").unwrap();
        assert_eq!(
            load(&p).volume,
            100.0,
            "out of range falls back to the default"
        );
    }

    #[test]
    fn the_preview_core_hears_and_draws_nothing_but_the_frame() {
        let p = preview_options();
        assert!(
            p.contains(&("aid", "no"))
                && p.contains(&("sid", "no"))
                && p.contains(&("pause", "yes"))
                && p.contains(&("hr-seek", "yes"))
        );
    }

    #[test]
    fn config_layers_are_the_existing_files_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let bundled = dir.path().join("bundled.conf");
        // ShellPaths reads ANIBEAM_MPV_CONF once, inside resolve, into a field, so setting
        // the variable afterwards would change nothing; and writing the process
        // environment races every other test in this binary. The override goes into the
        // field instead, which is the same thing the variable sets.
        let mut paths = ShellPaths::resolve(Some(dir.path())).unwrap();
        paths.bundled_mpv_conf_override = Some(bundled.clone());
        std::fs::create_dir_all(paths.config_dir()).unwrap();
        std::fs::create_dir_all(paths.user_mpv_conf().parent().unwrap()).unwrap();
        std::fs::write(&bundled, "hwdec=auto\n").unwrap();
        std::fs::write(paths.user_mpv_conf(), "osc=yes\n").unwrap();
        std::fs::write(paths.anibeam_mpv_conf(), "deband=no\n").unwrap();
        let off = config_layers(&paths, false);
        assert_eq!(off, vec![bundled.clone(), paths.anibeam_mpv_conf()]);
        let on = config_layers(&paths, true);
        assert_eq!(
            on,
            vec![bundled, paths.user_mpv_conf(), paths.anibeam_mpv_conf()]
        );
        // With no override the bundled layer is the packaged file, and it is a layer only
        // on a machine where the package installed it.
        paths.bundled_mpv_conf_override = None;
        let packaged = crate::paths::share_dir().join("mpv.conf");
        assert_eq!(
            config_layers(&paths, true).contains(&packaged),
            packaged.is_file()
        );
    }
    #[test]
    fn the_sample_preview_writes_its_srt_once_under_the_cache_dir() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ShellPaths::resolve(Some(dir.path())).unwrap();
        let (source, srt) = sample_preview(&paths);
        assert_eq!(source, "av://lavfi:color=c=0x101216:s=1280x720:d=3600");
        assert_eq!(srt, paths.cache_dir().join("sample.srt"));
        let contents = std::fs::read_to_string(&srt).unwrap();
        assert_eq!(
            contents,
            "1\n00:00:00,000 --> 01:00:00,000\nSample subtitle line\n"
        );
        // A second call finds the file already there and leaves it alone rather than
        // rewriting it on every preview open.
        std::fs::write(&srt, "already here\n").unwrap();
        let (_, srt_again) = sample_preview(&paths);
        assert_eq!(srt_again, srt);
        assert_eq!(std::fs::read_to_string(&srt).unwrap(), "already here\n");
    }
}
