use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

const COLUMNS: &str = "kind, prefix, next_value, padding, reset_yearly, last_year";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Numbering {
    pub kind: String,
    pub prefix: String,
    pub next_value: i64,
    pub padding: i64,
    pub reset_yearly: bool,
    pub last_year: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberingInput {
    #[serde(default)]
    pub prefix: String,
    #[serde(default = "default_next_value")]
    pub next_value: i64,
    #[serde(default = "default_padding")]
    pub padding: i64,
    #[serde(default)]
    pub reset_yearly: bool,
}

fn default_next_value() -> i64 {
    1
}

fn default_padding() -> i64 {
    4
}

fn map_row(row: &Row) -> rusqlite::Result<Numbering> {
    Ok(Numbering {
        kind: row.get(0)?,
        prefix: row.get(1)?,
        next_value: row.get(2)?,
        padding: row.get(3)?,
        reset_yearly: row.get::<_, i64>(4)? != 0,
        last_year: row.get(5)?,
    })
}

/// Read a sequence, creating a default row on first use.
pub fn get(conn: &Connection, kind: &str) -> rusqlite::Result<Numbering> {
    conn.execute("INSERT OR IGNORE INTO numbering (kind) VALUES (?1)", [kind])?;
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM numbering WHERE kind = ?1"),
        [kind],
        map_row,
    )
}

pub fn update(
    conn: &Connection,
    kind: &str,
    input: &NumberingInput,
) -> rusqlite::Result<Numbering> {
    get(conn, kind)?;
    conn.execute(
        "UPDATE numbering SET prefix = ?2, next_value = ?3, padding = ?4, reset_yearly = ?5
         WHERE kind = ?1",
        params![
            kind,
            input.prefix.trim(),
            input.next_value.max(1),
            input.padding.clamp(1, 12),
            input.reset_yearly as i64
        ],
    )?;
    get(conn, kind)
}

/// Consume the next number for `kind`, resetting first if the sequence is annual
/// and the year has rolled over.
pub fn next_number(conn: &Connection, kind: &str, year: i64) -> rusqlite::Result<String> {
    let current = get(conn, kind)?;

    let reset = current.reset_yearly && current.last_year != Some(year);
    let value = if reset { 1 } else { current.next_value.max(1) };
    let width = current.padding.clamp(1, 12) as usize;

    let number = if current.reset_yearly {
        format!("{}{year}-{value:0width$}", current.prefix)
    } else {
        format!("{}{value:0width$}", current.prefix)
    };

    let last_year = if current.reset_yearly {
        Some(year)
    } else {
        current.last_year
    };
    conn.execute(
        "UPDATE numbering SET next_value = ?2, last_year = ?3 WHERE kind = ?1",
        params![kind, value + 1, last_year],
    )?;

    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    fn config(prefix: &str, reset_yearly: bool) -> NumberingInput {
        NumberingInput {
            prefix: prefix.to_string(),
            next_value: 1,
            padding: 4,
            reset_yearly,
        }
    }

    #[test]
    fn sequence_increments_and_pads() {
        let t = temp_db();
        update(&t.conn, "invoice", &config("INV-", false)).unwrap();

        assert_eq!(next_number(&t.conn, "invoice", 2026).unwrap(), "INV-0001");
        assert_eq!(next_number(&t.conn, "invoice", 2026).unwrap(), "INV-0002");
        assert_eq!(get(&t.conn, "invoice").unwrap().next_value, 3);
    }

    #[test]
    fn annual_sequence_includes_and_resets_on_the_year() {
        let t = temp_db();
        update(&t.conn, "invoice", &config("INV-", true)).unwrap();

        assert_eq!(
            next_number(&t.conn, "invoice", 2026).unwrap(),
            "INV-2026-0001"
        );
        assert_eq!(
            next_number(&t.conn, "invoice", 2026).unwrap(),
            "INV-2026-0002"
        );
        assert_eq!(
            next_number(&t.conn, "invoice", 2027).unwrap(),
            "INV-2027-0001"
        );
    }

    #[test]
    fn next_value_can_be_seeded() {
        let t = temp_db();
        let mut seeded = config("INV-", false);
        seeded.next_value = 42;
        update(&t.conn, "invoice", &seeded).unwrap();

        assert_eq!(next_number(&t.conn, "invoice", 2026).unwrap(), "INV-0042");
    }
}
