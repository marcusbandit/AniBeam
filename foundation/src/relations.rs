//! The AniList relations a franchise is walked along: the closure's, and the
//! library's franchise boxes, which have to agree on them (#854).

/// The relations the walk follows. CHARACTER and OTHER are deliberately
/// missing: a shared character or a loose link is drawn, never walked, or
/// one cameo would glue every franchise it touches into a single blob.
pub const TRAVERSABLE: [&str; 11] = [
    "PREQUEL",
    "SEQUEL",
    "SIDE_STORY",
    "SPIN_OFF",
    "ALTERNATIVE",
    "PARENT",
    "CONTAINS",
    "SUMMARY",
    "COMPILATION",
    "SOURCE",
    "ADAPTATION",
];
