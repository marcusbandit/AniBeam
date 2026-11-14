-- Season-aware history keys -------------------------------------------------
-- An episode of any season but the first keys its history as S02E01 rather
-- than the bare number, so S01E01 and S02E01 stop sharing views, resume
-- points and completion. Season 1, and a file naming no season, keep the
-- bare number, which is what every key was, so their rows need no rewrite.
--
-- A bare history row whose number only a later season's file carries (a
-- folder holding season 2 alone, say) belonged to that file and follows it
-- to its new key. Where a season 1 file carries the number too, the row was
-- ambiguous and stays season 1's. The files' own keys are rewritten last,
-- once the history has been matched against the old ones; the next scan
-- writes the same keys again.

UPDATE completed SET episode_key = (
    SELECT 'S' || printf('%02d', f.season) || 'E' || CASE WHEN f.number = CAST(f.number AS INTEGER) THEN printf('%02d', CAST(f.number AS INTEGER)) ELSE CAST(f.number AS TEXT) END
    FROM files f
    WHERE f.series_id = completed.series_id AND f.kind = 'episode' AND f.season IS NOT NULL AND f.season <> 1 AND f.episode_key = completed.episode_key
    ORDER BY f.season LIMIT 1)
WHERE EXISTS (SELECT 1 FROM files f WHERE f.series_id = completed.series_id AND f.kind = 'episode' AND f.season IS NOT NULL AND f.season <> 1 AND f.episode_key = completed.episode_key)
  AND NOT EXISTS (SELECT 1 FROM files f WHERE f.series_id = completed.series_id AND f.kind = 'episode' AND (f.season IS NULL OR f.season = 1) AND f.episode_key = completed.episode_key);

UPDATE resume_points SET episode_key = (
    SELECT 'S' || printf('%02d', f.season) || 'E' || CASE WHEN f.number = CAST(f.number AS INTEGER) THEN printf('%02d', CAST(f.number AS INTEGER)) ELSE CAST(f.number AS TEXT) END
    FROM files f
    WHERE f.series_id = resume_points.series_id AND f.kind = 'episode' AND f.season IS NOT NULL AND f.season <> 1 AND f.episode_key = resume_points.episode_key
    ORDER BY f.season LIMIT 1)
WHERE EXISTS (SELECT 1 FROM files f WHERE f.series_id = resume_points.series_id AND f.kind = 'episode' AND f.season IS NOT NULL AND f.season <> 1 AND f.episode_key = resume_points.episode_key)
  AND NOT EXISTS (SELECT 1 FROM files f WHERE f.series_id = resume_points.series_id AND f.kind = 'episode' AND (f.season IS NULL OR f.season = 1) AND f.episode_key = resume_points.episode_key);

UPDATE views SET episode_key = (
    SELECT 'S' || printf('%02d', f.season) || 'E' || CASE WHEN f.number = CAST(f.number AS INTEGER) THEN printf('%02d', CAST(f.number AS INTEGER)) ELSE CAST(f.number AS TEXT) END
    FROM files f
    WHERE f.series_id = views.series_id AND f.kind = 'episode' AND f.season IS NOT NULL AND f.season <> 1 AND f.episode_key = views.episode_key
    ORDER BY f.season LIMIT 1)
WHERE EXISTS (SELECT 1 FROM files f WHERE f.series_id = views.series_id AND f.kind = 'episode' AND f.season IS NOT NULL AND f.season <> 1 AND f.episode_key = views.episode_key)
  AND NOT EXISTS (SELECT 1 FROM files f WHERE f.series_id = views.series_id AND f.kind = 'episode' AND (f.season IS NULL OR f.season = 1) AND f.episode_key = views.episode_key);

UPDATE files SET episode_key = 'S' || printf('%02d', season) || 'E' || CASE WHEN number = CAST(number AS INTEGER) THEN printf('%02d', CAST(number AS INTEGER)) ELSE CAST(number AS TEXT) END
WHERE kind = 'episode' AND season IS NOT NULL AND season <> 1;
