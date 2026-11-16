//! A show's schedule looked up by the coding agent on this computer (#419),
//! through a stand in that speaks Claude Code's stream protocol on stdin and
//! stdout: it reads the brief, says a line, searches, and answers.

#![cfg(unix)]

mod common;
mod fixtures;

use std::os::unix::fs::PermissionsExt;

use anibeam_core::*;

/// A stand in agent: reads the opening message, keeps it for the test, and
/// answers the way Claude Code's stream does.
fn stand_in(dir: &std::path::Path, answer: bool) -> std::path::PathBuf {
    let path = dir.join("agent.sh");
    let brief = dir.join("brief.json");
    let result = if answer {
        r#"{"type":"result","result":"Episode 8: 2026-11-14\nEpisode 9: TBC","is_error":false}"#
    } else {
        r#"{"type":"result","result":"","is_error":true}"#
    };
    let script = format!(
        r#"#!/bin/sh
read -r line
printf '%s\n' "$line" > '{}'
printf '%s\n' '{{"type":"system","subtype":"init"}}'
printf '%s\n' '{{"type":"assistant","message":{{"content":[{{"type":"text","text":"Looking it up."}},{{"type":"tool_use","name":"WebSearch","input":{{"query":"Show episodes"}}}}]}}}}'
printf '%s\n' '{result}'
cat > /dev/null
"#,
        brief.display()
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn show(core: &Core) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Show", "Show");
    fixtures::insert_episode_file(core, series, "/lib/Show/01.mkv", 1.0, now);
    series
}

/// One session: the agent is named for the shell, gets the show in its
/// brief, says what it does on the way, and its answer comes back for the
/// paste box. Then an agent that answers nothing fails the job with a line
/// saying so, and with no agent the call is refused.
#[test]
fn the_agent_looks_the_schedule_up_and_answers_for_the_paste_box() {
    let (dir, core, c) = common::open_core();
    let series = show(&core);
    let agent = stand_in(dir.path(), true);
    // One test sets the variable, and the binary is this file's own.
    unsafe { std::env::set_var("ANIBEAM_SCHEDULE_AGENT", &agent) };

    assert_eq!(
        core.call(Call::ScheduleAgent).unwrap(),
        Reply::Agent {
            name: Some("Claude Code".into())
        }
    );
    let job = match core.call(Call::AskScheduleAgent { series }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let done = common::wait_job(&c, job);
    match done.body {
        EventBody::AgentAnswered { series: s, text } => {
            assert_eq!(s, series);
            assert_eq!(text, "Episode 8: 2026-11-14\nEpisode 9: TBC");
        }
        other => panic!("{other:?}"),
    }
    let said: Vec<String> = c
        .events()
        .into_iter()
        .filter_map(|e| match e.body {
            EventBody::AgentSaid { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert_eq!(
        said,
        vec!["Looking it up.", "Searching the web for Show episodes"]
    );
    let brief = std::fs::read_to_string(dir.path().join("brief.json")).unwrap();
    let brief: serde_json::Value = serde_json::from_str(brief.trim()).unwrap();
    assert_eq!(brief["type"], "user");
    let text = brief["message"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Show: Show"), "{text}");
    // The answer parses into notes the sheet previews.
    match core
        .call(Call::ParseSchedule {
            text: "Episode 8: 2026-11-14\nEpisode 9: TBC".into(),
        })
        .unwrap()
    {
        Reply::Schedule { notes, .. } => assert_eq!(notes.len(), 2, "{notes:?}"),
        other => panic!("{other:?}"),
    }

    stand_in(dir.path(), false);
    let job = match core.call(Call::AskScheduleAgent { series }).unwrap() {
        Reply::Started { job } => job,
        other => panic!("{other:?}"),
    };
    let failed = common::wait_job(&c, job);
    assert!(
        matches!(failed.body, EventBody::JobFailed { .. }),
        "{failed:?}"
    );

    unsafe { std::env::set_var("ANIBEAM_SCHEDULE_AGENT", "none") };
    assert_eq!(
        core.call(Call::ScheduleAgent).unwrap(),
        Reply::Agent { name: None }
    );
    assert!(matches!(
        core.call(Call::AskScheduleAgent { series }),
        Err(CoreError::Unsupported { .. })
    ));
}
