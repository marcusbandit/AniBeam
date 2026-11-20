//! Control over adb (#1207): an agent drives the running app by name instead of by tapping
//! coordinates off a screenshot, and reads back what the screen shows. Slint hands Android
//! no accessibility tree, so `uiautomator` sees one opaque surface.
//!
//! `adb shell am broadcast -a com.marcusrosado.anibeam.CONTROL -p com.marcusrosado.anibeam
//! --es cmd '<command>'` reaches `Control.java`, which the sideload build registers at run
//! time with DUMP as the permission a sender must hold: only the shell user (adb) and the
//! system have it. Java hands the line to `serve` on a thread of its own, and the answer
//! goes back as the broadcast's result data, which `am broadcast` prints.
//! `scripts/ctl.sh` wraps it.
//!
//! `command.rs` reads the line, `ui.rs` acts on the window on the Slint event loop, and this
//! file runs between the two on the receiver's thread: the commands that wait or measure
//! (`wait`, `state`, `perf`) make their round trips to the UI loop from here, so the UI loop
//! never waits on itself. The Play build compiles none of it (build.rs, `cfg(control)`).

// Only Android has a receiver to call `serve`; the host keeps it all for the tests.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

mod command;
mod ui;

#[cfg(target_os = "android")]
mod android;

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use command::Command;
pub use ui::{rendered, rendering};

/// How long a command may keep the UI loop before the answer says it did not come back.
const ANSWER: Duration = Duration::from_secs(10);
/// How long a whole line may take. `am broadcast` waits on an ordered background broadcast,
/// and Android calls a receiver that keeps it past 60 s not responding.
const LINE: Duration = Duration::from_secs(45);
/// How long a line may take on Android 13 and older, where the background queue hands out
/// ordered broadcasts one at a time for the whole system: every other app's broadcast waits
/// behind a long line. ctl.sh splits a long `wait` into short ones.
const SHORT_LINE: Duration = Duration::from_secs(8);

/// The time a line gets on this phone.
fn line_time() -> Duration {
    #[cfg(target_os = "android")]
    if android::sdk() <= 33 {
        return SHORT_LINE;
    }
    LINE
}
/// The most an answer carries, in UTF-8 bytes. Java holds it as UTF-16, about twice this
/// for the mostly ASCII answers, and it crosses binder in one-way calls (to the system, then
/// to `am`) whose shared buffer is about 512 KB: past it a call fails, the broadcast never
/// finishes and it ends in an ANR.
const MOST: usize = 100_000;

/// The time a line has left.
#[derive(Clone, Copy)]
struct Budget(Instant);

impl Budget {
    fn left(self) -> Duration {
        self.0.saturating_duration_since(Instant::now())
    }

    /// `d`, or less when the line has less left.
    fn cap(self, d: Duration) -> Duration {
        d.min(self.left())
    }
}

/// Starts the channel: the window the commands act on, and on Android the receiver.
pub fn install(window: &crate::AppWindow) {
    ui::install(window);
    #[cfg(target_os = "android")]
    android::start();
}

/// Runs a command line and answers whether every command ran, and what each printed, one
/// after another. A command that fails stops the line, and its `error: ...` is the answer's
/// last line. The line has `LINE` in all: a command that would run past it is not started.
pub fn serve(line: &str, late: Duration) -> (bool, String) {
    let commands = match command::parse_line(line) {
        Ok(c) => c,
        Err(e) => return (false, format!("error: {e}")),
    };
    // Held back on its way (Android defers broadcasts to a cached app), the line has less
    // time left, so its answer still comes before ctl.sh stops waiting; one held too long
    // never runs (Control.java), so ctl.sh giving up never leaves a command to run later.
    let line = line_time();
    let budget = Budget(Instant::now() + line.saturating_sub(late));
    let total = commands.len();
    let mut out = Vec::new();
    let mut ok = true;
    for (i, c) in commands.into_iter().enumerate() {
        let said = match worst(&c) {
            Some(need) if need > budget.left() => format!(
                "error: the line would run past its {} s; {} of {total} not run, send them on a line of their own",
                line.as_secs(),
                total - i
            ),
            _ if budget.left().is_zero() => format!(
                "error: the line ran out of its {} s; {} of {total} not run",
                line.as_secs(),
                total - i
            ),
            _ => run(c, budget, i + 1 == total),
        };
        // A command that failed part way may say so on a later line, `perf`'s frames for one.
        let failed = said.lines().any(|l| l.starts_with("error:"));
        out.push(said);
        if failed {
            ok = false;
            let left = total - i - 1;
            if left > 0 && !out.last().is_some_and(|l| l.contains("not run")) {
                out.push(format!("error: {left} of {total} not run after it"));
            }
            break;
        }
    }
    (ok, cut(out.join("\n")))
}

/// A wait's answer once the line's time cut it short: when it is the line's last command,
/// it ends with the wait to send next, which ctl.sh sends as it is. Anywhere else the line
/// stops there and says what did not run, since a resend would drop the rest of the line.
fn again(said: String, label: &str, secs: f32, take: f32, last: bool) -> String {
    if last && take + 0.25 < secs && said.contains("did not show within") {
        format!("{said}\nagain: wait {} {:.1}s", quoted(label), secs - take)
    } else {
        said
    }
}

/// How long a command can take at most, for the ones that take long on purpose.
fn worst(c: &Command) -> Option<Duration> {
    match c {
        // These fit themselves to what the line has left (`fit`), so need only a moment.
        Command::Wait { .. } | Command::Perf(_) => Some(Duration::from_secs(1)),
        Command::Swipe { ms, .. } => Some(Duration::from_millis(*ms)),
        Command::Hold(_) => Some(ui::HOLD + Duration::from_millis(200)),
        _ => None,
    }
}

/// The answer, cut at a line under `MOST` bytes when it is longer.
fn cut(answer: String) -> String {
    if answer.len() <= MOST {
        return answer;
    }
    let mut end = MOST;
    while !answer.is_char_boundary(end) {
        end -= 1;
    }
    let end = answer[..end].rfind('\n').unwrap_or(end);
    // An error at the end stays at the end, all its lines (a list of matches follows it),
    // so the result code holds; it is not counted among the lines left out.
    let tail = answer
        .rfind("\nerror:")
        .filter(|&at| at >= end && answer.len() - at < 4000)
        .map(|at| &answer[at + 1..]);
    let left = answer[end..]
        .lines()
        .filter(|l| !l.is_empty())
        .count()
        .saturating_sub(tail.map_or(0, |t| t.lines().count()));
    let error = tail.map(|t| format!("\n{t}")).unwrap_or_default();
    format!(
        "{}\n... {left} more lines; narrow it with a filter{error}",
        &answer[..end]
    )
}

/// Where a command sent to the UI loop is: waiting its turn, started, or given up on.
const QUEUED: u8 = 0;
const STARTED: u8 = 1;
const GAVE_UP: u8 = 2;

/// Runs `f` on the UI loop and waits up to `timeout` for what it sends back. A command
/// still waiting its turn when the wait gives up is skipped when the turn comes, and the
/// answer says it was dropped; one that had started already says it ran, so a caller only
/// retries what never happened.
fn on_ui<T: Send + 'static>(
    timeout: Duration,
    f: impl FnOnce(mpsc::Sender<T>) + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(AtomicU8::new(QUEUED));
    let turn = state.clone();
    slint::invoke_from_event_loop(move || {
        if turn
            .compare_exchange(QUEUED, STARTED, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            crate::local::log("control: skipped a command its caller had given up on");
            return;
        }
        f(tx)
    })
    .map_err(|e| format!("error: the UI loop is gone: {e}"))?;
    rx.recv_timeout(timeout).map_err(|e| {
        // The UI side let go of its answer without sending it: a hold or swipe whose timer
        // went away, or the loop shutting down.
        if e == mpsc::RecvTimeoutError::Disconnected {
            return "error: the command ended without an answer".to_string();
        }
        let secs = timeout.as_secs_f32();
        match state.compare_exchange(QUEUED, GAVE_UP, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => format!(
                "error: the UI thread did not answer within {secs:.0} s; the command was dropped and never ran"
            ),
            Err(_) => format!(
                "error: the command ran but took over {secs:.0} s to answer; check where before sending it again"
            ),
        }
    })
}

/// How long a wait or a sample of `secs` may run on this line: what is left, less a margin
/// for its last round trip and the answer.
fn fit(secs: f32, budget: Budget) -> f32 {
    let room = budget.left().saturating_sub(Duration::from_millis(1500));
    secs.min(room.as_secs_f32()).max(0.5)
}

/// A label written for the command line's reader: in double quotes, `"` and `\` escaped.
fn quoted(label: &str) -> String {
    format!("\"{}\"", label.replace('\\', "\\\\").replace('"', "\\\""))
}

fn run(command: Command, budget: Budget, last: bool) -> String {
    match command {
        Command::Wait { label, secs } => {
            let take = fit(secs, budget);
            let said = wait(&label, take, budget);
            again(said, &label, secs, take, last)
        }
        Command::State => state(budget),
        Command::Perf(secs) => {
            let take = fit(secs, budget);
            let said = perf(take, budget);
            if take + 0.25 < secs {
                format!("{said}\n(cut to {take:.1} s, what a line has on this Android)")
            } else {
                said
            }
        }
        other => {
            let timeout = match &other {
                Command::Swipe { ms, .. } => ANSWER + Duration::from_millis(*ms),
                _ => ANSWER,
            };
            on_ui(budget.cap(timeout), move |tx| ui::exec(other, tx)).unwrap_or_else(|e| e)
        }
    }
}

/// Looks for the label twice a second, until a control with it shows. Each look walks the
/// window on the UI thread, so it looks no more often than that.
fn wait(label: &str, secs: f32, budget: Budget) -> String {
    let start = Instant::now();
    let until = Duration::from_secs_f32(secs);
    loop {
        let want = label.to_string();
        match on_ui(budget.cap(ANSWER), move |tx| {
            let _ = tx.send(ui::probe(&want));
        }) {
            Ok(Some(line)) => {
                return format!("{line} after {:.1} s", start.elapsed().as_secs_f32());
            }
            Ok(None) if start.elapsed() < until => {
                std::thread::sleep(Duration::from_millis(500));
            }
            Ok(None) => return format!("error: {label:?} did not show within {secs:.0} s"),
            Err(e) => return e,
        }
    }
}

/// How long a call to the UI loop waits before it runs: how busy the UI thread is.
fn ping(budget: Budget) -> Result<Duration, String> {
    let sent = Instant::now();
    on_ui(budget.cap(ANSWER), move |tx| {
        let _ = tx.send(sent.elapsed());
    })
}

/// Lines of /proc/self/status, by their names.
fn status(keys: &[&str]) -> Vec<String> {
    let text = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    keys.iter()
        .filter_map(|k| {
            text.lines()
                .find(|l| l.starts_with(&format!("{k}:")))
                .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .collect()
}

/// What the core is running now.
fn jobs() -> String {
    use anibeam_core::{Call, Reply};
    let Some(core) = crate::local::core() else {
        return "core: not open".into();
    };
    match core.call(Call::ListJobs) {
        Ok(Reply::Jobs { jobs }) if jobs.is_empty() => "core: no jobs running".into(),
        Ok(Reply::Jobs { jobs }) => {
            let mut out = vec![format!("core: {} jobs", jobs.len())];
            for j in jobs {
                let age = j.started_at.elapsed().unwrap_or_default().as_secs_f32();
                let progress = j
                    .progress
                    .map(|p| match p.total {
                        Some(t) => format!(", {} of {t} {}", p.done, p.label),
                        None => format!(", {} {}", p.done, p.label),
                    })
                    .unwrap_or_default();
                out.push(format!("  {:?} #{} for {age:.1} s{progress}", j.kind, j.id));
            }
            out.join("\n")
        }
        Ok(_) => "core: no job list".into(),
        Err(e) => format!("core: jobs unreadable: {e}"),
    }
}

fn state(budget: Budget) -> String {
    let ui = match on_ui(budget.cap(ANSWER), |tx| {
        let _ = tx.send(ui::state_lines());
    }) {
        Ok(lines) => lines,
        Err(e) => return e,
    };
    let mut out = vec![ui];
    // A ping that failed goes last, as the line's error.
    let (ping_line, failed) = match ping(budget) {
        Ok(d) => (
            format!("ui loop: answered in {:.1} ms", d.as_secs_f64() * 1000.0),
            None,
        ),
        Err(e) => ("ui loop: no answer".to_string(), Some(e)),
    };
    out.push(ping_line);
    out.push(status(&["Threads", "VmRSS", "VmHWM"]).join(", "));
    out.push(jobs());
    out.extend(failed);
    out.join("\n")
}

/// Every thread's name and CPU time so far, in clock ticks.
fn thread_times() -> Vec<(u32, String, u64)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir("/proc/self/task")
        .into_iter()
        .flatten()
        .flatten()
    {
        let Ok(tid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let path = entry.path();
        let name = std::fs::read_to_string(path.join("comm"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let stat = std::fs::read_to_string(path.join("stat")).unwrap_or_default();
        // The fields after the name, which sits in brackets and may hold blanks: the state,
        // then utime and stime as the 12th and 13th.
        let after = stat.rsplit_once(')').map(|(_, a)| a).unwrap_or("");
        let f: Vec<&str> = after.split_whitespace().collect();
        let ticks = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        out.push((tid, name, ticks(11) + ticks(12)));
    }
    out
}

/// Samples the UI loop for `secs`: how long its calls waited, the frames it drew, and the
/// threads that used the CPU meanwhile.
fn perf(secs: f32, budget: Budget) -> String {
    // A UI thread too busy to take the mark is the stall perf is there to measure: the
    // sample goes on without the frame count and says why.
    let asked = Instant::now();
    let mark_failed = on_ui(budget.cap(ANSWER), |tx| {
        ui::frames_mark();
        let _ = tx.send(());
    })
    .err();
    // How long the mark waited is a wait like the pings', the first and often the longest.
    let mark_wait = asked.elapsed();
    // The CPU times and the span they are divided by start at the same moment, after the
    // mark, which a busy UI thread can hold up for seconds.
    let start = Instant::now();
    let before = thread_times();
    let span = Duration::from_secs_f32(secs);
    // A ping that gets no answer is the stall perf is there to catch: it counts as a wait of
    // its whole timeout, and the sample goes on.
    let mut waits = vec![mark_wait];
    let mut unanswered = usize::from(mark_failed.is_some());
    while start.elapsed() < span && !budget.left().is_zero() {
        let cap = budget.cap(ANSWER);
        match ping(budget) {
            Ok(d) => waits.push(d),
            Err(_) => {
                waits.push(cap);
                unanswered += 1;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let after = thread_times();
    let elapsed = start.elapsed().as_secs_f32();
    // A frames read that failed goes last, as the line's error, so the sample counts as
    // failed rather than passing with a hole in it.
    let (frames, failed) = match &mark_failed {
        Some(e) => (
            "frames: not counted, the UI thread took no mark".to_string(),
            Some(e.clone()),
        ),
        None => match on_ui(budget.cap(ANSWER), |tx| {
            let _ = tx.send(ui::frames_since_mark());
        }) {
            Ok(f) => (f, None),
            Err(e) => ("frames: not read".to_string(), Some(e)),
        },
    };

    waits.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let avg = waits.iter().map(|d| ms(*d)).sum::<f64>() / waits.len().max(1) as f64;
    let p95 = waits
        .get((waits.len() * 95 / 100).min(waits.len().saturating_sub(1)))
        .copied()
        .unwrap_or_default();
    let max = waits.last().copied().unwrap_or_default();
    let mut out = vec![
        format!("perf over {elapsed:.1} s"),
        format!(
            "ui loop: {} calls, waited avg {avg:.1} ms, p95 {:.1} ms, max {:.1} ms{}",
            waits.len(),
            ms(p95),
            ms(max),
            if unanswered > 0 {
                format!(", {unanswered} got no answer (counted at their timeout)")
            } else {
                String::new()
            }
        ),
        frames,
    ];
    // Linux counts CPU time in ticks of 1/100 s on every Android ABI.
    let mut busy: Vec<(String, u32, u64)> = after
        .iter()
        .map(|(tid, name, t)| {
            let was = before
                .iter()
                .find(|(b, _, _)| b == tid)
                .map(|(_, _, t)| *t)
                .unwrap_or(0);
            (name.clone(), *tid, t.saturating_sub(was) * 10)
        })
        .filter(|(_, _, ms)| *ms > 0)
        .collect();
    busy.sort_by_key(|b| std::cmp::Reverse(b.2));
    out.push(format!(
        "threads: {}, {} used the CPU",
        after.len(),
        busy.len()
    ));
    for (name, tid, cpu) in busy.iter().take(8) {
        out.push(format!(
            "  {name} ({tid}) {cpu} ms cpu, {:.0}%",
            *cpu as f32 / (elapsed * 10.0)
        ));
    }
    out.extend(failed);
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_answer_is_cut_at_a_line_and_keeps_its_error_last() {
        assert_eq!(cut("short".into()), "short");
        let mut long: Vec<String> = (0..30_000)
            .map(|i| format!("  text \"row {i}\" at 1,2"))
            .collect();
        long.push("error: \"x\" matches 3; --nth n picks one:".into());
        long.push("  1: button \"x\" at 1,2 (3x4)".into());
        let out = cut(long.join("\n"));
        assert!(out.len() < MOST + 200, "{}", out.len());
        assert!(out.contains("more lines; narrow it with a filter"));
        // The whole error stays last, its list of matches included.
        assert!(
            out.ends_with(
                "\nerror: \"x\" matches 3; --nth n picks one:\n  1: button \"x\" at 1,2 (3x4)"
            ),
            "{}",
            &out[out.len() - 200..]
        );
        assert!(out.lines().all(|l| !l.ends_with("at 1,")));
    }

    #[test]
    fn only_a_last_wait_cut_short_says_to_send_it_again() {
        let missed = || "error: \"Play\" did not show within 6 s".to_string();
        let last = again(missed(), "Play", 30.0, 6.5, true);
        assert_eq!(
            last,
            "error: \"Play\" did not show within 6 s\nagain: wait \"Play\" 23.5s"
        );
        // Inside a line, a resend would drop what follows: the line stops and says so.
        assert_eq!(again(missed(), "Play", 30.0, 6.5, false), missed());
        // A wait that had its whole time, or found its label, has nothing to send again.
        assert_eq!(again(missed(), "Play", 6.5, 6.5, true), missed());
        let found = "button \"Play\" at 5,6 (7x8) after 1.0 s".to_string();
        assert_eq!(again(found.clone(), "Play", 30.0, 6.5, true), found);
    }

    #[test]
    fn a_line_that_arrived_late_has_less_of_its_time() {
        // Held back longer than its whole time, it runs nothing and says what did not run.
        let (ok, said) = serve("where; where", LINE + Duration::from_secs(1));
        assert!(!ok);
        assert!(said.starts_with("error: the line ran out"), "{said}");
        assert!(said.contains("2 of 2 not run"), "{said}");
    }

    #[test]
    fn the_long_commands_say_how_long_they_may_take() {
        let wait = Command::Wait {
            label: "x".into(),
            secs: 30.0,
        };
        // A wait and a sample fit themselves to the line, so they need only a moment of it.
        assert_eq!(worst(&wait), Some(Duration::from_secs(1)));
        assert_eq!(worst(&Command::Perf(5.0)), Some(Duration::from_secs(1)));
        assert_eq!(worst(&Command::Where), None);
        // A default wait fits the short line of an older Android, a margin left over.
        let short = Budget(Instant::now() + SHORT_LINE);
        let take = fit(command::WAIT, short);
        assert!(take < command::WAIT && take > 5.0, "{take}");
        assert_eq!(fit(2.0, short), 2.0);
        assert_eq!(quoted(r#"Say "hi" \ bye"#), r#""Say \"hi\" \\ bye""#);
        // Off the phone a line has the long budget.
        assert_eq!(line_time(), LINE);
        assert!(SHORT_LINE < LINE);
        let b = Budget(Instant::now() + Duration::from_secs(3));
        assert!(b.cap(ANSWER) <= Duration::from_secs(3));
    }
}
