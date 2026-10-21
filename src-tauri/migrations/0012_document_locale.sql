-- A document can be printed in a different language from the app UI. NULL means
-- inherit: an invoice or estimate takes its client's language, a client takes
-- Settings > Invoicing, and only then does the global pdf.locale apply. Storing
-- the override rather than a resolved value keeps old documents in step with a
-- client's language until one is explicitly fixed.
ALTER TABLE client ADD COLUMN locale TEXT;
ALTER TABLE invoice ADD COLUMN locale TEXT;
ALTER TABLE estimate ADD COLUMN locale TEXT;
