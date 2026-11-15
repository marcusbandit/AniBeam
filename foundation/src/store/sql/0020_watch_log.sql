-- The watch log (#593): every finish of an episode, each with its own time.
-- `completed` keeps one row per episode, the latest finish, which the series page
-- and the cards read; Stats reads this instead, so a rewatch adds to the year it
-- happened in and leaves the earlier one alone.
--
-- `how` says where a finish came from:
--   played    the player reached the end of the episode here
--   marked    progress set by hand; Stats leaves it out
--   imported  an export file's completion
--   copied    the history from before the log, one finish per episode
--
-- Keyed by series and episode key like the rest of the history, and moved with
-- it when a file's key changes. No row is ever unique: two finishes of one
-- episode are two rows.

CREATE TABLE watch_log (
    id            INTEGER PRIMARY KEY,
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    at            INTEGER NOT NULL,
    how           TEXT NOT NULL
);
CREATE INDEX watch_log_episode ON watch_log(series_id, episode_key);

-- Copied once: what was finished before the log, at its latest finish. A rewatch
-- from before the log is lost to it and stays counted once.
INSERT INTO watch_log (series_id, episode_key, at, how)
SELECT series_id, episode_key, at, 'copied' FROM completed ORDER BY at, series_id, episode_key;
