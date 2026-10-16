-- The Electron import stored every match as confirmed, its auto-matcher's
-- picks included, and the native auto-match only visits series with no
-- match, so a pick the native rules refuse could never be looked at again.
-- One did happen: a film matched to a one-minute ad of the same name. This
-- clears every match to a short, the same test as automatch::is_short (a
-- known duration under five minutes, or format MUSIC), and leaves no attempt
-- behind, so the next auto-match run matches the series again under the
-- native rules.
UPDATE series
   SET provider = NULL, anilist_id = NULL, mal_id = NULL, tmdb_id = NULL, tmdb_kind = NULL,
       tvmaze_id = NULL, confirmed = 0, matched_at = NULL,
       attempted_at = NULL, attempt_version = NULL
 WHERE anilist_id IN (SELECT id FROM anilist_media WHERE duration < 5 OR format = 'MUSIC');
