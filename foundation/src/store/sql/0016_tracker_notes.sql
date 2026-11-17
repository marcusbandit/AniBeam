-- A list entry's notes as the tracker keeps them (#211): AniList's notes,
-- MAL's comments. NULL is an entry with none, or one cached before this.
ALTER TABLE tracker_entries ADD COLUMN notes TEXT;
