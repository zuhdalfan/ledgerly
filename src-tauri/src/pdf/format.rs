//! Locale-free formatting helpers used by the PDF templates.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
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

fn group_digits(value: i64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(character);
    }
    out
}

/// Render integer minor units as e.g. `$1,234.50`, falling back to `CODE 1,234.50`.
pub fn format_money(minor: i64, currency: &str) -> String {
    let code = currency.trim().to_ascii_uppercase();
    let decimals = if has_no_decimals(&code) { 0 } else { 2 };
    let scale = 10i64.pow(decimals);

    let sign = if minor < 0 { "-" } else { "" };
    let absolute = minor.abs();
    let whole = group_digits(absolute / scale);

    let body = if decimals == 0 {
        whole
    } else {
        format!("{whole}.{:0width$}", absolute % scale, width = decimals as usize)
    };

    match symbol_for(&code) {
        Some(symbol) => format!("{sign}{symbol}{body}"),
        None if code.is_empty() => format!("{sign}{body}"),
        None => format!("{sign}{code} {body}"),
    }
}

/// Render an ISO `YYYY-MM-DD` date as `15 Mar 2026`, leaving anything else alone.
pub fn format_date(value: &str) -> String {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() < 3 {
        return value.to_string();
    }

    let month = parts[1]
        .parse::<usize>()
        .ok()
        .filter(|month| (1..=12).contains(month))
        .map(|month| MONTHS[month - 1]);

    match month {
        Some(month) => format!(
            "{} {month} {}",
            parts[2].trim_start_matches('0'),
            parts[0]
        ),
        None => value.to_string(),
    }
}

/// Render a tax rate in basis points as a compact percentage.
pub fn format_percent_bp(basis_points: i64) -> String {
    let percent = basis_points as f64 / 100.0;
    if (percent.fract()).abs() < f64::EPSILON {
        format!("{percent:.0}%")
    } else {
        format!("{percent:.2}%")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_money_with_symbol_and_grouping() {
        assert_eq!(format_money(123_456, "USD"), "$1,234.56");
        assert_eq!(format_money(0, "USD"), "$0.00");
        assert_eq!(format_money(-2_500, "EUR"), "-€25.00");
        assert_eq!(format_money(1_000, "JPY"), "¥1,000");
        assert_eq!(format_money(50_000, "XYZ"), "XYZ 500.00");
    }

    #[test]
    fn formats_dates() {
        assert_eq!(format_date("2026-03-15"), "15 Mar 2026");
        assert_eq!(format_date("2026-12-01"), "1 Dec 2026");
        assert_eq!(format_date("not-a-date"), "not-a-date");
    }

    #[test]
    fn formats_tax_percentages() {
        assert_eq!(format_percent_bp(1_000), "10%");
        assert_eq!(format_percent_bp(750), "7.50%");
        assert_eq!(format_percent_bp(0), "0%");
    }
}
