//! Document labels, so an invoice can be printed in a different language from
//! the app UI. English is complete; adding a language means adding a `Labels`
//! table and an arm in `Locale::parse`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
}

impl Locale {
    /// Unknown locales fall back to English rather than failing.
    pub fn parse(_tag: &str) -> Locale {
        // Only English ships today; add arms here as translations arrive.
        Locale::En
    }

    pub fn labels(self) -> &'static Labels {
        match self {
            Locale::En => &EN,
        }
    }
}

/// Every piece of fixed text that appears on a document.
pub struct Labels {
    pub invoice: &'static str,
    pub estimate: &'static str,
    pub statement: &'static str,
    pub bill_to: &'static str,
    pub issued: &'static str,
    pub due: &'static str,
    pub valid_until: &'static str,
    pub reference: &'static str,
    pub description: &'static str,
    pub quantity: &'static str,
    pub price: &'static str,
    pub discount: &'static str,
    pub tax: &'static str,
    pub total: &'static str,
    pub subtotal: &'static str,
    pub total_due: &'static str,
    pub total_plain: &'static str,
    pub total_outstanding: &'static str,
    pub balance: &'static str,
    pub notes: &'static str,
    pub terms: &'static str,
    pub payment_instructions: &'static str,
    pub no_client: &'static str,
    pub no_items: &'static str,
    pub page: &'static str,
    pub of: &'static str,
    pub void: &'static str,
}

static EN: Labels = Labels {
    invoice: "INVOICE",
    estimate: "ESTIMATE",
    statement: "STATEMENT",
    bill_to: "BILL TO",
    issued: "Issued",
    due: "Due",
    valid_until: "Valid until",
    reference: "Ref",
    description: "DESCRIPTION",
    quantity: "QTY",
    price: "PRICE",
    discount: "DISC",
    tax: "TAX",
    total: "TOTAL",
    subtotal: "Subtotal",
    total_due: "Total due",
    total_plain: "Total",
    total_outstanding: "Total outstanding",
    balance: "BALANCE",
    notes: "NOTES",
    terms: "TERMS",
    payment_instructions: "PAYMENT INSTRUCTIONS",
    no_client: "No client selected",
    no_items: "No line items",
    page: "Page",
    of: "of",
    void: "VOID",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_locales_fall_back_to_english() {
        assert_eq!(Locale::parse("en"), Locale::En);
        assert_eq!(Locale::parse("EN-GB"), Locale::En);
        assert_eq!(Locale::parse("fr"), Locale::En);
        assert_eq!(Locale::parse(""), Locale::En);
    }

    #[test]
    fn english_labels_are_complete() {
        let labels = Locale::En.labels();
        // Every label must be present and non-empty.
        for value in [
            labels.invoice,
            labels.estimate,
            labels.statement,
            labels.bill_to,
            labels.issued,
            labels.due,
            labels.valid_until,
            labels.reference,
            labels.description,
            labels.quantity,
            labels.price,
            labels.discount,
            labels.tax,
            labels.total,
            labels.subtotal,
            labels.total_due,
            labels.total_plain,
            labels.total_outstanding,
            labels.balance,
            labels.notes,
            labels.terms,
            labels.payment_instructions,
            labels.no_client,
            labels.no_items,
            labels.page,
            labels.of,
            labels.void,
        ] {
            assert!(!value.trim().is_empty(), "a label is missing");
        }
    }
}
