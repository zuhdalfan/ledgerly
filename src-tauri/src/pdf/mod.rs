//! Invoice PDF rendering.
//!
//! Everything is drawn here in Rust with an embedded font, so the output is
//! identical on every machine and printer, and nothing needs the network.

mod format;
mod labels;
mod text;

use printpdf::*;

use crate::db::business::Business;
use crate::db::clients::Client;
use crate::db::invoices::{self, Invoice, InvoiceItem};
use crate::db::line_items;
use crate::domain::totals::{self, InvoiceTotals, LineTotals};

use format::{format_date, format_money, format_percent_bp};
use labels::{Labels, Locale};
use text::TextMeasure;

pub use labels::Locale as DocumentLocale;

/// What to render, and how.
#[derive(Debug, Clone, Copy)]
pub struct PdfOptions {
    pub template: Template,
    /// Free invoices carry a small "Made with Ledgerly" footer.
    pub promo_footer: bool,
    /// Language of the labels on the document itself.
    pub locale: Locale,
}

static FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
static FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-Bold.ttf");

// A4 in millimetres.
const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 18.0;
const CONTENT_RIGHT: f32 = PAGE_W - MARGIN;
const CONTENT_BOTTOM: f32 = 24.0;
const FOOTER_Y: f32 = 12.0;

// Table column right edges.
const COL_QTY: f32 = 118.0;
const COL_PRICE: f32 = 142.0;
const COL_DISC: f32 = 162.0;
const COL_TAX: f32 = 178.0;
const COL_TOTAL: f32 = CONTENT_RIGHT;
const DESC_WIDTH: f32 = COL_QTY - 24.0 - MARGIN;

/// An RGB colour as plain floats, so themes stay cheap to copy.
type Color3 = [f32; 3];

fn rgb(color: Color3) -> Rgb {
    Rgb::new(color[0], color[1], color[2], None)
}

const INK: Color3 = [0.11, 0.11, 0.13];
const MUTED: Color3 = [0.45, 0.45, 0.50];
const RULE: Color3 = [0.80, 0.80, 0.84];
const SHADE: Color3 = [0.96, 0.96, 0.97];
const WATERMARK: Color3 = [0.90, 0.90, 0.92];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Classic,
    Modern,
    Minimal,
}

impl Template {
    pub fn parse(name: &str) -> Template {
        match name.trim().to_ascii_lowercase().as_str() {
            "modern" => Template::Modern,
            "minimal" => Template::Minimal,
            _ => Template::Classic,
        }
    }
}

#[derive(Clone, Copy)]
struct Theme {
    accent: Color3,
    ink: Color3,
    muted: Color3,
    rule: Color3,
    shade: Color3,
    band: Option<Color3>,
    shade_rows: bool,
    panel: bool,
    title_size: f32,
}

impl Theme {
    fn for_template(template: Template) -> Theme {
        match template {
            Template::Classic => Theme {
                accent: [0.12, 0.19, 0.33],
                ink: INK,
                muted: MUTED,
                rule: RULE,
                shade: SHADE,
                band: None,
                shade_rows: false,
                panel: false,
                title_size: 24.0,
            },
            Template::Modern => Theme {
                accent: [0.31, 0.27, 0.90],
                ink: INK,
                muted: MUTED,
                rule: RULE,
                shade: [0.95, 0.95, 0.99],
                band: Some([0.31, 0.27, 0.90]),
                shade_rows: true,
                panel: true,
                title_size: 22.0,
            },
            Template::Minimal => Theme {
                accent: [0.15, 0.15, 0.17],
                ink: INK,
                muted: [0.55, 0.55, 0.58],
                rule: [0.89, 0.89, 0.91],
                shade: SHADE,
                band: None,
                shade_rows: false,
                panel: false,
                title_size: 18.0,
            },
        }
    }
}

struct Painter {
    doc: PdfDocumentReference,
    font_regular: IndirectFontRef,
    font_bold: IndirectFontRef,
    measure: TextMeasure,
    bold_measure: TextMeasure,
    theme: Theme,
    promo: bool,
    labels: &'static Labels,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    pages: Vec<(PdfPageIndex, PdfLayerIndex)>,
}

impl Painter {
    fn layer(&self) -> PdfLayerReference {
        self.doc.get_page(self.page).get_layer(self.layer)
    }

    fn rect(&self, x: f32, y: f32, width: f32, height: f32, color: Color3) {
        let layer = self.layer();
        layer.set_fill_color(Color::Rgb(rgb(color)));
        layer.add_rect(Rect::new(Mm(x), Mm(y), Mm(x + width), Mm(y + height)));
    }

    // A drawing primitive: every argument is a distinct coordinate or style.
    #[allow(clippy::too_many_arguments)]
    fn text_on(
        &self,
        layer: &PdfLayerReference,
        value: &str,
        size: f32,
        x: f32,
        y: f32,
        bold: bool,
        color: Color3,
    ) {
        layer.set_fill_color(Color::Rgb(rgb(color)));
        let font = if bold {
            &self.font_bold
        } else {
            &self.font_regular
        };
        layer.use_text(value, size, Mm(x), Mm(y), font);
    }

    fn text(&self, value: &str, size: f32, x: f32, y: f32, bold: bool, color: Color3) {
        self.text_on(&self.layer(), value, size, x, y, bold, color);
    }

    fn width_mm(&self, value: &str, size: f32, bold: bool) -> f32 {
        if bold {
            self.bold_measure.width_mm(value, size)
        } else {
            self.measure.width_mm(value, size)
        }
    }

    fn text_right(&self, value: &str, size: f32, x_right: f32, y: f32, bold: bool, color: Color3) {
        let width = self.width_mm(value, size, bold);
        self.text(value, size, x_right - width, y, bold, color);
    }

    fn new_page(&mut self) {
        let (page, layer) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "content");
        self.page = page;
        self.layer = layer;
        self.pages.push((page, layer));
    }

    fn letterhead(&self, business: &Business, invoice: &Invoice) -> f32 {
        let theme = self.theme;

        if let Some(band) = theme.band {
            self.rect(0.0, PAGE_H - 8.0, PAGE_W, 8.0, band);
        }

        let mut y = PAGE_H - MARGIN - 2.0;
        self.text(
            display_name_or(&business.display_name, "Your business"),
            15.0,
            MARGIN,
            y,
            true,
            theme.ink,
        );
        y -= 5.5;
        for line in business_lines(business) {
            self.text(&line, 8.5, MARGIN, y, false, theme.muted);
            y -= 4.0;
        }

        let mut ry = PAGE_H - MARGIN - 2.0;
        self.text_right(
            self.labels.invoice,
            theme.title_size,
            CONTENT_RIGHT,
            ry,
            true,
            theme.accent,
        );
        ry -= 7.5;
        self.text_right(&invoice.number, 10.5, CONTENT_RIGHT, ry, true, theme.ink);
        ry -= 5.0;
        self.text_right(
            &format!(
                "{} {}",
                self.labels.issued,
                format_date(&invoice.issue_date)
            ),
            8.5,
            CONTENT_RIGHT,
            ry,
            false,
            theme.muted,
        );
        ry -= 4.2;

        if let Some(due) = non_empty(&invoice.due_date) {
            self.text_right(
                &format!("{} {}", self.labels.due, format_date(due)),
                8.5,
                CONTENT_RIGHT,
                ry,
                false,
                theme.muted,
            );
            ry -= 4.2;
        }

        if let Some(reference) = non_empty(&invoice.po_reference) {
            self.text_right(
                &format!("{} {reference}", self.labels.reference),
                8.5,
                CONTENT_RIGHT,
                ry,
                false,
                theme.muted,
            );
            ry -= 4.2;
        }

        y.min(ry) - 10.0
    }

    fn client_block(&self, client: Option<&Client>, mut y: f32) -> f32 {
        let theme = self.theme;
        self.text(self.labels.bill_to, 7.5, MARGIN, y, true, theme.muted);
        y -= 5.0;

        match client {
            Some(client) => {
                self.text(client.display_name.trim(), 11.0, MARGIN, y, true, theme.ink);
                y -= 5.0;
                for line in client_lines(client) {
                    self.text(&line, 9.0, MARGIN, y, false, theme.ink);
                    y -= 4.2;
                }
            }
            None => {
                self.text(self.labels.no_client, 10.0, MARGIN, y, false, theme.muted);
                y -= 5.0;
            }
        }

        y
    }

    fn table_header(&self, y: f32) -> f32 {
        let theme = self.theme;
        self.rect(MARGIN, y - 1.8, CONTENT_RIGHT - MARGIN, 0.4, theme.rule);

        let size = 7.5;
        self.text(self.labels.description, size, MARGIN, y, true, theme.muted);
        self.text_right(self.labels.quantity, size, COL_QTY, y, true, theme.muted);
        self.text_right(self.labels.price, size, COL_PRICE, y, true, theme.muted);
        self.text_right(self.labels.discount, size, COL_DISC, y, true, theme.muted);
        self.text_right(self.labels.tax, size, COL_TAX, y, true, theme.muted);
        self.text_right(self.labels.total, size, COL_TOTAL, y, true, theme.muted);

        y - 6.0
    }

    fn table_row(
        &mut self,
        mut y: f32,
        index: usize,
        item: &InvoiceItem,
        line: Option<&LineTotals>,
        currency: &str,
    ) -> f32 {
        let description = if item.description.trim().is_empty() {
            "—".to_string()
        } else {
            item.description.trim().to_string()
        };
        let lines = self.measure.wrap(&description, 9.0, DESC_WIDTH);
        let row_height = lines.len() as f32 * 4.4 + 2.6;

        if y - row_height < CONTENT_BOTTOM {
            self.new_page();
            y = PAGE_H - MARGIN - 6.0;
            y = self.table_header(y);
        }

        let theme = self.theme;

        if theme.shade_rows && index % 2 == 1 {
            self.rect(
                MARGIN - 1.5,
                y - row_height + 3.4,
                CONTENT_RIGHT - MARGIN + 3.0,
                row_height - 1.0,
                theme.shade,
            );
        }

        let mut dy = y;
        for (line_index, text) in lines.iter().enumerate() {
            self.text(text, 9.0, MARGIN, dy, false, theme.ink);

            if line_index == 0 {
                self.text_right(&item.quantity, 9.0, COL_QTY, dy, false, theme.ink);
                self.text_right(
                    &format_money(item.unit_price_minor, currency),
                    9.0,
                    COL_PRICE,
                    dy,
                    false,
                    theme.ink,
                );
                self.text_right(
                    &discount_label(item, currency),
                    9.0,
                    COL_DISC,
                    dy,
                    false,
                    theme.muted,
                );
                self.text_right(&tax_label(item), 9.0, COL_TAX, dy, false, theme.muted);
                self.text_right(
                    &format_money(line.map(|l| l.total_minor).unwrap_or(0), currency),
                    9.0,
                    COL_TOTAL,
                    dy,
                    false,
                    theme.ink,
                );
            }

            dy -= 4.4;
        }

        y - row_height
    }

    fn totals_block(&mut self, totals: &InvoiceTotals, currency: &str, mut y: f32) -> f32 {
        let theme = self.theme;
        y -= 6.0;

        if y - 36.0 < CONTENT_BOTTOM {
            self.new_page();
            y = PAGE_H - MARGIN - 10.0;
        }

        let label_x = COL_TOTAL - 34.0;

        if theme.panel {
            self.rect(
                label_x - 6.0,
                y - 22.0,
                CONTENT_RIGHT - label_x + 6.0,
                26.0,
                theme.shade,
            );
        }

        self.text_right(self.labels.subtotal, 9.0, label_x, y, false, theme.muted);
        self.text_right(
            &format_money(totals.subtotal_minor, currency),
            9.0,
            COL_TOTAL,
            y,
            false,
            theme.ink,
        );
        y -= 5.0;

        if totals.discount_minor != 0 {
            self.text_right(self.labels.discount, 9.0, label_x, y, false, theme.muted);
            self.text_right(
                &format!("-{}", format_money(totals.discount_minor, currency)),
                9.0,
                COL_TOTAL,
                y,
                false,
                theme.ink,
            );
            y -= 5.0;
        }

        self.text_right(self.labels.tax, 9.0, label_x, y, false, theme.muted);
        self.text_right(
            &format_money(totals.tax_minor, currency),
            9.0,
            COL_TOTAL,
            y,
            false,
            theme.ink,
        );
        y -= 6.0;

        self.rect(label_x, y + 3.4, COL_TOTAL - label_x, 0.4, theme.rule);
        self.text_right(
            self.labels.total_due,
            11.0,
            label_x,
            y - 2.4,
            true,
            theme.ink,
        );
        self.text_right(
            &format_money(totals.total_minor, currency),
            11.0,
            COL_TOTAL,
            y - 2.4,
            true,
            theme.accent,
        );

        y - 14.0
    }

    fn notes_block(&mut self, invoice: &Invoice, mut y: f32) -> f32 {
        let theme = self.theme;
        let sections: [(&str, Option<&str>); 3] = [
            (self.labels.notes, non_empty(&invoice.notes)),
            (self.labels.terms, non_empty(&invoice.terms)),
            (
                self.labels.payment_instructions,
                non_empty(&invoice.payment_instructions),
            ),
        ];

        for (label, content) in sections {
            let Some(content) = content else {
                continue;
            };

            let lines = self.measure.wrap(content, 9.0, CONTENT_RIGHT - MARGIN);
            let height = 10.0 + lines.len() as f32 * 4.2;

            if y - height < CONTENT_BOTTOM {
                self.new_page();
                y = PAGE_H - MARGIN - 6.0;
            }

            self.text(label, 7.5, MARGIN, y, true, theme.muted);
            y -= 4.8;
            for line in lines {
                self.text(&line, 9.0, MARGIN, y, false, theme.ink);
                y -= 4.2;
            }
            y -= 5.0;
        }

        y
    }

    fn watermark(&self, invoice: &Invoice) {
        if invoice.status != "void" {
            return;
        }

        let (page, layer_index) = match self.pages.first() {
            Some(first) => *first,
            None => return,
        };
        let layer = self.doc.get_page(page).get_layer(layer_index);

        let size = 64.0;
        let width = self.width_mm(self.labels.void, size, true);
        self.text_on(
            &layer,
            self.labels.void,
            size,
            (PAGE_W - width) / 2.0,
            PAGE_H / 2.0 - 10.0,
            true,
            WATERMARK,
        );
    }

    fn footers(&self, business: &Business) {
        let total = self.pages.len();

        for (index, (page, layer_index)) in self.pages.iter().enumerate() {
            let layer = self.doc.get_page(*page).get_layer(*layer_index);
            let page_label = format!(
                "{} {} {} {total}",
                self.labels.page,
                index + 1,
                self.labels.of
            );

            self.text_on(
                &layer,
                display_name_or(&business.display_name, "Ledgerly"),
                8.0,
                MARGIN,
                FOOTER_Y,
                false,
                self.theme.muted,
            );
            let width = self.width_mm(&page_label, 8.0, false);
            self.text_on(
                &layer,
                &page_label,
                8.0,
                CONTENT_RIGHT - width,
                FOOTER_Y,
                false,
                self.theme.muted,
            );

            if self.promo {
                let promo = self.labels.promo;
                let width = self.width_mm(promo, 7.0, false);
                self.text_on(
                    &layer,
                    promo,
                    7.0,
                    (PAGE_W - width) / 2.0,
                    FOOTER_Y,
                    false,
                    self.theme.muted,
                );
            }
        }
    }
}

fn display_name_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

fn non_empty(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn push_line(lines: &mut Vec<String>, value: &Option<String>) {
    if let Some(text) = non_empty(value) {
        lines.push(text.to_string());
    }
}

fn locality(
    city: &Option<String>,
    region: &Option<String>,
    postal_code: &Option<String>,
    country: &Option<String>,
) -> Option<String> {
    let parts: Vec<&str> = [city, region, postal_code, country]
        .iter()
        .filter_map(|field| non_empty(field))
        .collect();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn business_lines(business: &Business) -> Vec<String> {
    let mut lines = Vec::new();
    push_line(&mut lines, &business.address_line1);
    push_line(&mut lines, &business.address_line2);
    if let Some(line) = locality(
        &business.city,
        &business.region,
        &business.postal_code,
        &business.country,
    ) {
        lines.push(line);
    }
    push_line(&mut lines, &business.email);
    push_line(&mut lines, &business.phone);
    if let Some(tax_id) = non_empty(&business.tax_id) {
        lines.push(format!("Tax ID: {tax_id}"));
    }
    lines
}

fn client_lines(client: &Client) -> Vec<String> {
    let mut lines = Vec::new();
    push_line(&mut lines, &client.company);
    push_line(&mut lines, &client.address_line1);
    push_line(&mut lines, &client.address_line2);
    if let Some(line) = locality(
        &client.city,
        &client.region,
        &client.postal_code,
        &client.country,
    ) {
        lines.push(line);
    }
    push_line(&mut lines, &client.email);
    push_line(&mut lines, &client.phone);
    if let Some(tax_id) = non_empty(&client.tax_id) {
        lines.push(format!("Tax ID: {tax_id}"));
    }
    lines
}

fn discount_label(item: &InvoiceItem, currency: &str) -> String {
    match (
        item.discount_type.as_deref(),
        item.discount_value.as_deref(),
    ) {
        (Some("percent"), Some(value)) => {
            if value.trim().is_empty() {
                String::new()
            } else {
                format!("{}%", value.trim())
            }
        }
        (Some("fixed"), Some(value)) => match value.trim().parse::<i64>() {
            Ok(minor) if minor != 0 => format_money(minor, currency),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

fn tax_label(item: &InvoiceItem) -> String {
    match item.tax_rate_bp {
        Some(bp) if bp != 0 => format_percent_bp(bp),
        _ => String::new(),
    }
}

/// Render an invoice to PDF bytes.
pub fn render_invoice(
    invoice: &Invoice,
    business: &Business,
    client: Option<&Client>,
    options: PdfOptions,
) -> Result<Vec<u8>, String> {
    let measure = TextMeasure::new(FONT_REGULAR)
        .ok_or_else(|| "could not read the regular font".to_string())?;
    let bold_measure =
        TextMeasure::new(FONT_BOLD).ok_or_else(|| "could not read the bold font".to_string())?;

    let (doc, page, layer) = PdfDocument::new("Invoice", Mm(PAGE_W), Mm(PAGE_H), "content");

    let mut regular_reader = std::io::Cursor::new(FONT_REGULAR);
    let font_regular = doc
        .add_external_font(&mut regular_reader)
        .map_err(|error| error.to_string())?;
    let mut bold_reader = std::io::Cursor::new(FONT_BOLD);
    let font_bold = doc
        .add_external_font(&mut bold_reader)
        .map_err(|error| error.to_string())?;

    let mut painter = Painter {
        doc,
        font_regular,
        font_bold,
        measure,
        bold_measure,
        theme: Theme::for_template(options.template),
        promo: options.promo_footer,
        labels: options.locale.labels(),
        page,
        layer,
        pages: vec![(page, layer)],
    };

    let totals = totals::compute(
        &invoices::stored_line_inputs(&invoice.items),
        line_items::discount_from(&invoice.discount_type, &invoice.discount_value),
    );

    let mut y = painter.letterhead(business, invoice);
    y = painter.client_block(client, y);
    y -= 4.0;
    y = painter.table_header(y);

    for (index, item) in invoice.items.iter().enumerate() {
        y = painter.table_row(y, index, item, totals.lines.get(index), &invoice.currency);
    }

    if invoice.items.is_empty() {
        painter.text(
            painter.labels.no_items,
            9.0,
            MARGIN,
            y,
            false,
            painter.theme.muted,
        );
        y -= 6.0;
    }

    y = painter.totals_block(&totals, &invoice.currency, y);
    painter.notes_block(invoice, y);
    painter.watermark(invoice);
    painter.footers(business);

    painter
        .doc
        .save_to_bytes()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn business() -> Business {
        Business {
            id: "default".to_string(),
            display_name: "Zuhdi Studio".to_string(),
            legal_name: Some("Zuhdi Studio Pte Ltd".to_string()),
            email: Some("hi@example.com".to_string()),
            phone: Some("+65 5555 0000".to_string()),
            website: None,
            address_line1: Some("1 Main Street".to_string()),
            address_line2: Some("#10-01".to_string()),
            city: Some("Singapore".to_string()),
            region: None,
            postal_code: Some("018989".to_string()),
            country: Some("SG".to_string()),
            tax_id: Some("GST-123456".to_string()),
            default_currency: "SGD".to_string(),
            default_terms: None,
            default_notes: None,
            default_payment_instructions: None,
            updated_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    fn item(description: &str, quantity: &str, unit_price_minor: i64) -> InvoiceItem {
        InvoiceItem {
            id: uuid::Uuid::now_v7().to_string(),
            sort_order: 0,
            description: description.to_string(),
            quantity: quantity.to_string(),
            unit: Some("hour".to_string()),
            unit_price_minor,
            discount_type: None,
            discount_value: None,
            tax_rate_id: None,
            tax_rate_name: None,
            tax_rate_bp: None,
            tax_inclusive: false,
        }
    }

    fn invoice(items: Vec<InvoiceItem>, status: &str) -> Invoice {
        let totals = totals::compute(&invoices::stored_line_inputs(&items), None);

        Invoice {
            id: "invoice".to_string(),
            number: "INV-2026-0001".to_string(),
            client_id: None,
            client_name: Some("Acme Pte Ltd".to_string()),
            status: status.to_string(),
            issue_date: "2026-03-15".to_string(),
            due_date: Some("2026-04-14".to_string()),
            currency: "SGD".to_string(),
            po_reference: Some("PO-77".to_string()),
            discount_type: None,
            discount_value: None,
            subtotal_minor: totals.subtotal_minor,
            discount_minor: totals.discount_minor,
            tax_minor: totals.tax_minor,
            total_minor: totals.total_minor,
            notes: Some("Thanks for your business.".to_string()),
            terms: Some("Net 30".to_string()),
            payment_instructions: Some(
                "Bank transfer to Zuhdi Studio, ref the invoice number.".to_string(),
            ),
            voided_at: None,
            sent_at: None,
            created_at: "2026-03-15 00:00:00".to_string(),
            updated_at: "2026-03-15 00:00:00".to_string(),
            paid_minor: 0,
            balance_minor: totals.total_minor,
            days_past_due: None,
            display_status: status.to_string(),
            items,
        }
    }

    fn options(template: Template) -> PdfOptions {
        PdfOptions {
            template,
            promo_footer: false,
            locale: Locale::En,
        }
    }

    fn page_count(bytes: &[u8]) -> usize {
        lopdf::Document::load_mem(bytes)
            .expect("rendered bytes should parse as a PDF")
            .get_pages()
            .len()
    }

    #[test]
    fn renders_a_single_page_pdf() {
        let bytes = render_invoice(
            &invoice(vec![item("Design work", "2", 10_000)], "draft"),
            &business(),
            None,
            options(Template::Classic),
        )
        .unwrap();

        assert!(bytes.starts_with(b"%PDF-"));
        assert!(String::from_utf8_lossy(&bytes).contains("%%EOF"));
        assert_eq!(page_count(&bytes), 1);
    }

    #[test]
    fn paginates_long_invoices() {
        let items: Vec<InvoiceItem> = (0..45)
            .map(|index| {
                item(
                    &format!("Line item {index} with a fairly long description that wraps"),
                    "1",
                    1_000,
                )
            })
            .collect();

        let bytes = render_invoice(
            &invoice(items, "draft"),
            &business(),
            None,
            options(Template::Classic),
        )
        .unwrap();

        assert!(page_count(&bytes) > 1);
    }

    #[test]
    fn every_template_and_a_void_invoice_render() {
        for template in [Template::Classic, Template::Modern, Template::Minimal] {
            let bytes = render_invoice(
                &invoice(vec![item("Consulting", "1.5", 12_500)], "void"),
                &business(),
                None,
                options(template),
            )
            .unwrap();

            assert!(bytes.starts_with(b"%PDF-"), "{template:?} did not render");
            assert_eq!(page_count(&bytes), 1);
        }
    }

    #[test]
    fn renders_an_invoice_straight_from_the_database() {
        use crate::db::{clients, tax_rates, tests_support::temp_db};

        let t = temp_db();

        let client = clients::create(
            &t.conn,
            &clients::ClientInput {
                display_name: "Acme Pte Ltd".to_string(),
                company: Some("Acme".to_string()),
                contact_person: None,
                email: Some("billing@acme.test".to_string()),
                phone: None,
                address_line1: Some("9 Client Road".to_string()),
                address_line2: None,
                city: Some("Singapore".to_string()),
                region: None,
                postal_code: Some("049513".to_string()),
                country: Some("SG".to_string()),
                tax_id: Some("GST-999".to_string()),
                currency: None,
                notes: None,
            },
        )
        .unwrap();

        let gst = tax_rates::create(
            &t.conn,
            &tax_rates::TaxRateInput {
                name: "GST".to_string(),
                rate_bp: 900,
            },
        )
        .unwrap();

        let item = |description: &str, quantity: &str, price: i64, tax_rate_id: Option<String>| {
            invoices::LineItemInput {
                description: description.to_string(),
                quantity: quantity.to_string(),
                unit: Some("hour".to_string()),
                unit_price_minor: price,
                discount_type: None,
                discount_value: None,
                tax_rate_id,
                tax_inclusive: false,
            }
        };

        let created = invoices::create(
            &t.conn,
            &invoices::InvoiceInput {
                client_id: Some(client.id.clone()),
                issue_date: "2026-03-15".to_string(),
                due_date: Some("2026-04-14".to_string()),
                currency: "SGD".to_string(),
                po_reference: None,
                discount_type: Some("percent".to_string()),
                discount_value: Some("5".to_string()),
                notes: None,
                terms: None,
                payment_instructions: None,
                estimate_id: None,
                items: vec![
                    item("Consulting", "3", 12_345, Some(gst.id)),
                    item("Hosting", "1", 4_000, None),
                ],
            },
        )
        .unwrap();

        let loaded = invoices::get(&t.conn, &created.id).unwrap().unwrap();
        let business = crate::db::business::get(&t.conn).unwrap();

        let bytes = render_invoice(
            &loaded,
            &business,
            Some(&client),
            options(Template::Classic),
        )
        .unwrap();
        assert_eq!(page_count(&bytes), 1);

        // The PDF must show the same figures the list view shows.
        let recomputed = totals::compute(
            &invoices::stored_line_inputs(&loaded.items),
            line_items::discount_from(&loaded.discount_type, &loaded.discount_value),
        );
        assert_eq!(recomputed.subtotal_minor, loaded.subtotal_minor);
        assert_eq!(recomputed.discount_minor, loaded.discount_minor);
        assert_eq!(recomputed.tax_minor, loaded.tax_minor);
        assert_eq!(recomputed.total_minor, loaded.total_minor);
    }

    #[test]
    fn survives_unicode_and_empty_content() {
        let mut invoice = invoice(
            vec![item("Développement — 設計 ₹1,000", "1", 100_000)],
            "draft",
        );
        invoice.notes = None;
        invoice.terms = None;
        invoice.payment_instructions = None;
        invoice.client_name = None;

        let bytes =
            render_invoice(&invoice, &business(), None, options(Template::Minimal)).unwrap();
        assert_eq!(page_count(&bytes), 1);
    }
}
