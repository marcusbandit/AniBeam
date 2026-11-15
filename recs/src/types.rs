//! The data the recommender reads. Scores are always on AniList's 0 to 100 scale, whatever
//! format the person rates in, so nothing downstream has to know about score formats.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type ShowId = u64;
pub type Catalog = HashMap<ShowId, Show>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    Completed,
    Watching,
    Paused,
    Dropped,
    Planning,
    Repeating,
}

impl Status {
    pub fn from_anilist(s: &str) -> Option<Status> {
        Some(match s {
            "COMPLETED" => Status::Completed,
            "CURRENT" => Status::Watching,
            "PAUSED" => Status::Paused,
            "DROPPED" => Status::Dropped,
            "PLANNING" => Status::Planning,
            "REPEATING" => Status::Repeating,
            _ => return None,
        })
    }

    /// Far enough into a show that its sequel is something you could start.
    pub fn started(self) -> bool {
        matches!(
            self,
            Status::Completed | Status::Watching | Status::Repeating | Status::Paused
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Format {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Other,
}

impl Format {
    pub fn from_anilist(s: &str) -> Format {
        match s {
            "TV" => Format::Tv,
            "TV_SHORT" => Format::TvShort,
            "MOVIE" => Format::Movie,
            "SPECIAL" => Format::Special,
            "OVA" => Format::Ova,
            "ONA" => Format::Ona,
            "MUSIC" => Format::Music,
            _ => Format::Other,
        }
    }
}

/// The staff roles that decide how a show looks and moves. Writers and composers are left out
/// on purpose: this is the signal for "I dropped it because of the art or the direction".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LookRole {
    Director,
    CharacterDesign,
    AnimationDirector,
    ArtDirector,
    Cg,
    Photography,
    SourceArt,
    ColorDesign,
}

impl LookRole {
    /// AniList role strings carry episode notes in brackets, "Animation Director (eps 3, 7)".
    pub fn classify(role: &str) -> Option<LookRole> {
        let base = role
            .split('(')
            .next()
            .unwrap_or(role)
            .trim()
            .to_ascii_lowercase();
        Some(match base.as_str() {
            "director" | "chief director" | "series director" => LookRole::Director,
            "character design" | "animation character design" => LookRole::CharacterDesign,
            "chief animation director" | "animation director" => LookRole::AnimationDirector,
            "art director" | "art design" | "art setting" => LookRole::ArtDirector,
            "3d director" | "cg director" | "3dcg director" | "3d cg director" => LookRole::Cg,
            "director of photography" | "photography director" => LookRole::Photography,
            "original character design" => LookRole::SourceArt,
            "color design" => LookRole::ColorDesign,
            _ => return None,
        })
    }

    /// How much one credit in this role says about the look. The director shapes every frame;
    /// a colour designer shapes far less.
    pub fn weight(self) -> f32 {
        match self {
            LookRole::Director => 1.0,
            LookRole::CharacterDesign => 0.9,
            LookRole::AnimationDirector => 0.6,
            LookRole::ArtDirector => 0.6,
            LookRole::Cg => 0.5,
            LookRole::Photography => 0.4,
            LookRole::SourceArt => 0.4,
            LookRole::ColorDesign => 0.3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LookRole::Director => "director",
            LookRole::CharacterDesign => "character designer",
            LookRole::AnimationDirector => "animation director",
            LookRole::ArtDirector => "art director",
            LookRole::Cg => "CG director",
            LookRole::Photography => "director of photography",
            LookRole::SourceArt => "original character designer",
            LookRole::ColorDesign => "colour designer",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    /// How strongly the community ties the tag to the show, 0 to 100.
    pub rank: u8,
    /// AniList's "Technical" category: CGI, rotoscoping, stop motion. These are about the look.
    pub technical: bool,
    /// AniList hides it by default because it gives something away. It still counts for
    /// matching, but it is never shown.
    #[serde(default)]
    pub spoiler: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Credit {
    pub staff: u64,
    pub name: String,
    pub role: LookRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    Prequel,
    Sequel,
    Parent,
    SideStory,
    /// A recap of the related show.
    Summary,
    /// The same story told again: a remake, a retelling, another adaptation.
    Alternative,
    /// Cut together from the related show.
    Compilation,
    Other,
}

impl RelationKind {
    pub fn from_anilist(s: &str) -> RelationKind {
        match s {
            "PREQUEL" => RelationKind::Prequel,
            "SEQUEL" => RelationKind::Sequel,
            "PARENT" => RelationKind::Parent,
            "SIDE_STORY" => RelationKind::SideStory,
            "SUMMARY" => RelationKind::Summary,
            "ALTERNATIVE" => RelationKind::Alternative,
            "COMPILATION" => RelationKind::Compilation,
            _ => RelationKind::Other,
        }
    }
}

/// Where a show is in its run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiringStatus {
    Finished,
    Airing,
    NotYetAired,
    Cancelled,
    Hiatus,
}

impl AiringStatus {
    pub fn from_anilist(s: &str) -> Option<AiringStatus> {
        Some(match s {
            "FINISHED" => AiringStatus::Finished,
            "RELEASING" => AiringStatus::Airing,
            "NOT_YET_RELEASED" => AiringStatus::NotYetAired,
            "CANCELLED" => AiringStatus::Cancelled,
            "HIATUS" => AiringStatus::Hiatus,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Show {
    pub id: ShowId,
    pub title: String,
    pub format: Format,
    pub episodes: Option<u32>,
    /// Minutes per episode.
    pub minutes: Option<u32>,
    pub year: Option<i32>,
    pub genres: Vec<String>,
    pub tags: Vec<Tag>,
    pub studios: Vec<String>,
    pub credits: Vec<Credit>,
    /// AniList's average score, 0 to 100.
    pub community: Option<u8>,
    /// How many AniList users have it on a list. None when AniList did not say.
    pub popularity: Option<u32>,
    pub relations: Vec<(ShowId, RelationKind)>,
    pub adult: bool,
    /// None when the cached copy predates status being fetched.
    #[serde(default)]
    pub status: Option<AiringStatus>,
}

impl Show {
    /// Total minutes to watch all of it, when AniList knows both numbers.
    pub fn runtime(&self) -> Option<u32> {
        Some(self.episodes? * self.minutes?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub show: ShowId,
    pub status: Status,
    /// 1 to 100, or None when the person never rated it.
    pub score: Option<f32>,
    pub repeat: u32,
}

/// Why someone dropped a show, when they chose to say. Optional and never required: a drop with
/// no reason is read as "probably the look", which is how this person usually drops.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DropReason {
    /// Art style, animation or direction.
    Look,
    Story,
    Pacing,
    Other,
}

impl DropReason {
    pub fn parse(s: &str) -> Option<DropReason> {
        Some(match s.to_ascii_lowercase().as_str() {
            "look" | "art" | "animation" | "direction" => DropReason::Look,
            "story" | "plot" => DropReason::Story,
            "pacing" | "slow" => DropReason::Pacing,
            "other" => DropReason::Other,
            _ => return None,
        })
    }
}

pub type DropReasons = HashMap<ShowId, DropReason>;

/// Another AniList user whose list we read to find people with similar taste.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Neighbor {
    pub user: u64,
    pub entries: Vec<Entry>,
}
