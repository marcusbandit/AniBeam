//! player.toml and the mpv option lists the shell owns. Spec 4.4: the layers load through
//! `include` after init, and the options the shell owns are set last so no config line
//! can take them back.

use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, value};

use crate::paths::ShellPaths;

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

/// Set after every include, in this order. The subtitle defaults, the language orders,
/// volume and mute follow from their own tables.
pub fn owned_options() -> Vec<(&'static str, String)> {
    [
        ("vo", "libmpv"),
        ("osc", "no"),
        ("osd-level", "0"),
        ("input-default-bindings", "no"),
        ("input-vo-keyboard", "no"),
        ("input-media-keys", "no"),
        ("resume-playback", "no"),
        ("save-position-on-quit", "no"),
        ("keep-open", "always"),
        ("pause", "no"),
        ("fullscreen", "no"),
        ("loop-file", "no"),
        ("loop-playlist", "no"),
        ("ytdl", "no"),
        ("sub-auto", "no"),
        ("audio-file-auto", "no"),
        ("reset-on-next-file", "sub-delay"),
        ("volume-max", "100"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.to_string()))
    .collect()
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

fn colour(c: &anibeam_core::Colour) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", c.a, c.r, c.g, c.b)
}

/// mpv reads "1", not "1.0", and a whole number written whole is what a user editing the
/// option by hand would have typed.
fn num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// Every field of SubtitleDefaults as one mpv option, spec 4.4's table.
pub fn subtitle_options(d: &anibeam_core::SubtitleDefaults) -> Vec<(&'static str, String)> {
    use anibeam_core::AssOverride;
    let s = &d.text_style;
    let mut o = vec![
        ("slang", d.subtitle_languages.join(",")),
        ("alang", d.audio_languages.join(",")),
        ("sub-scale", num(d.scale)),
        (
            "sub-ass-override",
            match d.ass_override {
                AssOverride::AsScripted => "no",
                AssOverride::ScaleOnly => "scale",
                AssOverride::Force => "force",
            }
            .to_string(),
        ),
        ("sub-font", s.font.clone()),
        ("sub-color", colour(&s.colour)),
        ("sub-outline-size", num(s.outline_size)),
        ("sub-outline-color", colour(&s.outline_colour)),
        ("sub-shadow-offset", num(s.shadow_offset)),
        ("sub-bold", if s.bold { "yes" } else { "no" }.to_string()),
        ("sub-pos", num(s.position)),
    ];
    // The box is an opacity, not a colour: any opacity at all switches mpv's border style
    // to the box and sets the black behind it, and none of it leaves the outline alone.
    if s.box_opacity > 0.0 {
        o.push(("sub-border-style", "background-box".to_string()));
        o.push((
            "sub-back-color",
            format!(
                "#{:02X}000000",
                (s.box_opacity.clamp(0.0, 1.0) * 255.0).round() as u8
            ),
        ));
    } else {
        o.push(("sub-border-style", "outline-and-shadow".to_string()));
        o.push(("sub-back-color", "#00000000".to_string()));
    }
    o
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::ShellPaths;
    use std::path::Path;

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
    fn the_owned_options_end_with_what_the_spec_lists() {
        let o = owned_options();
        let names: Vec<&str> = o.iter().map(|(n, _)| *n).collect();
        // The order is the point, not the membership: these are set after the config
        // layers, in this sequence, and `vo=libmpv` is the first of them.
        assert_eq!(
            names,
            vec![
                "vo",
                "osc",
                "osd-level",
                "input-default-bindings",
                "input-vo-keyboard",
                "input-media-keys",
                "resume-playback",
                "save-position-on-quit",
                "keep-open",
                "pause",
                "fullscreen",
                "loop-file",
                "loop-playlist",
                "ytdl",
                "sub-auto",
                "audio-file-auto",
                "reset-on-next-file",
                "volume-max",
            ]
        );
        assert_eq!(o.iter().find(|(n, _)| *n == "vo").unwrap().1, "libmpv");
        assert_eq!(
            o.iter().find(|(n, _)| *n == "keep-open").unwrap().1,
            "always"
        );
        assert_eq!(
            o.iter()
                .find(|(n, _)| *n == "reset-on-next-file")
                .unwrap()
                .1,
            "sub-delay"
        );
        assert_eq!(o.iter().find(|(n, _)| *n == "volume-max").unwrap().1, "100");
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
        let packaged = Path::new("/usr/share/anibeam/mpv.conf").to_path_buf();
        assert_eq!(
            config_layers(&paths, true).contains(&packaged),
            packaged.is_file()
        );
    }
    #[test]
    fn subtitle_defaults_become_mpv_options() {
        let mut d = anibeam_core::SubtitleDefaults::default();
        let o = subtitle_options(&d);
        let get = |k: &str| {
            o.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(get("slang"), "en");
        assert_eq!(get("alang"), "ja");
        assert_eq!(get("sub-scale"), "1");
        assert_eq!(get("sub-ass-override"), "scale");
        assert_eq!(get("sub-font"), "sans-serif");
        assert_eq!(get("sub-color"), "#FFFFFFFF");
        assert_eq!(get("sub-outline-size"), "1.65");
        assert_eq!(get("sub-outline-color"), "#FF000000");
        assert_eq!(get("sub-shadow-offset"), "0");
        assert_eq!(get("sub-bold"), "no");
        assert_eq!(get("sub-pos"), "100");
        assert_eq!(get("sub-border-style"), "outline-and-shadow");
        d.text_style.box_opacity = 0.5;
        d.ass_override = anibeam_core::AssOverride::Force;
        d.subtitle_languages = vec!["en".into(), "ja".into()];
        let o = subtitle_options(&d);
        let get = |k: &str| {
            o.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(get("sub-border-style"), "background-box");
        assert_eq!(get("sub-back-color"), "#80000000");
        assert_eq!(get("sub-ass-override"), "force");
        assert_eq!(get("slang"), "en,ja");
    }
}
