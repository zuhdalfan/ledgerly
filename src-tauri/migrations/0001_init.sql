-- Application settings: a simple key/value store.
CREATE TABLE IF NOT EXISTS setting (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
