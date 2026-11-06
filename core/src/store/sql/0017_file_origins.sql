-- Which torrent each downloaded file came from (#241), so On disk tells this
-- release from another by the torrent rather than by the name. Keyed by the
-- file's path, the way the library and the torrent client both name it, so
-- a rescan that rewrites the files row keeps it.
CREATE TABLE file_origins (
    path          TEXT PRIMARY KEY,
    info_hash     TEXT NOT NULL,                 -- lower case hex
    torrent_name  TEXT NOT NULL,
    recorded_at   INTEGER NOT NULL
);
