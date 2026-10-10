use rusqlite::Connection;

/// Ordered schema migrations. Index 0 is migration version 1.
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
                 DROP TABLE estimate_item;
                 DROP TABLE estimate;
                 DROP TABLE payment;
                 DROP TABLE invoice_item;
                 DROP TABLE invoice;
                 DROP TABLE tax_rate;
                 DROP TABLE numbering;
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

        let mut stmt = t.conn.prepare("PRAGMA table_info(invoice)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .map(|name| name.unwrap())
            .collect();

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

    fn current_version(conn: &Connection) -> usize {
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap() as usize
    }
}
