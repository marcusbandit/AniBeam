-- MAL ids found by search (#782) ------------------------------------------------------
-- AniList carries no idMal for many new shows, specials and OVAs, and a write to MAL had
-- nothing to go to for them. The first write to a connected MAL account for such a show
-- searches MAL once and keeps the answer here, keyed by the AniList id: a hit for good, a
-- miss for a week before the show may be searched again. A series whose own mal_id is
-- NULL reads its MAL id here.
CREATE TABLE mal_lookups (
    anilist_id INTEGER PRIMARY KEY,
    mal_id     INTEGER,                      -- NULL when the search found nothing it could be sure of
    looked_at  INTEGER NOT NULL
);
