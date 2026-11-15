//! The playback team (#854): the player's sessions, Discord presence and
//! casting. The watch together room sits above it and hears about a session
//! through [`Follower`].

use std::sync::Arc;

use crate::cast::Cast;
use crate::presence::Presence;
use crate::session::Sessions;
use anibeam_downloads::Downloads;
use anibeam_foundation::above::Above;
use anibeam_foundation::services::Services;
use anibeam_net::Net;
use anibeam_trackers::Trackers;

/// What a team above hears about a session as it plays: the watch together
/// room, which drives the session it is attached to. Each runs where the
/// session rules once called the room, inline, before the rule returns.
pub trait Follower: Send + Sync {
    /// One tick of `session` from the player.
    fn tick(&self, session: u64, position: f64, paused: bool);
    /// `session` closed.
    fn closed(&self, session: u64);
    /// The skip windows job found where `session`'s opening starts.
    fn opening_known(&self, session: u64, start: f64);
}

pub struct Playback {
    pub(crate) services: Arc<Services>,
    pub(crate) net: Arc<Net>,
    pub(crate) trackers: Arc<Trackers>,
    pub(crate) downloads: Arc<Downloads>,
    /// Every open playback session, and the counter that names the next
    /// one. A session lives in memory for one run of the player: the rows
    /// its rules write are the record, and none of it survives a restart.
    pub sessions: Arc<Sessions>,
    /// Discord rich presence: what the player shows on Discord while the
    /// switch is on. Holds no thread or socket until it is first switched on.
    pub presence: Presence,
    /// Casting to Chromecasts and DLNA TVs (`Cast*` calls).
    #[cfg_attr(not(feature = "cast"), allow(dead_code))]
    pub(crate) cast: Cast,
    /// The watch together room, linked by `Core::open`; nothing to tell
    /// before that or once it is gone.
    pub follower: Above<dyn Follower>,
}

impl Playback {
    pub fn new(
        services: Arc<Services>,
        net: Arc<Net>,
        trackers: Arc<Trackers>,
        downloads: Arc<Downloads>,
    ) -> Playback {
        let presence = Presence::new(services.bus.clone());
        Playback {
            services,
            net,
            trackers,
            downloads,
            sessions: Arc::new(Sessions::default()),
            presence,
            cast: Cast::default(),
            follower: Above::default(),
        }
    }
}
