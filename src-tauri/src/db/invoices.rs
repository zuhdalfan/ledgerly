use rusqlite::{params, Connection, OptionalExtension, Row};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::line_items;
pub use super::line_items::LineItemInput;
use super::{numbering, trim_opt};
use crate::domain::totals::{InvoiceTotals, LineInput};

const PAID_EXPR: &str =
    "(SELECT IFNULL(SUM(p.amount_minor), 0) FROM payment p WHERE p.invoice_id = i.id)";
const DAYS_PAST_DUE_EXPR: &str =
    "CAST(julianday(date('now', 'localtime')) - julianday(i.due_date) AS INTEGER)";

fn columns() -> String {
    format!(
        "i.id, i.number, i.client_id, i.status, i.issue_date, i.due_date, \
         i.currency, i.po_reference, i.discount_type, i.discount_value, \
         i.subtotal_minor, i.discount_minor, i.tax_minor, i.total_minor, \
         i.notes, i.terms, i.payment_instructions, i.voided_at, i.sent_at, \
         i.created_at, i.updated_at, c.display_name, \
         {PAID_EXPR} AS paid_minor, {DAYS_PAST_DUE_EXPR} AS days_past_due"
    )
}

/// The lifecycle the user sees. Void and paid win; then an issued invoice that
/// is past its due date reads as overdue, then a part-paid one, then the
/// stored state. Drafts are never overdue because they were never issued.
fn display_status(
    status: &str,
    balance_minor: i64,
    paid_minor: i64,
    days_past_due: Option<i64>,
) -> String {
    if status == "void" {
        return "void".to_string();
    }
    if balance_minor <= 0 {
        return "paid".to_string();
    }
    if status == "sent" && days_past_due.unwrap_or(0) > 0 {
        return "overdue".to_string();
    }
    if paid_minor > 0 {
        return "partially_paid".to_string();
    }
    status.to_string()
}

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
    pub sent_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub paid_minor: i64,
    pub balance_minor: i64,
    pub days_past_due: Option<i64>,
    /// Derived from the stored lifecycle plus payments and the due date.
    pub display_status: String,
    pub items: Vec<InvoiceItem>,
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
    /// Set when the invoice was created by converting an estimate.
    #[serde(default)]
    pub estimate_id: Option<String>,
    #[serde(default)]
    pub items: Vec<LineItemInput>,
}

fn map_invoice(row: &Row, items: Vec<InvoiceItem>) -> rusqlite::Result<Invoice> {
    let status: String = row.get(3)?;
    let total_minor: i64 = row.get(13)?;
    let paid_minor: i64 = row.get(22)?;
    let days_past_due: Option<i64> = row.get(23)?;
    let balance_minor = total_minor - paid_minor;

    Ok(Invoice {
        id: row.get(0)?,
        number: row.get(1)?,
        client_id: row.get(2)?,
        status: status.clone(),
        issue_date: row.get(4)?,
        due_date: row.get(5)?,
        currency: row.get(6)?,
        po_reference: row.get(7)?,
        discount_type: row.get(8)?,
        discount_value: row.get(9)?,
        subtotal_minor: row.get(10)?,
        discount_minor: row.get(11)?,
        tax_minor: row.get(12)?,
        total_minor,
        notes: row.get(14)?,
        terms: row.get(15)?,
        payment_instructions: row.get(16)?,
        voided_at: row.get(17)?,
        sent_at: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
        client_name: row.get(21)?,
        paid_minor,
        balance_minor,
        days_past_due,
        display_status: display_status(&status, balance_minor, paid_minor, days_past_due),
        items,
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
        "SELECT {columns} FROM invoice i
         LEFT JOIN client c ON c.id = i.client_id
         WHERE (?1 IS NULL
                OR (?1 = 'void'    AND i.status = 'void')
                OR (?1 = 'draft'   AND i.status = 'draft')
                OR (?1 = 'sent'    AND i.status = 'sent')
                OR (?1 = 'unpaid'  AND i.status <> 'void' AND (i.total_minor - {PAID_EXPR}) > 0)
                OR (?1 = 'paid'    AND i.status <> 'void' AND (i.total_minor - {PAID_EXPR}) <= 0)
                OR (?1 = 'overdue' AND i.status = 'sent'
                                   AND {DAYS_PAST_DUE_EXPR} > 0
                                   AND (i.total_minor - {PAID_EXPR}) > 0))
           AND (?2 IS NULL OR i.number LIKE ?2 OR IFNULL(c.display_name, '') LIKE ?2)
         ORDER BY i.issue_date DESC, i.number DESC",
        columns = columns()
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![status, search], |row| map_invoice(row, Vec::new()))?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Invoice>> {
    let invoice = conn
        .query_row(
            &format!(
                "SELECT {} FROM invoice i
                 LEFT JOIN client c ON c.id = i.client_id
                 WHERE i.id = ?1",
                columns()
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
    line_items::preview_totals(
        conn,
        &input.discount_type,
        &input.discount_value,
        &input.items,
    )
}

/// Build totals inputs from stored line items (used for rendering and re-checks).
pub fn stored_line_inputs(items: &[InvoiceItem]) -> Vec<LineInput> {
    items
        .iter()
        .map(|item| LineInput {
            quantity: line_items::parse_decimal(&item.quantity, Decimal::ONE),
            unit_price_minor: item.unit_price_minor,
            discount: line_items::discount_from(&item.discount_type, &item.discount_value),
            tax_rate_bp: item.tax_rate_bp.unwrap_or(0),
            tax_inclusive: item.tax_inclusive,
        })
        .collect()
}

/// Insert an invoice and its lines *without* opening a transaction, so callers
/// that already have one (like estimate conversion) stay atomic.
pub(crate) fn insert_invoice(conn: &Connection, input: &InvoiceInput) -> rusqlite::Result<String> {
    let id = uuid::Uuid::now_v7().to_string();
    let year = line_items::year_of(conn, &input.issue_date)?;
    let number = numbering::next_number(conn, "invoice", year)?;
    let totals = preview_totals(conn, input)?;

    conn.execute(
        "INSERT INTO invoice (
            id, number, client_id, status, issue_date, due_date, currency, po_reference,
            discount_type, discount_value, subtotal_minor, discount_minor, tax_minor,
            total_minor, notes, terms, payment_instructions, estimate_id
         ) VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            id,
            number,
            trim_opt(&input.client_id),
            input.issue_date.trim(),
            trim_opt(&input.due_date),
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
            trim_opt(&input.estimate_id),
        ],
    )?;

    line_items::insert_items(conn, "invoice_item", "invoice_id", &id, &input.items)?;
    Ok(id)
}

pub fn create(conn: &Connection, input: &InvoiceInput) -> rusqlite::Result<Invoice> {
    let tx = conn.unchecked_transaction()?;
    let id = insert_invoice(&tx, input)?;
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
        ],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    tx.execute("DELETE FROM invoice_item WHERE invoice_id = ?1", [id])?;
    line_items::insert_items(&tx, "invoice_item", "invoice_id", id, &input.items)?;
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
        estimate_id: None,
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

/// Move an invoice between draft and sent. Void invoices are left untouched.
pub fn set_sent(conn: &Connection, id: &str, sent: bool) -> rusqlite::Result<Invoice> {
    conn.execute(
        "UPDATE invoice
         SET status = CASE WHEN ?2 = 1 THEN 'sent' ELSE 'draft' END,
             sent_at = CASE WHEN ?2 = 1 THEN IFNULL(sent_at, datetime('now')) ELSE NULL END,
             updated_at = datetime('now')
         WHERE id = ?1 AND status <> 'void'",
        params![id, sent as i64],
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
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
    use crate::db::{
        clients,
        tax_rates::{self, TaxRateInput},
        tests_support::temp_db,
    };

    fn item(description: &str, price: i64, tax_rate_id: Option<String>) -> LineItemInput {
        LineItemInput {
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

    fn input(items: Vec<LineItemInput>) -> InvoiceInput {
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
            estimate_id: None,
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

        assert!(
            invoice.number.starts_with("INV-"),
            "number was {}",
            invoice.number
        );
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
            line_items::discount_from(&invoice.discount_type, &invoice.discount_value),
        );

        assert_eq!(recomputed.subtotal_minor, invoice.subtotal_minor);
        assert_eq!(recomputed.discount_minor, invoice.discount_minor);
        assert_eq!(recomputed.tax_minor, invoice.tax_minor);
        assert_eq!(recomputed.total_minor, invoice.total_minor);
    }

    fn payment(amount_minor: i64) -> crate::db::payments::PaymentInput {
        crate::db::payments::PaymentInput {
            paid_on: "2026-03-20".to_string(),
            amount_minor,
            method: Some("bank".to_string()),
            reference: None,
            note: None,
        }
    }

    #[test]
    fn derived_status_follows_payments_and_due_date() {
        use crate::db::payments;
        let t = temp_db();

        let mut overdue = input(vec![item("Work", 10_000, None)]);
        overdue.due_date = Some("2020-01-01".to_string());
        let invoice = create(&t.conn, &overdue).unwrap();

        assert_eq!(invoice.display_status, "draft");
        assert_eq!(invoice.balance_minor, 10_000);

        let sent = set_sent(&t.conn, &invoice.id, true).unwrap();
        assert_eq!(sent.status, "sent");
        assert!(sent.sent_at.is_some());
        assert_eq!(sent.display_status, "overdue");

        payments::create(&t.conn, &invoice.id, &payment(4_000)).unwrap();
        let partial = get(&t.conn, &invoice.id).unwrap().unwrap();
        assert_eq!(partial.paid_minor, 4_000);
        assert_eq!(partial.balance_minor, 6_000);
        assert_eq!(
            partial.display_status, "overdue",
            "overdue outranks partial"
        );

        payments::create(&t.conn, &invoice.id, &payment(6_000)).unwrap();
        let paid = get(&t.conn, &invoice.id).unwrap().unwrap();
        assert_eq!(paid.balance_minor, 0);
        assert_eq!(paid.display_status, "paid");
    }

    #[test]
    fn a_future_due_date_reads_as_sent_not_overdue() {
        let t = temp_db();
        let mut future = input(vec![item("Work", 10_000, None)]);
        future.due_date = Some("2099-01-01".to_string());

        let invoice = create(&t.conn, &future).unwrap();
        let sent = set_sent(&t.conn, &invoice.id, true).unwrap();

        assert_eq!(sent.display_status, "sent");
        assert!(sent.days_past_due.unwrap_or(0) < 0);
    }

    #[test]
    fn a_draft_is_never_overdue_even_when_past_due() {
        use crate::db::payments;
        let t = temp_db();

        let mut past = input(vec![item("Work", 5_000, None)]);
        past.due_date = Some("2020-01-01".to_string());
        let invoice = create(&t.conn, &past).unwrap();

        assert!(invoice.days_past_due.unwrap_or(0) > 0);
        assert_eq!(invoice.display_status, "draft");

        payments::create(&t.conn, &invoice.id, &payment(2_000)).unwrap();
        assert_eq!(
            get(&t.conn, &invoice.id).unwrap().unwrap().display_status,
            "partially_paid"
        );
    }

    #[test]
    fn void_outranks_paid() {
        use crate::db::payments;
        let t = temp_db();
        let invoice = create(&t.conn, &input(vec![item("Work", 5_000, None)])).unwrap();
        set_sent(&t.conn, &invoice.id, true).unwrap();
        payments::create(&t.conn, &invoice.id, &payment(5_000)).unwrap();

        let voided = set_void(&t.conn, &invoice.id, true).unwrap();
        assert_eq!(voided.display_status, "void");
    }

    #[test]
    fn set_sent_leaves_void_invoices_alone() {
        let t = temp_db();
        let invoice = create(&t.conn, &input(vec![item("Work", 5_000, None)])).unwrap();
        set_void(&t.conn, &invoice.id, true).unwrap();

        let after = set_sent(&t.conn, &invoice.id, true).unwrap();
        assert_eq!(after.status, "void");
    }

    #[test]
    fn list_filters_by_payment_state() {
        use crate::db::payments;
        let t = temp_db();

        let mut past = input(vec![item("Work", 10_000, None)]);
        past.due_date = Some("2020-01-01".to_string());
        let overdue = create(&t.conn, &past).unwrap();
        set_sent(&t.conn, &overdue.id, true).unwrap();

        let mut future = input(vec![item("Work", 10_000, None)]);
        future.due_date = Some("2099-01-01".to_string());
        let settled = create(&t.conn, &future).unwrap();
        set_sent(&t.conn, &settled.id, true).unwrap();
        payments::create(&t.conn, &settled.id, &payment(10_000)).unwrap();

        assert_eq!(list(&t.conn, None, Some("overdue")).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, Some("paid")).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, Some("unpaid")).unwrap().len(), 1);
        assert_eq!(list(&t.conn, None, Some("sent")).unwrap().len(), 2);
    }
}
