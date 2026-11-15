//! anime-rec-lab's recommender. Pure functions over a person's list, a catalogue of shows and a
//! set of other AniList users: no network, no disk, no clock. That is what lets it move into
//! Anibeam's core unchanged if it earns a place there.
//!
//! Three models give an opinion on every show, each in points above or below your own mean:
//!
//! - story: the genres and story tags of shows you rated
//! - look: who directed and drew them, the studio, visual technique and era. Drops weigh more
//!   here, because drops are mostly about how a show looks
//! - people like you: users whose ratings move with yours
//!
//! A weighted blend of the three, plus a little of the community score, is the prediction.
//! `eval` hides part of the list and measures how well each model and the blend predict it.

pub mod affinity;
pub mod eval;
pub mod features;
pub mod franchise;
pub mod joint;
pub mod model;
pub mod neighbors;
pub mod recommend;
pub mod signal;
pub mod types;
pub mod why;

pub use types::*;
