//! The metadata team (#854): matching, airing, franchises, Recommended,
//! Airing now and the calendar feed.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::aired::Aired;
use crate::calendar::Calendar;
use crate::contract::{EventBody, Stage};
use crate::events::Signal;
use crate::foundation::services::Services;
use crate::library::Library;
use crate::metadata::OUTAGE_WINDOW;
use crate::metadata::automatch;
use crate::metadata::fetch::message_of;
use crate::net::Net;
use crate::net::jikan::JikanEpisode;
use crate::sync::recover;

pub struct Metadata {
    pub(crate) services: Arc<Services>,
    pub(crate) net: Arc<Net>,
    pub(crate) library: Arc<Library>,
    /// The calendar feed's listener and file lock (`calendar`).
    pub(crate) calendar: Calendar,
    pub(crate) aired: Aired,
    /// Airing now's read in flight per season, so a second ask for the same
    /// season joins it and the other season's read runs beside it.
    pub(crate) airing_reads: Mutex<HashMap<String, u64>>,
    /// The show page reads of the last few minutes, by AniList id: the job
    /// and when it started. A page opened twice joins the first opening's
    /// read, and a read that failed waits before it is tried again (#779).
    pub(crate) show_reads: Mutex<HashMap<u64, (u64, i64)>>,
    /// The least time, in milliseconds, between two of the recommender's
    /// requests to AniList, on top of the shared limiter: its first read is
    /// some seventy requests, and AniList's limit while degraded is thirty
    /// a minute (#732). A test shortens it.
    pub(crate) recommender_gap_ms: AtomicU64,
    /// When Jikan last failed and the core said so out loud. Jikan is the
    /// episode-title side-fetch, so an outage costs a series its titles
    /// rather than its match, and a job walking a whole library through
    /// one must neither write a warning per series into the log nor wait
    /// on Jikan once per series: inside the window it is not asked at all.
    jikan_outage: Mutex<Option<Instant>>,
    /// When each franchise root was last crawled on a page's behalf. A
    /// series page asks for its franchise every time it opens, and the
    /// crawl behind that read runs at most once a minute per root, so
    /// opening the same page twice costs AniList nothing the second time.
    pub(crate) crawl_recent: Mutex<HashMap<u64, Instant>>,
}

impl Metadata {
    pub(crate) fn new(services: Arc<Services>, net: Arc<Net>, library: Arc<Library>) -> Metadata {
        Metadata {
            services,
            net,
            library,
            calendar: Calendar::default(),
            aired: Aired::default(),
            airing_reads: Mutex::new(HashMap::new()),
            show_reads: Mutex::new(HashMap::new()),
            recommender_gap_ms: AtomicU64::new(crate::recommended::GAP_MS),
            jikan_outage: Mutex::new(None),
            crawl_recent: Mutex::new(HashMap::new()),
        }
    }

    /// Hears what the teams below tell this one (#854): a settled folder is
    /// worth a match, and a tracker list read again is what Recommended
    /// learns from. Set once, by `Core::open`.
    pub(crate) fn listen(self: &Arc<Self>) {
        let metadata = Arc::downgrade(self);
        self.services.bus.on_signal(Arc::new(move |signal| {
            let Some(metadata) = metadata.upgrade() else {
                return;
            };
            match signal {
                Signal::FolderSettled => {
                    automatch::start(&metadata);
                }
                Signal::TrackersChanged => crate::recommended::after_tracker_refresh(&metadata),
                Signal::SchedulesMoved => {}
            }
        }));
    }

    /// Jikan's episodes for `mal_id`, or `None` when Jikan has nothing to
    /// say: it failed just now, or it failed inside the last
    /// `OUTAGE_WINDOW` and is not asked again until the window has passed.
    /// A caller reads `None` as a Jikan half it does not have, which is not
    /// the same as an empty list.
    pub(crate) async fn jikan_episodes(&self, mal_id: u64) -> Option<Vec<JikanEpisode>> {
        if self.jikan_down() {
            return None;
        }
        match self.net.jikan.episodes(mal_id).await {
            Ok(episodes) => Some(episodes),
            Err(e) => {
                self.report_jikan_outage(&message_of(&e));
                None
            }
        }
    }

    /// Whether Jikan failed inside the outage window.
    fn jikan_down(&self) -> bool {
        recover(&self.jikan_outage).is_some_and(|at| at.elapsed() < OUTAGE_WINDOW)
    }

    /// Warns that Jikan is not answering, at most once every ten minutes,
    /// and starts the window in which it is not asked. A request already in
    /// flight when the window opened can still fail inside it; this is what
    /// keeps that failure out of the log.
    fn report_jikan_outage(&self, message: &str) {
        let mut last = recover(&self.jikan_outage);
        if last.is_some_and(|at| at.elapsed() < OUTAGE_WINDOW) {
            return;
        }
        *last = Some(Instant::now());
        // Dropped before the emit: a listener runs on this thread, and
        // nothing it does should be able to deadlock on this gate.
        drop(last);
        self.services.bus.warn(
            Stage::Metadata,
            format!("Jikan is not answering: {message}"),
            EventBody::Notice,
        );
    }
}
