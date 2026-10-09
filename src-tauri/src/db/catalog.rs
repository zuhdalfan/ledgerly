use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::trim_opt;

const COLUMNS: &str = "id, name, description, unit, unit_price_minor, currency, sku, \
                       archived_at, created_at, updated_at";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub unit_price_minor: i64,
    pub currency: Option<String>,
    pub sku: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub unit_price_minor: i64,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub sku: Option<String>,
}

fn map_row(row: &Row) -> rusqlite::Result<CatalogItem> {
    Ok(CatalogItem {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        unit: row.get(3)?,
        unit_price_minor: row.get(4)?,
        currency: row.get(5)?,
        sku: row.get(6)?,
        archived_at: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub fn list(
    conn: &Connection,
    query: Option<&str>,
    include_archived: bool,
) -> rusqlite::Result<Vec<CatalogItem>> {
    let search = query
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(|q| format!("%{q}%"));

    let sql = format!(
        "SELECT {COLUMNS} FROM catalog_item
         WHERE (?1 = 1 OR archived_at IS NULL)
           AND (?2 IS NULL
                OR name LIKE ?2
                OR IFNULL(sku, '') LIKE ?2)
         ORDER BY name COLLATE NOCASE"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![include_archived as i64, search], map_row)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<CatalogItem>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM catalog_item WHERE id = ?1"),
        [id],
        map_row,
    )
    .optional()
}

pub fn create(conn: &Connection, input: &CatalogItemInput) -> rusqlite::Result<CatalogItem> {
    let id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO catalog_item (
            id, name, description, unit, unit_price_minor, currency, sku
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.name.trim(),
            trim_opt(&input.description),
            trim_opt(&input.unit),
            input.unit_price_minor,
            trim_opt(&input.currency),
            trim_opt(&input.sku),
        ],
    )?;
    get(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &CatalogItemInput,
) -> rusqlite::Result<CatalogItem> {
    let changed = conn.execute(
        "UPDATE catalog_item SET
            name = ?2, description = ?3, unit = ?4, unit_price_minor = ?5,
            currency = ?6, sku = ?7, updated_at = datetime('now')
         WHERE id = ?1",
        params![
            id,
            input.name.trim(),
            trim_opt(&input.description),
            trim_opt(&input.unit),
            input.unit_price_minor,
            trim_opt(&input.currency),
            trim_opt(&input.sku),
        ],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool) -> rusqlite::Result<CatalogItem> {
    let changed = conn.execute(
        "UPDATE catalog_item
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    fn input(name: &str, price: i64) -> CatalogItemInput {
        CatalogItemInput {
            name: name.to_string(),
            description: Some("Design work".to_string()),
            unit: Some("hour".to_string()),
            unit_price_minor: price,
            currency: Some("USD".to_string()),
            sku: None,
        }
    }

    #[test]
    fn create_search_and_archive() {
        let t = temp_db();

        let item = create(&t.conn, &input("Design hour", 12_500)).unwrap();
        assert_eq!(item.unit_price_minor, 12_500);
        assert_eq!(item.unit.as_deref(), Some("hour"));

        create(&t.conn, &input("Consulting day", 90_000)).unwrap();

        assert_eq!(list(&t.conn, None, false).unwrap().len(), 2);
        assert_eq!(list(&t.conn, Some("design"), false).unwrap().len(), 1);

        set_archived(&t.conn, &item.id, true).unwrap();
        assert_eq!(list(&t.conn, None, false).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, true).unwrap().len(), 2);
    }

    #[test]
    fn update_round_trip() {
        let t = temp_db();

        let item = create(&t.conn, &input("Design hour", 12_500)).unwrap();
        let mut changed = input("Design hour", 15_000);
        changed.name = "Design hour (senior)".to_string();

        let updated = update(&t.conn, &item.id, &changed).unwrap();
        assert_eq!(updated.name, "Design hour (senior)");
        assert_eq!(updated.unit_price_minor, 15_000);
    }
}
