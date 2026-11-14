//! The net team (#854): one client per provider, each behind the shared
//! limiter, and the image cache.

use std::sync::Arc;
use std::time::Duration;

use crate::Upstream;
use crate::anilist::AnilistClient;
use crate::aniskip::AniSkipClient;
use crate::images::{CardsFor, ImageCache};
use crate::jikan::JikanClient;
use crate::limiter::ProviderClient;
use crate::mal::MalClient;
use crate::tvmaze::TvmazeClient;
use anibeam_foundation::services::Services;

/// The gap each upstream is paced to, from its published limit.
const ANILIST_GAP: Duration = Duration::from_millis(800);
const JIKAN_GAP: Duration = Duration::from_millis(1100);
const ANISKIP_GAP: Duration = Duration::from_millis(250);
const MAL_GAP: Duration = Duration::from_millis(500);
/// TVmaze allows 20 calls in any 10 seconds per IP, which is one every
/// 500 ms; the extra 50 keeps a run from sitting right on the line.
const TVMAZE_GAP: Duration = Duration::from_millis(550);

pub struct Net {
    pub(crate) services: Arc<Services>,
    /// The provider clients, built once so every job shares one limiter
    /// per upstream.
    pub anilist: Arc<AnilistClient>,
    pub jikan: Arc<JikanClient>,
    pub aniskip: Arc<AniSkipClient>,
    pub mal: Arc<MalClient>,
    pub tvmaze: Arc<TvmazeClient>,
    /// The poster, banner and portrait files under `<cache_dir>/images`,
    /// and the row per file that says what is there. Reads consult it,
    /// jobs fill it, and the sweep keeps it from growing without end.
    pub images: Arc<ImageCache>,
    /// The library's card read, for the `SeriesChanged` that follows a
    /// change to the images.
    pub(crate) cards_for: CardsFor,
}

impl Net {
    /// The clients over the services' one transport, and the image cache
    /// over the directory `Core::open` created, which also hands in the
    /// library's card read. Nothing here touches the network.
    pub fn new(services: Arc<Services>, cards_for: CardsFor) -> Net {
        let http = &services.http;
        let images = ImageCache::new(
            services.store.clone(),
            services.paths.images_dir(),
            http.clone(),
        );
        let anilist = Arc::new(AnilistClient::new(ProviderClient::new(
            Upstream::Anilist,
            http.clone(),
            ANILIST_GAP,
        )));
        let jikan = Arc::new(JikanClient::new(ProviderClient::new(
            Upstream::Jikan,
            http.clone(),
            JIKAN_GAP,
        )));
        let aniskip = Arc::new(AniSkipClient::new(ProviderClient::new(
            Upstream::AniSkip,
            http.clone(),
            ANISKIP_GAP,
        )));
        // Every MAL call is a tracker call with a user waiting on it, so
        // this client caps each request rather than each call site doing
        // it; AniList is shared with the metadata jobs and takes its cap
        // per call instead.
        let mal = Arc::new(MalClient::new(
            ProviderClient::new(Upstream::Mal, http.clone(), MAL_GAP)
                .with_attempt_timeout(super::TRACKER_TIMEOUT),
        ));
        let tvmaze = Arc::new(TvmazeClient::new(ProviderClient::new(
            Upstream::Tvmaze,
            http.clone(),
            TVMAZE_GAP,
        )));
        Net {
            services,
            anilist,
            jikan,
            aniskip,
            mal,
            tvmaze,
            images,
            cards_for,
        }
    }
}
