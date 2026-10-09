-- Estimates / quotes. Same shape as an invoice minus payments, plus a validity
-- date and a link to the invoice it was converted into.
CREATE TABLE estimate (
    id                   TEXT PRIMARY KEY NOT NULL,
    number               TEXT NOT NULL UNIQUE,
    client_id            TEXT REFERENCES client (id),
    status               TEXT NOT NULL DEFAULT 'draft',
    issue_date           TEXT NOT NULL,
    valid_until          TEXT,
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
    converted_invoice_id TEXT REFERENCES invoice (id),
    converted_at         TEXT,
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_estimate_client ON estimate (client_id);
CREATE INDEX idx_estimate_status ON estimate (status);

CREATE TABLE estimate_item (
    id               TEXT PRIMARY KEY NOT NULL,
    estimate_id      TEXT NOT NULL REFERENCES estimate (id) ON DELETE CASCADE,
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

CREATE INDEX idx_estimate_item_estimate ON estimate_item (estimate_id);

-- Estimates get their own numbering sequence.
INSERT INTO numbering (kind, prefix) VALUES ('estimate', 'EST-');

-- Link an invoice back to the estimate it came from.
ALTER TABLE invoice ADD COLUMN estimate_id TEXT REFERENCES estimate (id);
