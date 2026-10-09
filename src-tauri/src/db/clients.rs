use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::trim_opt;

const COLUMNS: &str = "id, display_name, company, contact_person, email, phone, \
                       address_line1, address_line2, city, region, postal_code, country, \
                       tax_id, currency, notes, archived_at, created_at, updated_at";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: String,
    pub display_name: String,
    pub company: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub tax_id: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInput {
    pub display_name: String,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub contact_person: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub address_line1: Option<String>,
    #[serde(default)]
    pub address_line2: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub tax_id: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn map_row(row: &Row) -> rusqlite::Result<Client> {
    Ok(Client {
        id: row.get(0)?,
        display_name: row.get(1)?,
        company: row.get(2)?,
        contact_person: row.get(3)?,
        email: row.get(4)?,
        phone: row.get(5)?,
        address_line1: row.get(6)?,
        address_line2: row.get(7)?,
        city: row.get(8)?,
        region: row.get(9)?,
        postal_code: row.get(10)?,
        country: row.get(11)?,
        tax_id: row.get(12)?,
        currency: row.get(13)?,
        notes: row.get(14)?,
        archived_at: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

pub fn list(
    conn: &Connection,
    query: Option<&str>,
    include_archived: bool,
) -> rusqlite::Result<Vec<Client>> {
    let search = query
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(|q| format!("%{q}%"));

    let sql = format!(
        "SELECT {COLUMNS} FROM client
         WHERE (?1 = 1 OR archived_at IS NULL)
           AND (?2 IS NULL
                OR display_name LIKE ?2
                OR IFNULL(company, '') LIKE ?2
                OR IFNULL(email, '') LIKE ?2)
         ORDER BY display_name COLLATE NOCASE"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![include_archived as i64, search], map_row)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Client>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM client WHERE id = ?1"),
        [id],
        map_row,
    )
    .optional()
}

pub fn create(conn: &Connection, input: &ClientInput) -> rusqlite::Result<Client> {
    let id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO client (
            id, display_name, company, contact_person, email, phone,
            address_line1, address_line2, city, region, postal_code, country,
            tax_id, currency, notes
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            id,
            input.display_name.trim(),
            trim_opt(&input.company),
            trim_opt(&input.contact_person),
            trim_opt(&input.email),
            trim_opt(&input.phone),
            trim_opt(&input.address_line1),
            trim_opt(&input.address_line2),
            trim_opt(&input.city),
            trim_opt(&input.region),
            trim_opt(&input.postal_code),
            trim_opt(&input.country),
            trim_opt(&input.tax_id),
            trim_opt(&input.currency),
            trim_opt(&input.notes),
        ],
    )?;
    get(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(conn: &Connection, id: &str, input: &ClientInput) -> rusqlite::Result<Client> {
    let changed = conn.execute(
        "UPDATE client SET
            display_name = ?2, company = ?3, contact_person = ?4, email = ?5, phone = ?6,
            address_line1 = ?7, address_line2 = ?8, city = ?9, region = ?10,
            postal_code = ?11, country = ?12, tax_id = ?13, currency = ?14, notes = ?15,
            updated_at = datetime('now')
         WHERE id = ?1",
        params![
            id,
            input.display_name.trim(),
            trim_opt(&input.company),
            trim_opt(&input.contact_person),
            trim_opt(&input.email),
            trim_opt(&input.phone),
            trim_opt(&input.address_line1),
            trim_opt(&input.address_line2),
            trim_opt(&input.city),
            trim_opt(&input.region),
            trim_opt(&input.postal_code),
            trim_opt(&input.country),
            trim_opt(&input.tax_id),
            trim_opt(&input.currency),
            trim_opt(&input.notes),
        ],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool) -> rusqlite::Result<Client> {
    let changed = conn.execute(
        "UPDATE client
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

    fn input(name: &str) -> ClientInput {
        ClientInput {
            display_name: name.to_string(),
            company: Some("  ".to_string()),
            contact_person: None,
            email: Some(format!("{}@example.com", name.to_lowercase())),
            phone: None,
            address_line1: Some("1 Main St".to_string()),
            address_line2: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            tax_id: None,
            currency: None,
            notes: None,
        }
    }

    #[test]
    fn create_get_update_round_trip() {
        let t = temp_db();

        let created = create(&t.conn, &input("Acme")).unwrap();
        assert_eq!(created.display_name, "Acme");
        assert_eq!(created.company, None, "blank strings are stored as NULL");

        let fetched = get(&t.conn, &created.id).unwrap().unwrap();
        assert_eq!(fetched.email.as_deref(), Some("acme@example.com"));

        let mut changed = input("Acme");
        changed.display_name = "Acme Ltd".to_string();
        let updated = update(&t.conn, &created.id, &changed).unwrap();
        assert_eq!(updated.display_name, "Acme Ltd");
        assert_eq!(updated.created_at, created.created_at);
    }

    #[test]
    fn search_and_archive() {
        let t = temp_db();

        create(&t.conn, &input("Acme")).unwrap();
        create(&t.conn, &input("Globex")).unwrap();

        assert_eq!(list(&t.conn, None, false).unwrap().len(), 2);
        assert_eq!(list(&t.conn, Some("glob"), false).unwrap().len(), 1);
        assert_eq!(list(&t.conn, Some("nobody"), false).unwrap().len(), 0);

        let acme = list(&t.conn, Some("acme"), false).unwrap().pop().unwrap();
        set_archived(&t.conn, &acme.id, true).unwrap();

        assert_eq!(list(&t.conn, None, false).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, true).unwrap().len(), 2);

        set_archived(&t.conn, &acme.id, false).unwrap();
        assert_eq!(list(&t.conn, None, false).unwrap().len(), 2);
    }

    #[test]
    fn update_missing_client_errors() {
        let t = temp_db();
        assert!(update(&t.conn, "missing", &input("X")).is_err());
    }
}
