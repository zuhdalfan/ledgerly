//! Estimates (quotes).
//!
//! Structurally an invoice without payments, plus a validity date and a link to
//! the invoice it was converted into. Line items and totals are shared with
//! invoices so both documents calculate money identically.

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::invoices::{self, Invoice, InvoiceInput};
use super::line_items::{self, LineItemInput};
use super::{numbering, trim_opt};
use crate::domain::totals::InvoiceTotals;
use crate::error::AppError;

const DAYS_UNTIL_EXPIRY_EXPR: &str =
    "CAST(julianday(i.valid_until) - julianday(date('now', 'localtime')) AS INTEGER)";

fn columns() -> String {
    format!(
        "i.id, i.number, i.client_id, i.status, i.issue_date, i.valid_until, i.currency, \
         i.po_reference, i.discount_type, i.discount_value, i.subtotal_minor, i.discount_minor, \
         i.tax_minor, i.total_minor, i.notes, i.terms, i.payment_instructions, \
         i.converted_invoice_id, i.converted_at, i.created_at, i.updated_at, c.display_name, \
         {DAYS_UNTIL_EXPIRY_EXPR} AS days_until_expiry, i.locale, c.locale"
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateItem {
    pub id: String,
    pub sort_order: i64,
    pub description: String,
    pub quantity: String,
    pub unit: Option<String>,
    pub unit_price_minor: i64,
    pub discount_type: Option<String>,
    pub discount_value: Option<String>,
    pub tax_rate_id: Option<String>,
    pub tax_rate_name: Option<String>,
    pub tax_rate_bp: Option<i64>,
    pub tax_inclusive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub id: String,
    pub number: String,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub status: String,
    pub issue_date: String,
    pub valid_until: Option<String>,
    pub currency: String,
    pub po_reference: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<String>,
    pub subtotal_minor: i64,
    pub discount_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub payment_instructions: Option<String>,
    pub converted_invoice_id: Option<String>,
    pub converted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub days_until_expiry: Option<i64>,
    /// Derived: an unaccepted estimate past its validity date reads as expired.
    pub display_status: String,
    /// Document language override; `None` inherits the client's language.
    pub locale: Option<String>,
    /// The client's own override, so a reader can resolve the fallback without
    /// a second query. `None` inherits the global default.
    pub client_locale: Option<String>,
    pub items: Vec<EstimateItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateInput {
    #[serde(default)]
    pub client_id: Option<String>,
    pub issue_date: String,
    #[serde(default)]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub po_reference: Option<String>,
    #[serde(default)]
    pub discount_type: Option<String>,
    #[serde(default)]
    pub discount_value: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub terms: Option<String>,
    #[serde(default)]
    pub payment_instructions: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub items: Vec<LineItemInput>,
}

fn display_status(status: &str, days_until_expiry: Option<i64>) -> String {
    match status {
        "converted" | "accepted" | "declined" => status.to_string(),
        _ if days_until_expiry.unwrap_or(0) < 0 => "expired".to_string(),
        _ => status.to_string(),
    }
}

fn map_estimate(row: &Row, items: Vec<EstimateItem>) -> rusqlite::Result<Estimate> {
    let status: String = row.get(3)?;
    let days_until_expiry: Option<i64> = row.get(22)?;

    Ok(Estimate {
        id: row.get(0)?,
        number: row.get(1)?,
        client_id: row.get(2)?,
        status: status.clone(),
        issue_date: row.get(4)?,
        valid_until: row.get(5)?,
        currency: row.get(6)?,
        po_reference: row.get(7)?,
        discount_type: row.get(8)?,
        discount_value: row.get(9)?,
        subtotal_minor: row.get(10)?,
        discount_minor: row.get(11)?,
        tax_minor: row.get(12)?,
        total_minor: row.get(13)?,
        notes: row.get(14)?,
        terms: row.get(15)?,
        payment_instructions: row.get(16)?,
        converted_invoice_id: row.get(17)?,
        converted_at: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
        client_name: row.get(21)?,
        days_until_expiry,
        display_status: display_status(&status, days_until_expiry),
        locale: row.get(23)?,
        client_locale: row.get(24)?,
        items,
    })
}

pub fn load_items(conn: &Connection, estimate_id: &str) -> rusqlite::Result<Vec<EstimateItem>> {
    let mut stmt = conn.prepare(
        "SELECT it.id, it.sort_order, it.description, it.quantity, it.unit,
                it.unit_price_minor, it.discount_type, it.discount_value,
                it.tax_rate_id, t.name, t.rate_bp, it.tax_inclusive
         FROM estimate_item it
         LEFT JOIN tax_rate t ON t.id = it.tax_rate_id
         WHERE it.estimate_id = ?1
         ORDER BY it.sort_order, it.created_at",
    )?;
    let rows = stmt.query_map([estimate_id], |row| {
        Ok(EstimateItem {
            id: row.get(0)?,
            sort_order: row.get(1)?,
            description: row.get(2)?,
            quantity: row.get(3)?,
            unit: row.get(4)?,
            unit_price_minor: row.get(5)?,
            discount_type: row.get(6)?,
            discount_value: row.get(7)?,
            tax_rate_id: row.get(8)?,
            tax_rate_name: row.get(9)?,
            tax_rate_bp: row.get(10)?,
            tax_inclusive: row.get::<_, i64>(11)? != 0,
        })
    })?;
    rows.collect()
}

pub fn list(
    conn: &Connection,
    query: Option<&str>,
    status: Option<&str>,
    client_id: Option<&str>,
) -> rusqlite::Result<Vec<Estimate>> {
    let search = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"));
    let status = status.map(str::trim).filter(|value| !value.is_empty());
    let client_id = client_id.map(str::trim).filter(|value| !value.is_empty());

    let sql = format!(
        // Filters match the *derived* status, so an expired draft appears under
        // "expired" and not under "draft" — the same thing the badge shows.
        "SELECT {columns} FROM estimate i
         LEFT JOIN client c ON c.id = i.client_id
         WHERE (?1 IS NULL
                OR (?1 = 'expired' AND i.status IN ('draft', 'sent') AND {days} < 0)
                OR (?1 IN ('draft', 'sent')
                                   AND i.status = ?1
                                   AND (i.valid_until IS NULL OR {days} >= 0))
                OR (?1 IN ('accepted', 'declined', 'converted') AND i.status = ?1))
           AND (?2 IS NULL OR i.number LIKE ?2 OR IFNULL(c.display_name, '') LIKE ?2)
           AND (?3 IS NULL OR i.client_id = ?3)
         ORDER BY i.issue_date DESC, i.number DESC",
        columns = columns(),
        days = DAYS_UNTIL_EXPIRY_EXPR
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![status, search, client_id], |row| {
        map_estimate(row, Vec::new())
    })?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Estimate>> {
    let estimate = conn
        .query_row(
            &format!(
                "SELECT {} FROM estimate i
                 LEFT JOIN client c ON c.id = i.client_id
                 WHERE i.id = ?1",
                columns()
            ),
            [id],
            |row| map_estimate(row, Vec::new()),
        )
        .optional()?;

    match estimate {
        None => Ok(None),
        Some(mut estimate) => {
            estimate.items = load_items(conn, id)?;
            Ok(Some(estimate))
        }
    }
}

pub fn preview_totals(conn: &Connection, input: &EstimateInput) -> rusqlite::Result<InvoiceTotals> {
    line_items::preview_totals(
        conn,
        &input.discount_type,
        &input.discount_value,
        &input.items,
    )
}

fn insert_estimate(conn: &Connection, input: &EstimateInput) -> rusqlite::Result<String> {
    let id = uuid::Uuid::now_v7().to_string();
    let year = line_items::year_of(conn, &input.issue_date)?;
    let number = numbering::next_number(conn, "estimate", year)?;
    let totals = preview_totals(conn, input)?;

    conn.execute(
        "INSERT INTO estimate (
            id, number, client_id, status, issue_date, valid_until, currency, po_reference,
            discount_type, discount_value, subtotal_minor, discount_minor, tax_minor,
            total_minor, notes, terms, payment_instructions, locale
         ) VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            id,
            number,
            trim_opt(&input.client_id),
            input.issue_date.trim(),
            trim_opt(&input.valid_until),
            line_items::currency_or_default(&input.currency),
            trim_opt(&input.po_reference),
            trim_opt(&input.discount_type),
            trim_opt(&input.discount_value),
            totals.subtotal_minor,
            totals.discount_minor,
            totals.tax_minor,
            totals.total_minor,
            trim_opt(&input.notes),
            trim_opt(&input.terms),
            trim_opt(&input.payment_instructions),
            trim_opt(&input.locale),
        ],
    )?;

    line_items::insert_items(conn, "estimate_item", "estimate_id", &id, &input.items)?;
    Ok(id)
}

pub fn create(conn: &Connection, input: &EstimateInput) -> rusqlite::Result<Estimate> {
    let tx = conn.unchecked_transaction()?;
    let id = insert_estimate(&tx, input)?;
    tx.commit()?;

    get(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(conn: &Connection, id: &str, input: &EstimateInput) -> rusqlite::Result<Estimate> {
    let tx = conn.unchecked_transaction()?;
    let totals = preview_totals(&tx, input)?;

    let changed = tx.execute(
        "UPDATE estimate SET
            client_id = ?2, issue_date = ?3, valid_until = ?4, currency = ?5, po_reference = ?6,
            discount_type = ?7, discount_value = ?8, subtotal_minor = ?9, discount_minor = ?10,
            tax_minor = ?11, total_minor = ?12, notes = ?13, terms = ?14,
            payment_instructions = ?15, locale = ?16, updated_at = datetime('now')
         WHERE id = ?1",
        params![
            id,
            trim_opt(&input.client_id),
            input.issue_date.trim(),
            trim_opt(&input.valid_until),
            line_items::currency_or_default(&input.currency),
            trim_opt(&input.po_reference),
            trim_opt(&input.discount_type),
            trim_opt(&input.discount_value),
            totals.subtotal_minor,
            totals.discount_minor,
            totals.tax_minor,
            totals.total_minor,
            trim_opt(&input.notes),
            trim_opt(&input.terms),
            trim_opt(&input.payment_instructions),
            trim_opt(&input.locale),
        ],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    tx.execute("DELETE FROM estimate_item WHERE estimate_id = ?1", [id])?;
    line_items::insert_items(&tx, "estimate_item", "estimate_id", id, &input.items)?;
    tx.commit()?;

    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn duplicate(conn: &Connection, id: &str) -> rusqlite::Result<Estimate> {
    let original = get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let today: String = conn.query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))?;

    let input = EstimateInput {
        client_id: original.client_id,
        issue_date: today,
        valid_until: None,
        currency: original.currency,
        po_reference: original.po_reference,
        discount_type: original.discount_type,
        discount_value: original.discount_value,
        notes: original.notes,
        terms: original.terms,
        payment_instructions: original.payment_instructions,
        locale: original.locale,
        items: original
            .items
            .iter()
            .map(|item| LineItemInput {
                description: item.description.clone(),
                quantity: item.quantity.clone(),
                unit: item.unit.clone(),
                unit_price_minor: item.unit_price_minor,
                discount_type: item.discount_type.clone(),
                discount_value: item.discount_value.clone(),
                tax_rate_id: item.tax_rate_id.clone(),
                tax_inclusive: item.tax_inclusive,
            })
            .collect(),
    };

    create(conn, &input)
}

/// Set the manual lifecycle state. `expired` and `converted` are derived, so
/// they are not accepted here.
pub fn set_status(conn: &Connection, id: &str, status: &str) -> Result<Estimate, AppError> {
    if !matches!(status, "draft" | "sent" | "accepted" | "declined") {
        return Err(AppError::Validation(format!(
            "'{status}' is not a status you can set."
        )));
    }

    let changed = conn.execute(
        "UPDATE estimate SET status = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![id, status],
    )?;
    if changed == 0 {
        return Err(AppError::Validation("Estimate not found.".to_string()));
    }

    get(conn, id)?.ok_or_else(|| AppError::Validation("Estimate not found.".to_string()))
}

/// Create an invoice from an estimate and link the two. Refuses to convert twice.
pub fn convert(conn: &Connection, id: &str) -> Result<Invoice, AppError> {
    let tx = conn.unchecked_transaction()?;

    let estimate =
        get(&tx, id)?.ok_or_else(|| AppError::Validation("Estimate not found.".to_string()))?;

    if estimate.converted_invoice_id.is_some() {
        return Err(AppError::Validation(
            "This estimate has already been converted to an invoice.".to_string(),
        ));
    }

    let issue_date: String =
        tx.query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))?;

    let input = InvoiceInput {
        client_id: estimate.client_id.clone(),
        issue_date,
        due_date: None,
        currency: estimate.currency.clone(),
        po_reference: estimate.po_reference.clone(),
        discount_type: estimate.discount_type.clone(),
        discount_value: estimate.discount_value.clone(),
        notes: estimate.notes.clone(),
        terms: estimate.terms.clone(),
        payment_instructions: estimate.payment_instructions.clone(),
        estimate_id: Some(estimate.id.clone()),
        locale: estimate.locale.clone(),
        items: estimate
            .items
            .iter()
            .map(|item| LineItemInput {
                description: item.description.clone(),
                quantity: item.quantity.clone(),
                unit: item.unit.clone(),
                unit_price_minor: item.unit_price_minor,
                discount_type: item.discount_type.clone(),
                discount_value: item.discount_value.clone(),
                tax_rate_id: item.tax_rate_id.clone(),
                tax_inclusive: item.tax_inclusive,
            })
            .collect(),
    };

    let invoice_id = invoices::insert_invoice(&tx, &input)?;

    tx.execute(
        "UPDATE estimate
         SET status = 'converted', converted_invoice_id = ?2,
             converted_at = datetime('now'), updated_at = datetime('now')
         WHERE id = ?1",
        params![id, invoice_id],
    )?;

    tx.commit()?;

    invoices::get(conn, &invoice_id)?
        .ok_or_else(|| AppError::Validation("The converted invoice could not be read.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::clients;
    use crate::db::tests_support::temp_db;

    fn item(description: &str, price: i64) -> LineItemInput {
        LineItemInput {
            description: description.to_string(),
            quantity: "1".to_string(),
            unit: None,
            unit_price_minor: price,
            discount_type: None,
            discount_value: None,
            tax_rate_id: None,
            tax_inclusive: false,
        }
    }

    fn input(items: Vec<LineItemInput>, valid_until: Option<&str>) -> EstimateInput {
        EstimateInput {
            client_id: None,
            issue_date: "2026-03-15".to_string(),
            valid_until: valid_until.map(str::to_string),
            currency: "USD".to_string(),
            po_reference: None,
            discount_type: None,
            discount_value: None,
            notes: None,
            terms: None,
            payment_instructions: None,
            locale: None,
            items,
        }
    }

    #[test]
    fn create_and_read_back() {
        let t = temp_db();
        let estimate = create(&t.conn, &input(vec![item("Work", 25_000)], None)).unwrap();

        assert!(
            estimate.number.starts_with("EST-"),
            "number was {}",
            estimate.number
        );
        assert_eq!(estimate.status, "draft");
        assert_eq!(estimate.total_minor, 25_000);
        assert_eq!(estimate.items.len(), 1);
    }

    #[test]
    fn estimates_and_invoices_number_independently() {
        let t = temp_db();
        let estimate = create(&t.conn, &input(vec![item("Work", 1_000)], None)).unwrap();

        let invoice = invoices::create(
            &t.conn,
            &InvoiceInput {
                client_id: None,
                issue_date: "2026-03-15".to_string(),
                due_date: None,
                currency: "USD".to_string(),
                po_reference: None,
                discount_type: None,
                discount_value: None,
                notes: None,
                terms: None,
                payment_instructions: None,
                estimate_id: None,
                locale: None,
                items: vec![item("Work", 1_000)],
            },
        )
        .unwrap();

        assert!(estimate.number.starts_with("EST-"));
        assert!(invoice.number.starts_with("INV-"));
    }

    #[test]
    fn expiry_is_derived_but_accepted_estimates_stay_accepted() {
        let t = temp_db();

        let expired = create(
            &t.conn,
            &input(vec![item("Work", 1_000)], Some("2020-01-01")),
        )
        .unwrap();
        assert!(expired.days_until_expiry.unwrap() < 0);
        assert_eq!(expired.display_status, "expired");

        let open = create(
            &t.conn,
            &input(vec![item("Work", 1_000)], Some("2099-01-01")),
        )
        .unwrap();
        assert_eq!(open.display_status, "draft");

        let accepted = set_status(&t.conn, &expired.id, "accepted").unwrap();
        assert_eq!(accepted.display_status, "accepted");
    }

    #[test]
    fn status_must_be_settable() {
        let t = temp_db();
        let estimate = create(&t.conn, &input(vec![item("Work", 1_000)], None)).unwrap();

        assert!(set_status(&t.conn, &estimate.id, "converted").is_err());
        assert!(set_status(&t.conn, &estimate.id, "nonsense").is_err());
        assert_eq!(
            set_status(&t.conn, &estimate.id, "sent").unwrap().status,
            "sent"
        );
    }

    #[test]
    fn convert_creates_a_linked_invoice_and_refuses_a_second_time() {
        let t = temp_db();

        let client = clients::create(
            &t.conn,
            &clients::ClientInput {
                display_name: "Acme".to_string(),
                company: None,
                contact_person: None,
                email: None,
                phone: None,
                address_line1: None,
                address_line2: None,
                city: None,
                region: None,
                postal_code: None,
                country: None,
                tax_id: None,
                currency: None,
                notes: None,
                locale: None,
            },
        )
        .unwrap();

        let mut with_client = input(
            vec![item("Consulting", 30_000), item("Hosting", 5_000)],
            Some("2099-01-01"),
        );
        with_client.client_id = Some(client.id.clone());
        let estimate = create(&t.conn, &with_client).unwrap();

        let invoice = convert(&t.conn, &estimate.id).unwrap();

        assert!(invoice.number.starts_with("INV-"));
        assert_eq!(invoice.client_id.as_deref(), Some(client.id.as_str()));
        assert_eq!(invoice.total_minor, estimate.total_minor);
        assert_eq!(invoice.items.len(), 2);
        assert_eq!(invoice.display_status, "draft");

        // Both sides of the link are recorded.
        let reloaded = get(&t.conn, &estimate.id).unwrap().unwrap();
        assert_eq!(reloaded.status, "converted");
        assert_eq!(reloaded.display_status, "converted");
        assert_eq!(
            reloaded.converted_invoice_id.as_deref(),
            Some(invoice.id.as_str())
        );
        assert!(reloaded.converted_at.is_some());

        // And it cannot happen twice.
        assert!(convert(&t.conn, &estimate.id).is_err());
    }

    #[test]
    fn list_filters_by_status_including_derived_expiry() {
        let t = temp_db();
        create(&t.conn, &input(vec![item("A", 1_000)], Some("2020-01-01"))).unwrap();
        create(&t.conn, &input(vec![item("B", 1_000)], Some("2099-01-01"))).unwrap();

        assert_eq!(list(&t.conn, None, None, None).unwrap().len(), 2);
        assert_eq!(list(&t.conn, None, Some("expired"), None).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, Some("draft"), None).unwrap().len(), 1);
    }
}
