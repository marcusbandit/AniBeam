-- Airing sort (#685): the rest of what AniList gives a season show that is worth ordering the
-- page by. Rows read before this stay blank until their season is read again, and the page
-- sorts a blank after every show that has the value.
ALTER TABLE season_shows ADD COLUMN trending   INTEGER;  -- AniList's trending score
ALTER TABLE season_shows ADD COLUMN favourites INTEGER;  -- how many users favourited it
ALTER TABLE season_shows ADD COLUMN duration   INTEGER;  -- minutes an episode runs
ALTER TABLE season_shows ADD COLUMN start_date INTEGER;  -- yyyymmdd; a missing month or day is 00
