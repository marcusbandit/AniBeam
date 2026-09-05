//! Subscriptions read from anirss, the download-automation tool this app
//! defers to rather than reimplements. `anirss -Qj` prints the feeds it is
//! watching as one JSON array; anything short of a clean list becomes one
//! of three named failures the shell can act on, rather than one raw
//! error string.
//!
//! Carried from Electron's `subscriptionsHandler.ts` and
//! `SubscriptionsPage.tsx`'s `decodeNyaaQuery`. Electron's ANSI strip
//! matched `[...m` on its own, with no escape byte required in front of
//! it; this one only removes a genuine `ESC [ ... m` sequence.

use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use crate::contract::*;
use crate::core::Core;
use crate::jobs::Finished;
use crate::metadata::apply::owner;

/// How long `anirss -Qj` gets before the job gives up and reports Timeout.
const ANIRSS_TIMEOUT: Duration = Duration::from_secs(15);

/// Starts the Subscriptions job. Unsupported outright on Windows, where
/// there is no anirss to run against; otherwise one at a time, like every
/// other kind `JobKind::one_at_a_time` names.
pub fn start(core: &Core) -> Result<u64, CoreError> {
    if cfg!(windows) {
        return Err(CoreError::Unsupported { what: "anirss on this platform".to_string() });
    }
    let owner = owner(core)?;
    Ok(owner.jobs.clone().start(JobKind::Subscriptions, move |_ctx| async move {
        let result = run().await?;
        let message = message_of(&result);
        Ok(Finished { level: Level::Debug, message, body: EventBody::SubscriptionsListed { result } })
    }))
}

/// Runs `anirss -Qj` to completion, or gives up on it. `kill_on_drop`
/// below is what keeps the child from outliving the job either way this
/// can end early: a timeout drops the future `wait_with_output` holds the
/// child in, and so does a cancelled job, and both send the kill the same
/// way.
async fn run() -> Result<SubscriptionsResult, CoreError> {
    let mut command = Command::new("anirss");
    command.arg("-Qj").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    if let Some(path) = child_path() {
        command.env("PATH", path);
    }
    let child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            return match spawn_outcome(&e) {
                Some(result) => Ok(result),
                // Present but not executable, or some other reason the OS
                // refused to start it: not the "go install it" story
                // `Missing` tells, so the job fails and says why.
                None => Err(CoreError::io_at("anirss", e)),
            };
        }
    };
    let output = match tokio::time::timeout(ANIRSS_TIMEOUT, child.wait_with_output()).await {
        Err(_elapsed) => return Ok(SubscriptionsResult::Timeout),
        // The child already started by this point, so whatever went wrong
        // reading it back is a failure to report, not a reason to claim
        // anirss was never there.
        Ok(Err(e)) => return Err(CoreError::io_at("anirss", e)),
        Ok(Ok(output)) => output,
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return classify_failure(&stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    match parse_output(&stdout) {
        Ok(feeds) => Ok(SubscriptionsResult::Ok { feeds }),
        Err(message) => Err(CoreError::Unsupported { what: format!("anirss returned unreadable JSON: {message}") }),
    }
}

/// Only a `NotFound` spawn error reads as `Missing`: that is the one
/// shape "go install anirss" actually answers. Anything else, a
/// permission error on a present-but-not-executable file included, is a
/// failure the job reports rather than a state the shell offers to fix.
fn spawn_outcome(err: &std::io::Error) -> Option<SubscriptionsResult> {
    (err.kind() == std::io::ErrorKind::NotFound).then_some(SubscriptionsResult::Missing)
}

/// The message for each of the four outcomes, verbatim.
fn message_of(result: &SubscriptionsResult) -> String {
    match result {
        SubscriptionsResult::Ok { feeds } => format!("subscriptions: {} feeds", feeds.len()),
        SubscriptionsResult::Missing => "anirss missing".to_string(),
        SubscriptionsResult::NeedsAuth => "anirss needs a qBittorrent session".to_string(),
        SubscriptionsResult::Timeout => "anirss timed out".to_string(),
    }
}

/// `~/.local/bin`, prepended to `PATH` when it is not already on it, so a
/// launch from a `.desktop` entry with a sparse `PATH` still finds a
/// user-installed anirss. Carried from Electron's `spawnEnv`. `None` when
/// the directory is already reachable, so the child inherits `PATH`
/// unchanged.
fn child_path() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let user_bin = format!("{home}/.local/bin");
    let path = std::env::var("PATH").unwrap_or_default();
    if path.split(':').any(|segment| segment == user_bin) {
        return None;
    }
    Some(if path.is_empty() { user_bin } else { format!("{user_bin}:{path}") })
}

/// A JSON array of `{ name, feed_url, save_path, rule_enabled,
/// torrent_count }` into the feeds the shell draws. An item with no name,
/// or an empty one, is dropped; `rule_enabled` reads active unless it is
/// present and explicitly `false`; the query comes from the feed's own
/// `q` parameter.
pub fn parse_output(stdout: &str) -> Result<Vec<Feed>, String> {
    let raw: Vec<serde_json::Value> = serde_json::from_str(stdout).map_err(|e| e.to_string())?;
    let mut feeds = Vec::with_capacity(raw.len());
    for item in raw {
        let Some(name) = item.get("name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) else {
            continue;
        };
        let feed_url = item.get("feed_url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let save_path = item.get("save_path").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let active = item.get("rule_enabled").and_then(|v| v.as_bool()) != Some(false);
        let torrents = item.get("torrent_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let query = decode_nyaa_query(&feed_url).unwrap_or_default();
        feeds.push(Feed { name: name.to_string(), active, torrents, query, save_path, url: feed_url });
    }
    Ok(feeds)
}

/// A feed's nyaa.si `q` parameter, percent-decoded with `+` read as a
/// space. `None` for a URL with no query, no `q` in it, or a `q` that
/// decodes to nothing.
pub fn decode_nyaa_query(url: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;
    let query = query.split('#').next().unwrap_or(query);
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "q" {
            let decoded = percent_decode(value);
            return if decoded.is_empty() { None } else { Some(decoded) };
        }
    }
    None
}

/// The form-encoded reading: `+` is a space, `%XX` is a byte, and anything
/// that is not a complete escape stands for itself rather than failing. A
/// hand parser rather than a dependency, in the same shape as the query
/// decoder in `trackers::oauth`.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => match (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                (Some(high), Some(low)) => {
                    out.push((high << 4) | low);
                    i += 3;
                }
                _ => {
                    out.push(b'%');
                    i += 1;
                }
            },
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// A non-zero exit's stderr into `NeedsAuth` or a hard job failure.
/// anirss prints a red `error: cancelled` when its password prompt has
/// nothing to read from and no cached qBittorrent session, and a plain
/// message when it cannot reach qBittorrent at all; anything else is a
/// failure the job reports rather than a state the shell offers to fix.
pub fn classify_failure(stderr: &str) -> Result<SubscriptionsResult, CoreError> {
    let stripped = strip_ansi(stderr);
    let trimmed = stripped.trim();
    let lower = trimmed.to_lowercase();
    if lower.contains("cancelled") || lower.contains("can't reach qbittorrent") {
        Ok(SubscriptionsResult::NeedsAuth)
    } else {
        Err(CoreError::Unsupported { what: format!("anirss exited: {trimmed}") })
    }
}

/// Strips a real `ESC [ 0-9 ; ]* m` sequence, escape byte included.
/// Electron's own strip matched from the literal `[` onward and left the
/// escape byte itself sitting in the string; this one only removes a
/// genuine escape sequence.
fn strip_ansi(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && bytes.get(i + 1) == Some(&b'[') {
            let mut j = i + 2;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b';') {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'm' {
                i = j + 1;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_nyaa_query_reads_the_q_parameter() {
        assert_eq!(decode_nyaa_query("https://nyaa.si/?page=rss&q=Frieren+1080p&c=1_2"), Some("Frieren 1080p".to_string()));
    }

    #[test]
    fn decode_nyaa_query_is_none_without_one() {
        assert_eq!(decode_nyaa_query("https://nyaa.si/?page=rss&c=1_2"), None);
    }

    #[test]
    fn parse_output_drops_a_nameless_item_and_defaults_rule_enabled_active() {
        let json = r#"[
            {"name": "Frieren", "feed_url": "https://nyaa.si/?q=Frieren+1080p", "save_path": "/downloads/frieren", "rule_enabled": true, "torrent_count": 3},
            {"feed_url": "https://nyaa.si/?q=Nameless", "save_path": "/downloads/x"},
            {"name": "Dandadan", "feed_url": "https://nyaa.si/?q=Dandadan", "save_path": "/downloads/dandadan", "torrent_count": 1}
        ]"#;
        let feeds = parse_output(json).unwrap();
        assert_eq!(feeds.len(), 2, "the nameless item should be dropped");
        assert_eq!(feeds[0].name, "Frieren");
        assert_eq!(feeds[0].query, "Frieren 1080p");
        assert!(feeds[0].active);
        assert_eq!(feeds[0].torrents, 3);
        assert_eq!(feeds[1].name, "Dandadan");
        assert!(feeds[1].active, "rule_enabled missing should read active");
    }

    #[test]
    fn classify_failure_reads_a_cancelled_prompt_as_needs_auth() {
        let result = classify_failure("\x1b[31merror: cancelled\x1b[0m").unwrap();
        assert_eq!(result, SubscriptionsResult::NeedsAuth);
    }

    #[test]
    fn classify_failure_fails_the_job_for_anything_else() {
        let err = classify_failure("boom, disk on fire").unwrap_err();
        match err {
            CoreError::Unsupported { what } => assert_eq!(what, "anirss exited: boom, disk on fire"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn spawn_outcome_reads_not_found_as_missing() {
        let err = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(spawn_outcome(&err), Some(SubscriptionsResult::Missing));
    }

    #[test]
    fn spawn_outcome_leaves_every_other_error_to_fail_the_job() {
        let err = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert_eq!(spawn_outcome(&err), None);
    }
}
