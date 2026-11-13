//! A show's schedule, looked up by the coding agent on this computer (#419).
//!
//! AniBeam runs a real session of the agent the person already has, the way
//! T3 Code does: Claude Code over its own stream protocol (`--input-format
//! stream-json --output-format stream-json`), a live session with a message
//! stream rather than a one-shot `claude -p`. The session runs in an empty
//! folder of its own, with only web search and web fetch, safe mode, and
//! nothing saved. The first message carries what the library knows about the
//! show and asks for its episode dates in the lines the schedule parser reads
//! (#418). What the agent says on the way streams to the shell as it comes;
//! its last answer goes into the schedule sheet's paste box, so the person
//! sees the preview on the tiles and nothing is saved until they press Save.
//!
//! The agent is found on the machine, never shipped. Claude Code is the one
//! session this speaks today; ACP (and Codex's app-server through it) is the
//! next step. Without one, `ScheduleAgent` answers no name and the shell
//! offers nothing.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::contract::*;
use crate::core::Core;
use crate::jobs::{Finished, JobCtx};
use crate::library::report::{self, AgentKind};

/// How long one session may take, web searches included.
const SESSION_TIMEOUT: Duration = Duration::from_secs(6 * 60);

/// The longest answer kept; a schedule is a few dozen lines. A longer one is
/// cut at the last whole line under it, so no line is cut into another date.
const ANSWER_CAP: usize = 8000;

/// The agent a schedule session runs, by path, and the name the shell shows.
/// `ANIBEAM_SCHEDULE_AGENT` names a stand in that speaks the same stream on
/// stdin and stdout (drives and tests), or `none`.
pub fn find() -> Option<(PathBuf, &'static str)> {
    if let Some(v) = std::env::var_os("ANIBEAM_SCHEDULE_AGENT") {
        if v == "none" {
            return None;
        }
        let path = PathBuf::from(v);
        return path.is_file().then_some((path, "Claude Code"));
    }
    // The installed agent alone: the quick fix's stand in speaks another
    // protocol, so its override is not this one's.
    report::find_installed()
        .filter(|a| a.kind == AgentKind::Claude)
        .map(|a| {
            let name = a.name();
            (a.path, name)
        })
}

/// The agent's name for the shell, or none when there is no session to
/// offer.
pub fn available() -> Option<String> {
    find().map(|(_, name)| name.to_string())
}

/// What the first message tells the agent about the show.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Brief {
    pub title: String,
    pub other_titles: Vec<String>,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub total: Option<u32>,
    pub site: Option<String>,
    /// Dates the library already has, as "Episode 3: 2026-10-30".
    pub known: Vec<String>,
}

/// The first message: the show, what is known, and the shape of the answer.
pub fn prompt(b: &Brief, today: &str) -> String {
    let mut out = String::new();
    out.push_str("You are helping AniBeam, a local anime library app, fill in the release schedule of one show.\n\n");
    out.push_str(&format!("Show: {}\n", b.title));
    if !b.other_titles.is_empty() {
        out.push_str(&format!("Also known as: {}\n", b.other_titles.join("; ")));
    }
    if let Some(y) = b.year {
        out.push_str(&format!("Year: {y}\n"));
    }
    if let Some(s) = b.season {
        out.push_str(&format!("Season: {s}\n"));
    }
    if let Some(t) = b.total {
        out.push_str(&format!("Episodes announced: {t}\n"));
    }
    if let Some(site) = &b.site {
        out.push_str(&format!("Record: {site}\n"));
    }
    out.push_str(&format!("Today is {today}.\n"));
    if !b.known.is_empty() {
        out.push_str("\nDates the app already has:\n");
        for line in &b.known {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(
        "\nSearch the web for this show's episode release dates, in its original airing. \
Prefer the official site, the broadcaster or a well kept episode list. Then answer with \
the schedule alone, one line per episode or run of episodes, and nothing else:\n\
Episode 8: 2026-11-14\n\
Episodes 9-12: weekly from 2026-11-21\n\
Episode 13: March 2027\n\
Episode 14: 2027\n\
Episode 15: TBC\n\
Use a full date where one is announced, a month or a year where only that is known, \
and TBC for an announced episode with no date. Leave out episodes you found nothing for.",
    );
    out
}

/// What one line of the agent's stream adds: something said on the way, or
/// the turn's last answer.
#[derive(Clone, Debug, PartialEq)]
pub enum Said {
    /// Text the agent wrote, or a tool it used, as a line for the panel.
    Line(String),
    /// The turn's result: its text, or why there is none.
    Done(Result<String, String>),
}

/// One stream-json line, read. A line that is not one of these says
/// nothing.
pub fn read_line(line: &str) -> Vec<Said> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return Vec::new();
    };
    match v["type"].as_str() {
        Some("assistant") => v["message"]["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|block| match block["type"].as_str() {
                Some("text") => block["text"]
                    .as_str()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(|t| Said::Line(t.to_string())),
                Some("tool_use") => Some(Said::Line(tool_line(
                    block["name"].as_str().unwrap_or(""),
                    &block["input"],
                ))),
                _ => None,
            })
            .collect(),
        Some("result") => {
            let text = v["result"].as_str().unwrap_or("").trim().to_string();
            if v["is_error"].as_bool().unwrap_or(false) || text.is_empty() {
                let why = if text.is_empty() {
                    crate::i18n::tr("core-agent-no-answer", &[])
                } else {
                    text
                };
                vec![Said::Done(Err(why))]
            } else {
                vec![Said::Done(Ok(text))]
            }
        }
        _ => Vec::new(),
    }
}

/// A tool call as a person reads it.
fn tool_line(name: &str, input: &serde_json::Value) -> String {
    match name {
        "WebSearch" => match input["query"].as_str() {
            Some(query) => crate::i18n::tr("core-agent-searching", &[("query", query.into())]),
            None => crate::i18n::tr("core-agent-searching-schedule", &[]),
        },
        "WebFetch" => match input["url"].as_str() {
            Some(url) => crate::i18n::tr("core-agent-reading", &[("url", url.into())]),
            None => crate::i18n::tr("core-agent-reading-page", &[]),
        },
        other => crate::i18n::tr("core-agent-using", &[("tool", other.into())]),
    }
}

/// The user message that opens the session, as one stream-json line.
fn user_message(text: &str) -> String {
    serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": text }] }
    })
    .to_string()
}

/// Starts a session for the series. Refused at once when no agent is here.
pub fn start(core: &Core, series: u64) -> Result<u64, CoreError> {
    let Some((path, name)) = find() else {
        return Err(CoreError::Unsupported {
            what: crate::i18n::tr("core-agent-none", &[]),
        });
    };
    let brief = brief(core, series)?;
    let owner = core.owner()?;
    Ok(owner
        .jobs
        .clone()
        .start(JobKind::ScheduleAgent, move |ctx| async move {
            run(ctx, series, path, name, brief).await
        }))
}

/// The brief, off the series page's own read.
fn brief(core: &Core, series: u64) -> Result<Brief, CoreError> {
    let detail = match crate::library::reads::get_series(core, series)? {
        Reply::SeriesDetail { detail } => detail,
        _ => {
            return Err(CoreError::NotFound {
                what: Entity::Series,
                id: series,
            });
        }
    };
    let card = &detail.card;
    let mut other_titles: Vec<String> = Vec::new();
    for t in [
        card.titles.romaji.clone(),
        card.titles.english.clone(),
        card.titles.native.clone(),
    ]
    .into_iter()
    .flatten()
    {
        if t != card.title && !other_titles.contains(&t) {
            other_titles.push(t);
        }
    }
    // The season the sheet edits, the card's own: only its dates go in
    let season = card.count_season;
    let known = detail
        .upcoming
        .iter()
        .filter(|u| !u.predicted && (season.is_none() || u.season == season))
        .map(|u| {
            let day = ::time::OffsetDateTime::from_unix_timestamp(crate::time::to_secs(u.at))
                .map(|t| t.date().to_string())
                .unwrap_or_default();
            format!("Episode {}: {day}", u.number)
        })
        .collect();
    Ok(Brief {
        title: card.title.clone(),
        other_titles,
        year: detail.year,
        season,
        total: detail.progress.total,
        site: detail.site_url.clone(),
        known,
    })
}

/// The session: the agent started, the brief sent, every line said on the
/// way passed to the shell, and the last answer kept. Cancelling the job,
/// or running out of time, ends the agent and everything it started.
async fn run(
    ctx: Arc<JobCtx>,
    series: u64,
    path: PathBuf,
    name: &'static str,
    brief: Brief,
) -> Result<Finished, CoreError> {
    let dir =
        report::fresh_dir("anibeam-schedule").map_err(|e| CoreError::internal(e.to_string()))?;
    // Whatever ends this session, an answer, a failure, a stop, or the job
    // runner dropping it on a cancel, the agent's group and its folder go.
    let mut guard = Cleanup { pid: None, dir };
    let mut cmd = tokio::process::Command::new(&path);
    cmd.args([
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--safe-mode",
        "--tools",
        "WebSearch,WebFetch",
        "--allowedTools",
        "WebSearch,WebFetch",
        "--no-session-persistence",
    ])
    .current_dir(&guard.dir)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .kill_on_drop(true);
    if let Some(p) = report::agent_path_env(&path) {
        cmd.env("PATH", p);
    }
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child = cmd
        .spawn()
        .map_err(|e| CoreError::internal(format!("{name} did not start: {e}")))?;
    guard.pid = child.id();
    let today = ::time::OffsetDateTime::now_utc().date().to_string();
    let mut stdin = child.stdin.take();
    if let Some(input) = stdin.as_mut() {
        let line = user_message(&prompt(&brief, &today)) + "\n";
        input
            .write_all(line.as_bytes())
            .await
            .map_err(|e| CoreError::internal(format!("{name} did not take the brief: {e}")))?;
        let _ = input.flush().await;
    }
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CoreError::internal(format!("{name} gave no output")))?;
    let mut lines = BufReader::new(stdout).lines();
    let session = async {
        while let Some(line) = lines.next_line().await.ok().flatten() {
            for said in read_line(&line) {
                match said {
                    Said::Line(text) => ctx.emit(
                        Level::Debug,
                        format!("{name}: {}", first_line(&text)),
                        EventBody::AgentSaid {
                            series,
                            text: text.clone(),
                        },
                    ),
                    Said::Done(Ok(text)) => return Outcome::Answer(text),
                    Said::Done(Err(why)) => return Outcome::Failed(why),
                }
            }
        }
        Outcome::Failed(crate::i18n::tr(
            "core-agent-ended",
            &[("name", name.into())],
        ))
    };
    let outcome = tokio::select! {
        done = session => done,
        _ = tokio::time::sleep(SESSION_TIMEOUT) => Outcome::Failed(crate::i18n::tr("core-agent-too-long", &[("name", name.into())])),
        _ = ctx.cancelled() => Outcome::Stopped,
    };
    // One turn is the whole session: closing stdin ends it, and the guard
    // takes the group and the folder with it.
    drop(stdin);
    drop(guard);
    let _ = child.kill().await;
    match outcome {
        Outcome::Answer(answer) => Ok(Finished {
            plain: None,
            level: Level::Info,
            message: format!("{name} answered a schedule for {}", brief.title),
            body: EventBody::AgentAnswered {
                series,
                text: whole_lines(answer, ANSWER_CAP),
            },
        }),
        Outcome::Stopped => Err(CoreError::internal("cancelled")),
        Outcome::Failed(why) => Err(CoreError::Service {
            service: name.to_string(),
            code: "no_answer".to_string(),
            message: why,
        }),
    }
}

/// How a session ended.
enum Outcome {
    Answer(String),
    Failed(String),
    Stopped,
}

/// The agent's process group and folder, ended and removed when dropped.
struct Cleanup {
    pid: Option<u32>,
    dir: PathBuf,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(pid) = self.pid.take() {
            report::kill_group(pid);
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// `text` kept to `cap` bytes, cut at the last whole line under it.
fn whole_lines(text: String, cap: usize) -> String {
    if text.len() <= cap {
        return text;
    }
    let mut cut = (0..=cap)
        .rev()
        .find(|i| text.is_char_boundary(*i))
        .unwrap_or(0);
    if let Some(nl) = text[..cut].rfind('\n') {
        cut = nl;
    }
    text[..cut].to_string()
}

/// The first line of something the agent said, for the developer log.
fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_answer_is_cut_at_a_whole_line() {
        let text = "Episode 1: 2026-01-01\nEpisode 40: 2027-03-14".to_string();
        assert_eq!(whole_lines(text.clone(), 1000), text);
        assert_eq!(whole_lines(text, 30), "Episode 1: 2026-01-01");
    }

    #[test]
    fn the_stream_says_text_tools_and_the_answer() {
        let said = read_line(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Looking it up."},{"type":"tool_use","name":"WebSearch","input":{"query":"Show episode dates"}}]}}"#,
        );
        assert_eq!(
            said,
            vec![
                Said::Line("Looking it up.".into()),
                Said::Line("Searching the web for Show episode dates".into())
            ]
        );
        assert_eq!(
            read_line(r#"{"type":"result","result":"Episode 8: 2026-11-14","is_error":false}"#),
            vec![Said::Done(Ok("Episode 8: 2026-11-14".into()))]
        );
        assert!(matches!(
            read_line(r#"{"type":"result","result":"","is_error":true}"#)[0],
            Said::Done(Err(_))
        ));
        assert!(read_line(r#"{"type":"system","subtype":"init"}"#).is_empty());
        assert!(read_line("not json").is_empty());
    }

    /// The brief names the show and what is known, and asks for lines the
    /// schedule parser reads; the example lines themselves parse.
    #[test]
    fn the_prompt_asks_for_lines_the_parser_reads() {
        let b = Brief {
            title: "Show".into(),
            other_titles: vec!["Shou".into()],
            year: Some(2026),
            season: Some(2),
            total: Some(12),
            site: Some("https://example.invalid/show".into()),
            known: vec!["Episode 3: 2026-10-30".into()],
        };
        let p = prompt(&b, "2026-11-06");
        assert!(p.contains("Show: Show") && p.contains("Also known as: Shou"));
        assert!(p.contains("Season: 2") && p.contains("Episode 3: 2026-10-30"));
        assert!(!p.contains(['\u{2013}', '\u{2014}']));
        let now = ::time::macros::datetime!(2026-11-06 12:00 UTC).unix_timestamp();
        let (_, notes) = crate::library::schedule_text::parse(
            "Episode 8: 2026-11-14\nEpisodes 9-12: weekly from 2026-11-21\nEpisode 13: March 2027\nEpisode 14: 2027\nEpisode 15: TBC",
            now,
        );
        let firsts: Vec<u32> = notes.iter().map(|n| n.first).collect();
        assert_eq!(firsts.first(), Some(&8));
        assert!(firsts.contains(&15), "{notes:?}");
    }
}
