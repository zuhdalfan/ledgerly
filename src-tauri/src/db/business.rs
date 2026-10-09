use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::trim_opt;

const COLUMNS: &str = "id, display_name, legal_name, email, phone, website, \
                       address_line1, address_line2, city, region, postal_code, country, \
                       tax_id, default_currency, default_terms, default_notes, \
                       default_payment_instructions, updated_at";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub id: String,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub tax_id: Option<String>,
    pub default_currency: String,
    pub default_terms: Option<String>,
    pub default_notes: Option<String>,
    pub default_payment_instructions: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessInput {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub legal_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
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
    pub default_currency: String,
    #[serde(default)]
    pub default_terms: Option<String>,
    #[serde(default)]
    pub default_notes: Option<String>,
    #[serde(default)]
    pub default_payment_instructions: Option<String>,
}

fn map_row(row: &Row) -> rusqlite::Result<Business> {
    Ok(Business {
        id: row.get(0)?,
        display_name: row.get(1)?,
        legal_name: row.get(2)?,
        email: row.get(3)?,
        phone: row.get(4)?,
        website: row.get(5)?,
        address_line1: row.get(6)?,
        address_line2: row.get(7)?,
        city: row.get(8)?,
        region: row.get(9)?,
        postal_code: row.get(10)?,
        country: row.get(11)?,
        tax_id: row.get(12)?,
        default_currency: row.get(13)?,
        default_terms: row.get(14)?,
        default_notes: row.get(15)?,
        default_payment_instructions: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

pub fn get(conn: &Connection) -> rusqlite::Result<Business> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM business WHERE id = 'default'"),
        [],
        map_row,
    )
}

pub fn update(conn: &Connection, input: &BusinessInput) -> rusqlite::Result<Business> {
    let trimmed_currency = input.default_currency.trim();
    let currency = if trimmed_currency.is_empty() {
        "USD"
    } else {
        trimmed_currency
    };

    conn.execute(
        "UPDATE business SET
            display_name = ?1, legal_name = ?2, email = ?3, phone = ?4, website = ?5,
            address_line1 = ?6, address_line2 = ?7, city = ?8, region = ?9,
            postal_code = ?10, country = ?11, tax_id = ?12, default_currency = ?13,
            default_terms = ?14, default_notes = ?15, default_payment_instructions = ?16,
            updated_at = datetime('now')
         WHERE id = 'default'",
        params![
            input.display_name.trim(),
            trim_opt(&input.legal_name),
            trim_opt(&input.email),
            trim_opt(&input.phone),
            trim_opt(&input.website),
            trim_opt(&input.address_line1),
            trim_opt(&input.address_line2),
            trim_opt(&input.city),
            trim_opt(&input.region),
            trim_opt(&input.postal_code),
            trim_opt(&input.country),
            trim_opt(&input.tax_id),
            currency,
            trim_opt(&input.default_terms),
            trim_opt(&input.default_notes),
            trim_opt(&input.default_payment_instructions),
        ],
    )?;

    get(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    fn input() -> BusinessInput {
        BusinessInput {
            display_name: "  Zuhdi Studio  ".to_string(),
            legal_name: None,
            email: Some("hi@example.com".to_string()),
            phone: None,
            website: None,
            address_line1: None,
            address_line2: None,
            city: Some("Singapore".to_string()),
            region: None,
            postal_code: None,
            country: Some("SG".to_string()),
            tax_id: None,
            default_currency: "sgd".to_string(),
            default_terms: Some("Net 30".to_string()),
            default_notes: None,
            default_payment_instructions: None,
        }
    }

    #[test]
    fn defaults_exist_on_fresh_database() {
        let t = temp_db();
        let business = get(&t.conn).unwrap();
        assert_eq!(business.display_name, "");
        assert_eq!(business.default_currency, "USD");
    }

    #[test]
    fn update_round_trip() {
        let t = temp_db();

        let updated = update(&t.conn, &input()).unwrap();
        assert_eq!(updated.display_name, "Zuhdi Studio");
        assert_eq!(updated.default_currency, "sgd");
        assert_eq!(updated.city.as_deref(), Some("Singapore"));

        let reloaded = get(&t.conn).unwrap();
        assert_eq!(reloaded.default_terms.as_deref(), Some("Net 30"));
    }

    #[test]
    fn blank_currency_falls_back_to_usd() {
        let t = temp_db();

        let mut blank = input();
        blank.default_currency = "   ".to_string();
        assert_eq!(update(&t.conn, &blank).unwrap().default_currency, "USD");
    }
}
