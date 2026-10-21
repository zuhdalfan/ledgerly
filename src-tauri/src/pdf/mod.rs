//! Invoice PDF rendering.
//!
//! Everything is drawn here in Rust with an embedded font, so the output is
//! identical on every machine and printer, and nothing needs the network.

mod format;
mod labels;
mod text;

use printpdf::image_crate::DynamicImage;
use printpdf::*;

use crate::db::business::Business;
use crate::db::clients::Client;
use crate::db::estimates::{Estimate, EstimateItem};
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
    /// Language of the labels on the document itself.
    pub locale: Locale,
}

/// The documents share one layout. Only the title, the date line and the final
/// total's wording differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentKind {
    Invoice,
    Estimate,
    Statement,
}

/// What the letterhead block prints, so a statement can use it without
/// pretending to be an invoice.
struct DocumentHead<'a> {
    number: &'a str,
    issued: &'a str,
    due: Option<&'a str>,
    reference: Option<&'a str>,
}

impl DocumentKind {
    fn title(self, labels: &Labels) -> &'static str {
        match self {
            DocumentKind::Invoice => labels.invoice,
            DocumentKind::Estimate => labels.estimate,
            DocumentKind::Statement => labels.statement,
        }
    }

    fn date_label(self, labels: &Labels) -> &'static str {
        match self {
            DocumentKind::Invoice => labels.due,
            DocumentKind::Estimate => labels.valid_until,
            DocumentKind::Statement => labels.due,
        }
    }

    fn total_label(self, labels: &Labels) -> &'static str {
        match self {
            DocumentKind::Invoice => labels.total_due,
            DocumentKind::Estimate => labels.total_plain,
            DocumentKind::Statement => labels.total_outstanding,
        }
    }

    fn page_name(self) -> &'static str {
        match self {
            DocumentKind::Invoice => "Invoice",
            DocumentKind::Estimate => "Estimate",
            DocumentKind::Statement => "Statement",
        }
    }
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

// The minimum clear space between two right-aligned columns.
const COL_GUTTER: f32 = 4.0;
/// Breathing room around the Modern template's totals panel.
const PANEL_PAD: f32 = 4.0;
/// A description column narrower than this is unreadable, so the money columns
/// give way rather than squeeze it further.
const DESC_MIN: f32 = 45.0;
/// Statement columns: documents rather than line items.
const COL_ISSUED: f32 = 78.0;
const COL_DUE: f32 = 118.0;

/// The letterhead's logo slot. A logo is scaled to fit inside it, and the
/// placeholder is drawn at its full size so the space is reserved either way.
const LOGO_MAX_W: f32 = 40.0;
const LOGO_MAX_H: f32 = 16.0;
/// Clear space between the logo and the business name beneath it. The gap is
/// measured to the name's baseline, so it has to clear the ascender too.
const LOGO_GAP: f32 = 6.0;

/// Right edges of the line-item columns.
///
/// These are derived from the widest figure the document actually contains
/// rather than fixed, because money strings grow as the amounts do: a column
/// wide enough for `$9.00` will overlap its neighbour once it has to hold
/// `S$2,795.85`.
#[derive(Clone, Copy)]
struct Columns {
    qty: f32,
    price: f32,
    disc: f32,
    tax: f32,
    total: f32,
    desc_width: f32,
}

impl Default for Columns {
    fn default() -> Self {
        // Only a fallback; a rendered invoice/estimate always lays columns out
        // from its own figures.
        Self {
            qty: 118.0,
            price: 142.0,
            disc: 162.0,
            tax: 178.0,
            total: CONTENT_RIGHT,
            desc_width: 76.0,
        }
    }
}

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
    labels: &'static Labels,
    /// Document language, which decides number and date formatting as well as
    /// the labels.
    locale: Locale,
    kind: DocumentKind,
    cols: Columns,
    /// The decoded logo, if one is set. `Image::add_to_layer` consumes its
    /// image, so the decoded form is kept and re-wrapped per document.
    logo: Option<DynamicImage>,
    /// Whether to stamp VOID across the page. The mark goes down before the
    /// content so the figures always read on top of it.
    watermark: bool,
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
        self.stamp_watermark();
    }

    /// Lay a VOID mark across the current page. Called the moment a page is
    /// created, before anything is drawn on it, so the watermark always sits
    /// behind the invoice and never blots out a figure.
    fn stamp_watermark(&self) {
        if !self.watermark {
            return;
        }

        let size = 64.0;
        let width = self.width_mm(self.labels.void, size, true);
        self.text(
            self.labels.void,
            size,
            (PAGE_W - width) / 2.0,
            PAGE_H / 2.0 - 10.0,
            true,
            WATERMARK,
        );
    }

    /// Draw the logo — or the placeholder that reserves its space — at the
    /// top-left, and report how much vertical room it took, gap included.
    fn logo_block(&self, top: f32) -> f32 {
        let theme = self.theme;
        let bottom = top - LOGO_MAX_H;

        let Some(logo) = self.logo.as_ref() else {
            self.rect(MARGIN, bottom, LOGO_MAX_W, LOGO_MAX_H, theme.shade);
            let label = self.labels.logo_placeholder;
            let width = self.width_mm(label, 8.0, true);
            self.text(
                label,
                8.0,
                MARGIN + (LOGO_MAX_W - width) / 2.0,
                bottom + LOGO_MAX_H / 2.0 - 1.2,
                true,
                theme.muted,
            );
            return LOGO_MAX_H + LOGO_GAP;
        };

        // Scale to fit the slot, never up: 300 dpi keeps a raster logo crisp on
        // paper, and a small file simply prints smaller rather than blurred.
        let dpi = 300.0;
        let natural_w = logo.width() as f32 * 25.4 / dpi;
        let natural_h = logo.height() as f32 * 25.4 / dpi;
        if natural_w <= 0.0 || natural_h <= 0.0 {
            return LOGO_MAX_H + LOGO_GAP;
        }

        let scale = (LOGO_MAX_W / natural_w)
            .min(LOGO_MAX_H / natural_h)
            .min(1.0);
        let height = natural_h * scale;

        Image::from_dynamic_image(logo).add_to_layer(
            self.layer(),
            ImageTransform {
                translate_x: Some(Mm(MARGIN)),
                translate_y: Some(Mm(top - height)),
                dpi: Some(dpi),
                scale_x: Some(scale),
                scale_y: Some(scale),
                ..Default::default()
            },
        );

        height + LOGO_GAP
    }

    fn letterhead(&self, business: &Business, head: &DocumentHead) -> f32 {
        let theme = self.theme;

        if let Some(band) = theme.band {
            self.rect(0.0, PAGE_H - 8.0, PAGE_W, 8.0, band);
        }

        let top = PAGE_H - MARGIN - 2.0;
        let mut y = top - self.logo_block(top);
        self.text(
            display_name_or(&business.display_name, "Your business"),
            15.0,
            MARGIN,
            y,
            true,
            theme.ink,
        );
        y -= 5.5;
        for line in business_lines(business, self.labels) {
            self.text(&line, 8.5, MARGIN, y, false, theme.muted);
            y -= 4.0;
        }

        let mut ry = PAGE_H - MARGIN - 2.0;
        self.text_right(
            self.kind.title(self.labels),
            theme.title_size,
            CONTENT_RIGHT,
            ry,
            true,
            theme.accent,
        );
        ry -= 7.5;
        self.text_right(head.number, 10.5, CONTENT_RIGHT, ry, true, theme.ink);
        ry -= 5.0;
        self.text_right(
            &format!(
                "{} {}",
                self.labels.issued,
                format_date(head.issued, self.locale)
            ),
            8.5,
            CONTENT_RIGHT,
            ry,
            false,
            theme.muted,
        );
        ry -= 4.2;

        if let Some(due) = head.due.filter(|value| !value.trim().is_empty()) {
            self.text_right(
                &format!(
                    "{} {}",
                    self.kind.date_label(self.labels),
                    format_date(due, self.locale)
                ),
                8.5,
                CONTENT_RIGHT,
                ry,
                false,
                theme.muted,
            );
            ry -= 4.2;
        }

        if let Some(reference) = head.reference.filter(|value| !value.trim().is_empty()) {
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
                for line in client_lines(client, self.labels) {
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
        let cols = self.cols;
        self.text_right(self.labels.quantity, size, cols.qty, y, true, theme.muted);
        self.text_right(self.labels.price, size, cols.price, y, true, theme.muted);
        self.text_right(self.labels.discount, size, cols.disc, y, true, theme.muted);
        self.text_right(self.labels.tax, size, cols.tax, y, true, theme.muted);
        self.text_right(self.labels.total, size, cols.total, y, true, theme.muted);

        y - 6.0
    }

    /// Column headings for a statement: documents, not line items.
    fn statement_header(&self, y: f32) -> f32 {
        let theme = self.theme;
        let size = 7.5;
        self.text(self.labels.invoice, size, MARGIN, y, true, theme.muted);
        self.text(self.labels.issued, size, COL_ISSUED, y, true, theme.muted);
        self.text(self.labels.due, size, COL_DUE, y, true, theme.muted);
        self.text_right(
            self.labels.balance,
            size,
            CONTENT_RIGHT,
            y,
            true,
            theme.muted,
        );
        y - 6.0
    }

    fn statement_row(&mut self, invoice: &Invoice, mut y: f32) -> f32 {
        let row_height = 6.0;
        if y - row_height < CONTENT_BOTTOM {
            self.new_page();
            y = PAGE_H - MARGIN - 6.0;
            y = self.statement_header(y);
        }

        let theme = self.theme;
        let size = 9.0;

        self.text(&invoice.number, size, MARGIN, y, false, theme.ink);
        self.text(
            &format_date(&invoice.issue_date, self.locale),
            size,
            COL_ISSUED,
            y,
            false,
            theme.ink,
        );
        self.text(
            &invoice
                .due_date
                .as_deref()
                .map(|date| format_date(date, self.locale))
                .unwrap_or_else(|| "—".to_string()),
            size,
            COL_DUE,
            y,
            false,
            theme.ink,
        );
        self.text_right(
            &format_money(invoice.balance_minor, &invoice.currency, self.locale),
            size,
            CONTENT_RIGHT,
            y,
            false,
            theme.ink,
        );

        let next = y - 5.0;
        self.rect(MARGIN, next + 2.0, CONTENT_RIGHT - MARGIN, 0.2, theme.rule);
        next
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
        let lines = self.measure.wrap(&description, 9.0, self.cols.desc_width);
        let row_height = lines.len() as f32 * 4.4 + 2.6;

        if y - row_height < CONTENT_BOTTOM {
            self.new_page();
            y = PAGE_H - MARGIN - 6.0;
            y = self.table_header(y);
        }

        let theme = self.theme;

        if theme.shade_rows && index % 2 == 1 {
            // Exactly the table's own width, so the band lines up with the
            // header rule instead of overhanging the description and the totals.
            self.rect(
                MARGIN,
                y - row_height + 3.4,
                CONTENT_RIGHT - MARGIN,
                row_height - 1.0,
                theme.shade,
            );
        }

        let mut dy = y;
        for (line_index, text) in lines.iter().enumerate() {
            self.text(text, 9.0, MARGIN, dy, false, theme.ink);

            if line_index == 0 {
                let cols = self.cols;
                self.text_right(&item.quantity, 9.0, cols.qty, dy, false, theme.ink);
                self.text_right(
                    &format_money(item.unit_price_minor, currency, self.locale),
                    9.0,
                    cols.price,
                    dy,
                    false,
                    theme.ink,
                );
                self.text_right(
                    &discount_label(item, currency, self.locale),
                    9.0,
                    cols.disc,
                    dy,
                    false,
                    theme.muted,
                );
                self.text_right(
                    &tax_label(item, self.locale),
                    9.0,
                    cols.tax,
                    dy,
                    false,
                    theme.muted,
                );
                self.text_right(
                    &format_money(
                        line.map(|l| l.total_minor).unwrap_or(0),
                        currency,
                        self.locale,
                    ),
                    9.0,
                    cols.total,
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

        let cols = self.cols;

        // Fix the rows up front so the panel can be measured against the ones
        // that will actually be drawn: there is no discount row when there is
        // no discount, which is what used to leave shade trailing under the
        // total.
        let has_discount = totals.discount_minor != 0;
        let subtotal_y = y;
        let discount_y = subtotal_y - 5.0;
        let tax_y = if has_discount {
            discount_y - 5.0
        } else {
            subtotal_y - 5.0
        };
        let rule_y = tax_y - 2.6;
        let total_y = tax_y - 8.4;

        let subtotal_value = format_money(totals.subtotal_minor, currency, self.locale);
        let discount_value = format!(
            "-{}",
            format_money(totals.discount_minor, currency, self.locale)
        );
        let tax_value = format_money(totals.tax_minor, currency, self.locale);
        let total_value = format_money(totals.total_minor, currency, self.locale);

        // Both the label column and the panel are sized to the widest string on
        // each side, so a long figure — a large amount, or a currency written
        // like `Rp` — pushes the labels aside instead of running into them.
        let mut widest_value = self
            .width_mm(&subtotal_value, 9.0, false)
            .max(self.width_mm(&tax_value, 9.0, false))
            .max(self.width_mm(&total_value, 11.0, true));
        if has_discount {
            widest_value = widest_value.max(self.width_mm(&discount_value, 9.0, false));
        }

        let widest_label = [
            (self.labels.subtotal, 9.0, false),
            (self.labels.discount, 9.0, false),
            (self.labels.tax, 9.0, false),
            (self.kind.total_label(self.labels), 11.0, true),
        ]
        .iter()
        .map(|(label, size, bold)| self.width_mm(label, *size, *bold))
        .fold(0.0, f32::max);

        let label_x = (cols.total - 34.0).min(cols.total - widest_value - COL_GUTTER);

        if theme.panel {
            let left = label_x - widest_label - PANEL_PAD;
            let top = subtotal_y + PANEL_PAD;
            let bottom = total_y - PANEL_PAD;
            self.rect(
                left,
                bottom,
                CONTENT_RIGHT + PANEL_PAD - left,
                top - bottom,
                theme.shade,
            );
        }

        self.text_right(
            self.labels.subtotal,
            9.0,
            label_x,
            subtotal_y,
            false,
            theme.muted,
        );
        self.text_right(
            &subtotal_value,
            9.0,
            cols.total,
            subtotal_y,
            false,
            theme.ink,
        );

        if has_discount {
            self.text_right(
                self.labels.discount,
                9.0,
                label_x,
                discount_y,
                false,
                theme.muted,
            );
            self.text_right(
                &discount_value,
                9.0,
                cols.total,
                discount_y,
                false,
                theme.ink,
            );
        }

        self.text_right(self.labels.tax, 9.0, label_x, tax_y, false, theme.muted);
        self.text_right(&tax_value, 9.0, cols.total, tax_y, false, theme.ink);

        self.rect(label_x, rule_y, cols.total - label_x, 0.4, theme.rule);
        self.text_right(
            self.kind.total_label(self.labels),
            11.0,
            label_x,
            total_y,
            true,
            theme.ink,
        );
        self.text_right(&total_value, 11.0, cols.total, total_y, true, theme.accent);

        tax_y - 20.0
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

fn business_lines(business: &Business, labels: &Labels) -> Vec<String> {
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
        lines.push(format!("{}: {tax_id}", labels.tax_id));
    }
    lines
}

fn client_lines(client: &Client, labels: &Labels) -> Vec<String> {
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
        lines.push(format!("{}: {tax_id}", labels.tax_id));
    }
    lines
}

fn discount_label(item: &InvoiceItem, currency: &str, locale: Locale) -> String {
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
            Ok(minor) if minor != 0 => format_money(minor, currency, locale),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

fn tax_label(item: &InvoiceItem, locale: Locale) -> String {
    match item.tax_rate_bp {
        Some(bp) if bp != 0 => format_percent_bp(bp, locale),
        _ => String::new(),
    }
}

/// Render an invoice to PDF bytes.
pub fn render_invoice(
    invoice: &Invoice,
    business: &Business,
    client: Option<&Client>,
    options: PdfOptions,
    logo: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    render_document(
        invoice,
        business,
        client,
        options,
        DocumentKind::Invoice,
        logo,
    )
}

/// Render an estimate to PDF bytes. An estimate is a quote: same layout, but it
/// says ESTIMATE, shows a validity date instead of a due date, and totals to
/// "Total" rather than "Total due".
pub fn render_estimate(
    estimate: &Estimate,
    business: &Business,
    client: Option<&Client>,
    options: PdfOptions,
    logo: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    render_document(
        &estimate_as_invoice(estimate),
        business,
        client,
        options,
        DocumentKind::Estimate,
        logo,
    )
}

/// An estimate carries the fields an invoice does, so it is viewed as one
/// rather than duplicating the renderer. Payments are zero and the validity
/// date takes the due-date slot; neither is printed for an estimate.
fn estimate_as_invoice(estimate: &Estimate) -> Invoice {
    Invoice {
        id: estimate.id.clone(),
        number: estimate.number.clone(),
        client_id: estimate.client_id.clone(),
        client_name: estimate.client_name.clone(),
        status: estimate.status.clone(),
        issue_date: estimate.issue_date.clone(),
        due_date: estimate.valid_until.clone(),
        currency: estimate.currency.clone(),
        po_reference: estimate.po_reference.clone(),
        discount_type: estimate.discount_type.clone(),
        discount_value: estimate.discount_value.clone(),
        subtotal_minor: estimate.subtotal_minor,
        discount_minor: estimate.discount_minor,
        tax_minor: estimate.tax_minor,
        total_minor: estimate.total_minor,
        notes: estimate.notes.clone(),
        terms: estimate.terms.clone(),
        payment_instructions: estimate.payment_instructions.clone(),
        voided_at: None,
        sent_at: None,
        created_at: estimate.created_at.clone(),
        updated_at: estimate.updated_at.clone(),
        paid_minor: 0,
        balance_minor: 0,
        days_past_due: None,
        display_status: estimate.display_status.clone(),
        // Chasing is an invoice concern; an estimate has no such record.
        last_chased_at: None,
        chase_count: 0,
        // The renderer reads the locale from the options, but the converted
        // document keeps the estimate's own values rather than inventing them.
        locale: estimate.locale.clone(),
        client_locale: estimate.client_locale.clone(),
        items: estimate
            .items
            .iter()
            .map(estimate_item_as_invoice_item)
            .collect(),
    }
}

fn estimate_item_as_invoice_item(item: &EstimateItem) -> InvoiceItem {
    InvoiceItem {
        id: item.id.clone(),
        sort_order: item.sort_order,
        description: item.description.clone(),
        quantity: item.quantity.clone(),
        unit: item.unit.clone(),
        unit_price_minor: item.unit_price_minor,
        discount_type: item.discount_type.clone(),
        discount_value: item.discount_value.clone(),
        tax_rate_id: item.tax_rate_id.clone(),
        tax_rate_name: item.tax_rate_name.clone(),
        tax_rate_bp: item.tax_rate_bp,
        tax_inclusive: item.tax_inclusive,
    }
}

/// Work out where each column ends from the widest value it actually has to
/// hold, and place the money columns from the right edge leftward. A fixed
/// layout cannot survive a figure that grows, which is exactly what happens as
/// amounts gain digits.
fn layout_columns(painter: &Painter, invoice: &Invoice, totals: &InvoiceTotals) -> Columns {
    let header = |label: &str| painter.width_mm(label, 7.5, true);
    let figure = |value: &str| painter.width_mm(value, 9.0, false);

    let mut qty = header(painter.labels.quantity);
    let mut price = header(painter.labels.price);
    let mut disc = header(painter.labels.discount);
    let mut tax = header(painter.labels.tax);
    let mut total = header(painter.labels.total);

    for (index, item) in invoice.items.iter().enumerate() {
        qty = qty.max(figure(&item.quantity));
        price = price.max(figure(&format_money(
            item.unit_price_minor,
            &invoice.currency,
            painter.locale,
        )));
        disc = disc.max(figure(&discount_label(
            item,
            &invoice.currency,
            painter.locale,
        )));
        tax = tax.max(figure(&tax_label(item, painter.locale)));
        let line_total = totals.lines.get(index).map_or(0, |line| line.total_minor);
        total = total.max(figure(&format_money(
            line_total,
            &invoice.currency,
            painter.locale,
        )));
    }

    let total_right = CONTENT_RIGHT;
    let tax_right = total_right - total - COL_GUTTER;
    let disc_right = tax_right - tax - COL_GUTTER;
    let price_right = disc_right - disc - COL_GUTTER;
    let qty_right = price_right - price - COL_GUTTER;
    let desc_width = (qty_right - qty - COL_GUTTER - MARGIN).max(DESC_MIN);

    Columns {
        qty: qty_right,
        price: price_right,
        disc: disc_right,
        tax: tax_right,
        total: total_right,
        desc_width,
    }
}

/// Decode a stored logo. Anything unreadable yields `None` so an export falls
/// back to the placeholder instead of failing outright.
fn decode_logo(bytes: &[u8]) -> Option<DynamicImage> {
    printpdf::image_crate::load_from_memory(bytes).ok()
}

/// A logo the renderer can embed: its MIME type and pixel size.
pub struct LogoInfo {
    pub mime: &'static str,
    pub width: u32,
    pub height: u32,
}

/// Check an uploaded logo. Only PNG and JPEG are accepted, which is what the
/// renderer can draw — refusing anything else at upload keeps a stored logo
/// always renderable.
pub fn inspect_logo(bytes: &[u8]) -> Option<LogoInfo> {
    use printpdf::image_crate::io::Reader;
    use printpdf::image_crate::{guess_format, ImageFormat};

    let mime = match guess_format(bytes).ok()? {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        _ => return None,
    };

    let (width, height) = Reader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;

    Some(LogoInfo {
        mime,
        width,
        height,
    })
}

fn build_painter(
    options: PdfOptions,
    kind: DocumentKind,
    watermark: bool,
    logo: Option<DynamicImage>,
) -> Result<Painter, String> {
    let measure = TextMeasure::new(FONT_REGULAR)
        .ok_or_else(|| "could not read the regular font".to_string())?;
    let bold_measure =
        TextMeasure::new(FONT_BOLD).ok_or_else(|| "could not read the bold font".to_string())?;

    let (doc, page, layer) = PdfDocument::new(kind.page_name(), Mm(PAGE_W), Mm(PAGE_H), "content");

    let mut regular_reader = std::io::Cursor::new(FONT_REGULAR);
    let font_regular = doc
        .add_external_font(&mut regular_reader)
        .map_err(|error| error.to_string())?;
    let mut bold_reader = std::io::Cursor::new(FONT_BOLD);
    let font_bold = doc
        .add_external_font(&mut bold_reader)
        .map_err(|error| error.to_string())?;

    let painter = Painter {
        doc,
        font_regular,
        font_bold,
        measure,
        bold_measure,
        theme: Theme::for_template(options.template),
        labels: options.locale.labels(),
        locale: options.locale,
        kind,
        cols: Columns::default(),
        logo,
        watermark,
        page,
        layer,
        pages: vec![(page, layer)],
    };
    // The first page is created here, not through `new_page`, so stamp it too.
    painter.stamp_watermark();
    Ok(painter)
}

fn render_document(
    invoice: &Invoice,
    business: &Business,
    client: Option<&Client>,
    options: PdfOptions,
    kind: DocumentKind,
    logo: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let mut painter = build_painter(
        options,
        kind,
        invoice.status == "void",
        logo.and_then(decode_logo),
    )?;

    let totals = totals::compute(
        &invoices::stored_line_inputs(&invoice.items),
        line_items::discount_from(&invoice.discount_type, &invoice.discount_value),
    );
    painter.cols = layout_columns(&painter, invoice, &totals);

    let mut y = painter.letterhead(
        business,
        &DocumentHead {
            number: &invoice.number,
            issued: &invoice.issue_date,
            due: invoice.due_date.as_deref(),
            reference: invoice.po_reference.as_deref(),
        },
    );
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
    painter.footers(business);

    painter
        .doc
        .save_to_bytes()
        .map_err(|error| error.to_string())
}

/// Render a statement: one client's open invoices and what they add up to.
///
/// A statement is addressed to the client and lists documents rather than
/// lines, so it shares the letterhead, themes and footer but not the invoice
/// table.
pub fn render_statement(
    client: &Client,
    open: &[Invoice],
    business: &Business,
    options: PdfOptions,
    issued: &str,
    logo: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let mut painter = build_painter(
        options,
        DocumentKind::Statement,
        false,
        logo.and_then(decode_logo),
    )?;

    // The client's name takes the slot an invoice uses for its number.
    let mut y = painter.letterhead(
        business,
        &DocumentHead {
            number: &client.display_name,
            issued,
            due: None,
            reference: None,
        },
    );

    for line in client_lines(client, painter.labels) {
        painter.text(&line, 8.5, MARGIN, y, false, painter.theme.muted);
        y -= 4.0;
    }

    y -= 2.0;
    y = painter.statement_header(y);

    for invoice in open {
        y = painter.statement_row(invoice, y);
    }

    let total: i64 = open.iter().map(|invoice| invoice.balance_minor).sum();
    let currency = open
        .first()
        .map(|invoice| invoice.currency.as_str())
        .unwrap_or(business.default_currency.as_str());

    let label_y = y - 5.0;
    painter.rect(
        MARGIN,
        label_y + 4.5,
        CONTENT_RIGHT - MARGIN,
        0.3,
        painter.theme.rule,
    );
    painter.text(
        painter.kind.total_label(painter.labels),
        11.0,
        MARGIN,
        label_y,
        true,
        painter.theme.ink,
    );
    painter.text_right(
        &format_money(total, currency, painter.locale),
        11.0,
        CONTENT_RIGHT,
        label_y,
        true,
        painter.theme.ink,
    );

    painter.footers(business);

    painter
        .doc
        .save_to_bytes()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The layout tests are not about the logo, so they pin it to none and stay
    // focused. The logo path has its own tests below.
    fn render_invoice(
        invoice: &Invoice,
        business: &Business,
        client: Option<&Client>,
        options: PdfOptions,
    ) -> Result<Vec<u8>, String> {
        super::render_invoice(invoice, business, client, options, None)
    }

    fn render_estimate(
        estimate: &Estimate,
        business: &Business,
        client: Option<&Client>,
        options: PdfOptions,
    ) -> Result<Vec<u8>, String> {
        super::render_estimate(estimate, business, client, options, None)
    }

    fn render_statement(
        client: &Client,
        open: &[Invoice],
        business: &Business,
        options: PdfOptions,
        issued: &str,
    ) -> Result<Vec<u8>, String> {
        super::render_statement(client, open, business, options, issued, None)
    }

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
            last_chased_at: None,
            chase_count: 0,
            locale: None,
            client_locale: None,
            items,
        }
    }

    fn client() -> Client {
        Client {
            id: "client-1".to_string(),
            display_name: "Acme Pte Ltd".to_string(),
            company: None,
            contact_person: None,
            email: Some("ap@acme.example".to_string()),
            phone: None,
            address_line1: Some("1 Example Street".to_string()),
            address_line2: None,
            city: Some("Singapore".to_string()),
            region: None,
            postal_code: Some("123456".to_string()),
            country: Some("Singapore".to_string()),
            tax_id: None,
            currency: None,
            notes: None,
            locale: None,
            archived_at: None,
            created_at: "2026-01-01 00:00:00".to_string(),
            updated_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    fn statement_invoices(count: usize) -> Vec<Invoice> {
        (0..count)
            .map(|index| {
                let mut open = invoice(vec![item("Work", "1", 10_000)], "sent");
                open.number = format!("INV-{index:04}");
                open
            })
            .collect()
    }

    #[test]
    fn renders_a_statement() {
        let bytes = render_statement(
            &client(),
            &statement_invoices(3),
            &business(),
            options(Template::Classic),
            "2026-10-10",
        )
        .expect("a statement should render");

        assert!(bytes.starts_with(b"%PDF-"));
        assert_eq!(page_count(&bytes), 1);
    }

    #[test]
    fn an_empty_statement_still_renders() {
        let bytes = render_statement(
            &client(),
            &[],
            &business(),
            options(Template::Classic),
            "2026-10-10",
        )
        .expect("a client who owes nothing should still produce a document");

        assert_eq!(page_count(&bytes), 1);
    }

    #[test]
    fn a_long_statement_paginates() {
        let bytes = render_statement(
            &client(),
            &statement_invoices(80),
            &business(),
            options(Template::Classic),
            "2026-10-10",
        )
        .expect("a long statement should render");

        assert!(
            page_count(&bytes) > 1,
            "the rows should continue onto another page"
        );
    }

    fn options(template: Template) -> PdfOptions {
        PdfOptions {
            template,
            locale: Locale::En,
        }
    }

    fn page_count(bytes: &[u8]) -> usize {
        lopdf::Document::load_mem(bytes)
            .expect("rendered bytes should parse as a PDF")
            .get_pages()
            .len()
    }

    fn estimate_item(description: &str, quantity: &str, unit_price_minor: i64) -> EstimateItem {
        EstimateItem {
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

    fn estimate(items: Vec<EstimateItem>, valid_until: Option<&str>) -> Estimate {
        Estimate {
            id: uuid::Uuid::now_v7().to_string(),
            number: "EST-0001".to_string(),
            client_id: None,
            client_name: Some("Acme".to_string()),
            status: "draft".to_string(),
            issue_date: "2026-01-15".to_string(),
            valid_until: valid_until.map(str::to_string),
            currency: "USD".to_string(),
            po_reference: None,
            discount_type: None,
            discount_value: None,
            subtotal_minor: 20_000,
            discount_minor: 0,
            tax_minor: 0,
            total_minor: 20_000,
            notes: None,
            terms: None,
            payment_instructions: None,
            converted_invoice_id: None,
            converted_at: None,
            created_at: "2026-01-15".to_string(),
            updated_at: "2026-01-15".to_string(),
            days_until_expiry: Some(31),
            display_status: "draft".to_string(),
            locale: None,
            client_locale: None,
            items,
        }
    }

    #[test]
    fn document_kinds_use_their_own_labels() {
        let labels = Locale::En.labels();

        assert_eq!(DocumentKind::Invoice.title(labels), "INVOICE");
        assert_eq!(DocumentKind::Estimate.title(labels), "ESTIMATE");

        assert_eq!(DocumentKind::Invoice.date_label(labels), "Due");
        assert_eq!(DocumentKind::Estimate.date_label(labels), "Valid until");

        // An estimate is a quote, so nothing is "due" yet.
        assert_eq!(DocumentKind::Invoice.total_label(labels), "Total due");
        assert_eq!(DocumentKind::Estimate.total_label(labels), "Total");
    }

    #[test]
    fn renders_an_estimate() {
        let bytes = render_estimate(
            &estimate(
                vec![estimate_item("Consulting", "2", 10_000)],
                Some("2026-02-15"),
            ),
            &business(),
            None,
            options(Template::Classic),
        )
        .expect("an estimate should render");

        assert!(bytes.starts_with(b"%PDF-"));
        assert_eq!(page_count(&bytes), 1);
    }

    #[test]
    fn an_estimate_without_a_validity_date_still_renders() {
        let bytes = render_estimate(
            &estimate(vec![estimate_item("Consulting", "2", 10_000)], None),
            &business(),
            None,
            options(Template::Classic),
        )
        .expect("a missing validity date must not break the document");

        assert_eq!(page_count(&bytes), 1);
    }

    #[test]
    fn an_estimate_paginates_like_an_invoice() {
        let items: Vec<EstimateItem> = (0..70)
            .map(|index| estimate_item(&format!("Line item {index}"), "1", 12_345))
            .collect();

        let bytes = render_estimate(
            &estimate(items, Some("2026-02-15")),
            &business(),
            None,
            options(Template::Classic),
        )
        .expect("a long estimate should render");

        assert!(
            page_count(&bytes) > 1,
            "the table should continue onto a second page"
        );
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
                locale: None,
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
                locale: None,
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

    /// A solid-colour PNG, encoded in memory.
    fn png(width: u32, height: u32) -> Vec<u8> {
        use printpdf::image_crate::{ImageBuffer, ImageFormat, Rgb};

        let buffer = ImageBuffer::from_pixel(width, height, Rgb([200u8, 40, 40]));
        let mut out = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(buffer)
            .write_to(&mut out, ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }

    #[test]
    fn a_logo_renders_in_every_template() {
        let invoice = invoice(vec![item("Design work", "2", 10_000)], "draft");
        let logo = png(320, 120);

        for template in [Template::Classic, Template::Modern, Template::Minimal] {
            let bytes =
                super::render_invoice(&invoice, &business(), None, options(template), Some(&logo))
                    .unwrap();

            assert!(bytes.starts_with(b"%PDF-"), "{template:?} did not render");
            assert_eq!(page_count(&bytes), 1, "{template:?} should stay one page");
        }
    }

    #[test]
    fn a_logo_that_cannot_be_decoded_falls_back_to_the_placeholder() {
        let invoice = invoice(vec![item("Design work", "2", 10_000)], "draft");

        let bytes = super::render_invoice(
            &invoice,
            &business(),
            None,
            options(Template::Classic),
            Some(b"this is not an image"),
        )
        .expect("a bad logo must not fail the export");

        assert_eq!(page_count(&bytes), 1);
    }

    #[test]
    fn an_embedded_logo_weighs_more_than_the_plain_placeholder() {
        let invoice = invoice(vec![item("Design work", "2", 10_000)], "draft");
        let plain = super::render_invoice(
            &invoice,
            &business(),
            None,
            options(Template::Classic),
            None,
        )
        .unwrap();
        let with_logo = super::render_invoice(
            &invoice,
            &business(),
            None,
            options(Template::Classic),
            Some(&png(400, 160)),
        )
        .unwrap();

        assert!(
            with_logo.len() > plain.len(),
            "the image should add bytes to the file ({} vs {})",
            with_logo.len(),
            plain.len()
        );
    }

    #[test]
    fn inspect_logo_accepts_png_and_refuses_anything_else() {
        let info = inspect_logo(&png(64, 32)).expect("a PNG should be accepted");
        assert_eq!(info.mime, "image/png");
        assert_eq!((info.width, info.height), (64, 32));

        assert!(inspect_logo(b"this is not an image").is_none());
        assert!(inspect_logo(b"").is_none());
    }
}
