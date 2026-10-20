pub mod business;
pub mod business_logo;
pub mod catalog;
pub mod clients;
pub mod estimates;
pub mod invoices;
pub mod line_items;
pub mod migrations;
pub mod numbering;
pub mod payments;
pub mod reports;
pub mod settings;
pub mod tax_rates;

use rusqlite::Connection;
use std::path::Path;

/// Open the application database and bring the schema up to date.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    // The MCP server runs as a separate process against the same file, so a
    // brief wait for a lock beats failing immediately with SQLITE_BUSY.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    // journal_mode reports the resulting mode, so read it rather than execute it.
    let _mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::run(&conn)?;
    Ok(conn)
}

/// Trim an optional string, treating an empty result as absent.
pub(crate) fn trim_opt(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
pub(crate) mod tests_support {
    use rusqlite::Connection;
    use std::path::PathBuf;

    /// A throwaway database in a temp directory, removed when dropped.
    pub(crate) struct TempDb {
        pub(crate) dir: PathBuf,
        pub(crate) conn: Connection,
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    pub(crate) fn temp_db() -> TempDb {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ledgerly-test-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        let conn = super::open(&dir.join("ledgerly.db")).unwrap();
        TempDb { dir, conn }
    }

    /// A database with only the first `version` migrations applied, so the rest
    /// can be exercised as a genuine upgrade rather than as a re-run.
    ///
    /// This matters because migrations are not all idempotent: `DROP TABLE IF
    /// EXISTS` can run twice, but `ALTER TABLE ADD COLUMN` cannot.
    pub(crate) fn temp_db_at(version: usize) -> TempDb {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ledgerly-test-{version}-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        let conn = Connection::open(dir.join("ledgerly.db")).unwrap();

        for sql in &super::migrations::MIGRATIONS[..version] {
            conn.execute_batch(&format!("BEGIN;\n{sql}\nCOMMIT;"))
                .unwrap();
        }
        conn.pragma_update(None, "user_version", version as i64)
            .unwrap();

        TempDb { dir, conn }
    }
}
