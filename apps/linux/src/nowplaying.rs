//! The two MPRIS lines, carried from Electron's `src/shared/nowPlaying.ts`. A title counts
//! as a name unless it is empty, the show's name, a bare episode token, or the show's name
//! followed by separators and such a token.

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

const SEP: &str = " · ";

fn fold(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// "Episode 5", "Ep 5", "Ep. 5", "E05", "5", "#5", with the number equal to `number`.
fn is_episode_token(s: &str, number: Option<u32>) -> bool {
    let s = s.trim();
    let rest = ["episode", "ep.", "ep", "e", "#"]
        .iter()
        .find_map(|p| s.strip_prefix(p))
        .unwrap_or(s)
        .trim_start();
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match number {
        Some(n) => rest.parse::<u32>().ok() == Some(n),
        None => true,
    }
}

pub fn is_real_episode_title(title: &str, show: &str, number: Option<u32>) -> bool {
    let t = fold(title);
    if t.is_empty() {
        return false;
    }
    let s = fold(show);
    if !s.is_empty() && t == s {
        return false;
    }
    let rest = if !s.is_empty() && t.starts_with(&s) {
        t[s.len()..]
            .trim_start_matches(|c: char| {
                c.is_whitespace() || matches!(c, '-' | '\u{2013}' | '\u{2014}' | ':' | '_')
            })
            .to_string()
    } else {
        t
    };
    !is_episode_token(&rest, number)
}

/// (title, artist).
pub fn lines(
    show: &str,
    episode_number: Option<u32>,
    episode_title: Option<&str>,
    extra_label: Option<&str>,
) -> (String, String) {
    let show = show.trim().to_string();
    if let Some(extra) = extra_label.map(str::trim).filter(|s| !s.is_empty()) {
        return (extra.to_string(), show);
    }
    let Some(n) = episode_number else {
        return (show, String::new());
    };
    let episode = format!("Episode {n}");
    if let Some(t) = episode_title.map(str::trim).filter(|s| !s.is_empty())
        && is_real_episode_title(t, &show, Some(n))
    {
        let artist = if show.is_empty() {
            episode
        } else {
            format!("{show}{SEP}{episode}")
        };
        return (t.to_string(), artist);
    }
    (show, episode)
}

/// Everything a path segment may not carry in a URL. `/` is not in the set, because the
/// path is split on it first and the separators are put back after.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// An absolute path as the `file://` URL MPRIS's `mpris:artUrl` wants.
pub fn art_url(path: &str) -> String {
    let segments: Vec<String> = path
        .split('/')
        .map(|s| utf8_percent_encode(s, PATH_SEGMENT).to_string())
        .collect();
    format!("file://{}", segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_real_episode_title_is_the_title_line() {
        assert_eq!(
            lines("Dungeon Meshi", Some(12), Some("Red Dragon"), None),
            ("Red Dragon".into(), "Dungeon Meshi · Episode 12".into())
        );
    }

    #[test]
    fn a_bare_token_or_the_show_name_is_not_a_title() {
        for t in [
            "Episode 5",
            "Ep 5",
            "Ep. 5",
            "E05",
            "5",
            "#5",
            "Dungeon Meshi",
            "Dungeon Meshi - Episode 5",
            "Dungeon Meshi: E05",
            "",
        ] {
            assert!(!is_real_episode_title(t, "Dungeon Meshi", Some(5)), "{t}");
            assert_eq!(
                lines("Dungeon Meshi", Some(5), Some(t), None),
                ("Dungeon Meshi".into(), "Episode 5".into())
            );
        }
        assert!(
            is_real_episode_title("Episode 6", "Dungeon Meshi", Some(5)),
            "a different number is a title"
        );
    }

    #[test]
    fn extras_and_films() {
        assert_eq!(
            lines("Girls und Panzer", None, None, Some("Opening 1")),
            ("Opening 1".into(), "Girls und Panzer".into())
        );
        assert_eq!(
            lines("Koe no Katachi", None, None, None),
            ("Koe no Katachi".into(), String::new())
        );
    }

    #[test]
    fn the_art_url_is_a_file_url_with_escaped_segments() {
        assert_eq!(
            art_url("/home/b/.cache/anibeam/images/ab/x y.jpg"),
            "file:///home/b/.cache/anibeam/images/ab/x%20y.jpg"
        );
        assert_eq!(art_url("/a/#b/c.png"), "file:///a/%23b/c.png");
    }
}
