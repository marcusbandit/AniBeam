//! The releases team (#854): endpoint repos, the endpoints installed from
//! them, and the feed.

use std::sync::Arc;

use crate::downloads::Downloads;
use crate::foundation::above::Above;
use crate::foundation::services::Services;
use crate::repos::Pending;

pub struct Releases {
    pub(crate) services: Arc<Services>,
    /// Repo indexes fetched for a preview and not trusted yet.
    pub(crate) pending: Pending,
    /// The engine an endpoint change goes to, and what anirss lists.
    pub(crate) downloads: Above<Downloads>,
}

impl Releases {
    pub(crate) fn new(services: Arc<Services>) -> Releases {
        Releases {
            services,
            pending: Pending::default(),
            downloads: Above::default(),
        }
    }
}
