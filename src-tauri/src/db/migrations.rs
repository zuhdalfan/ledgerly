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
    use crate::db::{tax_rates, tests_support::temp_db};

    #[test]
    fn upgrades_an_older_database_incrementally() {
        let t = temp_db();

        // Simulate a database created before the invoice/estimate schemas existed.
        t.conn
            .execute_batch(
                "DROP TABLE app_lock;
                 DROP TABLE trial;
                 DROP TABLE license;
                 DROP TABLE installation;
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
}
