use rusqlite::{params, Connection, OptionalExtension, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use super::{numbering, tax_rates, trim_opt};
use crate::domain::totals::{self, Discount, DiscountKind, InvoiceTotals, LineInput};

const COLUMNS: &str = "i.id, i.number, i.client_id, i.status, i.issue_date, i.due_date, \
                       i.currency, i.po_reference, i.discount_type, i.discount_value, \
                       i.subtotal_minor, i.discount_minor, i.tax_minor, i.total_minor, \
                       i.notes, i.terms, i.payment_instructions, i.voided_at, \
                       i.created_at, i.updated_at, c.display_name";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceItem {
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
pub struct Invoice {
    pub id: String,
    pub number: String,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub status: String,
    pub issue_date: String,
    pub due_date: Option<String>,
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
    pub voided_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub items: Vec<InvoiceItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceItemInput {
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_quantity")]
    pub quantity: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub unit_price_minor: i64,
    #[serde(default)]
    pub discount_type: Option<String>,
    #[serde(default)]
    pub discount_value: Option<String>,
    #[serde(default)]
    pub tax_rate_id: Option<String>,
    #[serde(default)]
    pub tax_inclusive: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceInput {
    #[serde(default)]
    pub client_id: Option<String>,
    pub issue_date: String,
    #[serde(default)]
    pub due_date: Option<String>,
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
    pub items: Vec<InvoiceItemInput>,
}

fn default_quantity() -> String {
    "1".to_string()
}

fn map_invoice(row: &Row, items: Vec<InvoiceItem>) -> rusqlite::Result<Invoice> {
    Ok(Invoice {
        id: row.get(0)?,
        number: row.get(1)?,
        client_id: row.get(2)?,
        status: row.get(3)?,
        issue_date: row.get(4)?,
        due_date: row.get(5)?,
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
        voided_at: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
        client_name: row.get(20)?,
        items,
    })
}

fn parse_decimal(value: &str, fallback: Decimal) -> Decimal {
    Decimal::from_str(value.trim()).unwrap_or(fallback)
}

pub(crate) fn discount_from(kind: &Option<String>, value: &Option<String>) -> Option<Discount> {
    let kind = kind.as_deref()?.trim();
    let value = value.as_deref()?.trim();
    if value.is_empty() {
        return None;
    }
    let value = Decimal::from_str(value).ok()?;
    let kind = match kind {
        "percent" => DiscountKind::Percent,
        "fixed" => DiscountKind::Fixed,
        _ => return None,
    };
    Some(Discount { kind, value })
}

fn currency_or_default(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "USD".to_string()
    } else {
        trimmed.to_string()
    }
}

fn year_of(conn: &Connection, date: &str) -> rusqlite::Result<i64> {
    if let Some(year) = date.get(0..4).and_then(|part| part.parse::<i64>().ok()) {
        return Ok(year);
    }
    conn.query_row("SELECT CAST(strftime('%Y', 'now') AS INTEGER)", [], |row| {
        row.get(0)
    })
}

pub fn load_items(conn: &Connection, invoice_id: &str) -> rusqlite::Result<Vec<InvoiceItem>> {
    let mut stmt = conn.prepare(
        "SELECT it.id, it.sort_order, it.description, it.quantity, it.unit,
                it.unit_price_minor, it.discount_type, it.discount_value,
                it.tax_rate_id, t.name, t.rate_bp, it.tax_inclusive
         FROM invoice_item it
         LEFT JOIN tax_rate t ON t.id = it.tax_rate_id
         WHERE it.invoice_id = ?1
         ORDER BY it.sort_order, it.created_at",
    )?;
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceItem {
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
) -> rusqlite::Result<Vec<Invoice>> {
    let search = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"));
    let status = status.map(str::trim).filter(|value| !value.is_empty());

    let sql = format!(
        "SELECT {COLUMNS} FROM invoice i
         LEFT JOIN client c ON c.id = i.client_id
         WHERE (?1 IS NULL OR i.status = ?1)
           AND (?2 IS NULL OR i.number LIKE ?2 OR IFNULL(c.display_name, '') LIKE ?2)
         ORDER BY i.issue_date DESC, i.number DESC"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![status, search], |row| map_invoice(row, Vec::new()))?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Invoice>> {
    let invoice = conn
        .query_row(
            &format!(
                "SELECT {COLUMNS} FROM invoice i
                 LEFT JOIN client c ON c.id = i.client_id
                 WHERE i.id = ?1"
            ),
            [id],
            |row| map_invoice(row, Vec::new()),
        )
        .optional()?;

    match invoice {
        None => Ok(None),
        Some(mut invoice) => {
            invoice.items = load_items(conn, id)?;
            Ok(Some(invoice))
        }
    }
}

/// Compute totals for a draft that has not been saved yet (live editor preview).
pub fn preview_totals(conn: &Connection, input: &InvoiceInput) -> rusqlite::Result<InvoiceTotals> {
    let rates = tax_rates::rate_map(conn)?;
    let lines: Vec<LineInput> = input
        .items
        .iter()
        .map(|item| to_line_input(item, &rates))
        .collect();
    Ok(totals::compute(
        &lines,
        discount_from(&input.discount_type, &input.discount_value),
    ))
}

fn to_line_input(item: &InvoiceItemInput, rates: &HashMap<String, i64>) -> LineInput {
    LineInput {
        quantity: parse_decimal(&item.quantity, Decimal::ONE),
        unit_price_minor: item.unit_price_minor,
        discount: discount_from(&item.discount_type, &item.discount_value),
        tax_rate_bp: item
            .tax_rate_id
            .as_ref()
            .and_then(|id| rates.get(id).copied())
            .unwrap_or(0),
        tax_inclusive: item.tax_inclusive,
    }
}

/// Build totals inputs from stored line items (used for rendering and re-checks).
pub fn stored_line_inputs(items: &[InvoiceItem]) -> Vec<LineInput> {
    items
        .iter()
        .map(|item| LineInput {
            quantity: parse_decimal(&item.quantity, Decimal::ONE),
            unit_price_minor: item.unit_price_minor,
            discount: discount_from(&item.discount_type, &item.discount_value),
            tax_rate_bp: item.tax_rate_bp.unwrap_or(0),
            tax_inclusive: item.tax_inclusive,
        })
        .collect()
}

fn normalized_quantity(value: &str) -> String {
    parse_decimal(value, Decimal::ONE).normalize().to_string()
}

fn insert_items(
    conn: &Connection,
    invoice_id: &str,
    items: &[InvoiceItemInput],
) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO invoice_item (
            id, invoice_id, sort_order, description, quantity, unit, unit_price_minor,
            discount_type, discount_value, tax_rate_id, tax_inclusive
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;
    for (index, item) in items.iter().enumerate() {
        stmt.execute(params![
            uuid::Uuid::now_v7().to_string(),
            invoice_id,
            index as i64,
            item.description.trim(),
            normalized_quantity(&item.quantity),
            trim_opt(&item.unit),
            item.unit_price_minor,
            trim_opt(&item.discount_type),
            trim_opt(&item.discount_value),
            trim_opt(&item.tax_rate_id),
            item.tax_inclusive as i64,
        ])?;
    }
    Ok(())
}

pub fn create(conn: &Connection, input: &InvoiceInput) -> rusqlite::Result<Invoice> {
    let tx = conn.unchecked_transaction()?;

    let id = uuid::Uuid::now_v7().to_string();
    let year = year_of(&tx, &input.issue_date)?;
    let number = numbering::next_number(&tx, "invoice", year)?;
    let totals = preview_totals(&tx, input)?;

    tx.execute(
        "INSERT INTO invoice (
            id, number, client_id, status, issue_date, due_date, currency, po_reference,
            discount_type, discount_value, subtotal_minor, discount_minor, tax_minor,
            total_minor, notes, terms, payment_instructions
         ) VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            id,
            number,
            trim_opt(&input.client_id),
            input.issue_date.trim(),
            trim_opt(&input.due_date),
            currency_or_default(&input.currency),
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
        ],
    )?;

    insert_items(&tx, &id, &input.items)?;
    tx.commit()?;

    get(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(conn: &Connection, id: &str, input: &InvoiceInput) -> rusqlite::Result<Invoice> {
    let tx = conn.unchecked_transaction()?;
    let totals = preview_totals(&tx, input)?;

    let changed = tx.execute(
        "UPDATE invoice SET
            client_id = ?2, issue_date = ?3, due_date = ?4, currency = ?5, po_reference = ?6,
            discount_type = ?7, discount_value = ?8, subtotal_minor = ?9, discount_minor = ?10,
            tax_minor = ?11, total_minor = ?12, notes = ?13, terms = ?14,
            payment_instructions = ?15, updated_at = datetime('now')
         WHERE id = ?1",
        params![
            id,
            trim_opt(&input.client_id),
            input.issue_date.trim(),
            trim_opt(&input.due_date),
            currency_or_default(&input.currency),
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
        ],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    tx.execute("DELETE FROM invoice_item WHERE invoice_id = ?1", [id])?;
    insert_items(&tx, id, &input.items)?;
    tx.commit()?;

    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn duplicate(conn: &Connection, id: &str) -> rusqlite::Result<Invoice> {
    let original = get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let issue_date: String = conn.query_row("SELECT date('now')", [], |row| row.get(0))?;

    let input = InvoiceInput {
        client_id: original.client_id,
        issue_date,
        due_date: None,
        currency: original.currency,
        po_reference: original.po_reference,
        discount_type: original.discount_type,
        discount_value: original.discount_value,
        notes: original.notes,
        terms: original.terms,
        payment_instructions: original.payment_instructions,
        items: original
            .items
            .iter()
            .map(|item| InvoiceItemInput {
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

pub fn set_void(conn: &Connection, id: &str, voided: bool) -> rusqlite::Result<Invoice> {
    let changed = conn.execute(
        "UPDATE invoice
         SET status = CASE WHEN ?2 = 1 THEN 'void' ELSE 'draft' END,
             voided_at = CASE WHEN ?2 = 1 THEN datetime('now') ELSE NULL END,
             updated_at = datetime('now')
         WHERE id = ?1",
        params![id, voided as i64],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{clients, tax_rates::TaxRateInput, tests_support::temp_db};

    fn item(description: &str, price: i64, tax_rate_id: Option<String>) -> InvoiceItemInput {
        InvoiceItemInput {
            description: description.to_string(),
            quantity: "1".to_string(),
            unit: None,
            unit_price_minor: price,
            discount_type: None,
            discount_value: None,
            tax_rate_id,
            tax_inclusive: false,
        }
    }

    fn input(items: Vec<InvoiceItemInput>) -> InvoiceInput {
        InvoiceInput {
            client_id: None,
            issue_date: "2026-03-15".to_string(),
            due_date: Some("2026-04-14".to_string()),
            currency: "USD".to_string(),
            po_reference: None,
            discount_type: None,
            discount_value: None,
            notes: None,
            terms: None,
            payment_instructions: None,
            items,
        }
    }

    #[test]
    fn create_persists_items_and_totals() {
        let t = temp_db();
        let vat = tax_rates::create(
            &t.conn,
            &TaxRateInput {
                name: "VAT".to_string(),
                rate_bp: 1_000,
            },
        )
        .unwrap();

        let invoice = create(
            &t.conn,
            &input(vec![
                item("Design", 20_000, Some(vat.id.clone())),
                item("Hosting", 5_000, None),
            ]),
        )
        .unwrap();

        assert!(invoice.number.starts_with("INV-"), "number was {}", invoice.number);
        assert_eq!(invoice.status, "draft");
        assert_eq!(invoice.items.len(), 2);
        assert_eq!(invoice.subtotal_minor, 25_000);
        assert_eq!(invoice.tax_minor, 2_000);
        assert_eq!(invoice.total_minor, 27_000);
    }

    #[test]
    fn update_replaces_items_and_recomputes() {
        let t = temp_db();
        let invoice = create(&t.conn, &input(vec![item("One", 10_000, None)])).unwrap();

        let mut changed = input(vec![item("One", 10_000, None), item("Two", 2_500, None)]);
        changed.discount_type = Some("percent".to_string());
        changed.discount_value = Some("10".to_string());

        let updated = update(&t.conn, &invoice.id, &changed).unwrap();

        assert_eq!(updated.items.len(), 2);
        assert_eq!(updated.subtotal_minor, 12_500);
        assert_eq!(updated.discount_minor, 1_250);
        assert_eq!(updated.total_minor, 11_250);
    }

    #[test]
    fn duplicate_gets_a_fresh_number_and_stays_draft() {
        let t = temp_db();
        let first = create(&t.conn, &input(vec![item("One", 10_000, None)])).unwrap();
        set_void(&t.conn, &first.id, true).unwrap();

        let copy = duplicate(&t.conn, &first.id).unwrap();

        assert_ne!(copy.number, first.number);
        assert_eq!(copy.status, "draft");
        assert_eq!(copy.total_minor, first.total_minor);
        assert_eq!(copy.items.len(), 1);
    }

    #[test]
    fn list_filters_by_status_and_search() {
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
            },
        )
        .unwrap();

        let mut with_client = input(vec![item("One", 1_000, None)]);
        with_client.client_id = Some(client.id);
        let first = create(&t.conn, &with_client).unwrap();
        create(&t.conn, &input(vec![item("Two", 2_000, None)])).unwrap();

        assert_eq!(list(&t.conn, None, None).unwrap().len(), 2);
        assert_eq!(list(&t.conn, Some("acme"), None).unwrap().len(), 1);

        set_void(&t.conn, &first.id, true).unwrap();
        assert_eq!(list(&t.conn, None, Some("void")).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, Some("draft")).unwrap().len(), 1);
    }

    #[test]
    fn update_missing_invoice_errors() {
        let t = temp_db();
        assert!(update(&t.conn, "missing", &input(vec![])).is_err());
    }

    #[test]
    fn cached_totals_match_the_totals_engine() {
        let t = temp_db();
        let vat = tax_rates::create(
            &t.conn,
            &TaxRateInput {
                name: "VAT".to_string(),
                rate_bp: 2_000,
            },
        )
        .unwrap();

        let mut with_discount = input(vec![
            item("A", 19_999, Some(vat.id)),
            item("B", 7_777, None),
        ]);
        with_discount.discount_type = Some("percent".to_string());
        with_discount.discount_value = Some("12.5".to_string());

        let invoice = create(&t.conn, &with_discount).unwrap();

        // What the PDF renders must equal what the list view shows.
        let recomputed = crate::domain::totals::compute(
            &stored_line_inputs(&invoice.items),
            discount_from(&invoice.discount_type, &invoice.discount_value),
        );

        assert_eq!(recomputed.subtotal_minor, invoice.subtotal_minor);
        assert_eq!(recomputed.discount_minor, invoice.discount_minor);
        assert_eq!(recomputed.tax_minor, invoice.tax_minor);
        assert_eq!(recomputed.total_minor, invoice.total_minor);
    }
}
