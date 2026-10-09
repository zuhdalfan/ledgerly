pub mod business;
pub mod catalog;
pub mod clients;
pub mod invoices;
pub mod migrations;
pub mod numbering;
pub mod payments;
pub mod settings;
pub mod tax_rates;

use rusqlite::Connection;
use std::path::Path;

/// Open the application database and bring the schema up to date.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
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
}
