-- Endpoint repos (#143): signed indexes added by URL, and the endpoints installed from them.
-- A repo is keyed by its index URL, so one site publishing several groups (general, nsfw) is
-- several rows. `payload` is the last index that passed its checks, kept byte for byte as it
-- was signed; a failed fetch keeps it and says why in `error`.
CREATE TABLE endpoint_repos (
    url        TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    nsfw       INTEGER NOT NULL DEFAULT 0,
    key        TEXT NOT NULL,
    serial     INTEGER NOT NULL,
    payload    BLOB NOT NULL,
    fetched_at INTEGER NOT NULL,
    error      TEXT
) STRICT;

-- An installed endpoint stays when its repo is removed or stops offering it; the person
-- removes it. Its credentials live in the secret store, never here.
CREATE TABLE installed_endpoints (
    install_id  TEXT PRIMARY KEY,
    repo_url    TEXT NOT NULL,
    endpoint_id TEXT NOT NULL,
    version     INTEGER NOT NULL,
    definition  TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    position    INTEGER NOT NULL,
    UNIQUE (repo_url, endpoint_id)
) STRICT;
