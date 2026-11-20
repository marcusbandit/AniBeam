//! A sidecar is a subtitle file beside its video, matched by base name within
//! the same folder, with an optional `.lang.title` suffix.

use std::path::Path;

use anibeam_contract::Sidecar;

pub const SUBTITLE_EXTENSIONS: [&str; 4] = ["srt", "vtt", "ass", "ssa"];

pub fn is_subtitle(name: &str) -> bool {
    anibeam_foundation::names::has_extension(name, &SUBTITLE_EXTENSIONS)
}

/// The part between the video's base name and the subtitle extension: the
/// first piece is the language when it is two or three ASCII letters, the
/// rest joined by spaces is the title.
pub fn parse_suffix(suffix: &str) -> (Option<String>, Option<String>) {
    let parts: Vec<&str> = suffix.split('.').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return (None, None);
    }
    let first = parts[0];
    let is_lang = (2..=3).contains(&first.len()) && first.chars().all(|c| c.is_ascii_alphabetic());
    if is_lang {
        let title = parts[1..].join(" ");
        (
            Some(first.to_lowercase()),
            if title.is_empty() { None } else { Some(title) },
        )
    } else {
        (None, Some(parts.join(" ")))
    }
}

/// `candidates` are the subtitle file paths in the video's folder. A subtitle
/// belongs to the video when its stem equals the video's stem, or starts with
/// the video's stem followed by a dot.
pub fn match_sidecars(video_path: &Path, candidates: &[String]) -> Vec<Sidecar> {
    let stem = video_path.file_stem().unwrap_or_default().to_string_lossy();
    let prefix = format!("{stem}.");
    let mut out: Vec<Sidecar> = candidates
        .iter()
        .filter_map(|c| {
            let cstem = Path::new(c)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let (language, title) = if cstem == stem {
                (None, None)
            } else {
                parse_suffix(cstem.strip_prefix(&prefix)?)
            };
            Some(Sidecar {
                path: c.clone(),
                language,
                title,
            })
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// A file's sidecar subtitles, off the JSON column they live in. A row that
/// does not parse is a file with no sidecars rather than a card or a
/// session that cannot be built.
pub fn sidecars_of(raw: &str) -> Vec<Sidecar> {
    match serde_json::from_str::<Vec<Sidecar>>(raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("a file's sidecars did not parse, treating it as none: {e}");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_lang_title_suffix() {
        assert_eq!(parse_suffix("en"), (Some("en".to_string()), None));
        assert_eq!(
            parse_suffix("en.forced"),
            (Some("en".to_string()), Some("forced".to_string()))
        );
        assert_eq!(
            parse_suffix("eng.Full"),
            (Some("eng".to_string()), Some("Full".to_string()))
        );
        assert_eq!(
            parse_suffix("en.forced.subs"),
            (Some("en".to_string()), Some("forced subs".to_string()))
        );
        assert_eq!(parse_suffix("forced"), (None, Some("forced".to_string())));
        assert_eq!(parse_suffix("english"), (None, Some("english".to_string())));
        assert_eq!(parse_suffix(""), (None, None));
    }

    #[test]
    fn matches_sidecars_by_stem_and_leaves_other_episodes_out() {
        let video = Path::new("/x/Show - 01.mkv");
        let candidates = vec![
            "/x/Show - 01.srt".to_string(),
            "/x/Show - 01.en.forced.srt".to_string(),
            "/x/Show - 10.srt".to_string(),
            "/x/Other.srt".to_string(),
        ];
        let out = match_sidecars(video, &candidates);
        assert_eq!(
            out,
            vec![
                Sidecar {
                    path: "/x/Show - 01.en.forced.srt".to_string(),
                    language: Some("en".to_string()),
                    title: Some("forced".to_string())
                },
                Sidecar {
                    path: "/x/Show - 01.srt".to_string(),
                    language: None,
                    title: None
                },
            ]
        );
    }

    #[test]
    fn a_prefix_episode_number_does_not_match_a_longer_one() {
        let video = Path::new("/x/Show - 1.mkv");
        let candidates = vec!["/x/Show - 10.srt".to_string()];
        assert_eq!(match_sidecars(video, &candidates), Vec::<Sidecar>::new());
    }

    #[test]
    fn is_subtitle_by_extension() {
        assert!(is_subtitle("a.SRT"));
        assert!(is_subtitle("a.ass"));
        assert!(!is_subtitle("a.mkv"));
    }
}
