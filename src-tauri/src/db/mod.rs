pub mod migrations;
pub mod settings;

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
