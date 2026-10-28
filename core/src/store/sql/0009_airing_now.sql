-- Airing now (#202) -----------------------------------------------------------
-- Every show AniList has on air, not only the library's. A season is 'NOW'
-- for everything releasing today, whatever season it started in, or a
-- named one such as 'WINTER-2027'. The page reads these rows and nothing
-- else, so a show here needs no anilist_media row and the library's tables
-- are never touched by a season the user only browsed.

CREATE TABLE season_shows (
    season        TEXT NOT NULL,
    anilist_id    INTEGER NOT NULL,
    rank          INTEGER NOT NULL,              -- AniList's popularity order within the season
    mal_id        INTEGER,
    title_romaji  TEXT,
    title_english TEXT,
    title_native  TEXT,
    format        TEXT,
    episodes      INTEGER,
    average_score INTEGER,                       -- AniList's 0 to 100; normalised at read
    cover_url     TEXT,
    site_url      TEXT,
    next_episode  INTEGER,
    next_at       INTEGER,
    PRIMARY KEY (season, anilist_id)
);

-- When each season was last read, so opening the page twice in a minute
-- asks AniList once.
CREATE TABLE season_fetches (
    season        TEXT PRIMARY KEY,
    fetched_at    INTEGER NOT NULL
);

-- Episodes that came out in the last two days, newest first on the page.
CREATE TABLE season_aired (
    anilist_id    INTEGER NOT NULL,
    episode       INTEGER NOT NULL,
    aired_at      INTEGER NOT NULL,
    PRIMARY KEY (anilist_id, episode)
);

-- The shows the user said they do not care about. One AniList entry each and
-- nothing else: a new season is a new entry and shows up again. The title and
-- cover are kept as they read when marked, so Settings can name a show no
-- season row carries any more.
CREATE TABLE not_interested (
    anilist_id    INTEGER PRIMARY KEY,
    title         TEXT NOT NULL,
    cover_url     TEXT,
    marked_at     INTEGER NOT NULL
);
