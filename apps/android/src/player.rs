//! The player page: one episode playing in libmpv, the desktop's PlayerPage ported.
//!
//! The rules are the desktop's, read from `apps/linux/qml/PlayerPage.qml`: the core opens a
//! session and owns the history (ticks once a second while playing and on every pause and
//! seek, a close with its reason), the track pick and the owned options come from the core,
//! the skip windows arrive with the session and again when AniSkip answers, auto-skip fires
//! once per kind and Undo disarms it, and the auto-next pill rises at the outro and counts
//! five seconds before the next episode. What differs is the phone's idiom: a tap shows the
//! chrome rather than pausing, a double tap on a side jumps ten seconds, and the screen
//! turns to landscape with the system bars hidden while a file plays.

use std::cell::RefCell;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use anibeam_core::playback::{options, tracks};
use anibeam_core::{
    AutoSkip, Call, CastDevice, CastKind, CastPlayState, CastState, CastSubtitle, Chapter, CloseReason, EventBody,
    PlaybackSession, Reply, SkipKind, SkipWindow, SubtitleChoice, Tracker, TrackerOutcome,
};
use slint::{ComponentHandle, Model, ModelRc, Timer, TimerMode, VecModel};

use crate::mpv::{Mpv, MpvEvent, Value};
use crate::thumbs::{Shot, Thumbs};
use crate::video::{Frame, Share, Video};
use crate::{AppWindow, Band, PickRow, PlayerData};

/// The chrome hides this long after the last touch.
const CHROME_FOR: Duration = Duration::from_millis(2500);
/// The auto-next countdown.
const NEXT_COUNT: Duration = Duration::from_secs(5);
/// The speeds the picker offers, the desktop's quarter steps over the useful range.
const SPEEDS: [f64; 8] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0];

/// The desktop's auto-skip and auto-next rules, kept apart from mpv and the UI so they can
/// be tested on their own.
#[derive(Debug, Default)]
pub struct Rules {
    pub duration: f64,
    pub position: f64,
    pub loaded: bool,
    pub windows: Vec<SkipWindow>,
    pub auto: AutoSkip,
    pub has_next: bool,
    /// Armed per kind for the length of the session; Undo disarms its kind for good.
    armed: [bool; 2],
    /// A seek put the position inside the window: the frames that follow must not skip out.
    landed: [bool; 2],
    last: Option<f64>,
    pub next_visible: bool,
    pub next_counting: bool,
    pub next_dismissed: bool,
}

/// What a new position asks the page to do.
#[derive(Debug, PartialEq)]
pub enum Step {
    /// Auto-skip: seek here and offer Undo back to `undo`.
    Skip { to: f64, undo: f64, kind: SkipKind },
    StartCount,
    StopCount,
}

fn slot(kind: &SkipKind) -> usize {
    match kind {
        SkipKind::Intro => 0,
        SkipKind::Outro => 1,
    }
}

impl Rules {
    pub fn new(windows: Vec<SkipWindow>, auto: AutoSkip, has_next: bool) -> Rules {
        Rules { windows, auto, has_next, armed: [true, true], ..Rules::default() }
    }

    pub fn window(&self, kind: SkipKind) -> Option<&SkipWindow> {
        self.windows.iter().find(|w| w.kind == kind)
    }

    fn inside(w: Option<&SkipWindow>, t: f64) -> bool {
        w.is_some_and(|w| t >= w.start && t < w.end)
    }

    /// The window the position is in, for the Skip chip.
    pub fn current(&self) -> Option<&SkipWindow> {
        [SkipKind::Intro, SkipKind::Outro].into_iter().filter_map(|k| self.window(k)).find(|w| Self::inside(Some(w), self.position))
    }

    pub fn disarm(&mut self, kind: &SkipKind) {
        self.armed[slot(kind)] = false;
    }

    /// The position moved: auto-skip, then the auto-next pill and its countdown.
    pub fn moved(&mut self, t: f64) -> Vec<Step> {
        self.position = t;
        let jumped = self.last.is_some_and(|l| (t - l).abs() > 2.0);
        self.last = Some(t);
        let mut steps = Vec::new();
        for (kind, on) in [(SkipKind::Intro, self.auto.intro), (SkipKind::Outro, self.auto.outro)] {
            let Some(w) = self.window(kind).cloned() else { continue };
            let i = slot(&kind);
            if !Self::inside(Some(&w), t) {
                self.landed[i] = false;
                continue;
            }
            if jumped {
                self.landed[i] = true;
                continue;
            }
            if !on || !self.armed[i] || self.landed[i] {
                continue;
            }
            self.armed[i] = false;
            steps.push(Step::Skip { to: w.end + 1.0, undo: w.start, kind });
        }
        steps.extend(self.update_next());
        steps
    }

    /// The pill appears when the outro starts, or eight seconds from the end with no outro;
    /// the countdown starts at the outro's end when the outro runs to the end of the file,
    /// else three seconds from the end. A seek back out of the counting zone stops it.
    pub fn update_next(&mut self) -> Option<Step> {
        if !self.has_next || self.next_dismissed || !self.loaded || self.duration <= 0.0 {
            self.next_visible = false;
            return self.stop_count();
        }
        let t = self.position;
        let remaining = self.duration - t;
        let outro = self.window(SkipKind::Outro).cloned();
        let count = outro.as_ref().is_some_and(|o| self.duration - o.end < 20.0 && t >= o.end) || remaining <= 3.0;
        let step = if count && !self.next_counting {
            self.next_counting = true;
            Some(Step::StartCount)
        } else if !count && self.next_counting {
            self.stop_count()
        } else {
            None
        };
        self.next_visible = self.next_counting || outro.map(|o| t >= o.start).unwrap_or(remaining <= 8.0);
        step
    }

    pub fn stop_count(&mut self) -> Option<Step> {
        if self.next_counting {
            self.next_counting = false;
            return Some(Step::StopCount);
        }
        None
    }

    pub fn stay(&mut self) {
        self.next_dismissed = true;
        self.next_visible = false;
        self.next_counting = false;
    }
}

fn tracker_name(t: &Tracker) -> &'static str {
    match t {
        Tracker::Anilist => "AniList",
        Tracker::Mal => "MyAnimeList",
    }
}

fn refusal_word(r: &anibeam_core::Refusal) -> &'static str {
    use anibeam_core::Refusal::*;
    match r {
        Hidden => "the show is hidden",
        NoMatch => "the show has no tracker match",
        NotNewer => "the tracker is already past this episode",
        Extra => "this is an extra",
        Unmatched => "the file is not matched to an episode",
        OnDisk => "the file is only on disk",
    }
}

/// "Tracked: AniList at 12; MyAnimeList at 12", the desktop's outcome line.
pub fn outcome_line(outcomes: &[TrackerOutcome]) -> String {
    let ok: Vec<String> = outcomes
        .iter()
        .filter(|o| o.ok)
        .map(|o| match o.progress {
            Some(p) => format!("{} at {p}", tracker_name(&o.tracker)),
            None => format!("{} ok", tracker_name(&o.tracker)),
        })
        .collect();
    let bad: Vec<&TrackerOutcome> = outcomes.iter().filter(|o| !o.ok).collect();
    let head = if !ok.is_empty() {
        "Tracked"
    } else if bad.iter().all(|o| o.reason.is_some()) {
        "Not tracked"
    } else {
        "Could not update your list"
    };
    let all: Vec<String> = ok
        .into_iter()
        .chain(bad.iter().map(|o| match (&o.reason, &o.message) {
            (Some(r), _) => format!("{}: {}", tracker_name(&o.tracker), refusal_word(r)),
            (None, Some(m)) => format!("{}: {m}", tracker_name(&o.tracker)),
            (None, None) => format!("{} could not be updated", tracker_name(&o.tracker)),
        }))
        .collect();
    if all.is_empty() { head.to_string() } else { format!("{head}: {}", all.join("; ")) }
}

/// The chapters as the core wants them, from mpv's `chapter-list` JSON.
pub fn chapters(json: &str) -> Vec<Chapter> {
    let list: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    list.as_array()
        .map(|a| {
            a.iter()
                .map(|c| Chapter {
                    title: c.get("title").and_then(|t| t.as_str()).unwrap_or_default().to_string(),
                    start: c.get("time").and_then(|t| t.as_f64()).unwrap_or(0.0),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A cast in progress from this page: the device has the picture, the page is the remote.
struct Casting {
    device: CastDevice,
    state: CastPlayState,
}

/// How long the page holds Wi-Fi multicast open while it looks for devices.
const LOOK_FOR: Duration = Duration::from_secs(10);

/// What the notice's action does.
enum Undo {
    Skip { to: f64, kind: SkipKind },
}

struct Player {
    window: slint::Weak<AppWindow>,
    session: PlaybackSession,
    /// The other device the session is open on, for an episode streamed from it; None
    /// for the phone's own.
    host: Option<Arc<crate::stream::Host>>,
    mpv: Arc<Mpv>,
    video: Option<Video>,
    rules: Rules,
    tracks: Vec<tracks::Track>,
    aid: i64,
    sid: i64,
    /// Whether loadfile has been sent.
    requested: bool,
    /// The renderer could not start at all; the file plays on sound alone.
    no_video: bool,
    /// Counts renderers, so a frame or a start from one that has stopped is ignored.
    video_gen: u64,
    paused: bool,
    seeking: bool,
    /// A drag on the bar is on: the preview bubble takes frames, the episode stays put.
    scrubbing: bool,
    /// The preview core, opened on the first drag.
    thumbs: Option<Thumbs>,
    ended: bool,
    closed: bool,
    episode_number: Option<f64>,
    sub_pos_setting: f64,
    sub_pos: f64,
    undo: Option<Undo>,
    tick: Timer,
    hide: Timer,
    next: Timer,
    notice: Timer,
    fallback: Timer,
    /// The cast this page is the remote for.
    cast: Option<Casting>,
    /// A device to cast to as soon as the file's tracks are known: the next episode after
    /// one that finished on the TV.
    cast_on_load: Option<CastDevice>,
    /// mpv's track list as it last came, for the track numbers a cast names.
    track_json: String,
    look: Timer,
}

thread_local! {
    static PLAYER: RefCell<Option<Player>> = const { RefCell::new(None) };
    /// Set by `open` before the player exists, read by the rendering notifier.
    static WINDOW: RefCell<Option<slint::Weak<AppWindow>>> = const { RefCell::new(None) };
    /// The Chromecasts and TVs found on this network, kept across episodes.
    static DEVICES: RefCell<Vec<CastDevice>> = const { RefCell::new(Vec::new()) };
    /// Carried from one episode's page to the next's when the cast goes on.
    static PENDING_CAST: RefCell<Option<CastDevice>> = const { RefCell::new(None) };
}

fn with_player(f: impl FnOnce(&mut Player)) {
    PLAYER.with(|p| {
        if let Ok(mut p) = p.try_borrow_mut()
            && let Some(p) = p.as_mut()
        {
            f(p);
        }
    });
}

fn data(window: &slint::Weak<AppWindow>, f: impl FnOnce(&PlayerData)) {
    if let Some(w) = window.upgrade() {
        f(&w.global::<PlayerData>());
    }
}

/// Wires the page's callbacks and the rendering notifier. Once, at startup.
pub fn install(window: &AppWindow) {
    WINDOW.with(|w| *w.borrow_mut() = Some(window.as_weak()));
    let d = window.global::<PlayerData>();
    d.on_back(|| close_page(CloseReason::Stopped));
    d.on_toggle_pause(|| with_player(|p| p.toggle_pause()));
    d.on_scrub(|t| with_player(|p| p.scrub(t as f64)));
    d.on_scrub_cancel(|| with_player(|p| p.scrubbing = false));
    d.on_seek(|t| with_player(|p| p.seek(t as f64)));
    d.on_seek_by(|by| with_player(|p| p.seek(p.rules.position + by as f64)));
    d.on_skip(|| with_player(|p| p.skip()));
    d.on_next(|| neighbour(true));
    d.on_prev(|| neighbour(false));
    d.on_stay(|| with_player(|p| p.stay()));
    d.on_replay_clicked(|| {
        let mut file = None;
        with_player(|p| file = Some((p.session.file, p.host.is_some())));
        if let Some((file, remote)) = file {
            close_page(CloseReason::Switched);
            if remote { open_remote(file) } else { open(file) }
        }
    });
    d.on_open_picker(|kind| with_player(|p| p.open_picker(kind)));
    d.on_pick(|i| with_player(|p| p.pick(i)));
    d.on_close_picker(|| with_player(|p| p.close_picker()));
    d.on_mark(|| with_player(|p| p.mark()));
    d.on_notice_clicked(|| with_player(|p| p.notice_clicked()));
    d.on_touched(|| with_player(|p| p.show_chrome()));
    d.on_cast(|| with_player(|p| p.open_picker(4)));

    let weak = window.as_weak();
    let r = window.window().set_rendering_notifier(move |state, _api| match state {
        slint::RenderingState::BeforeRendering => {
            let Some(share) = crate::video::current_share() else { return };
            let size = weak.upgrade().map(|w| w.window().size()).map(|s| (s.width, s.height)).unwrap_or_default();
            with_player(|p| p.ensure_video(share, size));
        }
        slint::RenderingState::RenderingTeardown => with_player(|p| p.drop_video()),
        _ => {}
    });
    if let Err(e) = r {
        crate::local::log(&format!("player: no rendering notifier: {e:?}"));
    }
}

/// Opens `file` in the player. A file the core will not open, one still downloading, say,
/// leaves the page shut and says why on the series page's log.
pub fn open(file: u64) {
    OPENING.with(|o| o.set(o.get() + 1));
    let Some(window) = WINDOW.with(|w| w.borrow().clone()) else { return };
    let Some(core) = crate::local::core() else { return };
    if PLAYER.with(|p| p.borrow().is_some()) {
        close_page(CloseReason::Switched);
    }
    let session = match core.call(Call::OpenPlayback { file }) {
        Ok(Reply::Playback { session }) => *session,
        Ok(_) => return,
        Err(e) => {
            failed(&window, e.to_string());
            return;
        }
    };
    // The session carries the series and the file, not the episode number or the tracker
    // ids, so the detail is read once, as the desktop does.
    let (episode_number, tracked) = match core.call(Call::GetSeries { series: session.series }) {
        Ok(Reply::SeriesDetail { detail }) => (
            detail.episodes.iter().find(|e| e.file == file).map(|e| e.number),
            detail.card.match_info.as_ref().is_some_and(|m| m.anilist_id.is_some() || m.mal_id.is_some()),
        ),
        _ => (None, false),
    };
    begin(window, session, None, episode_number, tracked, None);
}

thread_local! {
    /// The episode numbers of the other device's files, by file, for the player's Mark
    /// watched. Set by the app whenever it reads that device's library.
    static REMOTE_NUMBERS: RefCell<std::collections::HashMap<u64, f64>> = RefCell::new(Default::default());
}

thread_local! {
    /// Moves on with every open and every close, so an episode of the other device that
    /// answers after the user has moved on is let go instead of opened over the page.
    static OPENING: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub fn set_remote_numbers(numbers: std::collections::HashMap<u64, f64>) {
    REMOTE_NUMBERS.with(|n| *n.borrow_mut() = numbers);
}

/// Opens `file` of the other device: the session opens there, off this thread, and the page
/// says it is connecting meanwhile.
pub fn open_remote(file: u64) {
    let Some(window) = WINDOW.with(|w| w.borrow().clone()) else { return };
    if PLAYER.with(|p| p.borrow().is_some()) {
        close_page(CloseReason::Switched);
    }
    failed(&window, String::new());
    data(&window, |d| d.set_title("Connecting".into()));
    let quality = crate::stream::quality();
    let unmetered = crate::android::unmetered().unwrap_or(false);
    let ticket = OPENING.with(|o| {
        o.set(o.get() + 1);
        o.get()
    });
    std::thread::spawn(move || {
        let opened = crate::stream::open(file, quality, unmetered);
        let _ = slint::invoke_from_event_loop(move || {
            let window = WINDOW.with(|w| w.borrow().clone());
            let waiting = OPENING.with(|o| o.get()) == ticket;
            let (Some(window), true) = (window, waiting) else {
                // Gone back, or opened something else, while this one connected.
                if let Ok(o) = opened {
                    o.host.send(Call::ClosePlayback { session: o.session.session, position: 0.0, reason: CloseReason::Stopped });
                }
                return;
            };
            match opened {
                Ok(o) => {
                    let number = REMOTE_NUMBERS.with(|n| n.borrow().get(&file).copied());
                    let note = o.converted.map(|h| format!("Streaming from {} at {h}p", o.host.name));
                    // The other device decides what its tracker takes; the button is offered
                    // for every episode, and a refusal says why.
                    begin(window, o.session, Some(o.host), number, number.is_some(), note);
                }
                Err(e) => failed(&window, e),
            }
        });
    });
}

/// Opens the page on a session, the phone's own or one on the other device.
fn begin(
    window: slint::Weak<AppWindow>,
    session: PlaybackSession,
    host: Option<Arc<crate::stream::Host>>,
    episode_number: Option<f64>,
    tracked: bool,
    note: Option<String>,
) {
    if PLAYER.with(|p| p.borrow().is_some()) {
        close_page(CloseReason::Switched);
    }
    let auto = match crate::local::core().map(|core| core.call(Call::GetSettings)) {
        Some(Ok(Reply::Settings { settings })) => settings.auto_skip,
        _ => AutoSkip::default(),
    };

    let listener = {
        let session_id = session.session;
        Box::new(move |event: MpvEvent| {
            let _ = slint::invoke_from_event_loop(move || {
                with_player(|p| {
                    if p.session.session == session_id {
                        p.on_mpv(event);
                    }
                })
            });
        })
    };
    let mpv = match Mpv::new(listener) {
        Ok(mpv) => mpv,
        Err(e) => {
            send(&host, Call::ClosePlayback { session: session.session, position: 0.0, reason: CloseReason::Stopped });
            failed(&window, e);
            return;
        }
    };

    let has_next = session.next.is_some() && !session.is_extra;
    let rules = Rules::new(session.skip_windows.clone(), auto, has_next);
    let can_mark = tracked && !session.is_extra && episode_number.is_some();
    data(&window, |d| {
        d.set_open(true);
        d.set_error("".into());
        d.set_has_frame(false);
        d.set_title(session.series_title.clone().into());
        d.set_code(session.code.clone().into());
        let episode = session
            .episode_title
            .clone()
            .or_else(|| (!session.is_extra).then(|| episode_number.map(|n| format!("Episode {n}"))).flatten())
            .unwrap_or_else(|| session.path.rsplit('/').next().unwrap_or_default().to_string());
        d.set_episode(episode.into());
        d.set_position(session.resume_from.unwrap_or(0.0) as f32);
        d.set_duration(0.0);
        d.set_paused(false);
        d.set_loaded(false);
        d.set_buffering(false);
        d.set_replay(false);
        d.set_has_prev(session.prev.is_some() && !session.is_extra);
        d.set_has_next(has_next);
        d.set_next_visible(false);
        d.set_next_counting(false);
        d.set_next_count_ms(NEXT_COUNT.as_millis() as i32);
        d.set_skip_label("".into());
        d.set_bands(ModelRc::default());
        d.set_audio_choice(false);
        d.set_subs_available(false);
        d.set_subs_on(false);
        d.set_can_mark(can_mark);
        d.set_speed(1.0);
        d.set_notice("".into());
        d.set_notice_action("".into());
        d.set_picker(0);
        d.set_chrome(true);
        d.set_preview(slint::Image::default());
    });
    crate::android::player_screen(true);

    let mut player = Player {
        window,
        session,
        host,
        mpv,
        video: None,
        rules,
        tracks: Vec::new(),
        aid: -1,
        sid: -1,
        requested: false,
        no_video: false,
        video_gen: 0,
        paused: false,
        seeking: false,
        scrubbing: false,
        thumbs: None,
        ended: false,
        closed: false,
        episode_number,
        sub_pos_setting: 100.0,
        sub_pos: 100.0,
        undo: None,
        tick: Timer::default(),
        hide: Timer::default(),
        next: Timer::default(),
        notice: Timer::default(),
        fallback: Timer::default(),
        cast: None,
        cast_on_load: PENDING_CAST.with(|c| c.borrow_mut().take()),
        track_json: String::new(),
        look: Timer::default(),
    };
    player.start();
    player.show_cast();
    if let Some(note) = note {
        player.flash(&note, None);
    }
    // The file waits for the renderer, but never forever: with no picture after three
    // seconds it plays on sound, and the page says why.
    let session_id = player.session.session;
    player.fallback.start(TimerMode::SingleShot, Duration::from_secs(3), move || {
        with_player(|p| {
            if p.session.session == session_id && !p.requested {
                crate::local::log("player: no renderer after 3 s, playing without a picture");
                p.video_started(false);
            }
        })
    });
    player.show_bands();
    player.show_chrome();
    PLAYER.with(|p| *p.borrow_mut() = Some(player));
    WINDOW.with(|w| {
        if let Some(w) = w.borrow().as_ref().and_then(|w| w.upgrade()) {
            w.window().request_redraw();
        }
    });
}

/// One call to wherever the session lives, its answer not waited for. A call to the other
/// device goes from its own thread, so the page never waits on the network.
fn send(host: &Option<Arc<crate::stream::Host>>, call: Call) {
    match host {
        Some(host) => host.send(call),
        None => {
            if let Some(core) = crate::local::core() {
                let _ = core.call(call);
            }
        }
    }
}

/// The page with only a reason on it, nothing of a previous episode left under it.
fn failed(window: &slint::Weak<AppWindow>, reason: String) {
    data(window, |d| {
        d.set_open(true);
        d.set_error(reason.into());
        d.set_loaded(false);
        d.set_has_frame(false);
        d.set_frame(Default::default());
        d.set_title("".into());
        d.set_episode("".into());
        d.set_code("".into());
        d.set_duration(0.0);
        d.set_position(0.0);
        d.set_skip_label("".into());
        d.set_next_visible(false);
        d.set_replay(false);
        d.set_audio_choice(false);
        d.set_subs_available(false);
        d.set_can_mark(false);
        d.set_has_prev(false);
        d.set_has_next(false);
        d.set_picker(0);
        d.set_chrome(true);
    });
    crate::android::player_screen(true);
}

/// Closes the page: the session with its reason, mpv, and the landscape hold. A switch to
/// another episode keeps the screen as it is, so auto-next does not turn the phone twice.
pub fn close_page(reason: CloseReason) {
    OPENING.with(|o| o.set(o.get() + 1));
    let switching = reason == CloseReason::Switched;
    let player = PLAYER.with(|p| p.borrow_mut().take());
    if let Some(mut p) = player {
        p.close(reason);
        // Letting go of MediaCodec and AudioTrack can take a while, so mpv goes on a thread
        // of its own; the renderer first, since its render context belongs to the core.
        let (video, mpv) = (p.video.take(), p.mpv.clone());
        drop(p);
        let _ = std::thread::Builder::new().name("mpv-close".into()).spawn(move || {
            drop(video);
            drop(mpv);
        });
    }
    if !switching {
        crate::android::player_awake(false);
        crate::android::player_screen(false);
    }
    WINDOW.with(|w| {
        if let Some(w) = w.borrow().as_ref() {
            data(w, |d| {
                d.set_open(false);
                d.set_has_frame(false);
                d.set_frame(Default::default());
            });
        }
    });
}

/// Next or previous: the same neighbours, and the same rule, as the desktop's buttons.
fn neighbour(forward: bool) {
    let mut target = None;
    let mut remote = false;
    with_player(|p| {
        remote = p.host.is_some();
        if !p.session.is_extra {
            target = if forward { p.session.next } else { p.session.prev };
        }
        // A cast goes on with the next episode on the same device.
        if target.is_some()
            && let Some(c) = &p.cast
        {
            PENDING_CAST.with(|pc| *pc.borrow_mut() = Some(c.device.clone()));
        }
    });
    if let Some(file) = target {
        close_page(CloseReason::Switched);
        if remote { open_remote(file) } else { open(file) }
    }
}

/// The core's playback events: AniSkip's windows and the tracker's answers.
pub fn core_event(body: &EventBody) {
    let body = match body {
        EventBody::SkipWindowsReady { .. }
        | EventBody::Marked { .. }
        | EventBody::Scored { .. }
        | EventBody::CastChanged { .. }
        | EventBody::CastDevicesChanged { .. } => body.clone(),
        _ => return,
    };
    if let EventBody::CastDevicesChanged { devices } = &body {
        let devices = devices.clone();
        let _ = slint::invoke_from_event_loop(move || {
            DEVICES.with(|d| *d.borrow_mut() = devices);
            with_player(|p| p.show_tracks());
        });
        return;
    }
    let _ = slint::invoke_from_event_loop(move || {
        with_player(|p| match body {
            // This phone's core speaks of its own sessions and series; one of the other
            // device's has ids of that device's, so only the cast's word is for it.
            EventBody::SkipWindowsReady { .. } | EventBody::Marked { .. } | EventBody::Scored { .. }
                if p.host.is_some() => {}
            EventBody::SkipWindowsReady { session, windows } if session == p.session.session => {
                p.rules.windows = windows;
                p.show_bands();
                p.show_skip();
            }
            EventBody::Marked { series, outcomes, .. } if series == p.session.series => {
                p.flash(&outcome_line(&outcomes), None);
            }
            EventBody::Scored { series, outcomes, .. } if series == p.session.series => {
                let ok = outcomes.iter().all(|o| o.ok);
                p.flash(if ok { "Rated" } else { "Could not save the score" }, None);
            }
            EventBody::CastChanged { state } => p.on_cast(state),
            _ => {}
        });
    });
}

impl Player {
    fn d(&self, f: impl FnOnce(&PlayerData)) {
        data(&self.window, f);
    }

    /// The owned options, the subtitle table and the resume point, which seeks before the
    /// first frame through the start option. The file itself follows the renderer.
    fn start(&mut self) {
        for (k, v) in options::owned_options() {
            // The video output is set before mpv starts and cannot change after.
            if k != "vo" {
                self.mpv.set(k, &v);
            }
        }
        self.apply_subtitle_options();
        self.mpv.set("volume", "100");
        let start = self.session.resume_from.map(|r| r.to_string()).unwrap_or_else(|| "none".into());
        self.mpv.set("start", &start);
        // The next episode of a cast plays on the TV, not here.
        if self.cast_on_load.is_some() {
            self.mpv.set("pause", "yes");
        }
        // The file loads once the renderer is up: see `video_started`.
    }

    fn apply_subtitle_options(&mut self) {
        for (k, v) in options::subtitle_options(&self.session.subtitle_defaults) {
            if k == "sub-pos" {
                self.sub_pos_setting = v.parse().unwrap_or(100.0);
            }
            self.mpv.set(k, &v);
        }
        self.lift_subtitles();
    }

    /// While the controls are up they cover the foot of the picture, where the subtitles
    /// are, so sub-pos is raised until the bottom line clears them, and put back after.
    fn lift_subtitles(&mut self) {
        let mut pos = self.sub_pos_setting;
        self.d(|d| {
            let (top, height) = (d.get_controls_top(), d.get_page_height());
            if d.get_chrome() && height > 0.0 && top > 0.0 {
                pos = pos.min((100.0 * (top - 12.0) / height).floor() as f64).max(0.0);
            }
        });
        if pos != self.sub_pos {
            self.sub_pos = pos;
            self.mpv.set("sub-pos", &pos.to_string());
        }
    }

    fn ensure_video(&mut self, share: Share, size: (u32, u32)) {
        if self.no_video {
            return;
        }
        if let Some(v) = &self.video {
            if v.share() == share {
                v.resize(size);
                return;
            }
            // Slint came back with a new context: the old textures are not shared with it.
            self.drop_video();
        }
        self.video_gen += 1;
        let generation = self.video_gen;
        let window = self.window.clone();
        let session = self.session.session;
        let on_frame = Box::new(move |frame: Frame| {
            let window = window.clone();
            let _ = slint::invoke_from_event_loop(move || {
                PLAYER.with(|p| {
                    let current = p.try_borrow().ok().and_then(|p| p.as_ref().map(|p| (p.session.session, p.video_gen)));
                    if current != Some((session, generation)) {
                        return;
                    }
                    let Some(texture) = NonZeroU32::new(frame.texture) else { return };
                    // Safety: the texture lives in Slint's share group until the renderer
                    // stops, and the renderer stops before the page lets go of the image.
                    let image = unsafe {
                        slint::BorrowedOpenGLTextureBuilder::new_gl_2d_rgba_texture(
                            texture,
                            (frame.width, frame.height).into(),
                        )
                        .origin(slint::BorrowedOpenGLTextureOrigin::TopLeft)
                        .build()
                    };
                    data(&window, |d| {
                        d.set_frame(image);
                        d.set_has_frame(true);
                    });
                });
            });
        });
        let session = self.session.session;
        let on_started = Box::new(move |r: Result<(), String>| {
            let _ = slint::invoke_from_event_loop(move || {
                with_player(|p| {
                    if p.session.session == session && p.video_gen == generation {
                        p.video_started(r.is_ok());
                    }
                })
            });
        });
        self.video = Video::start(&self.mpv, share, size, on_frame, on_started);
        if self.video.is_none() {
            // No thread for the renderer: the file plays without a picture rather than not
            // at all, and nothing tries again every frame.
            self.no_video = true;
            self.video_started(false);
        }
    }

    /// mpv opens its video output when a file starts, so the file waits for the render
    /// context; one already playing, when Slint came back with a new context, gets its
    /// video track chosen again, which opens the output on the new one.
    fn video_started(&mut self, ok: bool) {
        if !ok {
            self.d(|d| d.set_error("The picture could not start. The sound plays on.".into()));
        }
        if !self.requested {
            self.requested = true;
            self.mpv.command(&["loadfile", &self.session.path]);
        } else if ok && self.rules.loaded {
            let vid = self.mpv.get("vid").filter(|v| v != "no").unwrap_or_else(|| "auto".into());
            self.mpv.set("vid", "no");
            self.mpv.set("vid", &vid);
        }
    }

    fn drop_video(&mut self) {
        self.video_gen += 1;
        self.video = None;
        self.d(|d| {
            d.set_has_frame(false);
            d.set_frame(Default::default());
        });
    }

    fn on_mpv(&mut self, event: MpvEvent) {
        match event {
            MpvEvent::FileLoaded => self.loaded(),
            MpvEvent::EndFile { error: Some(e) } if !self.rules.loaded => {
                self.d(|d| d.set_error(format!("This file did not play: {e}.").into()));
            }
            MpvEvent::EndFile { .. } => {}
            MpvEvent::Property(name, value) => self.observed(name, value),
        }
    }

    fn loaded(&mut self) {
        self.rules.loaded = true;
        self.mpv.set("start", "none");
        let duration = self.mpv.get("duration").and_then(|d| d.parse().ok()).unwrap_or(0.0);
        self.rules.duration = duration;
        if duration > 0.0 {
            let chapters = self.mpv.get("chapter-list").map(|j| chapters(&j)).unwrap_or_default();
            send(&self.host, Call::ReportChapters { session: self.session.session, chapters, duration });
            self.ask_for_windows();
        }
        // Every sidecar the core found beside the file; mpv adds none of its own.
        for s in &self.session.sidecars {
            let title = s.title.clone().unwrap_or_default();
            let lang = s.language.clone().unwrap_or_default();
            self.mpv.command_now(&["sub-add", &s.path, "auto", &title, &lang]);
        }
        // The pick is made before the first frame, so the list is asked for here.
        self.tracks = self.mpv.get("track-list").map(|j| parse_tracks(&j)).unwrap_or_default();
        let pick = tracks::pick(&self.tracks, &self.session.track_choice, &self.session.subtitle_defaults);
        self.mpv.set("aid", &pick.aid.map(|a| a.to_string()).unwrap_or_else(|| "no".into()));
        self.mpv.set("sid", &pick.sid.map(|s| s.to_string()).unwrap_or_else(|| "no".into()));
        self.aid = pick.aid.unwrap_or(-1);
        self.sid = pick.sid.unwrap_or(-1);
        if let Some(j) = self.mpv.get("track-list") {
            self.track_json = j;
        }
        self.show_tracks();
        self.d(|d| {
            d.set_loaded(true);
            d.set_duration(duration as f32);
        });
        if let Some(dev) = self.cast_on_load.take() {
            self.start_cast(dev, None);
        }
        self.show_bands();
        // Once a second while playing; the pause, seek and close send their own.
        let weak = self.session.session;
        self.tick.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            with_player(|p| {
                if p.session.session == weak && !p.paused {
                    p.send_tick();
                }
            })
        });
        crate::android::player_awake(true);
    }

    fn observed(&mut self, name: &str, value: Value) {
        match (name, value) {
            ("time-pos", Value::Double(t)) => {
                self.d(|d| d.set_position(t as f32));
                for step in self.rules.moved(t) {
                    self.apply(step);
                }
                self.show_next();
                self.show_skip();
            }
            ("duration", Value::Double(t)) if t > 0.0 => {
                self.rules.duration = t;
                self.d(|d| d.set_duration(t as f32));
                self.show_bands();
            }
            ("pause", Value::Flag(p)) => {
                let was = self.paused;
                self.paused = p;
                self.d(|d| d.set_paused(p));
                if was != p {
                    self.send_tick();
                    crate::android::player_awake(!p && self.rules.loaded);
                }
                // A pause stops the countdown; playing back into the zone starts it again.
                if p && let Some(step) = self.rules.stop_count() {
                    self.apply(step);
                    self.show_next();
                }
            }
            ("seeking", Value::Flag(s)) => {
                let was = self.seeking;
                self.seeking = s;
                if was && !s {
                    self.send_tick();
                }
            }
            ("paused-for-cache", Value::Flag(b)) => self.d(|d| d.set_buffering(b)),
            // A seek back from the end leaves the last frame: Replay goes, Play works again.
            ("eof-reached", Value::Flag(false)) if self.ended => {
                self.ended = false;
                self.d(|d| d.set_replay(false));
            }
            ("eof-reached", Value::Flag(true)) if !self.ended => {
                self.ended = true;
                self.close(CloseReason::Ended);
                // The last frame stays up; Replay sits on it unless the countdown runs.
                let counting = self.rules.next_counting;
                self.d(|d| d.set_replay(!counting));
                crate::android::player_awake(false);
            }
            ("track-list", Value::Text(j)) => {
                self.tracks = parse_tracks(&j);
                self.track_json = j;
                self.show_tracks();
            }
            ("aid", v) => {
                self.aid = track_id(&v);
                self.show_tracks();
            }
            ("sid", v) => {
                self.sid = track_id(&v);
                self.show_tracks();
            }
            _ => {}
        }
    }

    fn apply(&mut self, step: Step) {
        match step {
            Step::Skip { to, undo, kind } => {
                self.mpv.command(&["seek", &to.to_string(), "absolute"]);
                let line = if kind == SkipKind::Intro { "Skipped intro" } else { "Skipped outro" };
                self.flash(line, Some(Undo::Skip { to: undo, kind }));
            }
            Step::StartCount => {
                self.next.start(TimerMode::SingleShot, NEXT_COUNT, || neighbour(true));
            }
            Step::StopCount => self.next.stop(),
        }
    }

    fn send_tick(&self) {
        if self.closed {
            return;
        }
        send(&self.host, Call::Tick { session: self.session.session, position: self.rules.position, paused: self.paused });
    }

    fn close(&mut self, reason: CloseReason) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.tick.stop();
        self.look.stop();
        crate::android::multicast(false);
        let carried = PENDING_CAST.with(|c| c.borrow().is_some());
        if self.cast.is_some()
            && !(reason == CloseReason::Switched && carried)
            && let Some(core) = crate::local::core()
        {
            let _ = core.call(Call::CastStop);
        }
        send(&self.host, Call::ClosePlayback { session: self.session.session, position: self.rules.position, reason });
        crate::local::touch();
    }

    fn toggle_pause(&mut self) {
        if let Some(c) = &self.cast {
            let call = if c.state == CastPlayState::Playing { Call::CastPause } else { Call::CastPlay };
            self.cast_call(call);
            self.show_chrome();
            return;
        }
        self.mpv.set("pause", if self.paused { "no" } else { "yes" });
        self.show_chrome();
    }

    /// The finger is on the bar at `t`: the bubble asks the preview core for that frame. The
    /// episode itself does not move until the release.
    fn scrub(&mut self, t: f64) {
        if self.cast.is_some() {
            return;
        }
        let t = if self.rules.duration > 0.0 { t.clamp(0.0, self.rules.duration) } else { t.max(0.0) };
        // A new drag drops the last one's frame, so the bubble never shows the wrong spot.
        if !self.scrubbing {
            self.d(|d| d.set_preview(slint::Image::default()));
        }
        self.scrubbing = true;
        // A conversion is made in order as it is watched; a second reader jumping about in
        // it would start it over each time, so a converted stream has no previews.
        let converted = self.host.is_some() && self.session.path.contains("/hls/");
        if self.thumbs.is_none() && !converted {
            let session_id = self.session.session;
            self.thumbs = Thumbs::open(&self.session.path, move |shot: Shot| {
                let _ = slint::invoke_from_event_loop(move || {
                    with_player(|p| {
                        if p.session.session == session_id {
                            p.show_preview(shot);
                        }
                    })
                });
            });
        }
        if let Some(thumbs) = &self.thumbs {
            thumbs.at(t);
        }
    }

    fn show_preview(&mut self, shot: Shot) {
        if !self.scrubbing {
            return;
        }
        let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(shot.width, shot.height);
        buf.make_mut_bytes().copy_from_slice(&shot.rgba);
        self.d(|d| d.set_preview(slint::Image::from_rgba8(buf)));
    }

    fn seek(&mut self, t: f64) {
        self.scrubbing = false;
        let t = if self.rules.duration > 0.0 { t.clamp(0.0, self.rules.duration) } else { t.max(0.0) };
        if self.cast.is_some() {
            self.cast_call(Call::CastSeek { to: t });
        } else {
            self.mpv.command(&["seek", &t.to_string(), "absolute"]);
        }
        // The next jump starts from here, not from a time-pos that has not caught up yet.
        self.rules.position = t;
        self.d(|d| d.set_position(t as f32));
        self.show_chrome();
    }

    fn skip(&mut self) {
        if let Some(w) = self.rules.current().cloned() {
            self.seek(w.end + 1.0);
        }
    }

    fn stay(&mut self) {
        self.rules.stay();
        self.next.stop();
        self.show_next();
    }

    fn show_chrome(&mut self) {
        self.d(|d| d.set_chrome(true));
        self.lift_subtitles();
        let session = self.session.session;
        self.hide.start(TimerMode::SingleShot, CHROME_FOR, move || {
            with_player(|p| {
                if p.session.session != session {
                    return;
                }
                // An open picker holds the chrome; it hides 2.5 s after the picker closes.
                let mut held = false;
                p.d(|d| held = d.get_picker() != 0);
                if held {
                    p.show_chrome();
                    return;
                }
                p.d(|d| d.set_chrome(false));
                p.lift_subtitles();
            })
        });
    }

    fn show_skip(&self) {
        let label = match self.rules.current().map(|w| &w.kind) {
            Some(SkipKind::Intro) => "Skip intro",
            Some(SkipKind::Outro) => "Skip outro",
            None => "",
        };
        self.d(|d| {
            if d.get_skip_label() != label {
                d.set_skip_label(label.into());
            }
        });
    }

    fn show_next(&self) {
        let (visible, counting) = (self.rules.next_visible, self.rules.next_counting);
        self.d(|d| {
            d.set_next_visible(visible);
            d.set_next_counting(counting);
        });
    }

    fn show_bands(&self) {
        let duration = self.rules.duration;
        if duration <= 0.0 {
            return;
        }
        let bands: Vec<Band> = self
            .rules
            .windows
            .iter()
            .map(|w| Band {
                start: (w.start / duration).clamp(0.0, 1.0) as f32,
                end: (w.end / duration).clamp(0.0, 1.0) as f32,
                intro: w.kind == SkipKind::Intro,
            })
            .collect();
        self.d(|d| d.set_bands(ModelRc::new(VecModel::from(bands))));
    }

    fn show_tracks(&self) {
        let audio = self.tracks.iter().filter(|t| t.kind == "audio").count();
        let subs = self.tracks.iter().filter(|t| t.kind == "sub").count();
        let on = self.sid >= 0;
        self.d(|d| {
            d.set_audio_choice(audio > 1);
            d.set_subs_available(subs > 0);
            d.set_subs_on(on);
        });
        // A picker that is open follows the list.
        let mut kind = 0;
        self.d(|d| kind = d.get_picker());
        if kind != 0 {
            self.fill_picker(kind);
        }
    }

    fn picker_rows(&self, kind: i32) -> (String, Vec<PickRow>) {
        match kind {
            1 => (
                "Audio".into(),
                self.tracks
                    .iter()
                    .filter(|t| t.kind == "audio")
                    .map(|t| PickRow { label: tracks::label(t).into(), active: t.id == self.aid })
                    .collect(),
            ),
            2 => {
                let mut rows = vec![PickRow { label: "Off".into(), active: self.sid < 0 }];
                rows.extend(
                    self.tracks
                        .iter()
                        .filter(|t| t.kind == "sub")
                        .map(|t| PickRow { label: tracks::label(t).into(), active: t.id == self.sid }),
                );
                ("Subtitles".into(), rows)
            }
            4 => {
                let mut rows = Vec::new();
                if let Some(c) = &self.cast {
                    rows.push(PickRow { label: format!("Stop casting to {}", c.device.name).into(), active: false });
                }
                let devices = DEVICES.with(|d| d.borrow().clone());
                for dev in &devices {
                    let kind = match dev.kind {
                        CastKind::Chromecast => dev.model.clone().unwrap_or_else(|| "Chromecast".into()),
                        CastKind::Dlna => "TV".into(),
                    };
                    let active = self.cast.as_ref().is_some_and(|c| c.device.id == dev.id);
                    rows.push(PickRow { label: format!("{}   {kind}", dev.name).into(), active });
                }
                let title = if devices.is_empty() { "Looking for Chromecasts and TVs" } else { "Cast to" };
                (title.into(), rows)
            }
            _ => {
                let mut speed = 1.0;
                self.d(|d| speed = d.get_speed() as f64);
                (
                    "Speed".into(),
                    SPEEDS
                        .iter()
                        .map(|s| PickRow { label: format!("{s}×").into(), active: (s - speed).abs() < 0.01 })
                        .collect(),
                )
            }
        }
    }

    fn fill_picker(&self, kind: i32) {
        let (title, rows) = self.picker_rows(kind);
        self.d(|d| {
            d.set_picker_title(title.into());
            let model = d.get_picker_rows();
            if model.row_count() == rows.len() {
                for (i, r) in rows.into_iter().enumerate() {
                    if model.row_data(i).as_ref() != Some(&r) {
                        model.set_row_data(i, r);
                    }
                }
            } else {
                d.set_picker_rows(ModelRc::new(VecModel::from(rows)));
            }
        });
    }

    fn open_picker(&mut self, kind: i32) {
        if kind == 4 {
            self.look_for_devices();
        }
        self.d(|d| d.set_picker_rows(ModelRc::default()));
        self.fill_picker(kind);
        self.d(|d| d.set_picker(kind));
        self.show_chrome();
    }

    fn close_picker(&mut self) {
        self.d(|d| d.set_picker(0));
        self.show_chrome();
    }

    /// A row of the open picker. The choice of track is stored for the series, as on the
    /// desktop; the speed lasts for this file only.
    fn pick(&mut self, i: i32) {
        let mut kind = 0;
        self.d(|d| kind = d.get_picker());
        let i = i.max(0) as usize;
        match kind {
            1 => {
                let Some(t) = self.tracks.iter().filter(|t| t.kind == "audio").nth(i).cloned() else { return };
                self.mpv.set("aid", &t.id.to_string());
                let r = tracks::track_ref(&t);
                self.session.track_choice.audio = Some(r.clone());
                self.store_choice();
                self.aid = t.id;
                self.recast();
            }
            2 => {
                if i == 0 {
                    self.mpv.set("sid", "no");
                    self.session.track_choice.subtitle = Some(SubtitleChoice::Off);
                } else {
                    let Some(t) = self.tracks.iter().filter(|t| t.kind == "sub").nth(i - 1).cloned() else { return };
                    self.mpv.set("sid", &t.id.to_string());
                    self.session.track_choice.subtitle = Some(SubtitleChoice::Track { track: tracks::track_ref(&t) });
                    self.sid = t.id;
                }
                if i == 0 {
                    self.sid = -1;
                }
                self.store_choice();
                self.recast();
            }
            4 => {
                let offset = usize::from(self.cast.is_some());
                if self.cast.is_some() && i == 0 {
                    self.cast_call(Call::CastStop);
                } else if let Some(dev) = DEVICES.with(|d| d.borrow().get(i - offset).cloned()) {
                    self.start_cast(dev, None);
                }
            }
            3 => {
                let Some(s) = SPEEDS.get(i) else { return };
                self.mpv.set("speed", &s.to_string());
                self.d(|d| d.set_speed(*s as f32));
            }
            _ => {}
        }
        self.close_picker();
    }

    /// Looks around the network while the Cast to sheet is up, holding Wi-Fi multicast open
    /// for that long.
    fn look_for_devices(&mut self) {
        crate::android::multicast(true);
        if let Some(core) = crate::local::core()
            && let Ok(Reply::CastDevices { devices }) = core.call(Call::CastDevices)
        {
            DEVICES.with(|d| *d.borrow_mut() = devices);
        }
        self.look.start(TimerMode::SingleShot, LOOK_FOR, || crate::android::multicast(false));
    }

    /// The Matroska track numbers of the audio and subtitles playing here, from mpv's list:
    /// an embedded track's number is its ffmpeg index plus one; a subtitle file beside the
    /// episode goes by its path.
    fn cast_tracks(&self) -> (Option<u64>, CastSubtitle) {
        let list: serde_json::Value = serde_json::from_str(&self.track_json).unwrap_or_default();
        let entry = |id: i64, kind: &str| {
            list.as_array().and_then(|a| a.iter().find(|t| t["id"].as_i64() == Some(id) && t["type"] == kind)).cloned()
        };
        let audio = entry(self.aid, "audio").and_then(|t| t["ff-index"].as_u64()).map(|i| i + 1);
        let subtitle = match entry(self.sid, "sub") {
            Some(t) if t["external"].as_bool() == Some(true) => match t["external-filename"].as_str() {
                Some(path) => CastSubtitle::File { path: path.to_string() },
                None => CastSubtitle::None,
            },
            Some(t) => t["ff-index"].as_u64().map(|i| CastSubtitle::Track { number: i + 1 }).unwrap_or(CastSubtitle::None),
            None => CastSubtitle::None,
        };
        (audio, subtitle)
    }

    /// Hands the episode to `device` from where it is now, with the tracks chosen here.
    fn start_cast(&mut self, device: CastDevice, at: Option<f64>) {
        let Some(core) = crate::local::core() else { return };
        let (audio_track, subtitle) = self.cast_tracks();
        self.mpv.set("pause", "yes");
        let start = at.unwrap_or(self.rules.position);
        self.cast = Some(Casting { device: device.clone(), state: CastPlayState::Connecting });
        self.show_cast();
        let call = match &self.host {
            // The other device converts it for the TV, subtitles drawn in, and the phone
            // passes it on, since the TV has no way onto the tailnet.
            Some(host) => {
                let (audio, burn) = self.remote_picks();
                Call::CastStartStream {
                    device: device.id.clone(),
                    url: host.url(&anibeam_companion::api::hls_path(self.session.file, 1080, audio, burn)),
                    title: self.session.series_title.clone(),
                    start,
                    duration: self.rules.duration,
                }
            }
            None => Call::CastStart { device: device.id.clone(), file: self.session.file, start, audio_track, subtitle },
        };
        if let Err(e) = core.call(call) {
            self.cast = None;
            self.show_cast();
            self.flash(&e.to_string(), None);
        }
        crate::local::touch();
    }

    /// A session on the other device finds its skip windows there, a few seconds after the
    /// chapters are reported; this asks for them a few times, since its event never comes
    /// here.
    fn ask_for_windows(&self) {
        let Some(host) = self.host.clone() else { return };
        if !self.rules.windows.is_empty() {
            return;
        }
        let session = self.session.session;
        std::thread::spawn(move || {
            for _ in 0..6 {
                std::thread::sleep(Duration::from_secs(3));
                let Some(windows) = host.skip_windows(session).filter(|w| !w.is_empty()) else { continue };
                let _ = slint::invoke_from_event_loop(move || {
                    with_player(|p| {
                        if p.session.session == session && p.host.is_some() {
                            p.rules.windows = windows;
                            p.show_bands();
                            p.show_skip();
                        }
                    })
                });
                return;
            }
        });
    }

    /// The audio and subtitle tracks picked here, as indexes into the other device's probe
    /// of the file. The file's own tracks count from 1 in mpv; a converted stream's
    /// subtitles are the file's tracks loaded one by one, in order.
    fn remote_picks(&self) -> (Option<usize>, Option<usize>) {
        let list: serde_json::Value = serde_json::from_str(&self.track_json).unwrap_or_default();
        let entry = |id: i64, kind: &str| {
            list.as_array().and_then(|a| a.iter().find(|t| t["id"].as_i64() == Some(id) && t["type"] == kind)).cloned()
        };
        let converted = self.session.path.contains("/hls/");
        // A conversion has the one audio track it was asked for, named in its address.
        let asked = self.session.path.split(['?', '&']).find_map(|p| p.strip_prefix("audio=")?.parse().ok());
        let audio = match entry(self.aid, "audio") {
            _ if converted => asked,
            Some(t) if t["external"].as_bool() != Some(true) => usize::try_from(self.aid - 1).ok(),
            _ => None,
        };
        let burn = entry(self.sid, "sub").and_then(|t| {
            if t["external"].as_bool() == Some(true) {
                let name = t["external-filename"].as_str()?;
                name.rsplit_once("/subtitles/").and_then(|(_, i)| i.parse().ok())
            } else {
                usize::try_from(self.sid - 1).ok()
            }
        });
        (audio, burn)
    }

    /// A new audio or subtitle choice while casting starts the cast again with it.
    fn recast(&mut self) {
        if let Some(c) = &self.cast {
            let dev = c.device.clone();
            self.start_cast(dev, None);
        }
    }

    fn cast_call(&mut self, call: Call) {
        if let Some(core) = crate::local::core()
            && let Err(e) = core.call(call)
        {
            self.flash(&e.to_string(), None);
        }
    }

    /// What the device said: its position drives the page and the ticks the same as local
    /// playback would; an end, a failure or a stop hands the episode back to the phone.
    fn on_cast(&mut self, st: CastState) {
        let Some(c) = self.cast.as_mut() else { return };
        // A relayed stream is the other device's file, which this phone has no number for.
        if st.file != self.session.file && !(self.host.is_some() && st.file == 0) {
            return;
        }
        c.state = st.state;
        match st.state {
            CastPlayState::Playing | CastPlayState::Paused | CastPlayState::Buffering => {
                if st.duration > 0.0 {
                    self.rules.duration = st.duration;
                    self.d(|d| d.set_duration(st.duration as f32));
                }
                let was = self.paused;
                self.paused = st.state != CastPlayState::Playing;
                let paused = self.paused;
                self.d(|d| d.set_paused(paused));
                if was != paused {
                    self.send_tick();
                }
                self.d(|d| d.set_position(st.position as f32));
                for step in self.rules.moved(st.position) {
                    self.apply(step);
                }
                self.show_next();
                self.show_skip();
            }
            CastPlayState::Ended => {
                self.rules.position = st.position.max(self.rules.position);
                if self.rules.has_next && !self.rules.next_dismissed {
                    let dev = c.device.clone();
                    PENDING_CAST.with(|pc| *pc.borrow_mut() = Some(dev));
                    let _ = slint::invoke_from_event_loop(|| neighbour(true));
                } else {
                    self.close(CloseReason::Ended);
                    self.d(|d| d.set_replay(true));
                }
            }
            CastPlayState::Failed | CastPlayState::Stopped => {
                if let Some(e) = &st.error {
                    self.flash(e, None);
                }
                self.cast = None;
                // The phone takes over where the TV was, paused.
                let at = st.position.max(0.0);
                if at > 0.0 {
                    self.mpv.command(&["seek", &at.to_string(), "absolute"]);
                    self.rules.position = at;
                }
                self.paused = true;
                self.d(|d| d.set_paused(true));
            }
            CastPlayState::Connecting => {}
        }
        self.show_cast();
    }

    fn show_cast(&self) {
        let (casting, device, line) = match &self.cast {
            Some(c) => (
                true,
                c.device.name.clone(),
                match c.state {
                    CastPlayState::Connecting => "Connecting",
                    CastPlayState::Buffering => "Loading",
                    CastPlayState::Playing => "Playing",
                    CastPlayState::Paused => "Paused",
                    CastPlayState::Ended => "Finished",
                    CastPlayState::Failed => "Stopped",
                    CastPlayState::Stopped => "Stopped",
                }
                .to_string(),
            ),
            None => (false, String::new(), String::new()),
        };
        self.d(|d| {
            d.set_casting(casting);
            d.set_cast_device(device.into());
            d.set_cast_line(line.into());
        });
    }

    fn store_choice(&self) {
        let c = &self.session.track_choice;
        send(&self.host, Call::SetTrackChoice { series: self.session.series, audio: c.audio.clone(), subtitle: c.subtitle.clone() });
    }

    fn mark(&mut self) {
        let Some(n) = self.episode_number else { return };
        let call = Call::MarkEpisode { series: self.session.series, episode: n };
        let refused = |e: &anibeam_core::CoreError| match e {
            anibeam_core::CoreError::Refused { reason } => format!("Not tracked: {}.", refusal_word(reason)),
            other => other.to_string(),
        };
        match &self.host {
            // The other device's tracker answers there, so its word comes back here.
            Some(host) => {
                let (host, session) = (host.clone(), self.session.session);
                std::thread::spawn(move || {
                    let line = match host.call(&call) {
                        Ok(_) => "Marked watched".to_string(),
                        Err(e) => refused(&e),
                    };
                    let _ = slint::invoke_from_event_loop(move || {
                        with_player(|p| {
                            if p.session.session == session {
                                p.flash(&line, None);
                            }
                        })
                    });
                });
            }
            None => {
                if let Some(core) = crate::local::core()
                    && let Err(e) = core.call(call)
                {
                    self.flash(&refused(&e), None);
                }
            }
        }
        self.show_chrome();
    }

    fn flash(&mut self, line: &str, undo: Option<Undo>) {
        let action = if undo.is_some() { "Undo" } else { "" };
        self.undo = undo;
        self.d(|d| {
            d.set_notice(line.into());
            d.set_notice_action(action.into());
        });
        let session = self.session.session;
        self.notice.start(TimerMode::SingleShot, Duration::from_secs(4), move || {
            with_player(|p| {
                if p.session.session == session {
                    p.undo = None;
                    p.d(|d| d.set_notice("".into()));
                }
            })
        });
    }

    fn notice_clicked(&mut self) {
        if let Some(Undo::Skip { to, kind }) = self.undo.take() {
            self.rules.disarm(&kind);
            self.seek(to);
        }
        self.notice.stop();
        self.d(|d| d.set_notice("".into()));
    }
}

fn parse_tracks(json: &str) -> Vec<tracks::Track> {
    serde_json::from_str(json).map(|v| tracks::parse(&v)).unwrap_or_default()
}

fn track_id(v: &Value) -> i64 {
    match v {
        Value::Text(t) => t.parse().unwrap_or(-1),
        _ => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_core::SkipSource;

    fn w(kind: SkipKind, start: f64, end: f64) -> SkipWindow {
        SkipWindow { kind, start, end, source: SkipSource::Chapters }
    }

    fn rules(windows: Vec<SkipWindow>, intro: bool) -> Rules {
        let mut r = Rules::new(windows, AutoSkip { intro, outro: false }, true);
        r.loaded = true;
        r.duration = 1420.0;
        r
    }

    #[test]
    fn auto_skip_fires_once_on_playing_into_the_intro() {
        let mut r = rules(vec![w(SkipKind::Intro, 60.0, 150.0)], true);
        assert!(r.moved(59.5).is_empty());
        assert_eq!(r.moved(60.2), vec![Step::Skip { to: 151.0, undo: 60.0, kind: SkipKind::Intro }]);
        // Undo seeks back to the start; it is disarmed, so playing on does not skip again.
        assert!(r.moved(60.0).is_empty());
        assert!(r.moved(61.0).is_empty());
    }

    #[test]
    fn a_seek_into_a_window_does_not_skip_out_of_it() {
        let mut r = rules(vec![w(SkipKind::Intro, 60.0, 150.0)], true);
        r.moved(10.0);
        assert!(r.moved(100.0).is_empty(), "the seek landed inside");
        assert!(r.moved(100.5).is_empty(), "and the frames after it stay");
    }

    #[test]
    fn auto_skip_off_leaves_the_chip_only() {
        let mut r = rules(vec![w(SkipKind::Intro, 60.0, 150.0)], false);
        r.moved(59.8);
        assert!(r.moved(60.3).is_empty());
        assert_eq!(r.current().map(|w| w.kind.clone()), Some(SkipKind::Intro));
    }

    #[test]
    fn the_next_pill_rises_at_the_outro_and_counts_at_its_end() {
        let mut r = rules(vec![w(SkipKind::Outro, 1300.0, 1410.0)], false);
        r.moved(1290.0);
        assert!(!r.next_visible);
        r.moved(1300.5);
        assert!(r.next_visible && !r.next_counting);
        assert_eq!(r.moved(1410.2), vec![Step::StartCount]);
        // A seek back out of the zone stops the countdown.
        assert_eq!(r.moved(1200.0), vec![Step::StopCount]);
        assert!(!r.next_visible);
    }

    #[test]
    fn with_no_outro_the_pill_rises_eight_seconds_from_the_end() {
        let mut r = rules(vec![], false);
        r.moved(1411.0);
        assert!(!r.next_visible);
        r.moved(1412.5);
        assert!(r.next_visible && !r.next_counting);
        assert_eq!(r.moved(1417.5), vec![Step::StartCount]);
        r.stay();
        assert!(r.moved(1418.0).is_empty());
        assert!(!r.next_visible);
    }

    #[test]
    fn outcome_lines_read_like_the_desktops() {
        let ok = TrackerOutcome { tracker: Tracker::Anilist, ok: true, progress: Some(12), reason: None, message: None };
        let refused = TrackerOutcome {
            tracker: Tracker::Mal,
            ok: false,
            progress: None,
            reason: Some(anibeam_core::Refusal::NotNewer),
            message: None,
        };
        assert_eq!(outcome_line(&[ok.clone()]), "Tracked: AniList at 12");
        assert_eq!(
            outcome_line(&[ok, refused.clone()]),
            "Tracked: AniList at 12; MyAnimeList: the tracker is already past this episode"
        );
        assert_eq!(outcome_line(&[refused]), "Not tracked: MyAnimeList: the tracker is already past this episode");
    }

    #[test]
    fn chapters_come_from_mpvs_list() {
        let c = chapters(r#"[{"title":"Opening","time":0.0},{"title":"Part A","time":90.5},{"time":1300}]"#);
        assert_eq!(c.len(), 3);
        assert_eq!(c[1], Chapter { title: "Part A".into(), start: 90.5 });
        assert_eq!(c[2].title, "");
        assert!(chapters("not json").is_empty());
    }
}
