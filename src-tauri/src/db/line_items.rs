//! Line items shared by invoices and estimates.
//!
//! Both documents have identical line-item semantics, so the input type, the
//! decimal/discount parsing and the insert live here once.

use rusqlite::{params, Connection};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

use super::{tax_rates, trim_opt};
use crate::domain::totals::{self, Discount, DiscountKind, InvoiceTotals, LineInput};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineItemInput {
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

fn default_quantity() -> String {
    "1".to_string()
}

pub(crate) fn parse_decimal(value: &str, fallback: Decimal) -> Decimal {
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

fn to_line_input(item: &LineItemInput, rates: &HashMap<String, i64>) -> LineInput {
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

/// Totals for unsaved lines, used by both editors for the live preview.
pub(crate) fn preview_totals(
    conn: &Connection,
    discount_type: &Option<String>,
    discount_value: &Option<String>,
    items: &[LineItemInput],
) -> rusqlite::Result<InvoiceTotals> {
    let rates = tax_rates::rate_map(conn)?;
    let lines: Vec<LineInput> = items
        .iter()
        .map(|item| to_line_input(item, &rates))
        .collect();
    Ok(totals::compute(
        &lines,
        discount_from(discount_type, discount_value),
    ))
}

/// An empty currency falls back to USD rather than violating NOT NULL.
pub(crate) fn currency_or_default(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "USD".to_string()
    } else {
        trimmed.to_string()
    }
}

/// The year part of an ISO date, falling back to the current year.
pub(crate) fn year_of(conn: &Connection, date: &str) -> rusqlite::Result<i64> {
    if let Some(year) = date.get(0..4).and_then(|part| part.parse::<i64>().ok()) {
        return Ok(year);
    }
    conn.query_row("SELECT CAST(strftime('%Y', 'now') AS INTEGER)", [], |row| {
        row.get(0)
    })
}

/// Insert the line items for one document, preserving order.
pub(crate) fn insert_items(
    conn: &Connection,
    table: &str,
    foreign_key: &str,
    parent_id: &str,
    items: &[LineItemInput],
) -> rusqlite::Result<()> {
    let sql = format!(
        "INSERT INTO {table} (
            id, {foreign_key}, sort_order, description, quantity, unit, unit_price_minor,
            discount_type, discount_value, tax_rate_id, tax_inclusive
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    );
    let mut stmt = conn.prepare(&sql)?;

    for (index, item) in items.iter().enumerate() {
        stmt.execute(params![
            uuid::Uuid::now_v7().to_string(),
            parent_id,
            index as i64,
            item.description.trim(),
            parse_decimal(&item.quantity, Decimal::ONE)
                .normalize()
                .to_string(),
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
