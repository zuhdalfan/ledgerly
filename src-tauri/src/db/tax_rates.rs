use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const COLUMNS: &str = "id, name, rate_bp, archived_at, created_at, updated_at";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRate {
    pub id: String,
    pub name: String,
    pub rate_bp: i64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRateInput {
    pub name: String,
    #[serde(default)]
    pub rate_bp: i64,
}

fn map_row(row: &Row) -> rusqlite::Result<TaxRate> {
    Ok(TaxRate {
        id: row.get(0)?,
        name: row.get(1)?,
        rate_bp: row.get(2)?,
        archived_at: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn list(conn: &Connection, include_archived: bool) -> rusqlite::Result<Vec<TaxRate>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM tax_rate
         WHERE (?1 = 1 OR archived_at IS NULL)
         ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![include_archived as i64], map_row)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<TaxRate>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM tax_rate WHERE id = ?1"),
        [id],
        map_row,
    )
    .optional()
}

pub fn create(conn: &Connection, input: &TaxRateInput) -> rusqlite::Result<TaxRate> {
    let id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO tax_rate (id, name, rate_bp) VALUES (?1, ?2, ?3)",
        params![id, input.name.trim(), input.rate_bp],
    )?;
    get(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(conn: &Connection, id: &str, input: &TaxRateInput) -> rusqlite::Result<TaxRate> {
    let changed = conn.execute(
        "UPDATE tax_rate SET name = ?2, rate_bp = ?3, updated_at = datetime('now') WHERE id = ?1",
        params![id, input.name.trim(), input.rate_bp],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool) -> rusqlite::Result<TaxRate> {
    let changed = conn.execute(
        "UPDATE tax_rate
         SET archived_at = CASE WHEN ?2 = 1 THEN datetime('now') ELSE NULL END,
             updated_at = datetime('now')
         WHERE id = ?1",
        params![id, archived as i64],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Map of tax rate id to basis points, for total calculations.
pub fn rate_map(conn: &Connection) -> rusqlite::Result<HashMap<String, i64>> {
    let mut stmt = conn.prepare("SELECT id, rate_bp FROM tax_rate")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    fn input(name: &str, bp: i64) -> TaxRateInput {
        TaxRateInput {
            name: name.to_string(),
            rate_bp: bp,
        }
    }

    #[test]
    fn crud_and_rate_map() {
        let t = temp_db();

        let vat = create(&t.conn, &input("VAT", 2_000)).unwrap();
        create(&t.conn, &input("GST", 900)).unwrap();

        assert_eq!(list(&t.conn, false).unwrap().len(), 2);
        assert_eq!(
            rate_map(&t.conn).unwrap().get(&vat.id).copied(),
            Some(2_000)
        );

        update(&t.conn, &vat.id, &input("VAT (reduced)", 1_200)).unwrap();
        assert_eq!(
            rate_map(&t.conn).unwrap().get(&vat.id).copied(),
            Some(1_200)
        );

        set_archived(&t.conn, &vat.id, true).unwrap();
        assert_eq!(list(&t.conn, false).unwrap().len(), 1);
        assert_eq!(list(&t.conn, true).unwrap().len(), 2);
    }
}
