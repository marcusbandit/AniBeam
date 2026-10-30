-- Episode count set by hand (#239) ---------------------------------------------
-- How many episodes the person says a series has, for a show whose record
-- names none yet. NULL when unset. It stands until the schedule or a file
-- goes past it; the read gives way to what really happened and keeps the
-- number, so clearing it stays the person's call.
ALTER TABLE series ADD COLUMN episodes_set INTEGER;
