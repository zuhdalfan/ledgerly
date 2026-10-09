//! Invoice totals.
//!
//! All money is handled as integer minor units (e.g. cents) wrapped in `Decimal`
//! for intermediate math, then rounded back to whole minor units. Nothing is a
//! float, so totals never drift.
//!
//! Rounding is half-away-from-zero at the minor-unit boundary.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::Serialize;

/// Basis points per whole unit: 10_000 bp = 100%.
const BP_PER_UNIT: i64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscountKind {
    Percent,
    Fixed,
}

#[derive(Debug, Clone, Copy)]
pub struct Discount {
    pub kind: DiscountKind,
    /// Percent (e.g. 10 = 10%) or an absolute amount in minor units.
    pub value: Decimal,
}

impl Discount {
    fn amount(&self, base: Decimal) -> Decimal {
        match self.kind {
            DiscountKind::Percent => base * self.value / Decimal::ONE_HUNDRED,
            DiscountKind::Fixed => self.value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineInput {
    pub quantity: Decimal,
    pub unit_price_minor: i64,
    pub discount: Option<Discount>,
    /// Tax rate in basis points (1000 = 10%).
    pub tax_rate_bp: i64,
    /// When true the unit price already includes tax.
    pub tax_inclusive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineTotals {
    pub gross_minor: i64,
    pub discount_minor: i64,
    pub net_minor: i64,
    /// Base the tax was computed on, after all discounts.
    pub taxable_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceTotals {
    pub lines: Vec<LineTotals>,
    pub subtotal_minor: i64,
    /// Line discounts plus the invoice-level discount.
    pub discount_minor: i64,
    pub tax_minor: i64,
    pub total_minor: i64,
}

fn round_minor(value: Decimal) -> i64 {
    value
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(0)
}

struct Staged {
    gross_minor: i64,
    discount_minor: i64,
    net_minor: i64,
    tax_rate_bp: i64,
    tax_inclusive: bool,
}

pub fn compute(lines: &[LineInput], invoice_discount: Option<Discount>) -> InvoiceTotals {
    let mut staged = Vec::with_capacity(lines.len());
    let mut subtotal_minor = 0i64;
    let mut line_discount_minor = 0i64;

    for line in lines {
        let gross = Decimal::from(line.unit_price_minor) * line.quantity;

        let discount = match line.discount {
            Some(discount) if gross > Decimal::ZERO => {
                discount.amount(gross).clamp(Decimal::ZERO, gross)
            }
            _ => Decimal::ZERO,
        };

        let gross_minor = round_minor(gross);
        let discount_minor = round_minor(discount).clamp(0, gross_minor);
        let net_minor = gross_minor - discount_minor;

        subtotal_minor += gross_minor;
        line_discount_minor += discount_minor;
        staged.push(Staged {
            gross_minor,
            discount_minor,
            net_minor,
            tax_rate_bp: line.tax_rate_bp,
            tax_inclusive: line.tax_inclusive,
        });
    }

    let net_total: i64 = staged.iter().map(|line| line.net_minor).sum();

    let invoice_discount_total = match invoice_discount {
        Some(discount) if net_total > 0 => {
            round_minor(discount.amount(Decimal::from(net_total))).clamp(0, net_total)
        }
        _ => 0,
    };

    // Spread the invoice discount across lines in proportion to their net amount,
    // letting the last line absorb the rounding remainder so the shares sum exactly.
    let mut allocation = vec![0i64; staged.len()];
    if invoice_discount_total > 0 && net_total > 0 {
        let last = staged.iter().rposition(|line| line.net_minor > 0);
        let mut allocated = 0i64;

        for (index, line) in staged.iter().enumerate() {
            if Some(index) == last || line.net_minor == 0 {
                continue;
            }
            let share = Decimal::from(invoice_discount_total) * Decimal::from(line.net_minor)
                / Decimal::from(net_total);
            let share = round_minor(share).clamp(0, line.net_minor);
            allocation[index] = share;
            allocated += share;
        }

        if let Some(index) = last {
            allocation[index] =
                (invoice_discount_total - allocated).clamp(0, staged[index].net_minor);
        }
    }

    let mut out_lines = Vec::with_capacity(staged.len());
    let mut tax_minor = 0i64;
    let mut total_minor = 0i64;

    for (index, line) in staged.iter().enumerate() {
        let taxable_minor = line.net_minor - allocation[index];

        let tax = if line.tax_rate_bp > 0 {
            let rate = Decimal::from(line.tax_rate_bp);
            if line.tax_inclusive {
                round_minor(
                    Decimal::from(taxable_minor) * rate
                        / Decimal::from(BP_PER_UNIT + line.tax_rate_bp),
                )
            } else {
                round_minor(Decimal::from(taxable_minor) * rate / Decimal::from(BP_PER_UNIT))
            }
        } else {
            0
        };

        let line_total = if line.tax_inclusive {
            taxable_minor
        } else {
            taxable_minor + tax
        };

        tax_minor += tax;
        total_minor += line_total;
        out_lines.push(LineTotals {
            gross_minor: line.gross_minor,
            discount_minor: line.discount_minor,
            net_minor: line.net_minor,
            taxable_minor,
            tax_minor: tax,
            total_minor: line_total,
        });
    }

    InvoiceTotals {
        lines: out_lines,
        subtotal_minor,
        discount_minor: line_discount_minor + invoice_discount_total,
        tax_minor,
        total_minor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(value: &str) -> Decimal {
        Decimal::from_str(value).unwrap()
    }

    fn line(qty: &str, price: i64, bp: i64) -> LineInput {
        LineInput {
            quantity: d(qty),
            unit_price_minor: price,
            discount: None,
            tax_rate_bp: bp,
            tax_inclusive: false,
        }
    }

    fn percent(value: &str) -> Option<Discount> {
        Some(Discount {
            kind: DiscountKind::Percent,
            value: d(value),
        })
    }

    fn fixed(value: i64) -> Option<Discount> {
        Some(Discount {
            kind: DiscountKind::Fixed,
            value: Decimal::from(value),
        })
    }

    #[test]
    fn empty_invoice_is_all_zero() {
        let totals = compute(&[], None);
        assert_eq!(totals.subtotal_minor, 0);
        assert_eq!(totals.total_minor, 0);
        assert!(totals.lines.is_empty());
    }

    #[test]
    fn exclusive_tax_simple() {
        // 2 x 100.00 at 10% = 200.00 + 20.00
        let totals = compute(&[line("2", 10_000, 1_000)], None);
        assert_eq!(totals.subtotal_minor, 20_000);
        assert_eq!(totals.tax_minor, 2_000);
        assert_eq!(totals.total_minor, 22_000);
    }

    #[test]
    fn line_percent_discount_before_tax() {
        // 2 x 100.00 = 200.00, less 10% = 180.00, plus 10% tax = 198.00
        let mut item = line("2", 10_000, 1_000);
        item.discount = percent("10");
        let totals = compute(&[item], None);

        assert_eq!(totals.subtotal_minor, 20_000);
        assert_eq!(totals.discount_minor, 2_000);
        assert_eq!(totals.lines[0].taxable_minor, 18_000);
        assert_eq!(totals.tax_minor, 1_800);
        assert_eq!(totals.total_minor, 19_800);
    }

    #[test]
    fn tax_inclusive_backs_out_the_tax() {
        // 110.00 inclusive of 10% -> tax 10.00, total 110.00
        let mut item = line("1", 11_000, 1_000);
        item.tax_inclusive = true;
        let totals = compute(&[item], None);

        assert_eq!(totals.subtotal_minor, 11_000);
        assert_eq!(totals.lines[0].taxable_minor, 11_000);
        assert_eq!(totals.tax_minor, 1_000);
        assert_eq!(totals.total_minor, 11_000);
    }

    #[test]
    fn mixed_inclusive_and_exclusive() {
        let mut inclusive = line("1", 11_000, 1_000);
        inclusive.tax_inclusive = true;
        let exclusive = line("1", 10_000, 1_000);

        let totals = compute(&[inclusive, exclusive], None);
        assert_eq!(totals.tax_minor, 1_000 + 1_000);
        assert_eq!(totals.total_minor, 11_000 + 11_000);
    }

    #[test]
    fn invoice_discount_is_allocated_exactly() {
        // Three lines of 10.00; a 5.00 invoice discount cannot split evenly,
        // so the shares must still sum to exactly 5.00.
        let lines = [
            line("1", 1_000, 0),
            line("1", 1_000, 0),
            line("1", 1_000, 0),
        ];
        let totals = compute(&lines, fixed(500));

        let taxable: i64 = totals.lines.iter().map(|l| l.taxable_minor).sum();
        assert_eq!(taxable, 2_500);
        assert_eq!(totals.discount_minor, 500);
        assert_eq!(totals.total_minor, 2_500);
    }

    #[test]
    fn exclusive_invoice_keeps_subtotal_minus_discount_plus_tax_identity() {
        let mut a = line("3", 4_999, 750);
        a.discount = percent("12.5");
        let b = line("1", 12_345, 2_100);
        let c = line("2.5", 1_111, 0);

        let totals = compute(&[a, b, c], percent("7.5"));

        assert_eq!(
            totals.subtotal_minor - totals.discount_minor + totals.tax_minor,
            totals.total_minor
        );
        assert_eq!(
            totals.lines.iter().map(|l| l.total_minor).sum::<i64>(),
            totals.total_minor
        );
    }

    #[test]
    fn rounding_is_half_away_from_zero() {
        // 1.5 x 3.33 = 4.995 -> 5.00 (half away from zero)
        let mut item = line("1.5", 333, 1_000);
        item.tax_inclusive = false;
        let totals = compute(&[item], None);

        assert_eq!(totals.subtotal_minor, 500);
        assert_eq!(totals.tax_minor, 50);
        assert_eq!(totals.total_minor, 550);
    }

    #[test]
    fn discounts_are_clamped_to_the_line_amount() {
        let mut item = line("1", 1_000, 0);
        item.discount = fixed(5_000);
        let totals = compute(&[item], fixed(9_999));

        assert_eq!(totals.lines[0].discount_minor, 1_000);
        assert_eq!(totals.lines[0].net_minor, 0);
        assert_eq!(totals.total_minor, 0);
    }

    #[test]
    fn percent_invoice_discount_sums_exactly_across_many_lines() {
        let lines = [
            line("1", 1_111, 500),
            line("2", 2_222, 1_000),
            line("3", 3_333, 2_000),
            line("0.5", 4_444, 0),
        ];
        let totals = compute(&lines, percent("33.333"));

        let allocated: i64 = totals
            .lines
            .iter()
            .map(|l| l.net_minor - l.taxable_minor)
            .sum();

        // The allocated discount matches what we reported, to the cent.
        assert_eq!(allocated, totals.discount_minor);
        assert_eq!(
            totals.subtotal_minor - totals.discount_minor + totals.tax_minor,
            totals.total_minor
        );
    }
}
