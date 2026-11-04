-- Airing now's wide banners (#415) -----------------------------------------------
-- Out now draws each episode as a wide tile, and a tall cover cropped to that
-- shape is mostly blur. AniList's bannerImage is the wide art; the season keeps its
-- URL beside the cover's, and the image cache holds the file.
ALTER TABLE season_shows ADD COLUMN banner_url TEXT;
