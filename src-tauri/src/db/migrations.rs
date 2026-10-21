use rusqlite::Connection;

/// Ordered schema migrations. Index 0 is migration version 1.
///
/// **Append only.** An entry that has run on any database can never be edited:
/// the schema version is already recorded, so the change would silently never
/// apply there. Fix forward with a new migration instead.
pub(crate) const MIGRATIONS: &[&str] = &[
    include_str!("../../migrations/0001_init.sql"),
    include_str!("../../migrations/0002_identity.sql"),
    include_str!("../../migrations/0003_invoices.sql"),
    include_str!("../../migrations/0004_payments.sql"),
    include_str!("../../migrations/0005_estimates.sql"),
    include_str!("../../migrations/0006_licensing.sql"),
    include_str!("../../migrations/0007_security.sql"),
    include_str!("../../migrations/0008_drop_licensing.sql"),
    include_str!("../../migrations/0009_chasing.sql"),
    include_str!("../../migrations/0010_business_logo.sql"),
    include_str!("../../migrations/0011_business_logo_drop_updated_at.sql"),
    include_str!("../../migrations/0012_document_locale.sql"),
];

/// Apply every migration newer than the database's current `user_version`.
pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    let mut version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    while (version as usize) < MIGRATIONS.len() {
        let sql = MIGRATIONS[version as usize];
        conn.execute_batch(&format!("BEGIN;\n{sql}\nCOMMIT;"))?;
        version += 1;
        conn.pragma_update(None, "user_version", version)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        tax_rates,
        tests_support::{temp_db, temp_db_at},
    };

    #[test]
    fn upgrades_an_older_database_incrementally() {
        let t = temp_db();

        // Simulate a database created before the invoice/estimate schemas existed.
        t.conn
            .execute_batch(
                "DROP TABLE app_lock;
                 DROP TABLE business_logo;
                 DROP TABLE estimate_item;
                 DROP TABLE estimate;
                 DROP TABLE payment;
                 DROP TABLE invoice_item;
                 DROP TABLE invoice;
                 DROP TABLE tax_rate;
                 DROP TABLE numbering;
                 -- Version 2 predates document languages, and client survives
                 -- the drops above, so take its column back off to match.
                 ALTER TABLE client DROP COLUMN locale;
                 PRAGMA user_version = 2;",
            )
            .unwrap();

        run(&t.conn).unwrap();

        let version: i64 = t
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version as usize, MIGRATIONS.len());

        // The recreated tables are present and usable.
        assert!(tax_rates::list(&t.conn, true).unwrap().is_empty());
    }

    #[test]
    fn re_running_is_a_no_op() {
        let t = temp_db();
        let before = tax_rates::list(&t.conn, true).unwrap().len();

        run(&t.conn).unwrap();
        run(&t.conn).unwrap();

        assert_eq!(tax_rates::list(&t.conn, true).unwrap().len(), before);
    }

    #[test]
    fn drops_the_retired_licensing_tables() {
        // A database as it stood at v7: the retired tables are still present,
        // and the invoice table predates the chasing columns.
        let t = temp_db_at(7);

        let before: i64 = t
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('installation', 'license', 'trial')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            before, 3,
            "the fixture should start with the retired tables"
        );

        run(&t.conn).unwrap();

        let after: i64 = t
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name IN ('installation', 'license', 'trial')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(after, 0, "the retired licensing tables should be gone");
    }

    #[test]
    fn an_older_database_gains_the_chasing_columns() {
        let t = temp_db_at(8);

        run(&t.conn).unwrap();

        let columns = table_columns(&t.conn, "invoice");
        assert!(
            columns.iter().any(|name| name == "last_chased_at"),
            "chasing should have added last_chased_at, got {columns:?}"
        );
        assert!(
            columns.iter().any(|name| name == "chase_count"),
            "chasing should have added chase_count, got {columns:?}"
        );
        assert_eq!(current_version(&t.conn), MIGRATIONS.len());
    }

    /// Migration 0010 first shipped an `updated_at` column that nothing reads and
    /// no insert supplies. A database that ran that revision keeps the column, so
    /// without 0011 every logo upload fails with NOT NULL. This reproduces the
    /// shape and proves the repair keeps the stored logo.
    #[test]
    fn repairs_a_business_logo_table_left_with_the_old_updated_at_column() {
        let t = temp_db();

        t.conn
            .execute_batch(
                "DROP TABLE business_logo;
                 CREATE TABLE business_logo (
                     id         TEXT PRIMARY KEY CHECK (id = 'default'),
                     bytes      BLOB NOT NULL,
                     mime       TEXT NOT NULL,
                     updated_at TEXT NOT NULL
                 );
                 INSERT INTO business_logo (id, bytes, mime, updated_at)
                     VALUES ('default', x'89504E47', 'image/png', datetime('now'));
                 -- Version 10 predates document languages; these tables are not
                 -- rebuilt below, so take their columns off to match.
                 ALTER TABLE client DROP COLUMN locale;
                 ALTER TABLE invoice DROP COLUMN locale;
                 ALTER TABLE estimate DROP COLUMN locale;
                 PRAGMA user_version = 10;",
            )
            .unwrap();

        run(&t.conn).unwrap();

        let columns = table_columns(&t.conn, "business_logo");
        assert!(
            !columns.iter().any(|name| name == "updated_at"),
            "the unused column should be gone, got {columns:?}"
        );

        // The stored logo survived the rebuild, and an insert that omits the
        // column now succeeds.
        let mime: String = t
            .conn
            .query_row(
                "SELECT mime FROM business_logo WHERE id = 'default'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(mime, "image/png");

        crate::db::business_logo::set(&t.conn, b"\x89PNG", "image/png").unwrap();
        assert_eq!(current_version(&t.conn), MIGRATIONS.len());
    }

    fn table_columns(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .map(|name| name.unwrap())
            .collect()
    }

    fn current_version(conn: &Connection) -> usize {
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap() as usize
    }
}
