use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};

use anibeam_core::trackers::Secrets;
use anibeam_core::{
    Call, Core, CorePaths, Direction, Event, EventBody, EventListener, JobPhase, Level, Reply,
    Sort, Tab,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "anibeam-cli", version = anibeam_core::VERSION, about = "AniBeam's core from the terminal")]
struct Cli {
    /// Put every directory under this root instead of the XDG paths.
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Send any call and print the reply as JSON.
    Call {
        name: String,
        #[arg(long)]
        json: Option<String>,
        /// Print the events of the jobs the call starts. The CLI waits for them either way.
        #[arg(long)]
        wait: bool,
    },
    /// Print recent events, then the live stream with --follow.
    Events {
        #[arg(long)]
        follow: bool,
        #[arg(long, default_value = "info")]
        level: String,
    },
    /// The sources: id, path, available, series count.
    Sources,
    /// Run a scan to its end; with --wait, print its events on the way.
    Scan {
        #[arg(long)]
        source: Option<u64>,
        #[arg(long)]
        wait: bool,
    },
    /// One line per card: id, code, title, watched/total.
    List {
        #[arg(long, default_value = "all")]
        tab: String,
        #[arg(long, default_value = "alpha")]
        sort: String,
        #[arg(long, default_value = "asc")]
        direction: String,
        #[arg(long, default_value = "")]
        query: String,
    },
    /// One series page as JSON.
    Show { series: u64 },
    /// The jobs running right now: id, kind, started, progress.
    Jobs,
    /// Read an anibeam-export document into this library.
    Import {
        file: String,
        #[arg(long)]
        wait: bool,
    },
    /// Write this library out as an anibeam-export document.
    Export {
        file: String,
        /// Also carry the accounts, the keys, the history and the preferences.
        #[arg(long)]
        private: bool,
        #[arg(long)]
        wait: bool,
    },
}

/// Builds a Call from its variant name and an optional JSON object of fields.
pub fn parse_call(name: &str, json: Option<&str>) -> Result<Call, String> {
    let value = match json {
        Some(j) => {
            let fields: serde_json::Value =
                serde_json::from_str(j).map_err(|e| format!("bad --json: {e}"))?;
            serde_json::json!({ name: fields })
        }
        None => serde_json::Value::String(name.to_string()),
    };
    serde_json::from_value(value).map_err(|e| format!("unknown call {name}: {e}"))
}

/// Forwards every event it sees into a channel. Following a call's jobs and
/// `events --follow` both read from the other end with a blocking `recv`, so
/// neither ever sleep-polls.
struct ChannelListener(Sender<Event>);

impl EventListener for ChannelListener {
    fn on_event(&self, event: Event) {
        // Nowhere useful to report a closed receiver; the caller has
        // already stopped listening on purpose.
        let _ = self.0.send(event);
    }
}

/// A run under `--root` keeps its secrets in that root's own
/// `secrets.json` and never probes the machine's keyring: a sandbox is
/// asked for precisely so it touches nothing outside itself, and a keyring
/// prompt on the owner's screen is the opposite of that. Without a root the
/// core makes the choice itself, keyring first. A root also runs against the
/// owner's real anirss and qBittorrent, so it changes nothing there on its
/// own, as the desktop's sandbox runs do: no pruning, no endpoint writes.
fn open(root: Option<PathBuf>) -> Arc<Core> {
    let sandbox = root.is_some();
    let opened = match root {
        Some(r) => {
            let paths = CorePaths::under(&r);
            let secrets = Secrets::file_only(paths.secrets_path());
            Core::open_with_secrets(paths, secrets)
        }
        None => {
            let paths = CorePaths::xdg().unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(2);
            });
            Core::open(paths)
        }
    };
    let core = opened.unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    if sandbox {
        core.disable_unattended_changes();
    }
    core
}

/// Sends one call and prints the reply as JSON, then follows every job the
/// call started to its finish: the reply's own `Started` job, and any job it
/// started on the side, such as the scan behind `AddSource`. `main` shuts the
/// core down after this returns and the shutdown cancels whatever is still
/// running, so nothing is left behind. `wait` only decides whether the jobs'
/// events are printed on the way. Returns the exit code: 1 when the call or
/// one of its jobs failed or was cancelled.
fn send(core: &Core, call: Call, wait: bool) -> i32 {
    // Subscribed before the call goes in, so a fast job's first events are
    // never missed racing the reply.
    let (tx, rx) = mpsc::channel::<Event>();
    let _sub = core.subscribe(Arc::new(ChannelListener(tx)));

    // A job is registered as running before the call that starts it
    // returns, so the difference between these two lists is what the call
    // started.
    let before = running_jobs(core);
    match core.call(call) {
        Ok(reply) => {
            println!("{}", serde_json::to_string_pretty(&reply).unwrap());
            let mut jobs: HashSet<u64> = running_jobs(core).difference(&before).copied().collect();
            // Listed even when it already finished, or joined a run that was
            // going before the call: its Finished event is still on its way.
            if let Reply::Started { job } = reply {
                jobs.insert(job);
            }
            if jobs.is_empty() {
                0
            } else {
                follow(rx, jobs, wait)
            }
        }
        Err(e) => {
            eprintln!("{}", serde_json::to_string_pretty(&e).unwrap());
            1
        }
    }
}

fn running_jobs(core: &Core) -> HashSet<u64> {
    match core.call(Call::ListJobs) {
        Ok(Reply::Jobs { jobs }) => jobs.into_iter().map(|j| j.id).collect(),
        _ => HashSet::new(),
    }
}

/// Reads events until every job in `jobs` reaches its `Finished` phase,
/// printing theirs when `print` is set, and returns 1 when any of them
/// failed or was cancelled. A failure puts its error on stderr whether or
/// not the events were printed.
fn follow(events: impl IntoIterator<Item = Event>, mut jobs: HashSet<u64>, print: bool) -> i32 {
    let mut code = 0;
    for event in events {
        let Some(j) = event.job.as_ref().filter(|j| jobs.contains(&j.id)) else {
            continue;
        };
        if print {
            println!("{}", serde_json::to_string(&event).unwrap());
        }
        if j.phase != JobPhase::Finished {
            continue;
        }
        match &event.body {
            EventBody::JobFailed { error } => {
                eprintln!("{}", serde_json::to_string_pretty(error).unwrap());
                code = 1;
            }
            EventBody::JobCancelled => {
                eprintln!("job {} was cancelled", j.id);
                code = 1;
            }
            _ => {}
        }
        jobs.remove(&j.id);
        if jobs.is_empty() {
            return code;
        }
    }
    // Only when the event stream closes with jobs unfinished. The CLI's own
    // subscription keeps it open today, so this is a guard, not a path.
    eprintln!("the core stopped before its jobs finished");
    1
}

fn run_call(core: &Core, name: &str, json: Option<&str>, wait: bool) -> i32 {
    let call = parse_call(name, json).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    send(core, call, wait)
}

/// A call whose reply the view formats itself. An error ends the process
/// the same way `send` does.
fn ask(core: &Core, call: Call) -> Reply {
    core.call(call).unwrap_or_else(|e| {
        eprintln!("{}", serde_json::to_string_pretty(&e).unwrap());
        std::process::exit(1);
    })
}

/// A column with nothing in it reads as a dash, so every line has the same
/// shape and splits on tabs.
fn dash(value: Option<String>) -> String {
    value.unwrap_or_else(|| "-".to_string())
}

fn bad(what: &str, value: &str) -> ! {
    eprintln!("unknown {what}: {value}");
    std::process::exit(2);
}

fn run_sources(core: &Core) {
    if let Reply::Sources { sources } = ask(core, Call::ListSources) {
        for s in sources {
            println!("{}\t{}\t{}\t{}", s.id, s.path, s.available, s.series_count);
        }
    }
}

fn run_list(core: &Core, tab: &str, sort: &str, direction: &str, query: &str) {
    let tab = Tab::from_column(tab).unwrap_or_else(|| bad("tab", tab));
    let sort = Sort::from_column(sort).unwrap_or_else(|| bad("sort", sort));
    let direction =
        Direction::from_column(direction).unwrap_or_else(|| bad("direction", direction));
    let call = Call::ListSeries {
        tab,
        query: query.to_string(),
        sort,
        direction,
        reveal_hidden: false,
    };
    if let Reply::Series { series } = ask(core, call) {
        for c in series {
            let watched = dash(c.watched.map(|w| w.to_string()));
            let total = dash(c.total_episodes.map(|t| t.to_string()));
            println!("{}\t{}\t{}\t{watched}/{total}", c.id, dash(c.code), c.title);
        }
    }
}

/// One line per running job. `started` is unix seconds, the same shape the
/// events carry, and a job with nothing to report yet has a dash for its
/// progress rather than an empty column.
fn run_jobs(core: &Core) {
    if let Reply::Jobs { jobs } = ask(core, Call::ListJobs) {
        for j in jobs {
            let progress = j.progress.map_or_else(
                || "-".to_string(),
                |p| {
                    let total = dash(p.total.map(|t| t.to_string()));
                    format!("{}/{total} {}", p.done, p.label)
                },
            );
            println!(
                "{}\t{}\t{}\t{progress}",
                j.id,
                j.kind.as_str(),
                anibeam_core::time::to_secs(j.started_at)
            );
        }
    }
}

fn run_show(core: &Core, series: u64) {
    if let Reply::SeriesDetail { detail } = ask(core, Call::GetSeries { series }) {
        println!("{}", serde_json::to_string_pretty(&detail).unwrap());
    }
}

/// Prints the ring's recent events at or above `level`, then with `follow`
/// set, blocks on the live stream until the channel closes (Ctrl-C ends
/// the process through the default handler).
fn run_events(core: &Core, follow: bool, level: &str) {
    let min = Level::from_column(level).unwrap_or(Level::Info);
    if let Ok(Reply::Events { events }) = core.call(Call::RecentEvents { limit: 2000 }) {
        for e in events.into_iter().filter(|e| e.level >= min) {
            println!("{}", serde_json::to_string(&e).unwrap());
        }
    }
    if follow {
        let (tx, rx) = mpsc::channel::<Event>();
        let _sub = core.subscribe(Arc::new(ChannelListener(tx)));
        for event in rx {
            if event.level >= min {
                println!("{}", serde_json::to_string(&event).unwrap());
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let core = open(cli.root);
    let code = match cli.command {
        Command::Call { name, json, wait } => run_call(&core, &name, json.as_deref(), wait),
        Command::Events { follow, level } => {
            run_events(&core, follow, &level);
            0
        }
        Command::Sources => {
            run_sources(&core);
            0
        }
        Command::Scan { source, wait } => send(&core, Call::Scan { source }, wait),
        Command::List {
            tab,
            sort,
            direction,
            query,
        } => {
            run_list(&core, &tab, &sort, &direction, &query);
            0
        }
        Command::Show { series } => {
            run_show(&core, series);
            0
        }
        Command::Jobs => {
            run_jobs(&core);
            0
        }
        Command::Import { file, wait } => send(&core, Call::Import { path: file }, wait),
        Command::Export {
            file,
            private,
            wait,
        } => send(
            &core,
            Call::Export {
                path: file,
                private,
            },
            wait,
        ),
    };
    core.shutdown();
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::{follow, parse_call};
    use anibeam_core::{
        Call, CoreError, Event, EventBody, JobKind, JobPhase, JobRef, Level, Stage,
    };
    use std::collections::HashSet;

    fn event(job: u64, phase: JobPhase, body: EventBody) -> Event {
        Event {
            seq: 0,
            at: std::time::SystemTime::now(),
            level: Level::Info,
            stage: Stage::Store,
            message: String::new(),
            plain: None,
            job: Some(JobRef {
                id: job,
                kind: JobKind::Export,
                phase,
            }),
            body,
        }
    }

    #[test]
    fn parse_call_builds_unit_and_field_variants() {
        assert_eq!(parse_call("ListSources", None).unwrap(), Call::ListSources);
        assert_eq!(
            parse_call("Scan", Some(r#"{"source": null}"#)).unwrap(),
            Call::Scan { source: None }
        );
        assert!(parse_call("Nope", None).is_err());
    }

    #[test]
    fn follow_waits_for_every_job_and_fails_if_any_did() {
        let started = |id| {
            event(
                id,
                JobPhase::Started,
                EventBody::JobStarted {
                    kind: JobKind::Export,
                },
            )
        };
        let done = |id| event(id, JobPhase::Finished, EventBody::Notice);
        let failed = |id| {
            event(
                id,
                JobPhase::Finished,
                EventBody::JobFailed {
                    error: CoreError::internal("disk full"),
                },
            )
        };
        let cancelled = |id| event(id, JobPhase::Finished, EventBody::JobCancelled);
        let set = |ids: &[u64]| ids.iter().copied().collect::<HashSet<u64>>();

        // A job the call did not start finishing is not its result.
        assert_eq!(follow([started(1), done(9), done(1)], set(&[1]), false), 0);
        assert_eq!(
            follow([started(1), done(9), failed(1)], set(&[1]), false),
            1
        );
        assert_eq!(follow([started(1), cancelled(1)], set(&[1]), false), 1);
        // A side job failing after the reply's job succeeded still fails.
        assert_eq!(follow([done(1), failed(2)], set(&[1, 2]), false), 1);
        assert_eq!(follow([failed(2), done(1)], set(&[1, 2]), false), 1);
        // A second job still running keeps the CLI waiting.
        assert_eq!(follow([done(1)], set(&[1, 2]), false), 1);
    }
}
