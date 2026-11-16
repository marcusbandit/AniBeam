-- Words for the activity log (#792) -------------------------------------------------
-- A plain line keeps its catalog ID and arguments beside the words it was said in, so a
-- shell shows a line already in the log in whatever language it speaks now. JSON:
-- {"id": "scan-done", "args": {"count": 3}}; NULL for a line said before this column, which
-- keeps showing its old words.
ALTER TABLE events ADD COLUMN plain_words TEXT;
