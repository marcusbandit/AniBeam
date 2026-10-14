-- IMDb ratings ---------------------------------------------------------------
-- IMDb's own daily dump, every rated title in it, replaced whole by the
-- weekly refresh. Kept whole rather than trimmed to the library, so a
-- series matched later has its rating already, with no refill to wait on.

CREATE TABLE imdb_ratings (
    id              INTEGER PRIMARY KEY,         -- the number after "tt"
    rating          INTEGER NOT NULL,            -- tenths: 8.9 is 89
    votes           INTEGER NOT NULL
);

-- Wikidata's map from AniList and MAL ids to IMDb titles, since neither
-- provider carries an IMDb id of its own. Also replaced whole. One id can
-- link to several titles; the card takes the most voted.
CREATE TABLE imdb_links (
    provider        TEXT NOT NULL,               -- 'anilist' | 'mal', Provider's column names
    provider_id     INTEGER NOT NULL,
    imdb            INTEGER NOT NULL,            -- the number after "tt"
    PRIMARY KEY (provider, provider_id, imdb)
) WITHOUT ROWID;
