use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;

use anibeam_core::{Call, Core, CorePaths, Direction, Event, EventListener, JobPhase, Level, Reply, Sort, Tab};
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
        /// Stay attached to a Started job and print its events until the terminal one.
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
    /// Start a scan; with --wait, print its events until it finishes.
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
}

/// Builds a Call from its variant name and an optional JSON object of fields.
pub fn parse_call(name: &str, json: Option<&str>) -> Result<Call, String> {
    let value = match json {
        Some(j) => {
            let fields: serde_json::Value = serde_json::from_str(j).map_err(|e| format!("bad --json: {e}"))?;
            serde_json::json!({ name: fields })
        }
        None => serde_json::Value::String(name.to_string()),
    };
    serde_json::from_value(value).map_err(|e| format!("unknown call {name}: {e}"))
}

/// Forwards every event it sees into a channel. Both `--wait` and
/// `events --follow` read from the other end with a blocking `recv`, so
/// neither ever sleep-polls.
struct ChannelListener(Sender<Event>);

impl EventListener for ChannelListener {
    fn on_event(&self, event: Event) {
        // Nowhere useful to report a closed receiver; the caller has
        // already stopped listening on purpose.
        let _ = self.0.send(event);
    }
}

fn open(root: Option<PathBuf>) -> Arc<Core> {
    let paths = match root {
        Some(r) => CorePaths::under(&r),
        None => CorePaths::xdg().unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(2);
        }),
    };
    Core::open(paths).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    })
}

/// Sends one call, prints the reply as JSON, and with `wait` set, stays
/// attached to a `Started` job's own events until its `Finished` phase.
fn send(core: &Core, call: Call, wait: bool) {
    // Subscribed before the call goes in, so a fast job's first events are
    // never missed racing the reply.
    let (tx, rx) = mpsc::channel::<Event>();
    let _sub = core.subscribe(Arc::new(ChannelListener(tx)));

    match core.call(call) {
        Ok(reply) => {
            println!("{}", serde_json::to_string_pretty(&reply).unwrap());
            if wait
                && let Reply::Started { job } = reply
            {
                for event in rx {
                    if !event.job.as_ref().is_some_and(|j| j.id == job) {
                        continue;
                    }
                    println!("{}", serde_json::to_string(&event).unwrap());
                    if event.job.as_ref().is_some_and(|j| j.phase == JobPhase::Finished) {
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}", serde_json::to_string_pretty(&e).unwrap());
            std::process::exit(1);
        }
    }
}

fn run_call(core: &Core, name: &str, json: Option<&str>, wait: bool) {
    let call = parse_call(name, json).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    send(core, call, wait);
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
    let direction = Direction::from_column(direction).unwrap_or_else(|| bad("direction", direction));
    let call = Call::ListSeries { tab, query: query.to_string(), sort, direction, reveal_hidden: false };
    if let Reply::Series { series } = ask(core, call) {
        for c in series {
            let watched = dash(c.watched.map(|w| w.to_string()));
            let total = dash(c.total_episodes.map(|t| t.to_string()));
            println!("{}\t{}\t{}\t{watched}/{total}", c.id, dash(c.code), c.title);
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
    match cli.command {
        Command::Call { name, json, wait } => run_call(&core, &name, json.as_deref(), wait),
        Command::Events { follow, level } => run_events(&core, follow, &level),
        Command::Sources => run_sources(&core),
        Command::Scan { source, wait } => send(&core, Call::Scan { source }, wait),
        Command::List { tab, sort, direction, query } => run_list(&core, &tab, &sort, &direction, &query),
        Command::Show { series } => run_show(&core, series),
    }
    core.shutdown();
}

#[cfg(test)]
mod tests {
    use super::parse_call;
    use anibeam_core::Call;

    #[test]
    fn parse_call_builds_unit_and_field_variants() {
        assert_eq!(parse_call("ListSources", None).unwrap(), Call::ListSources);
        assert_eq!(parse_call("Scan", Some(r#"{"source": null}"#)).unwrap(), Call::Scan { source: None });
        assert!(parse_call("Nope", None).is_err());
    }
}
