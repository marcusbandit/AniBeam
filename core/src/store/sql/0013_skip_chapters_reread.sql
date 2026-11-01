-- Prologue is not an opening (#328) ----------------------------------------------
-- Windows read from a file's chapters were cached under a pattern that took a
-- Prologue chapter for the opening, and the player serves the cached row at open,
-- before the chapters are read again. Dropping every chapter-sourced row makes the
-- next play read the chapters afresh. AniSkip answers and misses stay.
DELETE FROM skip_windows WHERE source = 'chapters';
