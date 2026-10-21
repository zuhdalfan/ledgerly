//! Reporting queries.
//!
//! Anything that depends on how money is calculated (tax, discounts) is derived
//! by the shared totals engine rather than re-expressed in SQL, so reports
//! always reconcile with the invoices themselves.

use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::BTreeMap;

use super::invoices::{self, stored_line_inputs};
use super::line_items;
use crate::domain::totals;

#[derive(Debug, Clone, Copy)]
pub enum Granularity {
    Day,
    Week,
    Month,
    Year,
}

impl Granularity {
    pub fn parse(value: &str) -> Granularity {
        match value.trim().to_ascii_lowercase().as_str() {
            "day" => Granularity::Day,
            "week" => Granularity::Week,
            "year" => Granularity::Year,
            _ => Granularity::Month,
        }
    }

    fn expression(self, column: &str) -> String {
        match self {
            Granularity::Day => format!("strftime('%Y-%m-%d', {column})"),
            Granularity::Week => format!("strftime('%Y-W%W', {column})"),
            Granularity::Month => format!("strftime('%Y-%m', {column})"),
            Granularity::Year => format!("strftime('%Y', {column})"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodRow {
    pub period: String,
    pub invoice_count: i64,
    pub invoiced_minor: i64,
    pub tax_minor: i64,
    pub paid_minor: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRevenueRow {
    pub client_id: Option<String>,
    pub client_name: String,
    pub invoice_count: i64,
    pub invoiced_minor: i64,
    pub paid_minor: i64,
    pub outstanding_minor: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgedRow {
    pub label: String,
    pub invoice_count: i64,
    pub balance_minor: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRow {
    pub tax_rate_id: Option<String>,
    pub name: String,
    pub rate_bp: i64,
    pub taxable_minor: i64,
    pub tax_minor: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodRow {
    pub method: String,
    pub payment_count: i64,
    pub amount_minor: i64,
}

const PAID_EXPR: &str =
    "(SELECT IFNULL(SUM(p.amount_minor), 0) FROM payment p WHERE p.invoice_id = i.id)";

/// Invoiced and received amounts per period. Invoiced is grouped by issue date,
/// received by payment date.
pub fn revenue_by_period(
    conn: &Connection,
    from: &str,
    to: &str,
    granularity: Granularity,
) -> rusqlite::Result<Vec<PeriodRow>> {
    let mut rows: BTreeMap<String, PeriodRow> = BTreeMap::new();

    let invoiced_sql = format!(
        "SELECT {} AS period, COUNT(*), IFNULL(SUM(total_minor), 0), IFNULL(SUM(tax_minor), 0)
         FROM invoice
         WHERE status <> 'void' AND issue_date >= ?1 AND issue_date <= ?2
         GROUP BY period",
        granularity.expression("issue_date")
    );
    let mut stmt = conn.prepare(&invoiced_sql)?;
    let invoiced = stmt.query_map(params![from, to], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;
    for row in invoiced {
        let (period, count, invoiced_minor, tax_minor) = row?;
        let entry = rows.entry(period.clone()).or_insert_with(|| PeriodRow {
            period,
            invoice_count: 0,
            invoiced_minor: 0,
            tax_minor: 0,
            paid_minor: 0,
        });
        entry.invoice_count = count;
        entry.invoiced_minor = invoiced_minor;
        entry.tax_minor = tax_minor;
    }

    let received_sql = format!(
        "SELECT {} AS period, IFNULL(SUM(p.amount_minor), 0)
         FROM payment p JOIN invoice i ON i.id = p.invoice_id
         WHERE i.status <> 'void' AND p.paid_on >= ?1 AND p.paid_on <= ?2
         GROUP BY period",
        granularity.expression("p.paid_on")
    );
    let mut stmt = conn.prepare(&received_sql)?;
    let received = stmt.query_map(params![from, to], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in received {
        let (period, paid_minor) = row?;
        let entry = rows.entry(period.clone()).or_insert_with(|| PeriodRow {
            period,
            invoice_count: 0,
            invoiced_minor: 0,
            tax_minor: 0,
            paid_minor: 0,
        });
        entry.paid_minor = paid_minor;
    }

    Ok(rows.into_values().collect())
}

pub fn revenue_by_client(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<ClientRevenueRow>> {
    let sql = format!(
        "SELECT t.client_id, IFNULL(c.display_name, 'No client') AS name,
                COUNT(*), IFNULL(SUM(t.total_minor), 0), IFNULL(SUM(t.paid_minor), 0)
         FROM (
             SELECT i.id, i.client_id, i.total_minor, {PAID_EXPR} AS paid_minor
             FROM invoice i
             WHERE i.status <> 'void' AND i.issue_date >= ?1 AND i.issue_date <= ?2
         ) t
         LEFT JOIN client c ON c.id = t.client_id
         GROUP BY t.client_id
         ORDER BY 4 DESC, name COLLATE NOCASE"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![from, to], |row| {
        let invoiced_minor: i64 = row.get(3)?;
        let paid_minor: i64 = row.get(4)?;
        Ok(ClientRevenueRow {
            client_id: row.get(0)?,
            client_name: row.get(1)?,
            invoice_count: row.get(2)?,
            invoiced_minor,
            paid_minor,
            outstanding_minor: invoiced_minor - paid_minor,
        })
    })?;
    rows.collect()
}

/// Issued invoices that are still unpaid, bucketed by how long they are past due.
pub fn aged_receivables(conn: &Connection) -> rusqlite::Result<Vec<AgedRow>> {
    const DAYS: &str =
        "CAST(julianday(date('now', 'localtime')) - julianday(i.due_date) AS INTEGER)";

    let sql = format!(
        "SELECT
            CASE
                WHEN i.due_date IS NULL OR {DAYS} <= 0 THEN 0
                WHEN {DAYS} <= 30 THEN 1
                WHEN {DAYS} <= 60 THEN 2
                WHEN {DAYS} <= 90 THEN 3
                ELSE 4
            END AS bucket,
            COUNT(*),
            IFNULL(SUM(i.total_minor - {PAID_EXPR}), 0)
         FROM invoice i
         WHERE i.status = 'sent' AND (i.total_minor - {PAID_EXPR}) > 0
         GROUP BY bucket"
    );

    let labels = [
        "Not yet due",
        "1–30 days",
        "31–60 days",
        "61–90 days",
        "90+ days",
    ];
    let mut buckets: Vec<AgedRow> = labels
        .iter()
        .map(|label| AgedRow {
            label: (*label).to_string(),
            invoice_count: 0,
            balance_minor: 0,
        })
        .collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (index, count, balance) = row?;
        if let Some(bucket) = buckets.get_mut(index as usize) {
            bucket.invoice_count = count;
            bucket.balance_minor = balance;
        }
    }

    Ok(buckets)
}

/// Tax collected per rate, computed with the same engine that renders invoices.
pub fn tax_collected(conn: &Connection, from: &str, to: &str) -> rusqlite::Result<Vec<TaxRow>> {
    let mut ids: Vec<String> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id FROM invoice
             WHERE status <> 'void' AND issue_date >= ?1 AND issue_date <= ?2
             ORDER BY issue_date",
        )?;
        let rows = stmt.query_map(params![from, to], |row| row.get::<_, String>(0))?;
        for row in rows {
            ids.push(row?);
        }
    }

    let mut by_rate: BTreeMap<String, TaxRow> = BTreeMap::new();

    for id in ids {
        let Some(invoice) = invoices::get(conn, &id)? else {
            continue;
        };
        let totals = totals::compute(
            &stored_line_inputs(&invoice.items),
            line_items::discount_from(&invoice.discount_type, &invoice.discount_value),
        );

        for (item, line) in invoice.items.iter().zip(totals.lines.iter()) {
            let key = item.tax_rate_id.clone().unwrap_or_default();
            let entry = by_rate.entry(key.clone()).or_insert_with(|| TaxRow {
                tax_rate_id: item.tax_rate_id.clone(),
                name: item
                    .tax_rate_name
                    .clone()
                    .unwrap_or_else(|| "No tax".to_string()),
                rate_bp: item.tax_rate_bp.unwrap_or(0),
                taxable_minor: 0,
                tax_minor: 0,
            });
            entry.taxable_minor += line.taxable_minor;
            entry.tax_minor += line.tax_minor;
        }
    }

    Ok(by_rate.into_values().collect())
}

pub fn payments_by_method(
    conn: &Connection,
    from: &str,
    to: &str,
) -> rusqlite::Result<Vec<MethodRow>> {
    let mut stmt = conn.prepare(
        "SELECT IFNULL(NULLIF(TRIM(p.method), ''), 'Unspecified'), COUNT(*), IFNULL(SUM(p.amount_minor), 0)
         FROM payment p JOIN invoice i ON i.id = p.invoice_id
         WHERE i.status <> 'void' AND p.paid_on >= ?1 AND p.paid_on <= ?2
         GROUP BY 1
         ORDER BY 3 DESC",
    )?;
    let rows = stmt.query_map(params![from, to], |row| {
        Ok(MethodRow {
            method: row.get(0)?,
            payment_count: row.get(1)?,
            amount_minor: row.get(2)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::invoices::{InvoiceInput, LineItemInput};
    use crate::db::payments::{self, PaymentInput};
    use crate::db::tax_rates::{self, TaxRateInput};
    use crate::db::tests_support::temp_db;

    fn line(description: &str, price: i64, tax_rate_id: Option<String>) -> LineItemInput {
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

    fn invoice(conn: &Connection, issue_date: &str, items: Vec<LineItemInput>) -> String {
        invoices::create(
            conn,
            &InvoiceInput {
                client_id: None,
                issue_date: issue_date.to_string(),
                due_date: Some("2099-01-01".to_string()),
                currency: "USD".to_string(),
                po_reference: None,
                discount_type: None,
                discount_value: None,
                notes: None,
                terms: None,
                payment_instructions: None,
                estimate_id: None,
                locale: None,
                items,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn revenue_groups_by_month_and_separates_invoiced_from_received() {
        let t = temp_db();
        let january = invoice(&t.conn, "2026-01-15", vec![line("A", 10_000, None)]);
        invoice(&t.conn, "2026-02-10", vec![line("B", 25_000, None)]);

        payments::create(
            &t.conn,
            &january,
            &PaymentInput {
                paid_on: "2026-02-03".to_string(),
                amount_minor: 4_000,
                method: Some("bank".to_string()),
                reference: None,
                note: None,
            },
        )
        .unwrap();

        let rows =
            revenue_by_period(&t.conn, "2026-01-01", "2026-12-31", Granularity::Month).unwrap();
        assert_eq!(rows.len(), 2);

        assert_eq!(rows[0].period, "2026-01");
        assert_eq!(rows[0].invoiced_minor, 10_000);
        assert_eq!(rows[0].paid_minor, 0);
        assert_eq!(rows[0].invoice_count, 1);

        assert_eq!(rows[1].period, "2026-02");
        assert_eq!(rows[1].invoiced_minor, 25_000);
        assert_eq!(
            rows[1].paid_minor, 4_000,
            "payment lands in the month it was received"
        );
    }

    #[test]
    fn void_invoices_are_excluded_everywhere() {
        let t = temp_db();
        let id = invoice(&t.conn, "2026-01-15", vec![line("A", 10_000, None)]);
        invoices::set_void(&t.conn, &id, true).unwrap();

        assert!(
            revenue_by_period(&t.conn, "2026-01-01", "2026-12-31", Granularity::Month)
                .unwrap()
                .is_empty()
        );
        assert!(revenue_by_client(&t.conn, "2026-01-01", "2026-12-31")
            .unwrap()
            .is_empty());
        assert!(tax_collected(&t.conn, "2026-01-01", "2026-12-31")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn tax_collected_reconciles_with_the_invoices() {
        let t = temp_db();
        let vat = tax_rates::create(
            &t.conn,
            &TaxRateInput {
                name: "VAT".to_string(),
                rate_bp: 2_000,
            },
        )
        .unwrap();

        let first = invoice(
            &t.conn,
            "2026-01-10",
            vec![
                line("Taxed", 19_999, Some(vat.id.clone())),
                line("Untaxed", 5_000, None),
            ],
        );
        let second = invoice(
            &t.conn,
            "2026-02-10",
            vec![line("Taxed", 7_777, Some(vat.id))],
        );

        let rows = tax_collected(&t.conn, "2026-01-01", "2026-12-31").unwrap();

        // Two rates: the VAT line and the untaxed line.
        let total_tax: i64 = rows.iter().map(|row| row.tax_minor).sum();
        let invoice_tax: i64 = [&first, &second]
            .iter()
            .map(|id| invoices::get(&t.conn, id).unwrap().unwrap().tax_minor)
            .sum();

        assert_eq!(total_tax, invoice_tax, "tax report must equal the invoices");
        assert!(rows
            .iter()
            .any(|row| row.rate_bp == 2_000 && row.tax_minor > 0));
        assert!(rows.iter().any(|row| row.name == "No tax"));
    }

    #[test]
    fn revenue_by_client_reports_outstanding() {
        let t = temp_db();
        let id = invoice(&t.conn, "2026-01-15", vec![line("A", 10_000, None)]);
        payments::create(
            &t.conn,
            &id,
            &PaymentInput {
                paid_on: "2026-01-20".to_string(),
                amount_minor: 3_000,
                method: None,
                reference: None,
                note: None,
            },
        )
        .unwrap();

        let rows = revenue_by_client(&t.conn, "2026-01-01", "2026-12-31").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].client_name, "No client");
        assert_eq!(rows[0].invoiced_minor, 10_000);
        assert_eq!(rows[0].paid_minor, 3_000);
        assert_eq!(rows[0].outstanding_minor, 7_000);
    }

    #[test]
    fn aged_receivables_buckets_past_due_invoices() {
        let t = temp_db();

        let with_due = |due: &str, price: i64| {
            let mut input = InvoiceInput {
                client_id: None,
                issue_date: "2026-01-01".to_string(),
                due_date: Some(due.to_string()),
                currency: "USD".to_string(),
                po_reference: None,
                discount_type: None,
                discount_value: None,
                notes: None,
                terms: None,
                payment_instructions: None,
                estimate_id: None,
                locale: None,
                items: vec![line("A", price, None)],
            };
            input.due_date = Some(due.to_string());
            let created = invoices::create(&t.conn, &input).unwrap();
            invoices::set_sent(&t.conn, &created.id, true).unwrap();
            created.id
        };

        with_due("2099-01-01", 10_000); // not yet due
        with_due("2020-01-01", 20_000); // long past due

        let rows = aged_receivables(&t.conn).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].label, "Not yet due");
        assert_eq!(rows[0].balance_minor, 10_000);
        assert_eq!(rows[4].label, "90+ days");
        assert_eq!(rows[4].balance_minor, 20_000);

        let total: i64 = rows.iter().map(|row| row.balance_minor).sum();
        assert_eq!(total, 30_000);

        // Settling one removes it from the report.
        let paid = with_due("2099-01-01", 5_000);
        payments::create(
            &t.conn,
            &paid,
            &PaymentInput {
                paid_on: "2026-01-05".to_string(),
                amount_minor: 5_000,
                method: None,
                reference: None,
                note: None,
            },
        )
        .unwrap();
        assert_eq!(aged_receivables(&t.conn).unwrap()[0].invoice_count, 1);
    }

    #[test]
    fn payments_group_by_method() {
        let t = temp_db();
        let id = invoice(&t.conn, "2026-01-15", vec![line("A", 50_000, None)]);

        for (method, amount) in [(Some("bank".to_string()), 20_000), (None, 5_000)] {
            payments::create(
                &t.conn,
                &id,
                &PaymentInput {
                    paid_on: "2026-01-20".to_string(),
                    amount_minor: amount,
                    method,
                    reference: None,
                    note: None,
                },
            )
            .unwrap();
        }

        let rows = payments_by_method(&t.conn, "2026-01-01", "2026-12-31").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].method, "bank");
        assert_eq!(rows[0].amount_minor, 20_000);
        assert_eq!(rows[1].method, "Unspecified");
    }
}
