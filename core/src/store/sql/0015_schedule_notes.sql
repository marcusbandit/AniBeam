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
    year       INTEGER,                      -- set when only the year, or the year and month, is known
    month      INTEGER,                      -- 1 to 12, with year, when the month is known and the day is not
    set_at     INTEGER NOT NULL
);
CREATE INDEX schedule_notes_series ON schedule_notes(series_id);

-- Notes belong to the record they were written against: a cleared match or a
-- different show drops them, the way a new match drops a set episode count.
CREATE TRIGGER schedule_notes_follow_match
AFTER UPDATE OF anilist_id, mal_id, tvmaze_id ON series
WHEN OLD.anilist_id IS NOT NEW.anilist_id
  OR OLD.mal_id IS NOT NEW.mal_id
  OR OLD.tvmaze_id IS NOT NEW.tvmaze_id
BEGIN
    DELETE FROM schedule_notes WHERE series_id = NEW.id;
END;
