//! The playback team (#854): the player's sessions, Discord presence,
//! casting and watch together rooms.

use std::sync::Arc;

use crate::cast::Cast;
use crate::downloads::Downloads;
use crate::foundation::services::Services;
use crate::library::Library;
use crate::metadata::Metadata;
use crate::net::Net;
use crate::playback::session::Sessions;
use crate::presence::Presence;
use crate::room::Rooms;
use crate::trackers::Trackers;

pub struct Playback {
    pub(crate) services: Arc<Services>,
    pub(crate) net: Arc<Net>,
    pub(crate) trackers: Arc<Trackers>,
    pub(crate) downloads: Arc<Downloads>,
    /// The library a room's Get media brings a file into, and the matching
    /// that points it at the room's show (#889).
    pub(crate) library: Arc<Library>,
    pub(crate) metadata: Arc<Metadata>,
    /// Every open playback session, and the counter that names the next
    /// one. A session lives in memory for one run of the player: the rows
    /// its rules write are the record, and none of it survives a restart.
    pub(crate) sessions: Arc<Sessions>,
    /// Discord rich presence: what the player shows on Discord while the
    /// switch is on. Holds no thread or socket until it is first switched on.
    pub(crate) presence: Presence,
    /// Casting to Chromecasts and DLNA TVs (`Cast*` calls).
    #[cfg_attr(not(feature = "cast"), allow(dead_code))]
    pub(crate) cast: Cast,
    /// The watch together room this core is in, if any (`Room*` calls).
    pub(crate) rooms: Rooms,
}

impl Playback {
    pub(crate) fn new(
        services: Arc<Services>,
        net: Arc<Net>,
        trackers: Arc<Trackers>,
        downloads: Arc<Downloads>,
        library: Arc<Library>,
        metadata: Arc<Metadata>,
    ) -> Playback {
        let presence = Presence::new(services.bus.clone());
        Playback {
            services,
            net,
            trackers,
            downloads,
            library,
            metadata,
            sessions: Arc::new(Sessions::default()),
            presence,
            cast: Cast::default(),
            rooms: Rooms::default(),
        }
    }
}
