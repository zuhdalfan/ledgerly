//! The business logo, kept as a BLOB in the database.
//!
//! It lives here rather than on disk because the app promises one local file and
//! a backup is a copy of this database. A separate table also keeps the image out
//! of every `business::get` call, which the MCP server and each PDF export make.

use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension};

pub struct BusinessLogo {
    pub bytes: Vec<u8>,
    pub mime: String,
}

pub fn get(conn: &Connection) -> rusqlite::Result<Option<BusinessLogo>> {
    conn.query_row(
        "SELECT bytes, mime FROM business_logo WHERE id = 'default'",
        [],
        |row| {
            Ok(BusinessLogo {
                bytes: row.get(0)?,
                mime: row.get(1)?,
            })
        },
    )
    .optional()
}

pub fn set(conn: &Connection, bytes: &[u8], mime: &str) -> rusqlite::Result<BusinessLogo> {
    conn.execute(
        "INSERT INTO business_logo (id, bytes, mime)
         VALUES ('default', ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET
            bytes = excluded.bytes,
            mime = excluded.mime",
        params![bytes, mime],
    )?;

    get(conn)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn clear(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM business_logo WHERE id = 'default'", [])?;
    Ok(())
}

/// A `data:` URL for the settings preview; the CSP already allows `img-src data:`,
/// so no asset-protocol scope is needed.
pub fn data_url(logo: &BusinessLogo) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(&logo.bytes);
    format!("data:{};base64,{encoded}", logo.mime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    #[test]
    fn a_fresh_database_has_no_logo() {
        let t = temp_db();
        assert!(get(&t.conn).unwrap().is_none());
    }

    #[test]
    fn set_get_clear_round_trip() {
        let t = temp_db();

        let stored = set(&t.conn, b"\x89PNG\r\n\x1a\nfake", "image/png").unwrap();
        assert_eq!(stored.mime, "image/png");

        let loaded = get(&t.conn).unwrap().unwrap();
        assert_eq!(loaded.bytes, b"\x89PNG\r\n\x1a\nfake");

        clear(&t.conn).unwrap();
        assert!(get(&t.conn).unwrap().is_none());
    }

    #[test]
    fn setting_twice_replaces_rather_than_duplicates() {
        let t = temp_db();

        set(&t.conn, b"first", "image/png").unwrap();
        set(&t.conn, b"second", "image/jpeg").unwrap();

        let loaded = get(&t.conn).unwrap().unwrap();
        assert_eq!(loaded.bytes, b"second");
        assert_eq!(loaded.mime, "image/jpeg");

        let rows: i64 = t
            .conn
            .query_row("SELECT COUNT(*) FROM business_logo", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 1, "there is only ever one logo");
    }

    #[test]
    fn data_url_carries_the_mime_and_the_bytes() {
        let logo = BusinessLogo {
            bytes: b"abc".to_vec(),
            mime: "image/png".to_string(),
        };
        assert_eq!(data_url(&logo), "data:image/png;base64,YWJj");
    }
}
