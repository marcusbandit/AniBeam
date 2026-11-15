-- Recommended for you (#779) -----------------------------------------------------
-- What the recommender reads from AniList, kept apart from the library's tables the
-- way Airing now keeps its own: a show only recommended never reaches the franchise
-- graph or the image fill. Each row is a reply as fetched, read at once by the
-- recommender's own parser, with the moment it was read for its time to live.

-- Every show the recommender knows: the list's, the library's and the candidates.
-- `media` is NULL when AniList no longer has the show, so it is not asked for again
-- until the row is old. `page` is the extra read the show page makes for a show the
-- library does not have: relations, AniList's own recommendations, the trailer.
CREATE TABLE rec_shows (
    anilist_id      INTEGER PRIMARY KEY,
    media           TEXT,
    fetched_at      INTEGER NOT NULL,
    page            TEXT,
    page_fetched_at INTEGER
);

-- The latest finishers of one of the user's favourites, as `[[user, score], ...]`.
CREATE TABLE rec_raters (
    anilist_id    INTEGER PRIMARY KEY,
    raters        TEXT NOT NULL,
    fetched_at    INTEGER NOT NULL
);

-- Another AniList user's list, as `[[show, status, score, repeat], ...]` with the
-- score out of 100. `entries` is NULL for a list that is private or gone.
CREATE TABLE rec_users (
    user_id       INTEGER PRIMARY KEY,
    entries       TEXT,
    fetched_at    INTEGER NOT NULL
);

-- AniList's "if you liked this" for one of the user's favourites, as `[id, ...]`.
CREATE TABLE rec_similar (
    anilist_id    INTEGER PRIMARY KEY,
    similar       TEXT NOT NULL,
    fetched_at    INTEGER NOT NULL
);

-- The last fit, best first. `forecast` is the share of the user's rated shows the
-- show is predicted to beat, 0 to 100. `why` holds the user's shows it is closest
-- to and how many hidden tags the user loves are behind it, never their names.
CREATE TABLE rec_ranked (
    anilist_id    INTEGER PRIMARY KEY,
    rank          INTEGER NOT NULL,
    forecast      INTEGER NOT NULL,
    predicted     REAL NOT NULL,
    why           TEXT NOT NULL
);

-- One row: what the last run decided, so the next one reads only what changed.
CREATE TABLE rec_state (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    people        TEXT NOT NULL DEFAULT '[]',    -- the users read as neighbours
    pool          TEXT NOT NULL DEFAULT '[]',    -- the candidates beyond the list and library
    trusted       TEXT NOT NULL DEFAULT '[]',    -- AniList's recommendations of favourites
    fitted_at     INTEGER,
    read_at       INTEGER,                       -- the last run that read everything it set out to
    rated         INTEGER NOT NULL DEFAULT 0,    -- how many rated shows the fit learnt from
    state         TEXT NOT NULL DEFAULT 'NeverFitted'
);
