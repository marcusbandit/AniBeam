//! Starting the service behind an on-demand endpoint: Jackett for 1337x.
//!
//! anirss boots such a service on its own whenever a search reaches it, and
//! the search guard in `mod.rs` keeps that from happening behind the user's
//! back. This is the other door: the shell asks, after the user confirmed,
//! and the core runs the start command itself, waits for the endpoint's
//! ready URL to answer and reports what happened.
//!
//! The command is the `start` anirss's own config gives that endpoint (on
//! this kind of setup a `podman pod start`), else the `jackett_command`
//! setting, else a bare `jackett` on PATH. It runs through `sh -c` because
//! anirss's `start` is a shell string. It is detached: its own process
//! group, output into a log file, nothing killed when the core stops, since
//! a pod outlives the command that started it and a bare `jackett` is
//! meant to stay up for the next search.

use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::process::Command;

use crate::net::{Http, HttpRequest, Method};

/// How long a command with no ready URL to wait for is given to fail
/// before it counts as started.
const NO_READY_GRACE: Duration = Duration::from_secs(2);

/// The ready URL answers within a second or it is not up yet.
const PROBE_TIMEOUT: Duration = Duration::from_secs(1);

/// Why the service did not come up, in the words of the popover.
#[derive(Debug, PartialEq)]
pub enum StartFailure {
    /// The command was not found: Jackett is not installed.
    Missing(String),
    /// The command ran and failed, or could not run.
    Failed(String),
    /// The command was fine but the ready URL never answered, in seconds.
    Timeout(u64),
}

/// The `start` command anirss's config gives the named endpoint, or the
/// first endpoint that has one when `name` is `None`. Read apart from
/// `settings::anirss_ondemand`, which keeps only what the search needs.
pub fn anirss_start(path: &Path, name: Option<&str>) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let doc: toml::Table = toml::from_str(&text).ok()?;
    doc.get("endpoint")?
        .as_array()?
        .iter()
        .filter_map(|e| e.as_table())
        .filter(|t| name.is_none_or(|n| t.get("name").and_then(|v| v.as_str()) == Some(n)))
        .find_map(|t| {
            t.get("start")
                .and_then(|s| s.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        })
}

/// The command to run: anirss's own `start` for this endpoint, else the
/// setting, else `jackett`. The endpoint's own wins so one saved command can
/// never start the wrong service for a second on-demand endpoint.
pub fn resolve_command(setting: &str, anirss_config: Option<&Path>, endpoint: &str) -> String {
    if let Some(own) = anirss_config.and_then(|p| anirss_start(p, Some(endpoint))) {
        return own;
    }
    let setting = setting.trim();
    if !setting.is_empty() {
        return setting.to_string();
    }
    "jackett".to_string()
}

/// What the person reads in the popover for a failure.
pub fn explain(name: &str, failure: &StartFailure) -> (&'static str, String) {
    match failure {
        StartFailure::Missing(program) => (
            "missing",
            format!(
                "Jackett is not installed: {program} was not found. Install it, or change the command in Settings."
            ),
        ),
        StartFailure::Failed(why) => ("failed", format!("{name} could not start: {why}")),
        StartFailure::Timeout(secs) => (
            "timeout",
            format!("{name} started but did not answer within {secs} seconds."),
        ),
    }
}

async fn answers(http: &dyn Http, url: &str) -> bool {
    http.send(HttpRequest {
        method: Method::Get,
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        timeout: Some(PROBE_TIMEOUT),
    })
    .await
    .map(|r| r.is_success())
    .unwrap_or(false)
}

/// The last line of the log that says anything.
fn last_line(log: &Path) -> Option<String> {
    let text = std::fs::read_to_string(log).ok()?;
    text.lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.chars().take(200).collect())
}

/// The command's program, for the "not found" line.
fn program(command: &str) -> String {
    command
        .split_whitespace()
        .find(|w| !w.contains('=') && *w != "systemd-run" && !w.starts_with('-'))
        .or_else(|| command.split_whitespace().next())
        .unwrap_or(command)
        .to_string()
}

/// Runs `command` and waits until `ready` answers. A service that already
/// answers is left alone and the command never runs. Output goes to `log`.
/// With an empty `ready` there is nothing to wait for: the command counts
/// as started once it exits cleanly, or has stayed up for a moment.
pub async fn start(
    http: &dyn Http,
    command: &str,
    ready: &str,
    timeout: Duration,
    poll: Duration,
    log: &Path,
) -> Result<(), StartFailure> {
    if !ready.is_empty() && answers(http, ready).await {
        return Ok(());
    }
    if let Some(dir) = log.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let out = std::fs::File::create(log).map_err(|e| StartFailure::Failed(e.to_string()))?;
    let err = out
        .try_clone()
        .map_err(|e| StartFailure::Failed(e.to_string()))?;
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .kill_on_drop(false);
    #[cfg(unix)]
    cmd.process_group(0);
    if let Some(path) = super::anirss::child_path() {
        cmd.env("PATH", path);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| StartFailure::Failed(e.to_string()))?;
    let began = Instant::now();
    let mut exited_clean = false;
    loop {
        if !exited_clean && let Some(status) = child.try_wait().ok().flatten() {
            if status.code() == Some(127) {
                return Err(StartFailure::Missing(program(command)));
            }
            if !status.success() {
                let why =
                    last_line(log).unwrap_or_else(|| format!("the command exited with {status}"));
                return Err(StartFailure::Failed(why));
            }
            exited_clean = true;
        }
        if ready.is_empty() {
            if exited_clean || began.elapsed() >= NO_READY_GRACE {
                return Ok(());
            }
        } else if answers(http, ready).await {
            return Ok(());
        }
        if began.elapsed() >= timeout {
            return Err(StartFailure::Timeout(timeout.as_secs()));
        }
        tokio::time::sleep(poll).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::FakeHttp;

    const READY: &str = "http://127.0.0.1:9117/caps";

    fn log(dir: &tempfile::TempDir) -> std::path::PathBuf {
        dir.path().join("logs").join("start.log")
    }

    async fn run(
        http: &FakeHttp,
        command: &str,
        ready: &str,
        dir: &tempfile::TempDir,
    ) -> Result<(), StartFailure> {
        start(
            http,
            command,
            ready,
            Duration::from_secs(3),
            Duration::from_millis(20),
            &log(dir),
        )
        .await
    }

    #[test]
    fn the_command_is_the_setting_then_anirss_then_jackett() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("config.toml");
        std::fs::write(
            &cfg,
            "[[endpoint]]\nname = \"nyaa\"\nkind = \"nyaa\"\n\n[[endpoint]]\nname = \"1337x\"\nkind = \"rss\"\nstart = \" podman pod start x \"\nready = \"http://127.0.0.1:9117/caps\"\n",
        )
        .unwrap();
        assert_eq!(
            anirss_start(&cfg, None),
            Some("podman pod start x".to_string())
        );
        assert_eq!(anirss_start(&cfg, Some("nyaa")), None);
        assert_eq!(
            resolve_command(" my-start ", Some(&cfg), "1337x"),
            "podman pod start x",
            "the endpoint's own start wins over the setting"
        );
        assert_eq!(
            resolve_command(" my-start ", Some(&cfg), "nyaa"),
            "my-start"
        );
        assert_eq!(
            resolve_command("", Some(&cfg), "1337x"),
            "podman pod start x"
        );
        assert_eq!(resolve_command("", Some(&cfg), "nyaa"), "jackett");
        assert_eq!(resolve_command("", None, "1337x"), "jackett");
    }

    #[test]
    fn the_not_found_line_names_the_program_not_the_wrapper() {
        assert_eq!(program("jackett"), "jackett");
        assert_eq!(
            program("systemd-run --user --scope --quiet podman pod start x"),
            "podman"
        );
    }

    #[tokio::test]
    async fn a_service_that_already_answers_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let http = FakeHttp::new();
        http.push(200, "ok");
        // A command that would fail if it ran
        run(&http, "exit 9", READY, &dir).await.unwrap();
        assert!(!log(&dir).exists());
    }

    #[tokio::test]
    async fn it_runs_the_command_then_waits_for_the_ready_url() {
        let dir = tempfile::tempdir().unwrap();
        let http = FakeHttp::new();
        // The first look finds it down, the next two are still booting
        http.fail_next("refused");
        http.fail_next("refused");
        http.fail_next("refused");
        http.push(200, "ok");
        run(&http, "echo starting", READY, &dir).await.unwrap();
        assert_eq!(http.requests().len(), 4);
        assert!(
            std::fs::read_to_string(log(&dir))
                .unwrap()
                .contains("starting")
        );
    }

    #[tokio::test]
    async fn a_command_that_is_not_found_means_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let http = FakeHttp::new();
        http.fail_next("refused");
        let r = run(&http, "anibeam-no-such-jackett --port 9117", READY, &dir).await;
        assert_eq!(
            r,
            Err(StartFailure::Missing("anibeam-no-such-jackett".to_string()))
        );
        let (code, message) = explain("1337x", &r.unwrap_err());
        assert_eq!(code, "missing");
        assert!(message.starts_with("Jackett is not installed"));
    }

    #[tokio::test]
    async fn a_command_that_fails_reports_the_last_line_it_wrote() {
        let dir = tempfile::tempdir().unwrap();
        let http = FakeHttp::new();
        http.fail_next("refused");
        let r = run(
            &http,
            "echo 'no pod with name anirss-1337x found' >&2; exit 125",
            READY,
            &dir,
        )
        .await;
        assert_eq!(
            r,
            Err(StartFailure::Failed(
                "no pod with name anirss-1337x found".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn a_service_that_never_answers_times_out() {
        let dir = tempfile::tempdir().unwrap();
        let http = FakeHttp::new();
        let r = start(
            http.as_ref(),
            "true",
            READY,
            Duration::from_millis(200),
            Duration::from_millis(20),
            &log(&dir),
        )
        .await;
        assert_eq!(r, Err(StartFailure::Timeout(0)));
        assert_eq!(
            explain("1337x", &StartFailure::Timeout(60)).1,
            "1337x started but did not answer within 60 seconds."
        );
    }

    #[tokio::test]
    async fn with_no_ready_url_a_clean_exit_is_enough() {
        let dir = tempfile::tempdir().unwrap();
        let http = FakeHttp::new();
        run(&http, "true", "", &dir).await.unwrap();
        assert!(http.requests().is_empty());
    }
}
