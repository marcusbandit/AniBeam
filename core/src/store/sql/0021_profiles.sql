-- Profiles (#562, #630): several people on one tailnet each keep their own progress.
-- A profile is the Tailscale user a device belongs to. Profile 1 is the person this
-- computer belongs to; its Tailscale user is filled in once the host reads the tailnet.
-- History, resume points and tracker accounts carry a profile; the library and the
-- metadata stay shared. Every row already here becomes profile 1's, and nothing is lost.
--
-- `profile_id` defaults to 1, so a write that names no profile is the owner's. It comes
-- last in each key, so every read and delete by series or tracker keeps its index.

CREATE TABLE profiles (
    id            INTEGER PRIMARY KEY,
    tailnet_user  TEXT UNIQUE,                   -- Tailscale's stable user id; NULL until known
    login         TEXT,                          -- the Tailscale login, "someone@example.com"
    name          TEXT,                          -- the name Tailscale shows for them
    created_at    INTEGER NOT NULL
);
INSERT INTO profiles (id, created_at) VALUES (1, CAST(strftime('%s', 'now') AS INTEGER));

CREATE TABLE views_p (
    profile_id    INTEGER NOT NULL DEFAULT 1 REFERENCES profiles(id) ON DELETE CASCADE,
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    at            INTEGER NOT NULL,
    PRIMARY KEY (series_id, profile_id)
);
INSERT INTO views_p (profile_id, series_id, episode_key, at)
SELECT 1, series_id, episode_key, at FROM views;
DROP TABLE views;
ALTER TABLE views_p RENAME TO views;

CREATE TABLE completed_p (
    profile_id    INTEGER NOT NULL DEFAULT 1 REFERENCES profiles(id) ON DELETE CASCADE,
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    at            INTEGER NOT NULL,
    PRIMARY KEY (series_id, episode_key, profile_id)
);
INSERT INTO completed_p (profile_id, series_id, episode_key, at)
SELECT 1, series_id, episode_key, at FROM completed;
DROP TABLE completed;
ALTER TABLE completed_p RENAME TO completed;

CREATE TABLE resume_points_p (
    profile_id    INTEGER NOT NULL DEFAULT 1 REFERENCES profiles(id) ON DELETE CASCADE,
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    position      REAL NOT NULL,
    duration      REAL NOT NULL,
    at            INTEGER NOT NULL,
    PRIMARY KEY (series_id, episode_key, profile_id)
);
INSERT INTO resume_points_p (profile_id, series_id, episode_key, position, duration, at)
SELECT 1, series_id, episode_key, position, duration, at FROM resume_points;
DROP TABLE resume_points;
ALTER TABLE resume_points_p RENAME TO resume_points;

CREATE TABLE watch_log_p (
    id            INTEGER PRIMARY KEY,
    profile_id    INTEGER NOT NULL DEFAULT 1 REFERENCES profiles(id) ON DELETE CASCADE,
    series_id     INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    episode_key   TEXT NOT NULL,
    at            INTEGER NOT NULL,
    how           TEXT NOT NULL
);
INSERT INTO watch_log_p (id, profile_id, series_id, episode_key, at, how)
SELECT id, 1, series_id, episode_key, at, how FROM watch_log;
DROP TABLE watch_log;
ALTER TABLE watch_log_p RENAME TO watch_log;
CREATE INDEX watch_log_episode ON watch_log(series_id, episode_key, profile_id);

CREATE TABLE tracker_accounts_p (
    profile_id    INTEGER NOT NULL DEFAULT 1 REFERENCES profiles(id) ON DELETE CASCADE,
    tracker       TEXT NOT NULL,                 -- 'anilist' | 'mal'
    user_id       INTEGER,
    username      TEXT,
    client_id     TEXT,
    expires_at    INTEGER,
    connected_at  INTEGER,
    synced_at     INTEGER,                       -- last successful write to the tracker
    progress_fetched_at INTEGER,                 -- the five minute cache gate
    secret_store  TEXT,                          -- 'keyring' | 'file': where the token went
    PRIMARY KEY (tracker, profile_id)
);
INSERT INTO tracker_accounts_p (profile_id, tracker, user_id, username, client_id, expires_at,
                                connected_at, synced_at, progress_fetched_at, secret_store)
SELECT 1, tracker, user_id, username, client_id, expires_at,
       connected_at, synced_at, progress_fetched_at, secret_store FROM tracker_accounts;
DROP TABLE tracker_accounts;
ALTER TABLE tracker_accounts_p RENAME TO tracker_accounts;

CREATE TABLE tracker_entries_p (                 -- the progress cache, per profile and tracker
    profile_id    INTEGER NOT NULL DEFAULT 1 REFERENCES profiles(id) ON DELETE CASCADE,
    tracker       TEXT NOT NULL,
    media_id      INTEGER NOT NULL,              -- the AniList id or the MAL id, per tracker
    status        TEXT,
    progress      INTEGER NOT NULL DEFAULT 0,
    score         REAL,
    repeat        INTEGER NOT NULL DEFAULT 0,
    updated_at    INTEGER,
    fetched_at    INTEGER NOT NULL,
    notes         TEXT,
    PRIMARY KEY (tracker, media_id, profile_id)
);
INSERT INTO tracker_entries_p (profile_id, tracker, media_id, status, progress, score, repeat,
                               updated_at, fetched_at, notes)
SELECT 1, tracker, media_id, status, progress, score, repeat, updated_at, fetched_at, notes
FROM tracker_entries;
DROP TABLE tracker_entries;
ALTER TABLE tracker_entries_p RENAME TO tracker_entries;
