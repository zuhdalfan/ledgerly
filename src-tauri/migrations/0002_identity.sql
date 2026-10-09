-- Business profile: a single row (id = 'default').
CREATE TABLE business (
    id                           TEXT PRIMARY KEY NOT NULL CHECK (id = 'default'),
    display_name                 TEXT NOT NULL DEFAULT '',
    legal_name                   TEXT,
    email                        TEXT,
    phone                        TEXT,
    website                      TEXT,
    address_line1                TEXT,
    address_line2                TEXT,
    city                         TEXT,
    region                       TEXT,
    postal_code                  TEXT,
    country                      TEXT,
    tax_id                       TEXT,
    default_currency             TEXT NOT NULL DEFAULT 'USD',
    default_terms                TEXT,
    default_notes                TEXT,
    default_payment_instructions TEXT,
    updated_at                   TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO business (id) VALUES ('default');

-- Clients.
CREATE TABLE client (
    id             TEXT PRIMARY KEY NOT NULL,
    display_name   TEXT NOT NULL,
    company        TEXT,
    contact_person TEXT,
    email          TEXT,
    phone          TEXT,
    address_line1  TEXT,
    address_line2  TEXT,
    city           TEXT,
    region         TEXT,
    postal_code    TEXT,
    country        TEXT,
    tax_id         TEXT,
    currency       TEXT,
    notes          TEXT,
    archived_at    TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_client_display_name ON client (display_name);
CREATE INDEX idx_client_archived ON client (archived_at);

-- Reusable products/services.
CREATE TABLE catalog_item (
    id               TEXT PRIMARY KEY NOT NULL,
    name             TEXT NOT NULL,
    description      TEXT,
    unit             TEXT,
    unit_price_minor INTEGER NOT NULL DEFAULT 0,
    currency         TEXT,
    sku              TEXT,
    archived_at      TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_catalog_item_name ON catalog_item (name);
