//! Formatting helpers used by the PDF templates.
//!
//! The document language decides more than the words: a reader in Indonesia
//! expects `Rp1.234,56` and `15 Mei 2026`, not the American grouping and month
//! names. Keeping the locale here means every figure on the page agrees with
//! the labels around it.

use super::labels::Locale;

/// Indonesian month abbreviations. The rest coincide with English, but a
/// separate table keeps the two visible side by side when a language is added.
const MONTHS_EN: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_ID: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des",
];

/// Currencies with no minor unit.
fn has_no_decimals(code: &str) -> bool {
    matches!(code, "JPY" | "KRW" | "VND" | "CLP" | "ISK")
}

fn symbol_for(code: &str) -> Option<&'static str> {
    Some(match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "INR" => "₹",
        "SGD" => "S$",
        "AUD" => "A$",
        "CAD" => "C$",
        "NZD" => "NZ$",
        "HKD" => "HK$",
        "CHF" => "CHF ",
        "SEK" | "NOK" | "DKK" => "kr ",
        "ZAR" => "R",
        "BRL" => "R$",
        "MXN" => "MX$",
        "MYR" => "RM",
        "IDR" => "Rp",
        "THB" => "฿",
        "PHP" => "₱",
        "VND" => "₫",
        "KRW" => "₩",
        "TRY" => "₺",
        "PLN" => "zł ",
        "AED" => "AED ",
        _ => return None,
    })
}

/// Grouping and decimal separators, matching what the UI shows for the same
/// language (CLDR: `1,234.56` in English, `1.234,56` in Indonesian).
fn separators(locale: Locale) -> (char, char) {
    match locale {
        Locale::En => (',', '.'),
        Locale::Id => ('.', ','),
    }
}

fn group_digits(value: u64, separator: char) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(character);
    }
    out
}

/// Render integer minor units as e.g. `$1,234.50` in English or `Rp1.234,50`
/// in Indonesian, falling back to `CODE 1,234.50`.
pub fn format_money(minor: i64, currency: &str, locale: Locale) -> String {
    let code = currency.trim().to_ascii_uppercase();
    let decimals = if has_no_decimals(&code) { 0u32 } else { 2 };
    let scale = 10u64.pow(decimals);
    let (grouping, point) = separators(locale);

    let sign = if minor < 0 { "-" } else { "" };
    let absolute = minor.unsigned_abs();
    let whole = group_digits(absolute / scale, grouping);

    let body = if decimals == 0 {
        whole
    } else {
        format!(
            "{whole}{point}{:0width$}",
            absolute % scale,
            width = decimals as usize
        )
    };

    match symbol_for(&code) {
        Some(symbol) => format!("{sign}{symbol}{body}"),
        None if code.is_empty() => format!("{sign}{body}"),
        None => format!("{sign}{code} {body}"),
    }
}

/// Render an ISO `YYYY-MM-DD` date as `15 Mar 2026`, leaving anything else
/// alone. Indonesian uses the same day-month-year order with its own month
/// abbreviations (`15 Mei 2026`).
pub fn format_date(value: &str, locale: Locale) -> String {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() < 3 {
        return value.to_string();
    }

    let months = match locale {
        Locale::En => &MONTHS_EN,
        Locale::Id => &MONTHS_ID,
    };

    let month = parts[1]
        .parse::<usize>()
        .ok()
        .filter(|month| (1..=12).contains(month))
        .map(|month| months[month - 1]);

    match month {
        Some(month) => format!("{} {month} {}", parts[2].trim_start_matches('0'), parts[0]),
        None => value.to_string(),
    }
}

/// Render a tax rate in basis points as a compact percentage, with the
/// language's decimal separator (`7,50%` in Indonesian).
pub fn format_percent_bp(basis_points: i64, locale: Locale) -> String {
    let percent = basis_points as f64 / 100.0;
    let rendered = if (percent.fract()).abs() < f64::EPSILON {
        format!("{percent:.0}")
    } else {
        format!("{percent:.2}")
    };

    match locale {
        Locale::En => format!("{rendered}%"),
        Locale::Id => format!("{}%", rendered.replace('.', ",")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_money_with_symbol_and_grouping() {
        assert_eq!(format_money(123_456, "USD", Locale::En), "$1,234.56");
        assert_eq!(format_money(0, "USD", Locale::En), "$0.00");
        assert_eq!(format_money(-2_500, "EUR", Locale::En), "-€25.00");
        assert_eq!(format_money(1_000, "JPY", Locale::En), "¥1,000");
        assert_eq!(format_money(50_000, "XYZ", Locale::En), "XYZ 500.00");
    }

    /// An Indonesian invoice must not print `Rp1,234.56`: the separators are
    /// part of the language, not just the labels.
    #[test]
    fn indonesian_money_swaps_the_separators() {
        assert_eq!(
            format_money(123_456_789, "IDR", Locale::Id),
            "Rp1.234.567,89"
        );
        assert_eq!(format_money(123_456, "USD", Locale::Id), "$1.234,56");
        assert_eq!(format_money(-2_500, "EUR", Locale::Id), "-€25,00");
    }

    #[test]
    fn formats_dates() {
        assert_eq!(format_date("2026-03-15", Locale::En), "15 Mar 2026");
        assert_eq!(format_date("2026-12-01", Locale::En), "1 Dec 2026");
        assert_eq!(format_date("not-a-date", Locale::En), "not-a-date");
    }

    #[test]
    fn indonesian_dates_use_indonesian_months() {
        assert_eq!(format_date("2026-05-15", Locale::Id), "15 Mei 2026");
        assert_eq!(format_date("2026-08-17", Locale::Id), "17 Agu 2026");
        assert_eq!(format_date("2026-12-01", Locale::Id), "1 Des 2026");
        // Months whose abbreviations are shared stay put.
        assert_eq!(format_date("2026-03-15", Locale::Id), "15 Mar 2026");
    }

    #[test]
    fn formats_tax_percentages() {
        assert_eq!(format_percent_bp(1_000, Locale::En), "10%");
        assert_eq!(format_percent_bp(750, Locale::En), "7.50%");
        assert_eq!(format_percent_bp(0, Locale::En), "0%");
        assert_eq!(format_percent_bp(750, Locale::Id), "7,50%");
    }
}
