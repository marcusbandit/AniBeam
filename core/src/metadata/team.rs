//! The metadata team (#854): matching, airing, franchises, Recommended,
//! Airing now and the calendar feed.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::aired::Aired;
use crate::calendar::Calendar;
use crate::contract::{EventBody, Stage};
use crate::downloads::Downloads;
use crate::foundation::above::Above;
use crate::foundation::services::Services;
use crate::library::Library;
use crate::metadata::OUTAGE_WINDOW;
use crate::net::Net;
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
    /// When the core last said out loud that Jikan was not answering.
    /// Jikan is the episode-title side-fetch, so an outage costs a series
    /// its titles rather than its match, and a job walking a whole library
    /// through one must not write a warning per series into the log.
    jikan_outage: Mutex<Option<Instant>>,
    /// When each franchise root was last crawled on a page's behalf. A
    /// series page asks for its franchise every time it opens, and the
    /// crawl behind that read runs at most once a minute per root, so
    /// opening the same page twice costs AniList nothing the second time.
    pub(crate) crawl_recent: Mutex<HashMap<u64, Instant>>,
    /// The followed shows' next episodes, armed again when a schedule moves.
    pub(crate) downloads: Above<Downloads>,
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
            downloads: Above::default(),
        }
    }

    /// Warns that Jikan is not answering, at most once every ten minutes.
    /// Every caller reports every failure; this is what decides which of
    /// them the user is told about.
    pub(crate) fn report_jikan_outage(&self, message: &str) {
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
