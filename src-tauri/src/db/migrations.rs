use rusqlite::Connection;

/// Ordered schema migrations. Index 0 is migration version 1.
pub(crate) const MIGRATIONS: &[&str] = &[
    include_str!("../../migrations/0001_init.sql"),
    include_str!("../../migrations/0002_identity.sql"),
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
