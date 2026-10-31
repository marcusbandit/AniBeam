//! The mpv options every AniBeam player sets, whichever shell draws it: the ones the app
//! owns, and the subtitle defaults as options. The desktop and the phone share these so a
//! file plays the same on both.

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
        ("reset-on-next-file", "sub-delay,audio-delay"),
        ("volume-max", "100"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v.to_string()))
    .collect()
}

fn colour(c: &crate::Colour) -> String {
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
pub fn subtitle_options(d: &crate::SubtitleDefaults) -> Vec<(&'static str, String)> {
    use crate::AssOverride;
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

#[cfg(test)]
mod tests {
    use super::*;

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
            "sub-delay,audio-delay"
        );
        assert_eq!(o.iter().find(|(n, _)| *n == "volume-max").unwrap().1, "100");
    }

    #[test]
    fn subtitle_defaults_become_mpv_options() {
        let mut d = crate::SubtitleDefaults::default();
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
        d.ass_override = crate::AssOverride::Force;
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
