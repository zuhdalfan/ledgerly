use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

pub fn all(conn: &Connection) -> rusqlite::Result<Vec<Setting>> {
    let mut stmt = conn.prepare("SELECT key, value, updated_at FROM setting ORDER BY key")?;
    let rows = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM setting WHERE key = ?1", [key], |row| {
        row.get(0)
    })
    .optional()
}

pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO setting (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        (key, value),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, tests_support::temp_db};

    #[test]
    fn migrations_are_applied() {
        let t = temp_db();
        let version: i64 = t
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version as usize, migrations::MIGRATIONS.len());
    }

    #[test]
    fn settings_round_trip_and_upsert() {
        let t = temp_db();

        assert_eq!(get(&t.conn, "business.name").unwrap(), None);

        set(&t.conn, "business.name", "Acme").unwrap();
        assert_eq!(
            get(&t.conn, "business.name").unwrap().as_deref(),
            Some("Acme")
        );

        set(&t.conn, "business.name", "Acme Ltd").unwrap();
        assert_eq!(
            get(&t.conn, "business.name").unwrap().as_deref(),
            Some("Acme Ltd")
        );

        let all = all(&t.conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].value, "Acme Ltd");
    }
}
