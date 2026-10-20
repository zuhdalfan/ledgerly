-- 0010 first created this table with an `updated_at` column that nothing reads.
--
-- A migration that has already run cannot be edited: databases that applied the
-- earlier revision of 0010 still carry the column, and because it is NOT NULL
-- with no default, every insert that omits it now fails. So the column is
-- removed here instead, by rebuilding the table and carrying the row across.
--
-- The INSERT names only the columns both shapes share, which makes this safe
-- whether or not `updated_at` is present.
CREATE TABLE business_logo_new (
    id    TEXT PRIMARY KEY CHECK (id = 'default'),
    bytes BLOB NOT NULL,
    mime  TEXT NOT NULL
);

INSERT INTO business_logo_new (id, bytes, mime)
    SELECT id, bytes, mime FROM business_logo;

DROP TABLE business_logo;

ALTER TABLE business_logo_new RENAME TO business_logo;
