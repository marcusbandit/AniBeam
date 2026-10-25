-- Quick fixes (#170) ---------------------------------------------------------
-- What the user says a file is, against how the scan read its name. Keyed by
-- the path, as files.path is, so a fix outlives the file row: a rescan, a file
-- removed and put back, a series rebuilt by a moved folder above it. A renamed
-- file is read afresh, since its new name may well be right.

CREATE TABLE file_fixes (
    path          TEXT PRIMARY KEY,
    reading       TEXT NOT NULL,                 -- JSON FileReading: what the file is
    was           TEXT NOT NULL,                 -- JSON FileReading: what the scan read before the first fix
    fixed_at      INTEGER NOT NULL
);
