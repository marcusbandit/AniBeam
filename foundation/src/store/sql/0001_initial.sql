-- Library ---------------------------------------------------------------

CREATE TABLE sources (
    id            INTEGER PRIMARY KEY,
    path          TEXT NOT NULL UNIQUE,
    available     INTEGER NOT NULL DEFAULT 1,   -- 0 while the path is missing; nothing under it is touched
    added_at      INTEGER NOT NULL,
    scanned_at    INTEGER                        -- last completed scan
);

CREATE TABLE series (
    id            INTEGER PRIMARY KEY,
    source_id     INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    kind          TEXT NOT NULL,                 -- 'show' | 'movie'
    path          TEXT NOT NULL,                 -- a show is its folder, a film is its file
    folder_name   TEXT NOT NULL,                 -- the title of last resort
    hidden        INTEGER NOT NULL DEFAULT 0,
    missing_since INTEGER,                       -- set while the path is gone and the source available, or by an import for a path never seen
    added_at      INTEGER NOT NULL,
    -- the match: at most one, every column NULL when unmatched
    provider      TEXT,                          -- 'anilist' | 'mal' | 'tmdb'
    anilist_id    INTEGER REFERENCES anilist_media(id),
    mal_id        INTEGER,
    tmdb_id       INTEGER,                       -- carried in from an export, never fetched
    tmdb_kind     TEXT,                          -- 'tv' | 'movie'
    confirmed     INTEGER NOT NULL DEFAULT 0,    -- applied by the user or imported; auto-match never replaces it
    matched_at    INTEGER,
    attempted_at  INTEGER,                       -- the last auto-match attempt that found nothing
    attempt_version INTEGER,                     -- the matcher version that made that attempt
    -- playback memory
    track_choice  TEXT,                          -- JSON TrackChoice, NULL until the first pick
    UNIQUE (kind, path)
);
CREATE INDEX series_source  ON series(source_id);
CREATE INDEX series_anilist ON series(anilist_id);   -- the "owned?" join for recommendations, graph nodes and the watching list
CREATE INDEX series_mal     ON series(mal_id);

CREATE TABLE files (
    id            INTEGER PRIMARY KEY,
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    path          TEXT NOT NULL UNIQUE,
    size          INTEGER NOT NULL,
    mtime         INTEGER NOT NULL,              -- the Feed's "downloaded" instant
    kind          TEXT NOT NULL,                 -- 'episode' | 'extra'
    number        REAL,                          -- the episode number; NULL for an extra
    season        INTEGER,
    extra_kind    TEXT,                          -- 'op' | 'ed' | 'pv' | 'sp' | 'other'
    extra_index   INTEGER,                       -- the 1 in OP1
    label         TEXT NOT NULL,                 -- the classifier's label for the row
    episode_key   TEXT NOT NULL,                 -- the history key: the number for an episode, the file name for the rest
    sidecars      TEXT NOT NULL DEFAULT '[]',    -- JSON Vec<Sidecar>
    seen_at       INTEGER NOT NULL
);
CREATE INDEX files_series  ON files(series_id);
CREATE INDEX files_episode ON files(series_id, episode_key);   -- not unique: two encodes of one episode may sit side by side

-- Provider records --------------------------------------------------------
-- One row per AniList media the library has ever needed: a match, a recommendation,
-- a graph node, a watching entry. The franchise store is these rows plus relations;
-- roots and closures are computed on read.

CREATE TABLE anilist_media (
    id            INTEGER PRIMARY KEY,           -- the AniList id
    mal_id        INTEGER,
    media_type    TEXT,                          -- 'ANIME' | 'MANGA'; relations reach manga
    title_romaji  TEXT,
    title_english TEXT,
    title_native  TEXT,
    synonyms      TEXT NOT NULL DEFAULT '[]',    -- JSON, kept this time: the matcher scores against them
    format        TEXT,
    status        TEXT,                          -- AniList's own string
    season        TEXT,
    year          INTEGER,                       -- seasonYear, else the start year
    start_date    TEXT,
    end_date      TEXT,
    episodes      INTEGER,                       -- NULL while airing with no count
    duration      INTEGER,                       -- minutes
    description   TEXT,
    average_score INTEGER,                       -- AniList's 0 to 100; normalised at read
    genres        TEXT NOT NULL DEFAULT '[]',    -- JSON
    studios       TEXT NOT NULL DEFAULT '[]',    -- JSON { id, name, main, animation }, nothing flattened away
    studio        TEXT,                          -- the animation studio the detail page names
    tags          TEXT NOT NULL DEFAULT '[]',    -- JSON { name, rank, spoiler, adult, category }
    characters    TEXT NOT NULL DEFAULT '[]',    -- JSON { id, name, role, image_url }, the top 10
    cover_url     TEXT,
    banner_url    TEXT,
    site_url      TEXT,
    fetched_at    INTEGER,                       -- NULL on a stub: known from an edge, a recommendation, a list or an import
    airing_refreshed_at   INTEGER,
    relations_fetched_at  INTEGER,               -- NULL while the crawl owes this node its edges
    crawl_deferred_until  INTEGER,               -- rate limited: retry after
    raw           TEXT                           -- the reply as fetched, for a later migration to mine without a refetch
);

CREATE TABLE anilist_episodes (                  -- the airing schedule and the episode titles, future rows included
    anilist_id    INTEGER NOT NULL REFERENCES anilist_media(id) ON DELETE CASCADE,
    number        INTEGER NOT NULL,
    title         TEXT,
    aired_at      INTEGER,                       -- in the future for a scheduled episode
    PRIMARY KEY (anilist_id, number)
);

CREATE TABLE recommendations (
    anilist_id      INTEGER NOT NULL REFERENCES anilist_media(id) ON DELETE CASCADE,
    recommended_id  INTEGER NOT NULL REFERENCES anilist_media(id),
    rank            INTEGER NOT NULL,            -- AniList's order, the top 8 kept
    rating          INTEGER,
    PRIMARY KEY (anilist_id, recommended_id)
);

CREATE TABLE relations (
    from_id       INTEGER NOT NULL REFERENCES anilist_media(id) ON DELETE CASCADE,
    to_id         INTEGER NOT NULL REFERENCES anilist_media(id),
    relation      TEXT NOT NULL,                 -- AniList's relationType; CHARACTER and OTHER are display-only
    PRIMARY KEY (from_id, to_id, relation)
);
CREATE INDEX relations_to ON relations(to_id);

-- Trackers ------------------------------------------------------------------
-- Tokens and client secrets live in the keyring or its file fallback; this is the non-secret half.

CREATE TABLE tracker_accounts (
    tracker       TEXT PRIMARY KEY,              -- 'anilist' | 'mal'
    user_id       INTEGER,
    username      TEXT,
    client_id     TEXT,
    expires_at    INTEGER,
    connected_at  INTEGER,
    synced_at     INTEGER,                       -- last successful write to the tracker
    progress_fetched_at INTEGER,                 -- the five minute cache gate
    secret_store  TEXT                           -- 'keyring' | 'file': where the token went, so it stays found
);

CREATE TABLE tracker_entries (                   -- the progress cache, both trackers
    tracker       TEXT NOT NULL,
    media_id      INTEGER NOT NULL,              -- the AniList id or the MAL id, per tracker
    status        TEXT,                          -- watching, planning, completed, paused, dropped, repeating
    progress      INTEGER NOT NULL DEFAULT 0,
    score         REAL,                          -- 0 to 10 in tenths, NULL for unrated
    repeat        INTEGER NOT NULL DEFAULT 0,
    updated_at    INTEGER,                       -- the tracker's own timestamp; the watching list's sort key
    fetched_at    INTEGER NOT NULL,
    PRIMARY KEY (tracker, media_id)
);

-- History --------------------------------------------------------------------
-- Keyed by series and episode key, never by file id: a replaced file keeps its history,
-- and the import's entries land whether or not a file exists yet.

CREATE TABLE views (                             -- one per series, the latest session
    series_id     INTEGER PRIMARY KEY REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    at            INTEGER NOT NULL
);

CREATE TABLE completed (
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    at            INTEGER NOT NULL,
    PRIMARY KEY (series_id, episode_key)
);

CREATE TABLE resume_points (
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    position      REAL NOT NULL,
    duration      REAL NOT NULL,
    at            INTEGER NOT NULL,
    PRIMARY KEY (series_id, episode_key)
);

CREATE TABLE skip_windows (                      -- the cache behind ReportChapters
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    windows       TEXT NOT NULL,                 -- JSON Vec<SkipWindow>, may be empty
    source        TEXT NOT NULL,                 -- 'chapters' | 'aniskip' | 'none'
    fetched_at    INTEGER NOT NULL,
    PRIMARY KEY (series_id, episode_key)
);

-- Store ------------------------------------------------------------------------

CREATE TABLE settings (
    key           TEXT PRIMARY KEY,              -- preferences, subtitle_defaults, auto_skip, main_tracker, auto_match_version, watching_fetched_at
    value         TEXT NOT NULL                  -- JSON
);

CREATE TABLE images (
    url           TEXT PRIMARY KEY,
    path          TEXT NOT NULL,                 -- relative to <cache_dir>/images: <aa>/<sha256>.<ext>
    bytes         INTEGER NOT NULL,
    fetched_at    INTEGER NOT NULL,
    used_at       INTEGER NOT NULL               -- bumped at most once a day per image
);
CREATE INDEX images_used ON images(used_at);

CREATE TABLE events (                            -- the ring: Info and above, the last 2000
    seq           INTEGER PRIMARY KEY,
    at            INTEGER NOT NULL,
    level         TEXT NOT NULL,
    stage         TEXT NOT NULL,
    message       TEXT NOT NULL,
    job_id        INTEGER,
    job_kind      TEXT,
    job_phase     TEXT,
    body          TEXT NOT NULL                  -- JSON EventBody
);
