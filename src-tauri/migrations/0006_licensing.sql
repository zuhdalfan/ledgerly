-- Per-installation identity. The serial is stored sealed (AES-GCM under a key
-- held in the OS keychain); the machine fingerprint is a salted hash.
CREATE TABLE installation (
    id             TEXT PRIMARY KEY NOT NULL CHECK (id = 'default'),
    serial_sealed  TEXT,
    machine_fp     TEXT,
    sku            TEXT NOT NULL DEFAULT 'ledgerly',
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO installation (id) VALUES ('default');

-- The active licence, re-verified on every launch. `file_json` is the signed
-- payload exactly as it was imported, so the signature can always be re-checked.
CREATE TABLE license (
    id               TEXT PRIMARY KEY NOT NULL,
    key_id           TEXT NOT NULL,
    sku              TEXT NOT NULL,
    plan             TEXT NOT NULL,
    features_json    TEXT NOT NULL,
    issued_at        TEXT NOT NULL,
    expires_at       TEXT,
    binding          TEXT NOT NULL,
    file_json        TEXT NOT NULL,
    activated_at     TEXT NOT NULL DEFAULT (datetime('now')),
    last_verified_at TEXT
);

-- Optional one-off trial of the Pro features.
CREATE TABLE trial (
    id         TEXT PRIMARY KEY NOT NULL CHECK (id = 'default'),
    started_at TEXT,
    ends_at    TEXT
);

INSERT INTO trial (id) VALUES ('default');
