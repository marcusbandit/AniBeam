-- The season a hand-set episode count belongs to (#283) ---------------------
-- A TVmaze show numbers its episodes within a season, so a count set by hand
-- is that season's length, and it stops applying once the folder moves on to
-- another season. NULL for a record that numbers without seasons (AniList).
ALTER TABLE series ADD COLUMN episodes_set_season INTEGER;
