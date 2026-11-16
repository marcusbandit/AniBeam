//! File and release names: which files are video, and the key two spellings
//! of one title share. The library's scan and watcher, the downloads' file
//! lists, rolling downloads and the release matcher all read names through
//! here, so they agree on what a video is.

use std::path::Path;

/// Every extension the app plays. mpv plays them all.
pub const VIDEO_EXTENSIONS: [&str; 11] = [
    "mkv", "mp4", "avi", "mov", "webm", "m4v", "ts", "m2ts", "wmv", "flv", "ogm",
];

/// Whether `name`'s extension, in any case, is on `list`. A name with no
/// extension, a bare `.mkv` among them, has none.
pub fn has_extension(name: &str, list: &[&str]) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| list.contains(&e.to_string_lossy().to_lowercase().as_str()))
}

/// Whether `name` is a video file the app plays.
pub fn is_video(name: &str) -> bool {
    has_extension(name, &VIDEO_EXTENSIONS)
}

/// `name` without a trailing video extension, matched in any case.
pub fn without_video_extension(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((stem, ext)) if VIDEO_EXTENSIONS.iter().any(|v| ext.eq_ignore_ascii_case(v)) => stem,
        _ => name,
    }
}

/// A title, a folder or a torrent's name as lower-case letters and digits
/// only, without the video or `.torrent` extension a client adds or drops,
/// so two spellings of one title compare equal. Empty for a name of only
/// punctuation, which then matches nothing.
pub fn title_key(name: &str) -> String {
    let name = name.trim();
    let stem = match name.rsplit_once('.') {
        Some((stem, ext)) if ext.eq_ignore_ascii_case("torrent") => stem,
        _ => without_video_extension(name),
    };
    stem.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_extension_is_a_video_in_any_case() {
        for ext in VIDEO_EXTENSIONS {
            assert!(is_video(&format!("Show - 01.{ext}")), "{ext}");
            assert!(
                is_video(&format!("Show - 01.{}", ext.to_uppercase())),
                "{ext}"
            );
        }
        assert!(is_video("a.MKV"));
        assert!(!is_video("a.srt"));
        assert!(!is_video("a.torrent"));
        assert!(!is_video("mkv"));
        assert!(!is_video(".mkv"));
    }

    #[test]
    fn the_lists_that_disagreed_now_take_the_same_files() {
        // The library scan never took these two; rolling downloads did.
        assert!(is_video("[Group] Show - 01.m2ts"));
        assert!(is_video("[Group] Show - 01.ogm"));
        // A torrent's file list never counted these; the scan did.
        assert!(is_video("Show - 01.mov"));
        assert!(is_video("Show - 01.flv"));
    }

    #[test]
    fn only_a_video_extension_comes_off() {
        assert_eq!(without_video_extension("Show - 01.MKV"), "Show - 01");
        assert_eq!(without_video_extension("Show - 01.m2ts"), "Show - 01");
        assert_eq!(without_video_extension("Show - 01.srt"), "Show - 01.srt");
        assert_eq!(
            without_video_extension("Show - 01.torrent"),
            "Show - 01.torrent"
        );
        assert_eq!(without_video_extension("Show"), "Show");
    }

    #[test]
    fn a_title_key_is_letters_and_digits_without_the_extension() {
        let want = "lumenkinonotabi031080pabcd1234";
        for name in [
            "[Lumen] Kino no Tabi - 03 (1080p) [ABCD1234]",
            "[Lumen] Kino no Tabi - 03 (1080p) [ABCD1234].mkv",
            "[Lumen] Kino no Tabi - 03 (1080p) [ABCD1234].webm",
            "[Lumen] Kino no Tabi - 03 (1080p) [ABCD1234].torrent",
            "  [Lumen] Kino no Tabi - 03 (1080p) [ABCD1234].MKV  ",
        ] {
            assert_eq!(title_key(name), want, "{name}");
        }
        assert_eq!(title_key("Ranma 1_2"), title_key("Ranma 1 2"));
        assert_eq!(title_key("Kino no Tabi.srt"), "kinonotabisrt");
        assert_eq!(title_key("--"), "");
    }

    #[test]
    fn a_folder_or_a_query_word_keys_the_way_it_always_did() {
        // The library's folder key and the downloads' link key had no
        // extension to drop; a name without one keys as before.
        assert_eq!(title_key("Sousou no Frieren"), "sousounofrieren");
        assert_eq!(
            title_key("Kusuriya no Hitorigoto S2"),
            "kusuriyanohitorigotos2"
        );
        assert_eq!(title_key("Ünïcödé Tïtle"), "ünïcödétïtle");
        // A film's file now keys as its folder does.
        assert_eq!(title_key("Perfect Blue.mkv"), title_key("Perfect Blue"));
    }
}
