//! The join page a room link opens: `GET /{code}` on either relay.
//!
//! One self-contained page with no outside resources. Its Open Graph tags name
//! the series and episode, so a link pasted into a chat previews as what the
//! room watches. The button opens `anibeam://room/{code}` in the app; the other
//! link is for someone who does not have it yet.

use crate::proto::RoomView;

/// Where someone without the app gets it.
pub const DOWNLOAD_URL: &str = "https://anibeam.app/download/";

/// The join page for `code`. `room` is `None` when no such room is open, and the
/// page says so.
pub fn join_html(code: &str, room: Option<&RoomView>) -> String {
    let code = escape(code);
    let (title, og_title, og_description, heading, episode) = match room {
        Some(room) => {
            let series = escape(&room.episode.series_title);
            let episode = escape(&episode_line(room));
            let og_description = if episode.is_empty() {
                format!("Room {code}")
            } else {
                format!("{episode}. Room {code}")
            };
            (
                format!("{series} in Anibeam"),
                format!("Watch {series} together"),
                og_description,
                series,
                episode,
            )
        }
        None => (
            "Anibeam room".to_owned(),
            "Anibeam room".to_owned(),
            "This room is closed, or the code is wrong.".to_owned(),
            "Room not found".to_owned(),
            "It may have closed. Ask for a new link.".to_owned(),
        ),
    };
    let episode = if episode.is_empty() {
        String::new()
    } else {
        format!("\n<p class=\"episode\">{episode}</p>")
    };
    let open = if room.is_some() {
        format!("\n<a class=\"open\" href=\"anibeam://room/{code}\">Open in Anibeam</a>")
    } else {
        String::new()
    };
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<meta name="color-scheme" content="dark">
<meta name="theme-color" content="#07080a">
<title>{title}</title>
<meta property="og:site_name" content="Anibeam">
<meta property="og:type" content="website">
<meta property="og:title" content="{og_title}">
<meta property="og:description" content="{og_description}">
<style>
:root {{ --bg: #07080a; --card: #111214; --fg: #f4f5f7; --dim: #9a9ca3; --brand: #fdc901; --ink: #0d0e11; }}
* {{ box-sizing: border-box; }}
html, body {{ margin: 0; height: 100%; }}
body {{ background: var(--bg); color: var(--fg); font: 16px/1.4 system-ui, -apple-system, "Segoe UI", sans-serif; display: grid; place-items: center; padding: 16px; }}
main {{ width: 100%; max-width: 420px; background: var(--card); border-radius: 20px; padding: 32px 24px; text-align: center; }}
.kicker {{ margin: 0 0 8px; color: var(--dim); font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; }}
h1 {{ margin: 0; font-size: 26px; line-height: 1.2; overflow-wrap: anywhere; }}
.episode {{ margin: 8px 0 0; color: var(--dim); overflow-wrap: anywhere; }}
.code {{ margin: 24px 0; font: 600 32px/1 ui-monospace, "JetBrains Mono", monospace; letter-spacing: 0.18em; }}
.open {{ display: block; padding: 14px; border-radius: 12px; background: var(--brand); color: var(--ink); font-weight: 600; text-decoration: none; }}
.open:hover, .open:focus-visible {{ filter: brightness(1.08); }}
.get {{ display: inline-block; margin-top: 16px; color: var(--dim); }}
.get:hover, .get:focus-visible {{ color: var(--fg); }}
</style>
</head>
<body>
<main>
<p class="kicker">Watch together</p>
<h1>{heading}</h1>{episode}
<p class="code">{code}</p>{open}
<a class="get" href="{DOWNLOAD_URL}">Get Anibeam</a>
</main>
</body>
</html>
"##
    )
}

/// "Episode 5: Title", "Episode 5", "Title", or nothing, unescaped.
fn episode_line(room: &RoomView) -> String {
    let e = &room.episode;
    let title = e
        .episode_title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    match (e.number, title) {
        (Some(n), Some(t)) => format!("Episode {}: {t}", number(n)),
        (Some(n), None) => format!("Episode {}", number(n)),
        (None, Some(t)) => t.to_owned(),
        (None, None) => String::new(),
    }
}

fn number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e9 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Escapes text for HTML element content and quoted attribute values.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{EpisodeRef, MAX_MEMBERS, PlayState};

    fn view(series: &str, number: Option<f64>, title: Option<&str>) -> RoomView {
        RoomView {
            code: "K7QM2X".into(),
            host: None,
            members: Vec::new(),
            episode: EpisodeRef {
                anilist_id: None,
                mal_id: None,
                tvmaze_id: None,
                episode_key: "k".into(),
                number,
                series_title: series.into(),
                episode_title: title.map(Into::into),
                duration: None,
                opening_start: None,
            },
            state: PlayState {
                playing: false,
                position: 0.0,
                rate: 1.0,
                at_ms: 0,
                rev: 0,
            },
            max: MAX_MEMBERS,
        }
    }

    #[test]
    fn names_the_series_and_episode_in_the_preview_and_the_page() {
        let html = join_html(
            "K7QM2X",
            Some(&view("Mushishi", Some(3.0), Some("Tender Horns"))),
        );
        assert!(html.contains(r#"<meta property="og:title" content="Watch Mushishi together">"#));
        assert!(html.contains(
            r#"<meta property="og:description" content="Episode 3: Tender Horns. Room K7QM2X">"#
        ));
        assert!(html.contains("<h1>Mushishi</h1>"));
        assert!(html.contains(r#"<p class="episode">Episode 3: Tender Horns</p>"#));
        assert!(html.contains(r#"href="anibeam://room/K7QM2X""#));
        assert!(html.contains(r#"href="https://anibeam.app/download/""#));
    }

    #[test]
    fn escapes_what_the_host_typed() {
        let html = join_html(
            "K7QM2X",
            Some(&view(
                r#"<script>alert("x")</script> & 'co'"#,
                Some(1.5),
                Some("\"><img src=x>"),
            )),
        );
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(
            html.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp; &#39;co&#39;")
        );
        assert!(html.contains("Episode 1.5: &quot;&gt;&lt;img src=x&gt;"));
    }

    #[test]
    fn escapes_the_code_too() {
        let html = join_html("\"><b>", None);
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn episode_lines() {
        assert_eq!(episode_line(&view("S", Some(12.0), None)), "Episode 12");
        assert_eq!(episode_line(&view("S", None, Some(" Pilot "))), "Pilot");
        assert_eq!(episode_line(&view("S", None, Some("  "))), "");
        assert_eq!(episode_line(&view("S", None, None)), "");
        let html = join_html("K7QM2X", Some(&view("S", None, None)));
        assert!(!html.contains(r#"class="episode""#));
        assert!(html.contains(r#"content="Room K7QM2X""#));
    }

    #[test]
    fn a_missing_room_says_so_and_offers_no_open_button() {
        let html = join_html("K7QM2X", None);
        assert!(html.contains("Room not found"));
        assert!(!html.contains("anibeam://"));
        assert!(html.contains(DOWNLOAD_URL));
    }

    #[test]
    fn self_contained_and_dash_free() {
        for html in [
            join_html("K7QM2X", Some(&view("S", Some(1.0), Some("T")))),
            join_html("K7QM2X", None),
        ] {
            assert!(!html.contains('\u{2014}') && !html.contains('\u{2013}'));
            for needle in ["<link", "<script", "<img", "src=", "@import", "url("] {
                assert!(!html.contains(needle), "{needle}");
            }
            assert!(html.starts_with("<!doctype html>"));
        }
    }
}
