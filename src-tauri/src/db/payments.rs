use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::trim_opt;
use crate::error::AppError;

const COLUMNS: &str =
    "id, invoice_id, paid_on, amount_minor, method, reference, note, created_at, updated_at";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: String,
    pub invoice_id: String,
    pub paid_on: String,
    pub amount_minor: i64,
    pub method: Option<String>,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInput {
    pub paid_on: String,
    pub amount_minor: i64,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

fn map_row(row: &Row) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: row.get(0)?,
        invoice_id: row.get(1)?,
        paid_on: row.get(2)?,
        amount_minor: row.get(3)?,
        method: row.get(4)?,
        reference: row.get(5)?,
        note: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn invoice_total(conn: &Connection, invoice_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT total_minor FROM invoice WHERE id = ?1",
        [invoice_id],
        |row| row.get(0),
    )
}

/// How much of the invoice is still unpaid, optionally ignoring one payment
/// (so editing a payment does not count itself twice).
fn outstanding(
    conn: &Connection,
    invoice_id: &str,
    excluding: Option<&str>,
) -> rusqlite::Result<i64> {
    let total = invoice_total(conn, invoice_id)?;
    let paid: i64 = conn.query_row(
        "SELECT IFNULL(SUM(amount_minor), 0) FROM payment
         WHERE invoice_id = ?1 AND (?2 IS NULL OR id <> ?2)",
        params![invoice_id, excluding],
        |row| row.get(0),
    )?;
    Ok(total - paid)
}

fn validate_amount(amount_minor: i64, available: i64) -> Result<(), AppError> {
    if amount_minor <= 0 {
        return Err(AppError::Validation(
            "Payment amount must be greater than zero.".to_string(),
        ));
    }
    if amount_minor > available {
        return Err(AppError::Validation(
            "Payment is more than the outstanding balance. Split it into smaller payments."
                .to_string(),
        ));
    }
    Ok(())
}

pub fn list(conn: &Connection, invoice_id: &str) -> rusqlite::Result<Vec<Payment>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM payment WHERE invoice_id = ?1 ORDER BY paid_on, created_at"
    ))?;
    let rows = stmt.query_map([invoice_id], map_row)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Payment>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM payment WHERE id = ?1"),
        [id],
        map_row,
    )
    .optional()
}

pub fn create(
    conn: &Connection,
    invoice_id: &str,
    input: &PaymentInput,
) -> Result<Payment, AppError> {
    let available = outstanding(conn, invoice_id, None)?;
    validate_amount(input.amount_minor, available)?;

    let id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO payment (id, invoice_id, paid_on, amount_minor, method, reference, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            invoice_id,
            input.paid_on.trim(),
            input.amount_minor,
            trim_opt(&input.method),
            trim_opt(&input.reference),
            trim_opt(&input.note),
        ],
    )?;

    get(conn, &id)?.ok_or_else(|| AppError::Validation("Payment not found.".to_string()))
}

pub fn update(conn: &Connection, id: &str, input: &PaymentInput) -> Result<Payment, AppError> {
    let existing = match get(conn, id)? {
        Some(payment) => payment,
        None => return Err(AppError::Validation("Payment not found.".to_string())),
    };

    let available = outstanding(conn, &existing.invoice_id, Some(id))?;
    validate_amount(input.amount_minor, available)?;

    conn.execute(
        "UPDATE payment
         SET paid_on = ?2, amount_minor = ?3, method = ?4, reference = ?5, note = ?6,
             updated_at = datetime('now')
         WHERE id = ?1",
        params![
            id,
            input.paid_on.trim(),
            input.amount_minor,
            trim_opt(&input.method),
            trim_opt(&input.reference),
            trim_opt(&input.note),
        ],
    )?;

    get(conn, id)?.ok_or_else(|| AppError::Validation("Payment not found.".to_string()))
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let changed = conn.execute("DELETE FROM payment WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::Validation("Payment not found.".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::invoices::{self, InvoiceInput, InvoiceItemInput};
    use crate::db::tests_support::temp_db;

    fn invoice(conn: &Connection, total: i64) -> String {
        invoices::create(
            conn,
            &InvoiceInput {
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
                items: vec![InvoiceItemInput {
                    description: "Work".to_string(),
                    quantity: "1".to_string(),
                    unit: None,
                    unit_price_minor: total,
                    discount_type: None,
                    discount_value: None,
                    tax_rate_id: None,
                    tax_inclusive: false,
                }],
            },
        )
        .unwrap()
        .id
    }

    fn payment(amount: i64) -> PaymentInput {
        PaymentInput {
            paid_on: "2026-03-20".to_string(),
            amount_minor: amount,
            method: Some("bank".to_string()),
            reference: None,
            note: None,
        }
    }

    #[test]
    fn partial_then_full_payment() {
        let t = temp_db();
        let id = invoice(&t.conn, 100_000);

        let first = create(&t.conn, &id, &payment(40_000)).unwrap();
        assert_eq!(outstanding(&t.conn, &id, None).unwrap(), 60_000);

        create(&t.conn, &id, &payment(60_000)).unwrap();
        assert_eq!(outstanding(&t.conn, &id, None).unwrap(), 0);
        assert_eq!(list(&t.conn, &id).unwrap().len(), 2);

        delete(&t.conn, &first.id).unwrap();
        assert_eq!(outstanding(&t.conn, &id, None).unwrap(), 40_000);
    }

    #[test]
    fn rejects_overpayment_and_non_positive_amounts() {
        let t = temp_db();
        let id = invoice(&t.conn, 50_000);

        assert!(create(&t.conn, &id, &payment(50_001)).is_err());
        assert!(create(&t.conn, &id, &payment(0)).is_err());
        assert!(create(&t.conn, &id, &payment(-100)).is_err());

        create(&t.conn, &id, &payment(50_000)).unwrap();
        assert!(create(&t.conn, &id, &payment(1)).is_err());
    }

    #[test]
    fn editing_a_payment_does_not_double_count_itself() {
        let t = temp_db();
        let id = invoice(&t.conn, 100_000);
        let existing = create(&t.conn, &id, &payment(30_000)).unwrap();

        // Raising it to the full balance is fine; more than that is not.
        update(&t.conn, &existing.id, &payment(100_000)).unwrap();
        assert!(update(&t.conn, &existing.id, &payment(120_000)).is_err());
    }

    #[test]
    fn deleting_a_missing_payment_errors() {
        let t = temp_db();
        assert!(delete(&t.conn, "missing").is_err());
    }
}
