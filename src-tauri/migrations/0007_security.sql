-- Optional app lock. The passphrase itself is never stored: only an Argon2id
-- verifier (a PHC string with its own salt).
CREATE TABLE app_lock (
    id                TEXT PRIMARY KEY NOT NULL CHECK (id = 'default'),
    verifier          TEXT,
    enabled           INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
    auto_lock_minutes INTEGER NOT NULL DEFAULT 15,
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO app_lock (id) VALUES ('default');
