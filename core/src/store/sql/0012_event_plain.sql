-- Plain activity log (#287) ------------------------------------------------------
-- The line a person reads beside the developer's message. NULL for a line only
-- the developer log shows, which is every row written before this column.
ALTER TABLE events ADD COLUMN plain TEXT;
