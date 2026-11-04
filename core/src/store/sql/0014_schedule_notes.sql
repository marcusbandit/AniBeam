-- What the user told a series' schedule that its provider does not say
-- (#418): episodes on a date, episodes due some time in a year, or episodes
-- with no date at all. The predictor never guesses past a note without a
-- date, and a provider's own date still wins over a dated note.
CREATE TABLE schedule_notes (
    series_id  INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    season     INTEGER,                      -- the TVmaze season, NULL for a record numbering without seasons
    first      INTEGER NOT NULL,
    last       INTEGER NOT NULL,             -- first when the note covers one episode
    at         INTEGER,                      -- Unix seconds, noon UTC for a date with no time
    year       INTEGER,                      -- set when only the year is known
    set_at     INTEGER NOT NULL
);
CREATE INDEX schedule_notes_series ON schedule_notes(series_id);
