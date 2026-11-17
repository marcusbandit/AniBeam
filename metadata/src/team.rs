//! The metadata team (#854): matching, airing, franchises, Recommended,
//! Airing now and the calendar feed.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use crate::OUTAGE_WINDOW;
use crate::aired::Aired;
use crate::automatch;
use crate::calendar::Calendar;
use crate::fetch::message_of;
use crate::owed;
use anibeam_contract::{CoreError, EventBody, Stage};
use anibeam_foundation::events::Signal;
use anibeam_foundation::services::Services;
use anibeam_foundation::sync::recover;
use anibeam_library::Library;
use anibeam_net::Net;
use anibeam_net::jikan::{JIKAN_MAX_PAGES, JikanCap, JikanEpisodes};

pub struct Metadata {
    pub services: Arc<Services>,
    pub net: Arc<Net>,
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
    pub recommender_gap_ms: AtomicU64,
    /// When Jikan last failed and the core said so out loud. Jikan is the
    /// episode-title side-fetch, so an outage costs a series its titles
    /// rather than its match, and a job walking a whole library through
    /// one must neither write a warning per series into the log nor wait
    /// on Jikan once per series: inside the window it is not asked at all.
    jikan_outage: Mutex<Option<Instant>>,
    /// How long, in milliseconds, Jikan is left alone after it failed:
    /// `OUTAGE_WINDOW`, which a test shortens.
    pub jikan_window_ms: AtomicU64,
    /// Whether the walk over the titles an outage still owes is running
    /// (`owed`), so a launch and a reply after an outage start one, not two.
    pub(crate) owed_walking: AtomicBool,
    /// This team, for the reply that ends an outage to start that walk
    /// from. Set once, by `listen`.
    this: OnceLock<Weak<Metadata>>,
    /// When each franchise root was last crawled on a page's behalf. A
    /// series page asks for its franchise every time it opens, and the
    /// crawl behind that read runs at most once a minute per root, so
    /// opening the same page twice costs AniList nothing the second time.
    pub(crate) crawl_recent: Mutex<HashMap<u64, Instant>>,
}

impl Metadata {
    pub fn new(services: Arc<Services>, net: Arc<Net>, library: Arc<Library>) -> Metadata {
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
            jikan_window_ms: AtomicU64::new(OUTAGE_WINDOW.as_millis() as u64),
            owed_walking: AtomicBool::new(false),
            this: OnceLock::new(),
            crawl_recent: Mutex::new(HashMap::new()),
        }
    }

    /// Hears what the teams below tell this one (#854): a settled folder is
    /// worth a match, and a tracker list read again is what Recommended
    /// learns from. Set once, by `Core::open`.
    pub fn listen(self: &Arc<Self>) {
        let _ = self.this.set(Arc::downgrade(self));
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

    /// Jikan's episodes for `mal_id`. The list is `whole` only when Jikan
    /// answered every page; it is empty and not whole when Jikan has
    /// nothing to say: it failed just now, or it went down inside the last
    /// `OUTAGE_WINDOW` and is not asked again until the window has passed.
    /// A page after the first that fails leaves the pages before it, still
    /// not whole (#1040). A caller reads a list that is not whole as part
    /// of a Jikan half, which is not the same as all of it, and deletes
    /// nothing on its word. Only a failure that looks like Jikan itself
    /// being down opens the window; one series' bad answer costs that
    /// series its titles and nothing else.
    ///
    /// The failure that cut the read short, on the first page or a later
    /// one, is left in `stopped`, which is how `owes_titles` tells an
    /// outage from one series' bad answer. The first whole reply after the
    /// window has passed is Jikan back, and starts the walk over the
    /// titles the outage still owes (#1065).
    pub(crate) async fn jikan_episodes(&self, mal_id: u64) -> JikanEpisodes {
        self.jikan_episodes_from(mal_id, 1).await
    }

    /// `jikan_episodes` from the page holding episode `from` on, for a
    /// caller that holds everything before it already (#1105). A read that
    /// stopped at the page cap or its budget says so in one log line.
    pub(crate) async fn jikan_episodes_from(&self, mal_id: u64, from: u32) -> JikanEpisodes {
        if self.jikan_down() {
            return JikanEpisodes::default();
        }
        let read = match self.net.jikan.episodes_from(mal_id, from).await {
            Ok(read) => read,
            Err(e) => JikanEpisodes {
                stopped: Some(e),
                ..JikanEpisodes::default()
            },
        };
        match &read.stopped {
            Some(e) if is_outage(e) => self.report_jikan_outage(&message_of(e)),
            Some(_) => {}
            None if read.whole => self.jikan_back(),
            None => {}
        }
        if let Some(cap) = read.capped {
            let why = match cap {
                JikanCap::Pages => format!("at {JIKAN_MAX_PAGES} pages"),
                JikanCap::Budget => "at its time limit".to_string(),
            };
            self.services.bus.info(
                Stage::Metadata,
                format!(
                    "Jikan's episode list for MAL {mal_id} stopped {why}, {} episodes read",
                    read.episodes.len()
                ),
                EventBody::Notice,
            );
        }
        read
    }

    /// Whether Jikan failed inside the outage window.
    pub(crate) fn jikan_down(&self) -> bool {
        let window = self.jikan_window();
        recover(&self.jikan_outage).is_some_and(|at| at.elapsed() < window)
    }

    fn jikan_window(&self) -> Duration {
        Duration::from_millis(self.jikan_window_ms.load(Ordering::SeqCst))
    }

    /// A whole reply: an outage whose window has passed is over, and what
    /// it left owed is asked for again. The window is closed only here, so
    /// a reply from a request that was already in flight when Jikan went
    /// down neither ends the window nor starts a walk.
    fn jikan_back(&self) {
        let window = self.jikan_window();
        let ended = {
            let mut last = recover(&self.jikan_outage);
            let ended = last.is_some_and(|at| at.elapsed() >= window);
            if ended {
                *last = None;
            }
            ended
        };
        if ended && let Some(this) = self.this.get().and_then(Weak::upgrade) {
            owed::start(&this);
        }
    }

    /// Warns that Jikan is not answering, at most once every ten minutes,
    /// and starts the window in which it is not asked. A request already in
    /// flight when the window opened can still fail inside it; this is what
    /// keeps that failure out of the log.
    fn report_jikan_outage(&self, message: &str) {
        let window = self.jikan_window();
        let mut last = recover(&self.jikan_outage);
        if last.is_some_and(|at| at.elapsed() < window) {
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

/// Whether a read leaves its series owed Jikan's titles (#1065): it is not
/// whole, and not because of that series' own bad answer, which asking
/// again will not mend. A read skipped inside the outage window, one that
/// met the outage, and one cut short by its page budget all are.
pub(crate) fn owes_titles(read: &JikanEpisodes) -> bool {
    !read.whole && read.stopped.as_ref().is_none_or(is_outage)
}

/// Whether a Jikan failure says Jikan is down rather than that one series'
/// answer was bad: no answer at all (a timeout or a dropped connection), a
/// rate limit the limiter already waited out, or a server error.
fn is_outage(e: &CoreError) -> bool {
    match e {
        CoreError::Provider { status, .. } => match status {
            None => true,
            Some(code) => *code == 429 || *code >= 500,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anibeam_contract::Provider;

    fn provider(status: Option<u32>) -> CoreError {
        CoreError::provider(Provider::Mal, status, "Jikan")
    }

    #[test]
    fn only_a_jikan_that_is_down_opens_the_window() {
        assert!(is_outage(&provider(None)));
        assert!(is_outage(&provider(Some(429))));
        assert!(is_outage(&provider(Some(504))));
        assert!(!is_outage(&provider(Some(400))));
        assert!(!is_outage(&provider(Some(403))));
    }

    /// A read leaves its series owed titles unless it was whole or the
    /// series' own bad answer cut it short (#1065).
    #[test]
    fn only_a_read_short_of_a_bad_answer_owes_titles() {
        let read = |whole: bool, stopped: Option<CoreError>| JikanEpisodes {
            whole,
            stopped,
            ..JikanEpisodes::default()
        };
        assert!(!owes_titles(&read(true, None)));
        // Skipped inside the window, or cut short by the page budget.
        assert!(owes_titles(&read(false, None)));
        assert!(owes_titles(&read(false, Some(provider(Some(504))))));
        assert!(owes_titles(&read(false, Some(provider(None)))));
        assert!(!owes_titles(&read(false, Some(provider(Some(400))))));
    }
}
