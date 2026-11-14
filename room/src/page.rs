//! The two pages a relay serves to a browser, the same on either relay.
//!
//! - `GET /` is the find page: a field for a room code that goes to `/{code}`.
//!   Without script the form asks `/?code=...`, which [`code_from_query`] reads.
//! - `GET /{code}` is the join page a room link opens. Its Open Graph tags name
//!   the series and episode, so a link pasted into a chat previews as what the
//!   room watches. The button opens `anibeam://room/{code}` in the app; when the
//!   page is still showing a moment later, the app did not take the link, and
//!   the page turns to the code with a Copy button for the app's Join room.
//!
//! Both are self-contained: no outside resources, the script inline.

use crate::code;
use crate::proto::RoomView;

/// Where someone without the app gets it.
pub const DOWNLOAD_URL: &str = "https://anibeam.app/download/";

/// The look both pages share.
const STYLE: &str = r#"
:root { --bg: #07080a; --card: #111214; --field: #1a1b1e; --line: #2a2c31; --fg: #f4f5f7; --dim: #9a9ca3; --brand: #fdc901; --ink: #0d0e11; --red: #f26d6d; }
* { box-sizing: border-box; }
html, body { margin: 0; height: 100%; }
body { background: var(--bg); color: var(--fg); font: 16px/1.4 system-ui, -apple-system, "Segoe UI", sans-serif; display: grid; place-items: center; padding: 16px; }
main { width: 100%; max-width: 420px; background: var(--card); border-radius: 20px; padding: 32px 24px; text-align: center; }
.kicker { margin: 0 0 8px; color: var(--dim); font-size: 13px; letter-spacing: 0.08em; text-transform: uppercase; }
h1 { margin: 0; font-size: 26px; line-height: 1.2; overflow-wrap: anywhere; }
.episode { margin: 8px 0 0; color: var(--dim); overflow-wrap: anywhere; }
.code { margin: 24px 0; font: 600 32px/1 ui-monospace, "JetBrains Mono", monospace; letter-spacing: 0.18em; transition: font-size 0.2s ease; }
.open { display: block; width: 100%; padding: 14px; border: 0; border-radius: 12px; background: var(--brand); color: var(--ink); font-family: inherit; font-size: 16px; font-weight: 600; line-height: 1.4; text-decoration: none; cursor: pointer; }
.open:hover, .open:focus-visible { filter: brightness(1.08); }
.get { display: inline-block; margin-top: 16px; color: var(--dim); }
.get:hover, .get:focus-visible { color: var(--fg); }
.find { margin: 24px 0 0; display: grid; gap: 12px; }
.code-in { width: 100%; padding: 14px; border: 1px solid var(--line); border-radius: 12px; background: var(--field); color: var(--fg); font: 600 28px/1 ui-monospace, "JetBrains Mono", monospace; letter-spacing: 0.18em; text-align: center; text-transform: uppercase; outline: none; }
.code-in::placeholder { color: #4a4c52; }
.code-in:focus { border-color: var(--brand); }
.code-in[aria-invalid="true"] { border-color: var(--red); }
.missed { display: none; }
.gone .code { font-size: 44px; }
.gone .missed { display: block; }
.gone .retry { background: transparent; color: var(--fg); box-shadow: inset 0 0 0 1px var(--line); margin-top: 12px; }
.press { margin: 12px 0 0; color: var(--dim); }
"#;

/// A page around `body`: the shared head and look, `head` for the Open Graph tags.
fn page(title: &str, head: &str, body: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<meta name="color-scheme" content="dark">
<meta name="theme-color" content="#07080a">
<title>{title}</title>{head}
<style>{STYLE}</style>
</head>
<body>
{body}
</body>
</html>
"##
    )
}

/// The find page: a field for a room code and the button that goes there. `typed` is
/// what a script-less form sent that was not a code; it comes back in the field,
/// marked wrong.
pub fn find_html(typed: Option<&str>) -> String {
    let (value, invalid) = match typed {
        Some(t) => (
            format!(" value=\"{}\"", escape(t)),
            " aria-invalid=\"true\"",
        ),
        None => (String::new(), ""),
    };
    let alphabet = std::str::from_utf8(code::ALPHABET).unwrap_or_default();
    let len = code::LEN;
    let body = format!(
        r#"<main>
<p class="kicker">Watch together</p>
<h1>Join a room</h1>
<form class="find" id="find" action="/" method="get" autocomplete="off">
<input class="code-in" id="code" name="code" type="text" inputmode="text" autocapitalize="characters" spellcheck="false" maxlength="64" placeholder="K7QM2X" aria-label="Room code" autofocus{value}{invalid}>
<button class="open" type="submit">Join</button>
</form>
<a class="get" href="{DOWNLOAD_URL}">Get Anibeam</a>
</main>
<script>
(function () {{
  var A = "{alphabet}", N = {len};
  var form = document.getElementById("find"), field = document.getElementById("code");
  function clean(v) {{
    var parts = v.split("/").filter(function (p) {{ return p.trim() !== ""; }});
    v = (parts.length ? parts[parts.length - 1] : "").toUpperCase().replace(/[\s-]/g, "");
    return v;
  }}
  field.addEventListener("input", function () {{ field.removeAttribute("aria-invalid"); }});
  form.addEventListener("submit", function (e) {{
    e.preventDefault();
    var c = clean(field.value), ok = c.length === N;
    for (var i = 0; ok && i < c.length; i++) ok = A.indexOf(c[i]) >= 0;
    if (!ok) {{ field.setAttribute("aria-invalid", "true"); field.focus(); return; }}
    location.href = "/" + c;
  }});
}})();
</script>"#
    );
    page("Watch together in Anibeam", "", &body)
}

/// The room code in a find page's `?code=` query, canonical, when it reads as one.
/// A pasted room link counts too: its last path segment is the code. `Err` holds
/// what was typed when it was not a code, for the page to show again; `Ok(None)`
/// means the query asked nothing.
pub fn code_from_query(query: Option<&str>) -> Result<Option<String>, String> {
    let Some(raw) = query
        .unwrap_or("")
        .split('&')
        .find_map(|pair| pair.strip_prefix("code="))
    else {
        return Ok(None);
    };
    let typed = percent_decode(raw);
    if typed.trim().is_empty() {
        return Ok(None);
    }
    let last = typed
        .split('/')
        .rfind(|p| !p.trim().is_empty())
        .unwrap_or("");
    match code::normalize(last) {
        Some(c) => Ok(Some(c)),
        None => Err(typed),
    }
}

/// A form value as the browser sent it: `+` for a space and `%XX` escapes.
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match std::str::from_utf8(&bytes[i + 1..i + 3])
                    .ok()
                    .filter(|h| h.bytes().all(|b| b.is_ascii_hexdigit()))
                    .and_then(|h| u8::from_str_radix(h, 16).ok())
                {
                    Some(b) => {
                        out.push(b);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

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
                format!("{series} in AniBeam"),
                format!("Watch {series} together"),
                og_description,
                series,
                episode,
            )
        }
        None => (
            "AniBeam room".to_owned(),
            "AniBeam room".to_owned(),
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
    // The app takes the link and comes up over the browser; when the page is still
    // showing two seconds later, nothing took it (an install that never registered
    // anibeam://, or none at all), so the page turns to the code to type in by hand.
    let open = if room.is_some() {
        format!(
            r#"
<div class="missed">
<button class="open" id="copy" type="button">Copy code</button>
<p class="press">Open Anibeam and press Join room</p>
</div>
<a class="open retry" id="open" href="anibeam://room/{code}">Open in AniBeam</a>
<script>
(function () {{
  var main = document.querySelector("main"), open = document.getElementById("open");
  var copy = document.getElementById("copy"), code = document.getElementById("code");
  var timer = 0;
  open.addEventListener("click", function () {{
    clearTimeout(timer);
    timer = setTimeout(function () {{
      if (document.visibilityState === "visible") main.classList.add("gone");
    }}, 2000);
  }});
  document.addEventListener("visibilitychange", function () {{ if (document.hidden) clearTimeout(timer); }});
  function copied() {{ copy.textContent = "Copied"; }}
  function select() {{
    var r = document.createRange(), s = window.getSelection();
    r.selectNodeContents(code); s.removeAllRanges(); s.addRange(r);
    try {{ document.execCommand("copy"); copied(); }} catch (e) {{}}
  }}
  copy.addEventListener("click", function () {{
    if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(code.textContent).then(copied, select);
    else select();
  }});
}})();
</script>"#
        )
    } else {
        String::new()
    };
    let head = format!(
        r#"
<meta property="og:site_name" content="AniBeam">
<meta property="og:type" content="website">
<meta property="og:title" content="{og_title}">
<meta property="og:description" content="{og_description}">"#
    );
    let body = format!(
        r#"<main>
<p class="kicker">Watch together</p>
<h1>{heading}</h1>{episode}
<p class="code" id="code">{code}</p>{open}
<a class="get" href="{DOWNLOAD_URL}">Get AniBeam</a>
</main>"#
    );
    page(&title, &head, &body)
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
    fn the_join_page_falls_back_to_the_code_when_the_app_does_not_open() {
        let html = join_html("K7QM2X", Some(&view("S", Some(1.0), None)));
        assert!(html.contains(r#"<p class="code" id="code">K7QM2X</p>"#));
        assert!(html.contains(r#"id="copy""#));
        assert!(html.contains("Open Anibeam and press Join room"));
        assert!(html.contains("setTimeout"));
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
        assert!(!html.contains("<script>alert"));
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
        let html = find_html(Some("\"><b>"));
        assert!(!html.contains("<b>"));
        assert!(html.contains(r#"aria-invalid="true""#));
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
        assert!(!html.contains("<script"));
        assert!(html.contains(DOWNLOAD_URL));
    }

    #[test]
    fn the_find_page_asks_for_a_code_and_offers_the_app() {
        let html = find_html(None);
        assert!(html.contains(r#"name="code""#));
        assert!(html.contains(r#"action="/""#));
        assert!(html.contains(DOWNLOAD_URL));
        assert!(html.contains("ABCDEFGHJKLMNPQRSTUVWXYZ23456789"));
        assert!(!html.contains(r#"aria-invalid="true">"#));
    }

    #[test]
    fn a_query_code_is_read_like_a_typed_one() {
        assert_eq!(code_from_query(None), Ok(None));
        assert_eq!(code_from_query(Some("")), Ok(None));
        assert_eq!(code_from_query(Some("code=")), Ok(None));
        assert_eq!(code_from_query(Some("code=+")), Ok(None));
        assert_eq!(
            code_from_query(Some("code=k7qm2x")),
            Ok(Some("K7QM2X".into()))
        );
        assert_eq!(
            code_from_query(Some("x=1&code=k7q+m-2x")),
            Ok(Some("K7QM2X".into()))
        );
        assert_eq!(
            code_from_query(Some("code=k7q%20m%2D2x")),
            Ok(Some("K7QM2X".into()))
        );
        assert_eq!(
            code_from_query(Some("code=https%3A%2F%2Frooms.anibeam.app%2FK7QM2X%2F")),
            Ok(Some("K7QM2X".into()))
        );
        assert_eq!(code_from_query(Some("code=K7QM")), Err("K7QM".into()));
        assert_eq!(code_from_query(Some("code=I0I0I0")), Err("I0I0I0".into()));
        assert_eq!(code_from_query(Some("code=%zz")), Err("%zz".into()));
        assert_eq!(code_from_query(Some("code=%4")), Err("%4".into()));
    }

    #[test]
    fn self_contained_and_dash_free() {
        for html in [
            join_html("K7QM2X", Some(&view("S", Some(1.0), Some("T")))),
            join_html("K7QM2X", None),
            find_html(None),
            find_html(Some("nope")),
        ] {
            assert!(!html.contains('\u{2014}') && !html.contains('\u{2013}'));
            for needle in ["<link", "<script src", "<img", "src=", "@import", "url("] {
                assert!(!html.contains(needle), "{needle}");
            }
            assert!(html.starts_with("<!doctype html>"));
        }
    }
}
