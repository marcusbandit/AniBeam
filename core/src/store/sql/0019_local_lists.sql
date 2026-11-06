-- The list kept on this device (#213): a series' list status and score for
-- someone with no tracker account, or for a series no connected tracker
-- carries. A tracker's own entry wins wherever there is one.
CREATE TABLE local_entries (
    series_id   INTEGER PRIMARY KEY REFERENCES series(id) ON DELETE CASCADE,
    status      TEXT,                        -- watching, planning, completed, paused, dropped
    score       REAL,                        -- 0 to 10 in tenths, NULL for unrated
    updated_at  INTEGER NOT NULL
);
