-- TVmaze, the second match provider ----------------------------------------
-- The shows AniList has no entry for: western animation and live-action
-- television. One row per show a series has been matched to, written whole
-- by every fetch, which is one request with the episodes and images embedded.

CREATE TABLE tvmaze_shows (
    id              INTEGER PRIMARY KEY,         -- the TVmaze show id
    name            TEXT,
    url             TEXT,                        -- the show's page on tvmaze.com
    type            TEXT,                        -- 'Scripted' | 'Animation' | 'Reality' | ...
    language        TEXT,
    genres          TEXT NOT NULL DEFAULT '[]',  -- JSON
    status          TEXT,                        -- TVmaze's own: 'Running' | 'Ended' | 'To Be Determined' | 'In Development'
    premiered       TEXT,                        -- YYYY-MM-DD
    ended           TEXT,
    average_runtime INTEGER,                     -- minutes
    rating          REAL,                        -- already 0 to 10
    network         TEXT,                        -- the network's name, else the web channel's
    summary         TEXT,                        -- plain text, the HTML stripped at write
    poster_url      TEXT,                        -- the main poster, else the show's image
    background_url  TEXT,                        -- the main background, else the first
    imdb            TEXT,
    thetvdb         INTEGER,
    fetched_at      INTEGER NOT NULL             -- the last fetch; the airing refresh's clock too
);

CREATE TABLE tvmaze_episodes (                   -- numbered episodes only; a special without a number has no place in a running order
    show_id         INTEGER NOT NULL REFERENCES tvmaze_shows(id) ON DELETE CASCADE,
    season          INTEGER NOT NULL,
    number          INTEGER NOT NULL,
    name            TEXT,
    airstamp        INTEGER,                     -- Unix seconds, in the future for a scheduled episode
    runtime         INTEGER,
    PRIMARY KEY (show_id, season, number)
);

-- No foreign key: an imported match carries its id before the next refresh
-- writes the show row behind it.
ALTER TABLE series ADD COLUMN tvmaze_id INTEGER;   -- provider = 'tvmaze'
CREATE INDEX series_tvmaze ON series(tvmaze_id);
