-- The shows a subscription follows (#215), as the last read of the torrent
-- client's rules linked them to the library, so the core can look for each
-- new episode when it airs without reaching the client at launch. A miss is
-- said once per episode, never again after a restart.
CREATE TABLE follows (
    series_id   INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    feed_url    TEXT NOT NULL,                -- a show two rules follow has two rows
    read_at     INTEGER NOT NULL,
    PRIMARY KEY (series_id, feed_url)
);

CREATE TABLE follow_misses (
    series_id   INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode     INTEGER NOT NULL,
    said_at     INTEGER NOT NULL,
    PRIMARY KEY (series_id, episode)
);
