-- Payments recorded against an invoice. The invoice's paid/balance figures and
-- its effective status (paid / partially paid / overdue) are derived from these.
CREATE TABLE payment (
    id           TEXT PRIMARY KEY NOT NULL,
    invoice_id   TEXT NOT NULL REFERENCES invoice (id) ON DELETE CASCADE,
    paid_on      TEXT NOT NULL,
    amount_minor INTEGER NOT NULL,
    method       TEXT,
    reference    TEXT,
    note         TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_payment_invoice ON payment (invoice_id);

-- When the invoice was marked as sent (used for aging).
ALTER TABLE invoice ADD COLUMN sent_at TEXT;
