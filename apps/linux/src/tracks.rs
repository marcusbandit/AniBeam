//! The track pick, spec 4.4: the series' track choice first (exact kind, language and
//! title; then kind and language; then language), then the language orders (a sidecar
//! beats an embedded track, dialogue beats signs), then the first subtitle track and the
//! file's default audio, so a file never plays unsubbed by accident.

use anibeam_core::{SubtitleChoice, SubtitleDefaults, TrackChoice, TrackKind, TrackRef};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: i64,
    pub kind: String,
    pub lang: Option<String>,
    pub title: Option<String>,
    pub default: bool,
    pub external: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pick {
    pub aid: Option<i64>,
    pub sid: Option<i64>,
}

/// mpv's `track-list`, observed as a node. An entry without an id or a type is not a
/// track, so it is dropped rather than guessed at.
pub fn parse(list: &Value) -> Vec<Track> {
    list.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| {
                    Some(Track {
                        id: t.get("id")?.as_i64()?,
                        kind: t.get("type")?.as_str()?.to_string(),
                        lang: t.get("lang").and_then(Value::as_str).map(String::from),
                        title: t.get("title").and_then(Value::as_str).map(String::from),
                        default: t.get("default").and_then(Value::as_bool).unwrap_or(false),
                        external: t.get("external").and_then(Value::as_bool).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The two and three letter codes for the languages a library like this one carries. The
/// list is deliberately short: a code not on it still matches itself.
const PAIRS: [(&str, &str); 14] = [
    ("en", "eng"),
    ("ja", "jpn"),
    ("de", "ger"),
    ("de", "deu"),
    ("fr", "fre"),
    ("fr", "fra"),
    ("es", "spa"),
    ("it", "ita"),
    ("pt", "por"),
    ("ru", "rus"),
    ("zh", "chi"),
    ("zh", "zho"),
    ("ko", "kor"),
    ("ar", "ara"),
];

/// One code down to its two letter form, with the region suffix gone and mpv's "und",
/// undetermined, down to nothing.
fn canon(l: &str) -> String {
    let l = l.to_ascii_lowercase();
    let base = l.split(['-', '_']).next().unwrap_or(&l);
    // The whole code, not a substring: a replace would reach inside any code that happened
    // to spell those three letters, and it left an undetermined track unable to match even
    // the ref taken from itself.
    if base == "und" {
        return String::new();
    }
    PAIRS
        .iter()
        .find(|(_, three)| base == *three)
        .map(|(two, _)| (*two).to_string())
        .unwrap_or_else(|| base.to_string())
}

/// Two and three letter codes are the same to mpv, so they are the same here. An
/// undetermined tag is no language, so it is the same as another undetermined tag and as
/// nothing else: a language order never reaches for it, but a choice stored on one still
/// finds it again.
pub fn same_lang(a: &str, b: &str) -> bool {
    canon(a) == canon(b)
}

fn lang_matches(t: &Track, lang: &str) -> bool {
    t.lang.as_deref().is_some_and(|l| same_lang(l, lang))
}

/// A track meant to carry the dialogue rather than the signs, the songs, a forced line or
/// a commentary. Nothing in the title is the ordinary case, so it says yes.
fn is_dialogue(t: &Track) -> bool {
    let title = t.title.as_deref().unwrap_or("").to_ascii_lowercase();
    !["signs", "songs", "forced", "commentary"]
        .iter()
        .any(|w| title.contains(w))
}

/// The stored choice against what this file actually carries, loosening a step at a time:
/// kind, language and title; then kind and language; then the language alone.
fn by_ref<'a>(tracks: &'a [Track], kind: &str, r: &TrackRef) -> Option<&'a Track> {
    let of_kind: Vec<&Track> = tracks.iter().filter(|t| t.kind == kind).collect();
    let kind_ok = |t: &Track| match r.kind {
        TrackKind::Sidecar => t.external,
        TrackKind::Embedded => !t.external,
    };
    let lang_ok = |t: &Track| match &r.language {
        Some(l) => lang_matches(t, l),
        None => t.lang.is_none(),
    };
    let title_ok = |t: &Track| {
        r.title
            .as_deref()
            .is_none_or(|x| x.eq_ignore_ascii_case(t.title.as_deref().unwrap_or("")))
    };
    of_kind
        .iter()
        .copied()
        .find(|t| kind_ok(t) && lang_ok(t) && title_ok(t))
        .or_else(|| of_kind.iter().copied().find(|t| kind_ok(t) && lang_ok(t)))
        .or_else(|| {
            r.language
                .as_deref()
                .and_then(|l| of_kind.iter().copied().find(|t| lang_matches(t, l)))
        })
}

pub fn pick(tracks: &[Track], choice: &TrackChoice, defaults: &SubtitleDefaults) -> Pick {
    let subs: Vec<&Track> = tracks.iter().filter(|t| t.kind == "sub").collect();
    let audio: Vec<&Track> = tracks.iter().filter(|t| t.kind == "audio").collect();

    let sid = match &choice.subtitle {
        Some(SubtitleChoice::Off) => None,
        // A stored track that this file still has wins outright; one it no longer has
        // falls through to the language orders rather than leaving the file unsubbed.
        Some(SubtitleChoice::Track { track }) if by_ref(tracks, "sub", track).is_some() => {
            by_ref(tracks, "sub", track).map(|t| t.id)
        }
        _ => defaults
            .subtitle_languages
            .iter()
            .find_map(|lang| {
                let in_lang: Vec<&Track> = subs
                    .iter()
                    .copied()
                    .filter(|t| lang_matches(t, lang))
                    .collect();
                if in_lang.is_empty() {
                    return None;
                }
                in_lang
                    .iter()
                    .copied()
                    .find(|t| t.external && is_dialogue(t))
                    .or_else(|| in_lang.iter().copied().find(|t| t.external))
                    .or_else(|| in_lang.iter().copied().find(|t| is_dialogue(t)))
                    .or_else(|| in_lang.first().copied())
                    .map(|t| t.id)
            })
            .or_else(|| subs.first().map(|t| t.id)),
    };

    let aid = choice
        .audio
        .as_ref()
        .and_then(|r| by_ref(tracks, "audio", r))
        .map(|t| t.id)
        .or_else(|| {
            defaults.audio_languages.iter().find_map(|lang| {
                audio
                    .iter()
                    .copied()
                    .find(|t| lang_matches(t, lang))
                    .map(|t| t.id)
            })
        })
        .or_else(|| {
            audio
                .iter()
                .copied()
                .find(|t| t.default)
                .or_else(|| audio.first().copied())
                .map(|t| t.id)
        });

    Pick { aid, sid }
}

pub fn track_ref(t: &Track) -> TrackRef {
    TrackRef {
        kind: if t.external {
            TrackKind::Sidecar
        } else {
            TrackKind::Embedded
        },
        language: t.lang.clone(),
        title: t.title.clone(),
    }
}

/// "Full Dialogue (eng)", "English (eng, sidecar)", "spa", "Track 2".
pub fn label(t: &Track) -> String {
    let mut extra: Vec<&str> = Vec::new();
    if let Some(l) = &t.lang {
        extra.push(l);
    }
    if t.external {
        extra.push("sidecar");
    }
    match (&t.title, extra.is_empty()) {
        (Some(title), true) => title.clone(),
        (Some(title), false) => format!("{title} ({})", extra.join(", ")),
        (None, false) => extra.join(", "),
        (None, true) => format!("Track {}", t.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::{SubtitleChoice, SubtitleDefaults, TrackChoice, TrackKind, TrackRef};
    use serde_json::json;

    fn list() -> Vec<Track> {
        parse(&json!([
            { "id": 1, "type": "video" },
            { "id": 1, "type": "audio", "lang": "jpn", "title": "Japanese", "default": true },
            { "id": 2, "type": "audio", "lang": "eng", "title": "English Dub" },
            { "id": 1, "type": "sub", "lang": "eng", "title": "Signs & Songs" },
            { "id": 2, "type": "sub", "lang": "eng", "title": "Full Dialogue" },
            { "id": 3, "type": "sub", "lang": "eng", "title": "English", "external": true, "external-filename": "/x/ep.en.srt" },
            { "id": 4, "type": "sub", "lang": "spa" }
        ]))
    }

    fn defaults() -> SubtitleDefaults {
        SubtitleDefaults::default()
    }

    #[test]
    fn language_orders_prefer_a_sidecar_and_dialogue_and_the_first_audio_match() {
        let p = pick(&list(), &TrackChoice::default(), &defaults());
        assert_eq!(
            p.sid,
            Some(3),
            "the sidecar beats the embedded English tracks"
        );
        assert_eq!(p.aid, Some(1), "ja is first in the audio order");
        let no_sidecar: Vec<Track> = list().into_iter().filter(|t| !t.external).collect();
        assert_eq!(
            pick(&no_sidecar, &TrackChoice::default(), &defaults()).sid,
            Some(2),
            "dialogue beats signs"
        );
        let mut d = defaults();
        d.subtitle_languages = vec!["es".into()];
        d.audio_languages = vec!["en".into()];
        let p = pick(&list(), &TrackChoice::default(), &d);
        assert_eq!(
            (p.aid, p.sid),
            (Some(2), Some(4)),
            "two letter codes match three letter tags"
        );
    }

    #[test]
    fn a_track_choice_wins_by_kind_language_and_title_then_loosens() {
        let exact = TrackChoice {
            audio: Some(TrackRef {
                kind: TrackKind::Embedded,
                language: Some("eng".into()),
                title: Some("English Dub".into()),
            }),
            subtitle: Some(SubtitleChoice::Track {
                track: TrackRef {
                    kind: TrackKind::Embedded,
                    language: Some("en".into()),
                    title: Some("Signs & Songs".into()),
                },
            }),
        };
        let p = pick(&list(), &exact, &defaults());
        assert_eq!((p.aid, p.sid), (Some(2), Some(1)));
        let loose = TrackChoice {
            audio: None,
            subtitle: Some(SubtitleChoice::Track {
                track: TrackRef {
                    kind: TrackKind::Sidecar,
                    language: Some("en".into()),
                    title: Some("gone".into()),
                },
            }),
        };
        assert_eq!(
            pick(&list(), &loose, &defaults()).sid,
            Some(3),
            "kind and language match when the title is gone"
        );
        let lang_only = TrackChoice {
            audio: None,
            subtitle: Some(SubtitleChoice::Track {
                track: TrackRef {
                    kind: TrackKind::Embedded,
                    language: Some("es".into()),
                    title: None,
                },
            }),
        };
        assert_eq!(pick(&list(), &lang_only, &defaults()).sid, Some(4));
        let off = TrackChoice {
            audio: None,
            subtitle: Some(SubtitleChoice::Off),
        };
        assert_eq!(
            pick(&list(), &off, &defaults()).sid,
            None,
            "off applies as off"
        );
    }

    #[test]
    fn with_nothing_matching_the_first_subtitle_and_the_default_audio_play() {
        let mut d = defaults();
        d.subtitle_languages = vec!["fr".into()];
        d.audio_languages = vec!["fr".into()];
        let p = pick(&list(), &TrackChoice::default(), &d);
        assert_eq!((p.aid, p.sid), (Some(1), Some(1)));
    }

    #[test]
    fn refs_and_labels() {
        let l = list();
        let r = track_ref(&l[5]);
        assert_eq!(r.kind, TrackKind::Sidecar);
        assert_eq!(r.language.as_deref(), Some("eng"));
        assert_eq!(r.title.as_deref(), Some("English"));
        assert_eq!(label(&l[4]), "Full Dialogue (eng)");
        assert_eq!(label(&l[5]), "English (eng, sidecar)");
        assert_eq!(label(&l[6]), "spa");
        assert!(
            same_lang("en", "eng")
                && same_lang("ja", "jpn")
                && same_lang("EN", "en")
                && !same_lang("en", "es")
        );
    }

    #[test]
    fn an_undetermined_tag_is_no_language_but_still_finds_its_own_ref() {
        let tracks = parse(&json!([
            { "id": 1, "type": "sub", "lang": "und", "title": "Unknown" },
            { "id": 2, "type": "sub", "lang": "eng", "title": "English" }
        ]));
        assert!(
            same_lang("und", "und") && !same_lang("und", "en"),
            "an undetermined tag is only ever itself"
        );
        assert!(
            !same_lang("sund", "s"),
            "the whole code is tested, so a code that spells und inside itself is left alone"
        );
        let stored = TrackChoice {
            audio: None,
            subtitle: Some(SubtitleChoice::Track {
                track: track_ref(&tracks[0]),
            }),
        };
        assert_eq!(
            pick(&tracks, &stored, &defaults()).sid,
            Some(1),
            "a choice stored on an undetermined track finds it again"
        );
        assert_eq!(
            pick(&tracks, &TrackChoice::default(), &defaults()).sid,
            Some(2),
            "but the en order never reaches for it"
        );
    }
}
