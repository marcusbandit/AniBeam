//! Folder classification and the walk of one source. The walk decides what
//! each folder is by its structure alone, in three contexts: the source
//! itself, a Movies context, and everything else. Carried from
//! src/main/handlers/folderHandler.ts.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;

use regex::Regex;

use super::classifier::{
    Classified, classify, clean_episode_title, episode_key, strip_brackets_and_ext,
};
use super::labels::extra_label;
use super::sidecar::{is_subtitle, match_sidecars};
use super::{BRACKETS, SPACES, has_extension};
use crate::contract::{CoreError, SeriesKind, Sidecar};

pub const VIDEO_EXTENSIONS: [&str; 9] = [
    "mkv", "mp4", "avi", "mov", "webm", "m4v", "ts", "wmv", "flv",
];

#[derive(Clone, Debug, PartialEq)]
pub struct ScannedFile {
    pub path: String,
    pub size: u64,
    pub mtime: i64,
    pub classified: Classified,
    pub label: String,
    pub episode_key: String,
    pub sidecars: Vec<Sidecar>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScannedSeries {
    pub kind: SeriesKind,
    pub path: String,
    pub name: String,
    pub season_hint: Option<u32>,
    pub part_hint: Option<u32>,
    pub files: Vec<ScannedFile>,
}

pub fn is_video(name: &str) -> bool {
    has_extension(name, &VIDEO_EXTENSIONS)
}

/// Dot entries and the names a downloader gives an unfinished file.
pub fn is_ignored_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    name.starts_with('.')
        || lower.ends_with(".part")
        || lower.ends_with(".crdownload")
        || lower.ends_with(".tmp")
}

pub fn is_movies_folder_name(name: &str) -> bool {
    name.to_lowercase() == "movies"
}

static SEASON_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)Season\s*([0-9]+)").unwrap(),
        Regex::new(r"(?i)\bS([0-9]+)\b").unwrap(),
    ]
});
static PART_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)Part\s*([0-9]+)").unwrap(),
        Regex::new(r"(?i)\bP([0-9]+)\b").unwrap(),
    ]
});
static EPISODE_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\s+-\s+[0-9]+\s*[~\x{2013}-]\s*[0-9]+(\s+END)?\s*$").unwrap()
});
static TRAILING_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s+-?\s*END\s*$").unwrap());
static OVA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(?:OVA|OAD|ONA)s?\b").unwrap());
static SPECIAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bSpecials?\b").unwrap());
static ORDINAL_SEASON: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b([0-9]+)(?:st|nd|rd|th)\s+Season\b").unwrap());
static PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\(.*?\)\s*").unwrap());
static DOT_YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.[0-9]{4}\.").unwrap());
static PURE_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([0-9]+)$").unwrap());

fn first_number(patterns: &[Regex], s: &str) -> Option<u32> {
    patterns
        .iter()
        .find_map(|re| re.captures(s).and_then(|m| m[1].parse().ok()))
}

pub fn extract_season_number(folder_name: &str) -> Option<u32> {
    first_number(&SEASON_PATTERNS, folder_name)
}
pub fn extract_part_number(folder_name: &str) -> Option<u32> {
    first_number(&PART_PATTERNS, folder_name)
}

/// Release-group brackets, a trailing episode range and a trailing END gone.
pub fn clean_folder_title(name: &str) -> String {
    let s = BRACKETS.replace_all(name, " ");
    let s = EPISODE_RANGE.replace(&s, "");
    let s = TRAILING_END.replace(&s, "");
    SPACES.replace_all(&s, " ").trim().to_string()
}

enum SubfolderLabel {
    Season(u32),
    Ova,
    Special,
}

fn classify_subfolder_label(folder_name: &str) -> Option<SubfolderLabel> {
    let cleaned = clean_folder_title(folder_name);
    if OVA.is_match(&cleaned) {
        return Some(SubfolderLabel::Ova);
    }
    if SPECIAL.is_match(&cleaned) {
        return Some(SubfolderLabel::Special);
    }
    if let Some(m) = ORDINAL_SEASON.captures(&cleaned) {
        return m[1].parse().ok().map(SubfolderLabel::Season);
    }
    extract_season_number(&cleaned).map(SubfolderLabel::Season)
}

/// The byte range of `needle` in `haystack`, matched case-insensitively.
/// Walks `haystack` a character at a time and compares the lowercase of a
/// same-char-count window against the lowercase needle, so the returned
/// range always falls on `haystack`'s own char boundaries: a naive
/// `haystack.to_lowercase().find(&needle.to_lowercase())` instead returns a
/// byte offset into the LOWERCASED string, which drifts from the original
/// the moment a character's lowercase form has a different UTF-8 length
/// (`İ` lowercases to two chars, `ẞ` to a shorter one), producing an offset
/// that can split a multi-byte character or run past the string's end.
fn find_case_insensitive(haystack: &str, needle: &str) -> Option<std::ops::Range<usize>> {
    let needle_lower = needle.to_lowercase();
    let needle_chars = needle.chars().count();
    if needle_chars == 0 {
        return None;
    }
    let positions: Vec<usize> = haystack.char_indices().map(|(i, _)| i).collect();
    let total_chars = positions.len();
    if needle_chars > total_chars {
        return None;
    }
    for start in 0..=(total_chars - needle_chars) {
        let start_byte = positions[start];
        let end_byte = positions
            .get(start + needle_chars)
            .copied()
            .unwrap_or(haystack.len());
        if haystack[start_byte..end_byte].to_lowercase() == needle_lower {
            return Some(start_byte..end_byte);
        }
    }
    None
}

/// A wrapper's subfolder named from the wrapper, which the user named and
/// which is treated as canonical.
pub fn derive_subfolder_series_name(
    subfolder_name: &str,
    wrapper_name: &str,
) -> (String, Option<u32>) {
    let cleaned_sub = clean_folder_title(subfolder_name);
    let wrapper = wrapper_name.trim();
    if let Some(range) = find_case_insensitive(&cleaned_sub, wrapper) {
        let suffix = cleaned_sub.get(range.end..).unwrap_or("").trim();
        if suffix.is_empty() {
            return (wrapper.to_string(), None);
        }
        if let Some(m) = PURE_NUMBER.captures(suffix) {
            let n: u32 = m[1].parse().unwrap_or(0);
            return (format!("{wrapper} {n}"), Some(n));
        }
        return (format!("{wrapper} {suffix}"), None);
    }
    match classify_subfolder_label(subfolder_name) {
        Some(SubfolderLabel::Season(n)) if n <= 1 => (wrapper.to_string(), Some(n)),
        Some(SubfolderLabel::Season(n)) => (format!("{wrapper} Season {n}"), Some(n)),
        Some(SubfolderLabel::Ova) => (format!("{wrapper} OVA"), None),
        Some(SubfolderLabel::Special) => (format!("{wrapper} Specials"), None),
        None => (
            if cleaned_sub.is_empty() {
                wrapper.to_string()
            } else {
                cleaned_sub
            },
            None,
        ),
    }
}

/// Extension and brackets gone, then parentheses, `.YYYY.`, dots and
/// underscores to spaces, whitespace collapsed.
pub fn clean_movie_title(file_name: &str) -> String {
    let s = strip_brackets_and_ext(file_name);
    let s = PARENS.replace_all(&s, "");
    let s = DOT_YEAR.replace_all(&s, " ");
    let s = s.replace(['.', '_'], " ");
    SPACES.replace_all(&s, " ").trim().to_string()
}

struct Listing {
    dirs: Vec<PathBuf>,
    videos: Vec<PathBuf>,
    subtitles: Vec<String>,
}

fn list(dir: &Path) -> Listing {
    let mut out = Listing {
        dirs: Vec::new(),
        videos: Vec::new(),
        subtitles: Vec::new(),
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if is_ignored_name(&name) {
            continue;
        }
        let Ok(meta) = fs::metadata(&p) else { continue };
        if meta.is_dir() {
            out.dirs.push(p);
        } else if meta.is_file() && is_video(&name) {
            out.videos.push(p);
        } else if meta.is_file() && is_subtitle(&name) {
            out.subtitles.push(p.to_string_lossy().into_owned());
        }
    }
    out
}

/// A path's canonical form, symlinks and `..` resolved; the path itself when
/// canonicalisation fails (gone, a permission error), so a transient error
/// never hides a directory rather than merely failing to dedupe it.
fn canonical_path(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// One walk's memo. `listings` maps a literal path to the `Listing` already
/// read from it, so `classify_folder`'s shallow lookahead into a subfolder,
/// `collect`'s own listing of it moments later and a film's revisit of its
/// parent for sidecars all land on the same entry: a directory is only ever
/// read off disk once per walk.
///
/// `visited` is every directory's canonical identity, recorded the moment it
/// is accepted into some `Listing.dirs`. `list` drops a directory whose
/// canonical form is already in the set from the `dirs` it returns, before
/// anything classifies or recurses into it, so a `link -> ..` loop (ordinary
/// on a media share) can never regrow itself into the tree: the loop's
/// target is always an ancestor that was entered first, so the second sight
/// of it is filtered out at the source, and every consumer of `Listing.dirs`
/// (`has_videos_shallow`, `classify_folder`, `collect`, `collect_videos_in_subtree`)
/// sees a tree with no cycle to overflow the stack on.
struct WalkState {
    listings: HashMap<PathBuf, Rc<Listing>>,
    visited: HashSet<PathBuf>,
}

impl WalkState {
    fn new(root: &Path) -> Self {
        let mut visited = HashSet::new();
        visited.insert(canonical_path(root));
        Self {
            listings: HashMap::new(),
            visited,
        }
    }

    fn list(&mut self, dir: &Path) -> Rc<Listing> {
        if let Some(l) = self.listings.get(dir) {
            return Rc::clone(l);
        }
        let raw = list(dir);
        let dirs: Vec<PathBuf> = raw
            .dirs
            .into_iter()
            .filter(|d| self.visited.insert(canonical_path(d)))
            .collect();
        let l = Rc::new(Listing {
            dirs,
            videos: raw.videos,
            subtitles: raw.subtitles,
        });
        self.listings.insert(dir.to_path_buf(), Rc::clone(&l));
        l
    }
}

/// True when `dir` holds a video directly or one level down.
fn has_videos_shallow(state: &mut WalkState, dir: &Path) -> bool {
    let l = state.list(dir);
    if !l.videos.is_empty() {
        return true;
    }
    l.dirs.iter().any(|d| !state.list(d).videos.is_empty())
}

fn has_loose_videos(state: &mut WalkState, dir: &Path) -> bool {
    !state.list(dir).videos.is_empty()
}

enum Shape {
    Series,
    Wrapper(Vec<PathBuf>),
    Passthrough(PathBuf),
}

fn classify_folder(state: &mut WalkState, loose_videos: usize, dirs: &[PathBuf]) -> Shape {
    let video_bearing: Vec<PathBuf> = dirs
        .iter()
        .filter(|d| has_videos_shallow(state, d))
        .cloned()
        .collect();
    if video_bearing.len() >= 2 {
        return Shape::Wrapper(video_bearing);
    }
    if !video_bearing.is_empty() && loose_videos >= 1 {
        return Shape::Wrapper(video_bearing);
    }
    if video_bearing.len() == 1 && loose_videos == 0 && !has_loose_videos(state, &video_bearing[0])
    {
        return Shape::Passthrough(video_bearing[0].clone());
    }
    Shape::Series
}

fn file_meta(p: &Path) -> (u64, i64) {
    fs::metadata(p)
        .map(|m| {
            (
                m.len(),
                m.modified().ok().map(crate::time::to_secs).unwrap_or(0),
            )
        })
        .unwrap_or((0, 0))
}

fn scanned_file(p: &Path, folder_season: Option<u32>, subtitles: &[String]) -> ScannedFile {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut classified = classify(&name);
    if classified.season.is_none() {
        classified.season = folder_season;
    }
    let label = match classified.extra {
        Some(kind) => extra_label(
            kind,
            classified.extra_index,
            classified.extra_variant.as_deref(),
            classified.raw_label.as_deref(),
        ),
        None => clean_episode_title(&name),
    };
    let (size, mtime) = file_meta(p);
    ScannedFile {
        path: p.to_string_lossy().into_owned(),
        size,
        mtime,
        episode_key: episode_key(&classified, &name),
        classified,
        label,
        sidecars: match_sidecars(p, subtitles),
    }
}

fn film(state: &mut WalkState, p: &Path, use_folder_title: bool) -> ScannedSeries {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let folder = p
        .parent()
        .and_then(|d| d.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let title = if use_folder_title {
        clean_movie_title(&folder)
    } else {
        clean_movie_title(&name)
    };
    let listing = p.parent().map(|d| state.list(d));
    let subtitles: &[String] = listing
        .as_deref()
        .map(|l| l.subtitles.as_slice())
        .unwrap_or(&[]);
    let mut f = scanned_file(p, None, subtitles);
    f.classified.season = None;
    f.episode_key = name;
    ScannedSeries {
        kind: SeriesKind::Movie,
        path: p.to_string_lossy().into_owned(),
        name: title,
        season_hint: None,
        part_hint: None,
        files: vec![f],
    }
}

/// Every video under `root`, the folder's season hint passing to the files
/// beneath it, a `Season 2` subfolder overriding it. `WalkState.list` has
/// already dropped any subdirectory whose canonical form was seen before, so
/// this recursion always terminates even on a symlink loop back to an
/// ancestor.
fn collect_videos_in_subtree(
    state: &mut WalkState,
    root: &Path,
    inherited: Option<u32>,
) -> Vec<ScannedFile> {
    fn visit(
        state: &mut WalkState,
        dir: &Path,
        parent_season: Option<u32>,
        out: &mut Vec<ScannedFile>,
        seen: &mut HashSet<String>,
    ) {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let season_here = extract_season_number(&name).or(parent_season);
        let l = state.list(dir);
        for v in &l.videos {
            let f = scanned_file(v, season_here, &l.subtitles);
            if seen.insert(f.path.clone()) {
                out.push(f);
            }
        }
        for d in &l.dirs {
            visit(state, d, season_here, out, seen);
        }
    }
    let mut out = Vec::new();
    visit(state, root, inherited, &mut out, &mut HashSet::new());
    out
}

#[derive(Clone)]
struct Context {
    name: String,
    season_hint: Option<u32>,
}

fn collect(
    state: &mut WalkState,
    folder: &Path,
    results: &mut Vec<ScannedSeries>,
    is_root: bool,
    in_movies: bool,
    wrapper: Option<Context>,
    passthrough: Option<Context>,
) {
    let folder_name = folder
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let is_movies_container = is_movies_folder_name(&folder_name);
    let movies = in_movies || is_movies_container;
    let l = state.list(folder);

    if is_root {
        for v in &l.videos {
            results.push(film(state, v, false));
        }
        for d in &l.dirs {
            collect(state, d, results, false, movies, None, None);
        }
        return;
    }
    if movies {
        let use_folder = !is_movies_container && l.videos.len() == 1 && l.dirs.is_empty();
        for v in &l.videos {
            results.push(film(state, v, use_folder));
        }
        for d in &l.dirs {
            collect(state, d, results, false, true, None, None);
        }
        return;
    }
    let canonical = passthrough
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| folder_name.clone());
    match classify_folder(state, l.videos.len(), &l.dirs) {
        Shape::Passthrough(sub) => {
            let hint = passthrough
                .as_ref()
                .and_then(|p| p.season_hint)
                .or_else(|| extract_season_number(&folder_name));
            collect(
                state,
                &sub,
                results,
                false,
                false,
                None,
                Some(Context {
                    name: canonical,
                    season_hint: hint,
                }),
            );
        }
        Shape::Wrapper(subs) => {
            for v in &l.videos {
                results.push(film(state, v, false));
            }
            for sub in subs {
                let sub_name = sub
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let (name, season_hint) = derive_subfolder_series_name(&sub_name, &canonical);
                collect(
                    state,
                    &sub,
                    results,
                    false,
                    false,
                    Some(Context { name, season_hint }),
                    None,
                );
            }
        }
        Shape::Series => {
            let context = wrapper.or(passthrough);
            let season = extract_season_number(&folder_name)
                .or_else(|| context.as_ref().and_then(|c| c.season_hint));
            let part = extract_part_number(&folder_name);
            let mut files = collect_videos_in_subtree(state, folder, season);
            if files.is_empty() {
                return;
            }
            files.sort_by(|a, b| {
                let sa = a.classified.season.unwrap_or(0);
                let sb = b.classified.season.unwrap_or(0);
                sa.cmp(&sb)
                    .then(
                        a.classified
                            .number
                            .partial_cmp(&b.classified.number)
                            .unwrap_or(std::cmp::Ordering::Equal),
                    )
                    .then(a.path.cmp(&b.path))
            });
            let name = context
                .map(|c| c.name)
                .unwrap_or_else(|| folder_name.trim().to_string());
            results.push(ScannedSeries {
                kind: SeriesKind::Show,
                path: folder.to_string_lossy().into_owned(),
                name,
                season_hint: season,
                part_hint: part,
                files,
            });
        }
    }
}

/// The walk of one source. Blocking; the scan job runs it on a blocking thread.
///
/// The root's own listing is read here rather than left to `list`, which
/// treats a directory it cannot read as a directory with nothing in it.
/// That reading is right for a folder deep inside a library and wrong for
/// the root: a mount point whose drive is gone is still a directory, and an
/// empty answer from one would mark every series under it missing.
pub fn scan_source(root: &Path) -> Result<Vec<ScannedSeries>, CoreError> {
    let meta = fs::metadata(root).map_err(|e| CoreError::io_at(root.to_string_lossy(), e))?;
    if !meta.is_dir() {
        return Err(CoreError::Io {
            path: Some(root.to_string_lossy().into_owned()),
            message: "not a directory".into(),
        });
    }
    fs::read_dir(root).map_err(|e| CoreError::io_at(root.to_string_lossy(), e))?;
    let mut results = Vec::new();
    let mut state = WalkState::new(root);
    collect(&mut state, root, &mut results, true, false, None, None);
    Ok(results)
}

/// Every folder named Movies under the source, never descending into one.
/// `visited` is this call's own canonical-path memo (root registered up
/// front), so a symlink loop back to an ancestor is dropped rather than
/// recursed into forever.
pub fn find_movie_folders(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, found: &mut Vec<String>, visited: &mut HashSet<PathBuf>) {
        for d in list(dir).dirs {
            if !visited.insert(canonical_path(&d)) {
                continue;
            }
            let name = d
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if is_movies_folder_name(&name) {
                found.push(d.to_string_lossy().into_owned());
                continue;
            }
            walk(&d, found, visited);
        }
    }
    let mut found = Vec::new();
    if fs::metadata(root).is_ok_and(|m| m.is_dir()) {
        let mut visited = HashSet::new();
        visited.insert(canonical_path(root));
        walk(root, &mut found, &mut visited);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"x").unwrap();
    }

    #[test]
    fn helpers() {
        assert_eq!(extract_season_number("Season 2"), Some(2));
        assert_eq!(extract_season_number("S01"), Some(1));
        assert_eq!(extract_season_number("86"), None);
        assert_eq!(extract_part_number("Part 2"), Some(2));
        assert_eq!(extract_part_number("P01"), Some(1));
        assert_eq!(
            clean_folder_title(
                "[Erai-raws] Karakai Jouzu no Takagi-san 2 - 01 ~ 12 [1080p][Multiple Subtitle]"
            ),
            "Karakai Jouzu no Takagi-san 2"
        );
        assert_eq!(clean_folder_title("Show - 01-12 END"), "Show");
        assert_eq!(
            derive_subfolder_series_name(
                "[Erai-raws] Karakai Jouzu no Takagi-san 2 - 01 ~ 12 [1080p]",
                "Karakai Jouzu no Takagi-san"
            ),
            ("Karakai Jouzu no Takagi-san 2".to_string(), Some(2))
        );
        assert_eq!(
            derive_subfolder_series_name("[Judas] Kaminomi - S3", "Kami Nomi zo Shiru Sekai"),
            ("Kami Nomi zo Shiru Sekai Season 3".to_string(), Some(3))
        );
        assert_eq!(
            derive_subfolder_series_name("[Judas] Kaminomi - S1", "Kami Nomi zo Shiru Sekai"),
            ("Kami Nomi zo Shiru Sekai".to_string(), Some(1))
        );
        assert_eq!(
            derive_subfolder_series_name("[Judas] Kaminomi - OVAs", "Kami Nomi zo Shiru Sekai"),
            ("Kami Nomi zo Shiru Sekai OVA".to_string(), None)
        );
        assert_eq!(
            derive_subfolder_series_name("Specials", "Some Show"),
            ("Some Show Specials".to_string(), None)
        );
        assert_eq!(
            derive_subfolder_series_name("2nd Season", "Some Show"),
            ("Some Show Season 2".to_string(), Some(2))
        );
        assert_eq!(
            clean_movie_title("Perfect.Blue.1997.1080p.BluRay.mkv"),
            "Perfect Blue 1080p BluRay"
        );
        assert_eq!(
            clean_movie_title("Perfect Blue (1997) [Group].mkv"),
            "Perfect Blue"
        );
        assert_eq!(clean_movie_title("Your_Name.2016.mkv"), "Your Name 2016");
        assert!(is_movies_folder_name("MOVIES"));
        assert!(is_video("a.MKV"));
        assert!(!is_video("a.srt"));
        assert!(is_ignored_name(".stfolder"));
        assert!(is_ignored_name("ep01.mkv.part"));
    }

    #[test]
    fn derive_subfolder_series_name_survives_unicode_case_folding() {
        // İ (U+0130) lowercases to two chars ("i" plus a combining dot), so a
        // naive byte offset computed against the lowercased haystack lands
        // one byte short of "Show" in the original and used to panic slicing
        // mid-character.
        let (name, _) = derive_subfolder_series_name("İShowあ", "Show");
        assert!(!name.is_empty());
        // ẞ (U+1E9E) lowercases to the shorter ß, so the naive offset used to
        // run past the end of the two-byte original and panic out of range.
        let (name, _) = derive_subfolder_series_name("ß", "ẞ");
        assert!(!name.is_empty());
        // KELVIN SIGN (U+212A) lowercases to ASCII "k", three bytes shorter
        // than the original in UTF-8, so the naive offset used to land two
        // bytes early and silently keep "ow" as part of the suffix.
        assert_eq!(
            derive_subfolder_series_name("KShowあ", "Show"),
            ("Show あ".to_string(), None)
        );
    }

    #[test]
    fn takagi_wrapper_splits_into_three_series_and_a_film() {
        let tmp = tempfile::tempdir().unwrap();
        let show = tmp.path().join("Karakai Jouzu no Takagi-san");
        let seasons = [
            "[Erai-raws] Karakai Jouzu no Takagi-san - 01 ~ 12 [1080p][Multiple Subtitle]",
            "[Erai-raws] Karakai Jouzu no Takagi-san 2 - 01 ~ 12 [1080p][Multiple Subtitle]",
            "[Erai-raws] Karakai Jouzu no Takagi-san 3 - 01 ~ 12 [1080p][Multiple Subtitle]",
        ];
        for s in seasons {
            let stem = s
                .trim_start_matches("[Erai-raws] ")
                .split(" - 01 ~ 12")
                .next()
                .unwrap();
            for ep in 1..=12 {
                let tag = if ep == 12 { " END" } else { "" };
                touch(&show.join(s).join(format!(
                    "[Erai-raws] {stem} - {ep:02}{tag} [1080p][Multiple Subtitle].mkv"
                )));
            }
        }
        touch(&show.join("[Erai-raws] Karakai Jouzu no Takagi-san - Movie [1080p][49CCAF8A].mkv"));
        touch(&show.join("screenshots").join("cap01.png"));

        let results = scan_source(tmp.path()).unwrap();
        let series: Vec<&ScannedSeries> = results
            .iter()
            .filter(|r| r.kind == SeriesKind::Show)
            .collect();
        let movies: Vec<&ScannedSeries> = results
            .iter()
            .filter(|r| r.kind == SeriesKind::Movie)
            .collect();
        assert_eq!(
            series.len(),
            3,
            "{:?}",
            series.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
        assert_eq!(movies.len(), 1);
        for s in &series {
            assert_eq!(s.files.len(), 12);
            let mut eps: Vec<f64> = s.files.iter().map(|f| f.classified.number).collect();
            eps.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(eps, (1..=12).map(|n| n as f64).collect::<Vec<_>>());
            assert_ne!(Path::new(&s.path), show.as_path());
            assert!(s.path.starts_with(&format!("{}/", show.display())));
        }
        let by_name: std::collections::HashMap<&str, &ScannedSeries> =
            series.iter().map(|s| (s.name.as_str(), *s)).collect();
        assert_eq!(by_name["Karakai Jouzu no Takagi-san"].season_hint, None);
        assert_eq!(
            by_name["Karakai Jouzu no Takagi-san 2"].season_hint,
            Some(2)
        );
        assert_eq!(
            by_name["Karakai Jouzu no Takagi-san 3"].season_hint,
            Some(3)
        );
        assert_eq!(Path::new(&movies[0].path).parent().unwrap(), show.as_path());
        assert!(movies[0].name.contains("Karakai") && movies[0].name.contains("Movie"));
    }

    #[test]
    fn a_transparent_release_group_wrapper_stays_one_series() {
        let tmp = tempfile::tempdir().unwrap();
        for ep in 1..=3 {
            touch(
                &tmp.path()
                    .join("My Show")
                    .join("[Some-Group]")
                    .join(format!("My Show - {ep:02}.mkv")),
            );
        }
        let results = scan_source(tmp.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, SeriesKind::Show);
        assert_eq!(results[0].files.len(), 3);
        assert_eq!(results[0].name, "My Show");
    }

    #[test]
    fn a_flat_series_is_one_series() {
        let tmp = tempfile::tempdir().unwrap();
        for ep in 1..=4 {
            touch(
                &tmp.path()
                    .join("Plain Show")
                    .join(format!("Plain Show - {ep:02}.mkv")),
            );
        }
        let results = scan_source(tmp.path()).unwrap();
        assert_eq!(
            (results.len(), results[0].kind, results[0].files.len()),
            (1, SeriesKind::Show, 4)
        );
    }

    #[test]
    fn one_subdir_plus_one_loose_video_is_a_wrapper() {
        let tmp = tempfile::tempdir().unwrap();
        let show = tmp.path().join("Edge Show");
        for ep in 1..=3 {
            touch(
                &show
                    .join("[Group]")
                    .join(format!("Edge Show - {ep:02}.mkv")),
            );
        }
        touch(&show.join("Edge Show - OVA.mkv"));
        let results = scan_source(tmp.path()).unwrap();
        assert_eq!(
            results
                .iter()
                .filter(|r| r.kind == SeriesKind::Show)
                .count(),
            1
        );
        assert_eq!(
            results
                .iter()
                .filter(|r| r.kind == SeriesKind::Movie)
                .count(),
            1
        );
    }

    #[test]
    fn season_folders_behind_a_release_folder_split_by_season() {
        let tmp = tempfile::tempdir().unwrap();
        let show = tmp.path().join("Kami Nomi zo Shiru Sekai");
        let release = show.join("[Judas] Kami Nomi zo Shiru Sekai (The World God Only Knows) (Seasons 1-3 + OVAs) [BD 1080p][HEVC x265 10bit][Dual-Audio][Eng-Subs]");
        for s in 1..=3 {
            for ep in 1..=12 {
                touch(
                    &release
                        .join(format!("[Judas] Kaminomi - S{s}"))
                        .join(format!(
                            "[Judas] Kami nomi zo Shiru Sekai - S0{s}E{ep:02}.mkv"
                        )),
                );
            }
        }
        for f in [
            "[Judas] Kaminomi OAD - Four Plus an Idol.mkv",
            "[Judas] Kaminomi OVA - Magical Star Kanon 100%.mkv",
            "[Judas] Kaminomi OVA - Tenri Arc 01.mkv",
            "[Judas] Kaminomi OVA - Tenri Arc 02.mkv",
        ] {
            touch(&release.join("[Judas] Kaminomi - OVAs").join(f));
        }
        let results = scan_source(tmp.path()).unwrap();
        let series: Vec<&ScannedSeries> = results
            .iter()
            .filter(|r| r.kind == SeriesKind::Show)
            .collect();
        assert_eq!(
            series.len(),
            4,
            "{:?}",
            series.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
        let files: Vec<&str> = series
            .iter()
            .flat_map(|s| s.files.iter().map(|f| f.path.as_str()))
            .collect();
        assert_eq!(files.len(), 40);
        assert_eq!(
            files.iter().collect::<std::collections::HashSet<_>>().len(),
            40
        );
        let by_name: std::collections::HashMap<&str, &ScannedSeries> =
            series.iter().map(|s| (s.name.as_str(), *s)).collect();
        for (name, eps, season) in [
            ("Kami Nomi zo Shiru Sekai", 12, Some(1)),
            ("Kami Nomi zo Shiru Sekai Season 2", 12, Some(2)),
            ("Kami Nomi zo Shiru Sekai Season 3", 12, Some(3)),
            ("Kami Nomi zo Shiru Sekai OVA", 4, None),
        ] {
            let s = by_name
                .get(name)
                .unwrap_or_else(|| panic!("missing {name}"));
            assert_eq!(s.files.len(), eps);
            assert_eq!(s.season_hint, season);
            assert!(s.path.starts_with(&format!("{}/", release.display())));
        }
        for name in [
            "Kami Nomi zo Shiru Sekai",
            "Kami Nomi zo Shiru Sekai Season 2",
            "Kami Nomi zo Shiru Sekai Season 3",
        ] {
            let mut eps: Vec<f64> = by_name[name]
                .files
                .iter()
                .map(|f| f.classified.number)
                .collect();
            eps.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(eps, (1..=12).map(|n| n as f64).collect::<Vec<_>>());
        }
    }

    #[test]
    fn plain_season_subfolders_take_the_shows_name() {
        let tmp = tempfile::tempdir().unwrap();
        for (dir, n) in [("Season 1", 3), ("Season 2", 3), ("Specials", 2)] {
            for ep in 1..=n {
                touch(
                    &tmp.path()
                        .join("Some Show")
                        .join(dir)
                        .join(format!("{dir} - {ep:02}.mkv")),
                );
            }
        }
        let mut names: Vec<String> = scan_source(tmp.path())
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == SeriesKind::Show)
            .map(|s| s.name)
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec!["Some Show", "Some Show Season 2", "Some Show Specials"]
        );
    }

    #[test]
    fn movies_folders_films_and_sidecars() {
        let tmp = tempfile::tempdir().unwrap();
        touch(&tmp.path().join("Movies").join("Perfect Blue (1997).mkv"));
        touch(&tmp.path().join("Movies").join("Perfect Blue (1997).en.srt"));
        touch(
            &tmp.path()
                .join("Movies")
                .join("Akira")
                .join("Akira.1988.BluRay.mkv"),
        );
        touch(&tmp.path().join("Loose Film.mkv"));
        touch(&tmp.path().join("Show").join("Show - 01.mkv"));
        touch(&tmp.path().join("Show").join("Show - 01.en.forced.srt"));
        touch(&tmp.path().join("Show").join("Show - 01.ass"));
        touch(&tmp.path().join("Show").join("Show - 10.srt"));
        touch(&tmp.path().join("Show").join(".hidden.mkv"));
        touch(&tmp.path().join("Show").join("Show - 02.mkv.part"));
        let results = scan_source(tmp.path()).unwrap();
        let films: Vec<&ScannedSeries> = results
            .iter()
            .filter(|r| r.kind == SeriesKind::Movie)
            .collect();
        let names: std::collections::HashSet<&str> =
            films.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            ["Perfect Blue", "Akira", "Loose Film"]
                .into_iter()
                .collect()
        );
        let blue = films.iter().find(|f| f.name == "Perfect Blue").unwrap();
        assert_eq!(blue.files[0].episode_key, "Perfect Blue (1997).mkv");
        assert_eq!(
            blue.files[0].sidecars,
            vec![Sidecar {
                path: tmp
                    .path()
                    .join("Movies/Perfect Blue (1997).en.srt")
                    .to_string_lossy()
                    .into_owned(),
                language: Some("en".into()),
                title: None
            }]
        );
        let show = results.iter().find(|r| r.name == "Show").unwrap();
        assert_eq!(show.files.len(), 1);
        let mut langs: Vec<(Option<String>, Option<String>)> = show.files[0]
            .sidecars
            .iter()
            .map(|s| (s.language.clone(), s.title.clone()))
            .collect();
        langs.sort();
        assert_eq!(
            langs,
            vec![(None, None), (Some("en".into()), Some("forced".into()))]
        );
        assert_eq!(
            find_movie_folders(tmp.path()),
            vec![tmp.path().join("Movies").to_string_lossy().into_owned()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_loop_does_not_overflow_the_stack() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let lib = tmp.path().join("lib");
        touch(&lib.join("Show").join("ep01.mkv"));
        symlink("..", lib.join("Show").join("loop")).unwrap();
        let results = scan_source(&lib).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, SeriesKind::Show);
        assert_eq!(results[0].files.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn find_movie_folders_survives_the_same_symlink_loop() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let lib = tmp.path().join("lib");
        touch(&lib.join("Show").join("ep01.mkv"));
        symlink("..", lib.join("Show").join("loop")).unwrap();
        assert_eq!(find_movie_folders(&lib), Vec::<String>::new());
    }
}
