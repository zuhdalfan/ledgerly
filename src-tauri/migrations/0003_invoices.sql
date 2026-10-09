-- Tax rates (rate stored in basis points: 10000 bp = 100%).
CREATE TABLE tax_rate (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    rate_bp     INTEGER NOT NULL DEFAULT 0,
    archived_at TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Document numbering sequences.
CREATE TABLE numbering (
    kind         TEXT PRIMARY KEY NOT NULL,
    prefix       TEXT NOT NULL DEFAULT '',
    next_value   INTEGER NOT NULL DEFAULT 1,
    padding      INTEGER NOT NULL DEFAULT 4,
    reset_yearly INTEGER NOT NULL DEFAULT 0 CHECK (reset_yearly IN (0, 1)),
    last_year    INTEGER
);

INSERT INTO numbering (kind, prefix) VALUES ('invoice', 'INV-');

-- Invoices. Money is stored as integer minor units; totals are cached and
-- recomputed from the line items on every write.
CREATE TABLE invoice (
    id                   TEXT PRIMARY KEY NOT NULL,
    number               TEXT NOT NULL UNIQUE,
    client_id            TEXT REFERENCES client (id),
    status               TEXT NOT NULL DEFAULT 'draft',
    issue_date           TEXT NOT NULL,
    due_date             TEXT,
    currency             TEXT NOT NULL DEFAULT 'USD',
    po_reference         TEXT,
    discount_type        TEXT,
    discount_value       TEXT,
    subtotal_minor       INTEGER NOT NULL DEFAULT 0,
    discount_minor       INTEGER NOT NULL DEFAULT 0,
    tax_minor            INTEGER NOT NULL DEFAULT 0,
    total_minor          INTEGER NOT NULL DEFAULT 0,
    notes                TEXT,
    terms                TEXT,
    payment_instructions TEXT,
    voided_at            TEXT,
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_invoice_client ON invoice (client_id);
CREATE INDEX idx_invoice_status ON invoice (status);
CREATE INDEX idx_invoice_issue_date ON invoice (issue_date);

-- Invoice line items. Quantity and discount values are stored as decimal strings
-- to keep money math exact.
CREATE TABLE invoice_item (
    id               TEXT PRIMARY KEY NOT NULL,
    invoice_id       TEXT NOT NULL REFERENCES invoice (id) ON DELETE CASCADE,
    sort_order       INTEGER NOT NULL DEFAULT 0,
    description      TEXT NOT NULL DEFAULT '',
    quantity         TEXT NOT NULL DEFAULT '1',
    unit             TEXT,
    unit_price_minor INTEGER NOT NULL DEFAULT 0,
    discount_type    TEXT,
    discount_value   TEXT,
    tax_rate_id      TEXT REFERENCES tax_rate (id),
    tax_inclusive    INTEGER NOT NULL DEFAULT 0 CHECK (tax_inclusive IN (0, 1)),
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_invoice_item_invoice ON invoice_item (invoice_id);
