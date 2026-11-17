//! The writes that run from jobs of their own, Move up, the On hold sweep
//! and its undo, and Plan to watch, wait their turn in the series' write
//! line beside the status menu's writes (#1136). Each test fires one of
//! them together with a status from the menu, against a stand-in AniList
//! that holds the job's own save or live read back, so a write that does
//! not wait its turn lands first. In turn, the status asked for last is
//! what the list says.

mod common;
use common::started;
mod fixtures;

use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use anibeam_core::net::{Body, BoxFuture, Http, HttpError, HttpRequest, HttpResponse};
use anibeam_core::*;

const DAY: i64 = 86_400;

/// How long a held request waits for another save before it goes on. The
/// limiter spaces AniList requests most of a second apart, so a write that
/// does not wait its turn needs a few of those gaps to land in between.
const HOLD: Duration = Duration::from_secs(3);

/// A stand-in AniList that keeps one list entry and answers from it: the
/// list read and an entry read get the entry as it stands, a save changes
/// it. A save of `hold_save`'s status waits until another save has landed,
/// or for [`HOLD`] when none comes. With `hold_read`, the first live read
/// (the one On hold and Plan to watch make, which asks for the airing
/// state too) says so on `read_seen`, then answers with the entry as it was
/// when it came in, once another save has landed or [`HOLD`] has passed.
struct StandIn {
    media_id: u64,
    entry: Mutex<Option<serde_json::Value>>,
    hold_save: Option<&'static str>,
    hold_read: bool,
    reads: Mutex<u32>,
    read_seen: Mutex<Option<mpsc::Sender<()>>>,
    landed: tokio::sync::Notify,
}

impl StandIn {
    fn new(media_id: u64, entry: Option<serde_json::Value>) -> StandIn {
        StandIn {
            media_id,
            entry: Mutex::new(entry),
            hold_save: None,
            hold_read: false,
            reads: Mutex::new(0),
            read_seen: Mutex::new(None),
            landed: tokio::sync::Notify::new(),
        }
    }

    fn status(&self) -> Option<String> {
        self.entry
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|e| e["status"].as_str().map(str::to_string))
    }

    fn collection(&self) -> serde_json::Value {
        let entries: Vec<serde_json::Value> = self
            .entry
            .lock()
            .unwrap()
            .iter()
            .map(|e| {
                serde_json::json!({ "progress": e["progress"], "status": e["status"], "score": 0,
                                    "repeat": 0, "updatedAt": e["updatedAt"],
                                    "media": { "id": self.media_id } })
            })
            .collect();
        serde_json::json!({ "data": { "MediaListCollection": { "lists": [{ "entries": entries }] } } })
    }

    fn read(entry: Option<serde_json::Value>) -> serde_json::Value {
        match entry {
            Some(mut e) => {
                e["repeat"] = 0.into();
                e["media"] = serde_json::json!({ "status": "FINISHED", "episodes": 12, "nextAiringEpisode": null });
                serde_json::json!({ "data": { "MediaList": e } })
            }
            None => serde_json::json!({
                "errors": [{ "status": 404, "message": "Not Found." }],
                "data": { "MediaList": null }
            }),
        }
    }

    fn save(&self, variables: &serde_json::Value) -> serde_json::Value {
        let now = anibeam_core::time::now_secs();
        let mut entry = self.entry.lock().unwrap();
        let e = entry.get_or_insert_with(|| serde_json::json!({ "progress": 0 }));
        for key in ["progress", "status"] {
            if let Some(v) = variables.get(key) {
                e[key] = v.clone();
            }
        }
        e["updatedAt"] = now.into();
        serde_json::json!({ "data": { "SaveMediaListEntry": { "id": 9, "status": e["status"], "updatedAt": now } } })
    }
}

impl Http for StandIn {
    fn send(&self, req: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, HttpError>> {
        Box::pin(async move {
            let reply = |status: u16, body: String| HttpResponse {
                status,
                headers: vec![],
                body: body.into_bytes(),
            };
            let Some(Body::Json(v)) = req.body.as_ref().filter(|_| req.url.contains("anilist"))
            else {
                return Ok(reply(500, format!("no canned reply for {}", req.url)));
            };
            let query = v["query"].as_str().unwrap_or_default();
            let variables = &v["variables"];
            if query.contains("MediaListCollection") {
                return Ok(reply(200, self.collection().to_string()));
            }
            if !query.contains("SaveMediaListEntry") {
                let entry = self.entry.lock().unwrap().clone();
                let live = query.contains("nextAiringEpisode");
                let first = live && {
                    let mut reads = self.reads.lock().unwrap();
                    *reads += 1;
                    *reads == 1
                };
                if self.hold_read && first {
                    if let Some(seen) = self.read_seen.lock().unwrap().take() {
                        let _ = seen.send(());
                    }
                    let _ = tokio::time::timeout(HOLD, self.landed.notified()).await;
                }
                return Ok(reply(200, StandIn::read(entry).to_string()));
            }
            if self.hold_save.is_some() && variables["status"].as_str() == self.hold_save {
                let _ = tokio::time::timeout(HOLD, self.landed.notified()).await;
            }
            let body = self.save(variables).to_string();
            self.landed.notify_one();
            Ok(reply(200, body))
        })
    }
}

/// A stand-in that holds the first live read, and the receiver that hears
/// when it arrived.
fn holding_read(entry: Option<serde_json::Value>) -> (Arc<StandIn>, mpsc::Receiver<()>) {
    let mut anilist = StandIn::new(1, entry);
    anilist.hold_read = true;
    let (seen, read_seen) = mpsc::channel();
    *anilist.read_seen.lock().unwrap() = Some(seen);
    (Arc::new(anilist), read_seen)
}

fn read_arrives(read_seen: &mpsc::Receiver<()>) {
    read_seen
        .recv_timeout(Duration::from_secs(30))
        .expect("the job read the entry live");
}

/// One matched show with a file, so it is listed: AniList id only, twelve
/// episodes, finished airing.
fn show(core: &Core, anilist: u64) -> u64 {
    let now = anibeam_core::time::now_secs();
    let src = fixtures::insert_source(core, "/lib");
    let series = fixtures::insert_series(core, src, SeriesKind::Show, "/lib/Bebop", "Bebop");
    fixtures::insert_episode_file(core, series, "/lib/Bebop/01.mkv", 1.0, now);
    fixtures::media(anilist, "Bebop")
        .episodes(12)
        .status("FINISHED")
        .insert(core);
    fixtures::match_series(core, series, Some(anilist), None);
    series
}

fn dropped(core: &Core, series: u64) -> u64 {
    started(
        core,
        Call::SetListStatus {
            series,
            status: ListStatus::Dropped,
        },
    )
}

/// A move up of a show On hold here, then Dropped from the menu: the move
/// was asked for first, so its On hold lands first.
#[test]
fn a_status_set_after_a_move_up_lands_after_it() {
    let mut anilist = StandIn::new(1, None);
    anilist.hold_save = Some("PAUSED");
    let anilist = Arc::new(anilist);
    let (_dir, core, c) = common::open_core_with_http(anilist.clone());
    let series = show(&core, 1);
    core.call(Call::SetListStatus {
        series,
        status: ListStatus::Paused,
    })
    .unwrap();
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    let moved = started(
        &core,
        Call::MoveLists {
            tracker: Tracker::Anilist,
            series: vec![series],
        },
    );
    let status = dropped(&core, series);
    common::wait_job(&c, moved);
    common::wait_job(&c, status);
    assert_eq!(anilist.status().as_deref(), Some("DROPPED"));
}

/// Dropped from the menu, then a move up of the same show while Dropped is
/// still being saved: the tracker's Dropped is newer than the On hold kept
/// here by the time the move's turn comes, so the move leaves it.
#[test]
fn a_move_up_after_a_status_set_leaves_it() {
    let mut anilist = StandIn::new(1, None);
    anilist.hold_save = Some("DROPPED");
    let anilist = Arc::new(anilist);
    let (_dir, core, c) = common::open_core_with_http(anilist.clone());
    let series = show(&core, 1);
    core.call(Call::SetListStatus {
        series,
        status: ListStatus::Paused,
    })
    .unwrap();
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    let status = dropped(&core, series);
    let moved = started(
        &core,
        Call::MoveLists {
            tracker: Tracker::Anilist,
            series: vec![series],
        },
    );
    common::wait_job(&c, status);
    common::wait_job(&c, moved);
    assert_eq!(anilist.status().as_deref(), Some("DROPPED"));
}

/// Plan to watch has read the list live when Dropped is asked for from the
/// menu: Planning lands first.
#[test]
fn a_status_set_after_plan_to_watch_lands_after_it() {
    let (anilist, read_seen) = holding_read(None);
    let (_dir, core, c) = common::open_core_with_http(anilist.clone());
    let series = show(&core, 1);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    let planned = started(&core, Call::PlanToWatch { anilist_id: 1 });
    read_arrives(&read_seen);
    let status = dropped(&core, series);
    common::wait_job(&c, planned);
    common::wait_job(&c, status);
    assert_eq!(anilist.status().as_deref(), Some("DROPPED"));
}

/// A play of a series on hold is taking it back to Watching, and has read
/// the list live, when Dropped is asked for from the menu: Watching lands
/// first.
#[test]
fn a_status_set_after_a_play_takes_a_show_off_hold_lands_after_it() {
    let now = anibeam_core::time::now_secs();
    let (anilist, read_seen) = holding_read(Some(
        serde_json::json!({ "status": "PAUSED", "progress": 0, "updatedAt": now - 90 * DAY }),
    ));
    let (_dir, core, c) = common::open_core_with_http(anilist.clone());
    let series = show(&core, 1);
    let file = match core.call(Call::GetSeries { series }).unwrap() {
        Reply::SeriesDetail { detail } => detail.episodes[0].file,
        other => panic!("{other:?}"),
    };
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");
    fixtures::insert_tracker_entry(&core, "anilist", 1, 0, "paused", None);

    let session = match core.call(Call::OpenPlayback { file }).unwrap() {
        Reply::Playback { session } => session.session,
        other => panic!("{other:?}"),
    };
    for step in 0..35 {
        core.call(Call::Tick {
            session,
            position: f64::from(step),
            paused: false,
        })
        .unwrap();
    }
    read_arrives(&read_seen);
    let status = dropped(&core, series);
    common::wait_job(&c, status);
    let back = c.wait_for(
        |events| {
            events.iter().any(|e| {
                e.job.as_ref().is_some_and(|j| {
                    j.kind == JobKind::SetStatus && j.phase == JobPhase::Finished && j.id != status
                })
            })
        },
        Duration::from_secs(30),
    );
    assert!(back, "the play's job ended: {:#?}", c.events());
    assert_eq!(anilist.status().as_deref(), Some("DROPPED"));
}

/// The On hold sweep has read an entry live and is about to move it when
/// Dropped is asked for from the menu: the move lands first.
#[test]
fn a_status_set_while_the_sweep_moves_a_show_lands_after_it() {
    let now = anibeam_core::time::now_secs();
    let (anilist, read_seen) = holding_read(Some(
        serde_json::json!({ "status": "CURRENT", "progress": 3, "updatedAt": now - 90 * DAY }),
    ));
    let (_dir, core, c) = common::open_core_with_http(anilist.clone());
    let series = show(&core, 1);
    fixtures::connect_tracker(&core, Tracker::Anilist, 42, "atok");

    let refresh = started(&core, Call::RefreshProgress { tracker: None });
    read_arrives(&read_seen);
    let status = dropped(&core, series);
    common::wait_job(&c, refresh);
    common::wait_job(&c, status);
    let swept = c.wait_for(
        |events| {
            events.iter().any(|e| {
                e.job
                    .as_ref()
                    .is_some_and(|j| j.kind == JobKind::OnHold && j.phase == JobPhase::Finished)
            })
        },
        Duration::from_secs(30),
    );
    assert!(swept, "the sweep ended: {:#?}", c.events());
    assert_eq!(anilist.status().as_deref(), Some("DROPPED"));
}
