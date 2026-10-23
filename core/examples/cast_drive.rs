//! Drives a whole cast from a throwaway core: finds devices, casts the first episode of a
//! folder to the device whose name contains `<device>`, pauses, seeks, plays, and stops,
//! printing every cast event. For trying the protocols against real or fake receivers.
//! cargo run --example cast_drive --features cast -- <library dir> <device name part> [sub]

use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anibeam_core::{
    Call, CastPlayState, CastSubtitle, Core, CorePaths, Direction, Event, EventBody, EventListener,
    Reply, Sort, Tab,
};

struct Tx(mpsc::Sender<Event>);

impl EventListener for Tx {
    fn on_event(&self, e: Event) {
        let _ = self.0.send(e);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (library, want) = (&args[1], args[2].to_lowercase());
    let with_subs = args.get(3).is_some_and(|a| a == "sub");
    let root = tempfile_dir();
    let core = Core::open(CorePaths::under(&root)).expect("core");
    let (tx, rx) = mpsc::channel();
    let _sub = core.subscribe(Arc::new(Tx(tx)));
    core.call(Call::AddSource {
        path: library.clone(),
    })
    .expect("source");
    core.start().expect("start");

    // The first file the scan finds.
    let file = wait_for(Duration::from_secs(30), || {
        let Ok(Reply::Library { series, .. }) = core.call(Call::ListLibrary {
            tab: Tab::All,
            query: String::new(),
            sort: Sort::Alpha,
            direction: Direction::Asc,
            reveal_hidden: false,
        }) else {
            return None;
        };
        let s = series.first()?;
        let Ok(Reply::SeriesDetail { detail }) = core.call(Call::GetSeries { series: s.id }) else {
            return None;
        };
        detail
            .episodes
            .iter()
            .chain(detail.unmatched_files.iter())
            .map(|e| e.file)
            .next()
    })
    .expect("a file in the library");
    println!("file {file}");

    let device = wait_for(Duration::from_secs(12), || {
        let Ok(Reply::CastDevices { devices }) = core.call(Call::CastDevices) else {
            return None;
        };
        devices
            .into_iter()
            .find(|d| d.name.to_lowercase().contains(&want))
    })
    .expect("the device");
    println!("device {device:?}");

    let subtitle = if with_subs {
        CastSubtitle::Track { number: 4 }
    } else {
        CastSubtitle::None
    };
    let r = core.call(Call::CastStart {
        device: device.id.clone(),
        file,
        start: 5.0,
        audio_track: None,
        subtitle,
    });
    println!("start: {r:?}");

    let script: Vec<(u64, Call)> = vec![
        (12, Call::CastPause),
        (16, Call::CastPlay),
        (20, Call::CastSeek { to: 60.0 }),
        (30, Call::CastStop),
    ];
    let t0 = Instant::now();
    let mut next = 0;
    while t0.elapsed() < Duration::from_secs(34) {
        if let Some((at, call)) = script.get(next)
            && t0.elapsed() >= Duration::from_secs(*at)
        {
            println!(
                "{:5.1}s -> {call:?}: {:?}",
                t0.elapsed().as_secs_f64(),
                core.call(call.clone())
            );
            next += 1;
        }
        if let Ok(e) = rx.recv_timeout(Duration::from_millis(200))
            && let EventBody::CastChanged { state } = &e.body
        {
            println!(
                "{:5.1}s {:?} at {:.1}/{:.1}{}",
                t0.elapsed().as_secs_f64(),
                state.state,
                state.position,
                state.duration,
                state
                    .error
                    .as_ref()
                    .map(|e| format!(" error: {e}"))
                    .unwrap_or_default()
            );
            if matches!(state.state, CastPlayState::Failed) {
                break;
            }
        }
    }
    core.shutdown();
    let _ = std::fs::remove_dir_all(&root);
}

fn wait_for<T>(limit: Duration, mut f: impl FnMut() -> Option<T>) -> Option<T> {
    let t = Instant::now();
    while t.elapsed() < limit {
        if let Some(v) = f() {
            return Some(v);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    None
}

fn tempfile_dir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("anibeam-cast-drive-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}
