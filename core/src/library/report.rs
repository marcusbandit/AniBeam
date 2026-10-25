//! The quick fix's GitHub issue (#170): a filled in new-issue page for the
//! user to read and send. It carries the fix twice, as a table for people and
//! as a JSON record under `<!-- anibeam:quick-fix v1 -->` for the triage
//! process. Nothing is sent from here; the page opens in the browser and the
//! user sends it or not.
//!
//! With Claude Code or Codex installed, the core first asks that agent for a
//! short triage of the misreading, run in an empty folder with the name
//! reader's own source in the prompt: Claude Code with no tools at all and
//! nothing saved, Codex in its read only sandbox with its shell tool off. What
//! it answers goes into the issue, which the user reads before sending. An
//! agent that fails or runs out of time costs the issue nothing but its triage.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::{Connection, params, params_from_iter};
use serde_json::json;

use crate::contract::*;
use crate::core::Core;
use crate::library::{cards, readings};
use crate::percent;
use crate::store::sql::placeholders;

const NEW_ISSUE: &str = "https://github.com/marcusbandit/AniBeam/issues/new";
/// What the triage process looks for in a quick fix's issue.
pub const MARKER: &str = "<!-- anibeam:quick-fix v1 -->";
/// GitHub answers a long enough URL with 414 and documents no number; reports
/// put the edge between two and eight thousand characters. The page stays
/// under this, trimming the triage and then the file list to fit.
const URL_BUDGET: usize = 7000;
const TRIAGE_TIMEOUT: Duration = Duration::from_secs(180);
const TRIAGE_CAP: usize = 1800;
/// The name reader, handed to the agent so it reads the rules it is judging.
const CLASSIFIER: &str = include_str!("classifier.rs");

struct Facts {
    app: String,
    title: String,
    folder: String,
    anilist: Option<u64>,
    mal: Option<u64>,
    tvmaze: Option<u64>,
    format: Option<String>,
    episodes: Option<u32>,
    files: Vec<Fixed>,
}

struct Fixed {
    name: String,
    was: FileReading,
    now: FileReading,
}

fn facts(
    conn: &Connection,
    images_dir: &Path,
    series: u64,
    files: &[u64],
) -> Result<Facts, CoreError> {
    let title = cards::cards_for(conn, images_dir, &[series])?
        .first()
        .map(|c| c.title.clone())
        .ok_or(CoreError::NotFound {
            what: Entity::Series,
            id: series,
        })?;
    let (kind, path, anilist, mal, tvmaze, format, episodes) = conn.query_row(
        "SELECT s.kind, s.path, s.anilist_id, s.mal_id, s.tvmaze_id, m.format, m.episodes
         FROM series s LEFT JOIN anilist_media m ON m.id = s.anilist_id WHERE s.id = ?1",
        params![series as i64],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                r.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                r.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<i64>>(6)?.map(|v| v as u32),
            ))
        },
    )?;
    let folder = folder_of(&kind, &path);
    let mut fixed = Vec::new();
    if !files.is_empty() {
        let sql = format!(
            "SELECT f.path, x.was, x.reading FROM files f JOIN file_fixes x ON x.path = f.path
             WHERE f.series_id = ? AND f.id IN ({}) ORDER BY f.path",
            placeholders(files.len())
        );
        let mut stmt = conn.prepare(&sql)?;
        let ids = std::iter::once(series as i64).chain(files.iter().map(|f| *f as i64));
        let rows = stmt.query_map(params_from_iter(ids), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (path, was, now) = row?;
            if let (Ok(was), Ok(now)) = (serde_json::from_str(&was), serde_json::from_str(&now)) {
                fixed.push(Fixed {
                    name: plain(readings::file_name(&path)).unwrap_or_else(|| "file".to_string()),
                    was,
                    now,
                });
            }
        }
    }
    Ok(Facts {
        app: crate::VERSION.to_string(),
        title,
        folder,
        anilist,
        mal,
        tvmaze,
        format,
        episodes,
        files: fixed,
    })
}

/// One plain path component: what the triage process (docs/agents/triage.md)
/// makes a file or folder of, so it refuses anything else. Not empty, not `.`
/// or `..`, no `/`, no NUL.
fn plain(component: String) -> Option<String> {
    let ok = !component.is_empty()
        && component != "."
        && component != ".."
        && !component.contains(['/', '\0']);
    ok.then_some(component)
}

/// The folder the series' files sit in, by its name on disk: a show's own
/// folder (a `Season 2` subfolder stays `Season 2`, where the series is titled
/// off its parent), a film's parent folder. The scan reads names in that
/// folder's context, so the triage rebuilds the case in a folder named the same.
fn folder_of(kind: &str, path: &str) -> String {
    let path = Path::new(path);
    let dir = if kind == "movie" {
        path.parent()
    } else {
        Some(path)
    };
    dir.and_then(Path::file_name)
        .and_then(|n| plain(n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "series".to_string())
}

// The agents --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum AgentKind {
    Claude,
    Codex,
}

struct Agent {
    kind: AgentKind,
    path: PathBuf,
}

impl Agent {
    fn name(&self) -> &'static str {
        match self.kind {
            AgentKind::Claude => "Claude Code",
            AgentKind::Codex => "Codex",
        }
    }
}

/// Where the agents install when not on PATH: an app started from a desktop
/// launcher, the Mac's Finder especially, gets a short PATH of its own.
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        for sub in [
            ".local/bin",
            ".claude/local",
            ".npm-global/bin",
            ".bun/bin",
            ".volta/bin",
        ] {
            dirs.push(home.join(sub));
        }
    }
    for dir in [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/home/linuxbrew/.linuxbrew/bin",
        "/usr/bin",
    ] {
        dirs.push(PathBuf::from(dir));
    }
    let mut seen = std::collections::HashSet::new();
    dirs.retain(|d| seen.insert(d.clone()));
    dirs
}

fn executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        p.metadata()
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

/// Claude Code first, then Codex. `ANIBEAM_TRIAGE_AGENT` names a stand in
/// that reads the prompt on stdin and answers on stdout the way Claude Code
/// does (drives and tests), or `none`.
fn find_agent() -> Option<Agent> {
    if let Some(v) = std::env::var_os("ANIBEAM_TRIAGE_AGENT") {
        if v == "none" {
            return None;
        }
        let path = PathBuf::from(v);
        return executable(&path).then_some(Agent {
            kind: AgentKind::Claude,
            path,
        });
    }
    let dirs = search_dirs();
    [(AgentKind::Claude, "claude"), (AgentKind::Codex, "codex")]
        .into_iter()
        .find_map(|(kind, bin)| {
            dirs.iter()
                .map(|d| d.join(bin))
                .find(|p| executable(p))
                .map(|path| Agent { kind, path })
        })
}

/// A pipe read to its end on a thread of its own, its text sent once it ends.
fn drain(pipe: Option<impl Read + Send + 'static>) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut out = String::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_string(&mut out);
        }
        let _ = tx.send(out);
    });
    rx
}

/// How long the pipes may stay open after the agent itself has exited: a
/// helper it left behind can hold them open for good.
const PIPE_GRACE: Duration = Duration::from_secs(5);

/// Ends the agent and everything it started. It runs in a process group of its
/// own (`process_group(0)`), so the group shares its pid; a wrapper such as an
/// npm shim takes the real agent down with it. Called only while the agent or
/// something it started is still running, so the group id is still theirs.
fn end_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// One read only run of the agent on `prompt`, fed on stdin in a fresh empty
/// folder: Claude Code with no tools, no customisations and nothing saved,
/// Codex in its read only sandbox with its shell tool off. Its answer, or why
/// there is none.
fn run_agent(agent: &Agent, prompt: &str, timeout: Duration) -> Result<String, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("anibeam-triage-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let answer = dir.join("answer.md");
    let mut cmd = Command::new(&agent.path);
    match agent.kind {
        AgentKind::Claude => {
            cmd.args([
                "-p",
                "--safe-mode",
                "--tools",
                "",
                "--no-session-persistence",
                "--output-format",
                "text",
            ]);
        }
        AgentKind::Codex => {
            // The read only sandbox still reads the whole disk, and the prompt
            // carries names from torrent releases, so the shell tool goes off
            // too: an answer in text, nothing run.
            cmd.args([
                "exec",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "-c",
                "features.shell_tool=false",
                "--output-last-message",
            ])
            .arg(&answer)
            .arg("-");
        }
    }
    // Its own folder first on the PATH, so a script-installed agent finds the
    // runtime installed beside it.
    let mut path = Vec::new();
    if let Some(own) = agent.path.parent() {
        path.push(own.to_path_buf());
    }
    path.extend(search_dirs());
    if let Ok(joined) = std::env::join_paths(path) {
        cmd.env("PATH", joined);
    }
    cmd.current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let result = (|| {
        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        let stdin = child.stdin.take();
        let prompt = prompt.to_string();
        let writer = thread::spawn(move || {
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(prompt.as_bytes());
            }
        });
        let out = drain(child.stdout.take());
        let err = drain(child.stderr.take());
        let deadline = Instant::now() + timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    end_group(&mut child);
                    return Err(format!("no answer within {}", span(timeout)));
                }
                Ok(None) => thread::sleep(Duration::from_millis(200)),
                Err(e) => {
                    end_group(&mut child);
                    return Err(e.to_string());
                }
            }
        };
        let stdout = out.recv_timeout(PIPE_GRACE);
        let stderr = err.recv_timeout(PIPE_GRACE);
        // A pipe still open means something the agent started still holds it,
        // so its group is alive and goes too. Only then: once the group is gone
        // its id is free for another process to take.
        if stdout.is_err() || stderr.is_err() {
            end_group(&mut child);
        }
        let (stdout, stderr) = (stdout.unwrap_or_default(), stderr.unwrap_or_default());
        let _ = writer.join();
        if !status.success() {
            let why = stderr
                .lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("");
            return Err(match status.code() {
                Some(code) if why.is_empty() => format!("it stopped with code {code}"),
                _ if !why.is_empty() => why.trim().to_string(),
                _ => "it stopped".to_string(),
            });
        }
        let text = match agent.kind {
            AgentKind::Claude => stdout,
            AgentKind::Codex => std::fs::read_to_string(&answer).unwrap_or(stdout),
        };
        let text = text.trim();
        if text.is_empty() {
            return Err("it answered nothing".to_string());
        }
        Ok(cap(text, TRIAGE_CAP))
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// `3 minutes`, `1 second`.
fn span(d: Duration) -> String {
    let s = d.as_secs();
    match (s % 60, s / 60) {
        (0, 1) => "1 minute".to_string(),
        (0, m) if m > 0 => format!("{m} minutes"),
        _ if s == 1 => "1 second".to_string(),
        _ => format!("{s} seconds"),
    }
}

/// At most `max` bytes, cut at a line, else at a char, and said so.
fn cap(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let cut = text[..end]
        .rfind('\n')
        .filter(|i| *i > max / 2)
        .unwrap_or(end);
    format!("{}\n\n_(cut short)_", text[..cut].trim_end())
}

fn prompt(f: &Facts) -> String {
    let mut lines = vec![
        "You are triaging a bug report for AniBeam, an open source anime library app (https://github.com/marcusbandit/AniBeam). AniBeam reads each video file's name to decide whether it is an episode, and which number, or an extra: an opening, ending, preview, special or other. A user found files it read wrong and corrected them by hand with the app's quick fix. Find why the name reader got these wrong, and propose the smallest change to its rules that reads them right without breaking what its tests cover.".to_string(),
        String::new(),
        format!("Series: {}. {}", f.title, record_line(f)),
        format!("Folder: {}", f.folder),
        String::new(),
        "Files, how AniBeam read them, and what the user says they are:".to_string(),
    ];
    for x in &f.files {
        lines.push(format!(
            "- `{}`: read as {}; is {}",
            x.name,
            readings::describe(&x.was),
            readings::describe(&x.now)
        ));
    }
    lines.extend([
        String::new(),
        "Past the name, AniBeam applies one rule: a series whose record is format SPECIAL or OVA and that holds no regular episode reads its numbered specials as its episodes.".to_string(),
        String::new(),
        "The name reader, core/src/library/classifier.rs:".to_string(),
        String::new(),
        "```rust".to_string(),
        CLASSIFIER.to_string(),
        "```".to_string(),
        String::new(),
        "Answer in GitHub markdown, under 200 words, with exactly these three parts and nothing before them:".to_string(),
        "**Cause.** Which rule misread the names, and why.".to_string(),
        "**Fix.** The smallest change to the rules, as a short code sketch.".to_string(),
        "**Test.** One assertion in the style of the tests in classifier.rs.".to_string(),
    ]);
    lines.join("\n")
}

fn format_word(format: &str) -> String {
    match format {
        "TV" => "TV".to_string(),
        // The app's own word for it.
        "MOVIE" => "Film".to_string(),
        "TV_SHORT" => "TV short".to_string(),
        "OVA" | "ONA" => format.to_string(),
        other => {
            let lower = other.replace('_', " ").to_lowercase();
            let mut chars = lower.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        }
    }
}

/// `AniList 15811, Special, 6 episodes`, the record as far as it is known.
fn record_line(f: &Facts) -> String {
    let mut parts = Vec::new();
    match (f.anilist, f.mal, f.tvmaze) {
        (Some(id), _, _) => parts.push(format!("AniList {id}")),
        (None, Some(id), _) => parts.push(format!("MyAnimeList {id}")),
        (None, None, Some(id)) => parts.push(format!("TVmaze {id}")),
        _ => parts.push("not matched".to_string()),
    }
    if let Some(format) = &f.format {
        parts.push(format_word(format));
    }
    if let Some(n) = f.episodes {
        parts.push(if n == 1 {
            "1 episode".to_string()
        } else {
            format!("{n} episodes")
        });
    }
    parts.join(", ")
}

fn record_link(f: &Facts) -> String {
    match (f.anilist, f.mal, f.tvmaze) {
        (Some(id), _, _) => format!("[AniList {id}](https://anilist.co/anime/{id})"),
        (None, Some(id), _) => format!("[MyAnimeList {id}](https://myanimelist.net/anime/{id})"),
        (None, None, Some(id)) => format!("[TVmaze {id}](https://www.tvmaze.com/shows/{id})"),
        _ => "Not matched".to_string(),
    }
}

fn title(f: &Facts) -> String {
    let n = f.files.len();
    format!(
        "Quick fix: {} of {} read wrong",
        if n == 1 {
            "1 file".to_string()
        } else {
            format!("{n} files")
        },
        f.title
    )
}

/// The issue body, `shown` files of the fix listed and the triage as given.
fn body(f: &Facts, shown: usize, triage: Option<(&str, &str)>) -> String {
    let mut out = vec![
        MARKER.to_string(),
        if f.files.len() == 1 {
            format!(
                "AniBeam read this file of **{}** wrong, and it was fixed by hand with the app's quick fix.",
                f.title
            )
        } else {
            format!(
                "AniBeam read these files of **{}** wrong, and they were fixed by hand with the app's quick fix.",
                f.title
            )
        },
        String::new(),
        {
            let mut facts = vec![record_link(f)];
            facts.extend(record_line(f).split(", ").skip(1).map(str::to_string));
            facts.push(format!("folder `{}`", f.folder));
            facts.join(", ")
        },
        String::new(),
        "| File | AniBeam read | Fixed to |".to_string(),
        "| --- | --- | --- |".to_string(),
    ];
    for x in f.files.iter().take(shown) {
        out.push(format!(
            "| `{}` | {} | {} |",
            cell(&x.name),
            readings::describe(&x.was),
            readings::describe(&x.now)
        ));
    }
    if shown < f.files.len() {
        out.push(format!("| and {} more | | |", f.files.len() - shown));
    }
    if let Some((who, text)) = triage {
        out.extend([
            String::new(),
            format!("### Triage by {who}"),
            String::new(),
            text.to_string(),
        ]);
    }
    let record = json!({
        "version": 1,
        "app": f.app,
        "os": std::env::consts::OS,
        "series": {
            "title": f.title,
            "folder": f.folder,
            "anilist": f.anilist,
            "mal": f.mal,
            "tvmaze": f.tvmaze,
            "format": f.format,
            "episodes": f.episodes,
        },
        "files": f.files.iter().take(shown).map(|x| json!({ "name": x.name, "was": x.was, "now": x.now })).collect::<Vec<_>>(),
    });
    out.extend([
        String::new(),
        "<details><summary>Fix record</summary>".to_string(),
        String::new(),
        "```json".to_string(),
        record.to_string(),
        "```".to_string(),
        String::new(),
        "</details>".to_string(),
        String::new(),
        format!("AniBeam {} on {}", f.app, os_word()),
    ]);
    out.join("\n")
}

/// A file name as a table cell: inside backticks, so nothing in it renders,
/// with what would end the cell or the code span replaced.
fn cell(name: &str) -> String {
    name.replace(['\n', '\r'], " ")
        .replace('|', "\\|")
        .replace('`', "'")
}

fn os_word() -> &'static str {
    match std::env::consts::OS {
        "linux" => "Linux",
        "macos" => "macOS",
        "android" => "Android",
        "windows" => "Windows",
        other => other,
    }
}

fn url(title: &str, body: &str) -> String {
    format!(
        "{NEW_ISSUE}?title={}&body={}",
        percent::encode(title),
        percent::encode(body)
    )
}

/// The page to open: everything when it fits, else the triage cut down, then
/// fewer files listed, the marker and the record always kept.
fn draft(f: &Facts, triage: Option<(&str, &str)>) -> (String, String, String) {
    let title = title(f);
    let mut shown = f.files.len();
    let mut cut: Option<String> = triage.map(|(_, t)| t.to_string());
    loop {
        let body = body(f, shown, triage.map(|(who, _)| who).zip(cut.as_deref()));
        let link = url(&title, &body);
        if link.len() <= URL_BUDGET {
            return (title, body, link);
        }
        match cut.as_deref() {
            Some(t) if t.len() > 400 => cut = Some(cap(t, t.len() / 2)),
            Some(_) => cut = None,
            None if shown > 1 => shown = (shown / 2).max(1),
            None => return (title, body, link),
        }
    }
}

// The calls -----------------------------------------------------------------

pub fn triage_agent() -> Result<Reply, CoreError> {
    Ok(Reply::TriageAgent {
        name: find_agent().map(|a| a.name().to_string()),
    })
}

/// Drafts the issue for the fixes on `files`, asking an installed agent for a
/// triage first. Blocks while the agent works, up to three minutes.
pub fn report_fix(
    core: &Core,
    series: u64,
    files: Vec<u64>,
    app: Option<String>,
) -> Result<Reply, CoreError> {
    let images_dir = core.paths.images_dir();
    let mut facts = core.store.read(|c| facts(c, &images_dir, series, &files))?;
    if let Some(app) = app.filter(|a| !a.trim().is_empty()) {
        facts.app = app;
    }
    if facts.files.is_empty() {
        return Err(CoreError::invalid(
            "files",
            "none of these files has a fix to report",
        ));
    }
    let (triage, triaged_by, triage_error) = match find_agent() {
        Some(agent) => match run_agent(&agent, &prompt(&facts), TRIAGE_TIMEOUT) {
            Ok(text) => (Some(text), Some(agent.name().to_string()), None),
            Err(why) => (
                None,
                None,
                Some(format!("{} wrote no triage: {why}", agent.name())),
            ),
        },
        None => (None, None, None),
    };
    let (title, body, url) = draft(&facts, triaged_by.as_deref().zip(triage.as_deref()));
    Ok(Reply::FixReport {
        report: FixReport {
            url,
            title,
            body,
            triaged_by,
            triage_error,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gup(n: usize) -> Facts {
        Facts {
            app: "2.1.0".into(),
            title: "Girls und Panzer OVA Series".into(),
            folder: "Girls und Panzer - Specials".into(),
            anilist: Some(15811),
            mal: None,
            tvmaze: None,
            format: Some("SPECIAL".into()),
            episodes: Some(6),
            files: (1..=n)
                .map(|i| Fixed {
                    name: format!(
                        "[ak-Submarines] Girls und Panzer - OVA {i} [BD 1080p][F37D267C].mkv"
                    ),
                    was: FileReading::Extra {
                        kind: ExtraKind::Sp,
                        index: None,
                    },
                    now: FileReading::Episode {
                        number: i as f64,
                        season: None,
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn the_issue_carries_the_marker_a_table_and_the_record() {
        let (title, body, link) =
            draft(&gup(6), Some(("Claude Code", "**Cause.** The OVA token.")));
        assert_eq!(
            title,
            "Quick fix: 6 files of Girls und Panzer OVA Series read wrong"
        );
        assert!(body.starts_with(MARKER));
        assert!(body.contains("[AniList 15811](https://anilist.co/anime/15811), Special, 6 episodes, folder `Girls und Panzer - Specials`"));
        assert!(body.contains("| `[ak-Submarines] Girls und Panzer - OVA 1 [BD 1080p][F37D267C].mkv` | Special | Episode 1 |"));
        assert!(body.contains("### Triage by Claude Code"));
        assert!(
            body.contains(r#""was":{"Extra":{"index":null,"kind":"Sp"}}"#)
                || body.contains(r#""was":{"Extra":{"kind":"Sp","index":null}}"#)
        );
        assert!(
            link.starts_with(
                "https://github.com/marcusbandit/AniBeam/issues/new?title=Quick%20fix"
            )
        );
        // Labels would 404 the page for anyone without triage rights.
        assert!(!link.contains("labels="));
        assert!(!body.contains('\u{2014}') && !body.contains('\u{2013}'));
    }

    #[test]
    fn a_long_issue_is_cut_to_fit_and_keeps_the_marker_and_the_record() {
        let long = "word ".repeat(2000);
        let (_, body, link) = draft(&gup(200), Some(("Codex", &long)));
        assert!(link.len() <= URL_BUDGET, "{}", link.len());
        assert!(body.starts_with(MARKER));
        assert!(body.contains("```json"));
        assert!(body.contains("more |"));
    }

    #[test]
    fn the_prompt_names_every_file_and_hands_over_the_rules() {
        let p = prompt(&gup(2));
        assert!(p.contains("read as Special; is Episode 2"));
        assert!(p.contains("AniList 15811, Special, 6 episodes"));
        assert!(p.contains("fn find_extra_token"));
    }

    #[cfg(unix)]
    #[test]
    fn a_stand_in_agent_answers_on_stdout_and_a_slow_one_is_cut_off() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, script: &str| {
            let p = dir.path().join(name);
            std::fs::write(&p, script).unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            p
        };
        let echo = write(
            "echo-agent",
            "#!/bin/sh\ncat >/dev/null\nprintf '**Cause.** here\\n'\n",
        );
        let agent = Agent {
            kind: AgentKind::Claude,
            path: echo,
        };
        assert_eq!(
            run_agent(&agent, "prompt", Duration::from_secs(20)).unwrap(),
            "**Cause.** here"
        );
        let slow = write("slow-agent", "#!/bin/sh\nsleep 30\n");
        let agent = Agent {
            kind: AgentKind::Claude,
            path: slow,
        };
        let started = Instant::now();
        assert!(
            run_agent(&agent, "prompt", Duration::from_secs(1))
                .unwrap_err()
                .contains("within 1 second")
        );
        assert!(started.elapsed() < Duration::from_secs(10));
        let failing = write(
            "failing-agent",
            "#!/bin/sh\necho 'not logged in' >&2\nexit 3\n",
        );
        let agent = Agent {
            kind: AgentKind::Claude,
            path: failing,
        };
        assert_eq!(
            run_agent(&agent, "prompt", Duration::from_secs(20)).unwrap_err(),
            "not logged in"
        );
    }

    #[test]
    fn the_record_names_the_folder_on_disk_as_one_plain_component() {
        assert_eq!(folder_of("show", "/lib/Takagi-san/Season 2"), "Season 2");
        assert_eq!(
            folder_of("show", "/lib/Girls und Panzer/Girls und Panzer - Specials"),
            "Girls und Panzer - Specials"
        );
        assert_eq!(
            folder_of("movie", "/lib/Movies/Perfect Blue (1997).mkv"),
            "Movies"
        );
        assert_eq!(folder_of("show", "/"), "series");
        assert_eq!(plain("..".into()), None);
        assert_eq!(plain("".into()), None);
        assert_eq!(plain("a/b".into()), None);
        assert_eq!(plain("a\\b".into()).as_deref(), Some("a\\b"));
        assert_eq!(plain("OVA 1.mkv".into()).as_deref(), Some("OVA 1.mkv"));
    }

    #[test]
    fn cap_cuts_at_a_line() {
        assert_eq!(cap("short", 100), "short");
        let text = format!("{}\n{}", "a".repeat(60), "b".repeat(60));
        assert_eq!(
            cap(&text, 100),
            format!("{}\n\n_(cut short)_", "a".repeat(60))
        );
    }
}
