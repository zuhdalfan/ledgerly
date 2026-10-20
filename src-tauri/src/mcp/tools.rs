//! The tools the MCP server exposes, and their JSON schemas.
//!
//! Handlers are plain functions over a `&Connection` so the whole surface can
//! be unit-tested without touching stdio. Every write tool is flagged so the
//! consent gate can refuse it while "Allow changes" is off.

use std::path::PathBuf;

use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

use crate::backup;
use crate::db::{
    business, catalog, clients, estimates, invoices, numbering, payments, reports, settings,
    tax_rates,
};
use crate::error::{AppError, AppResult};

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub write: bool,
    pub schema: Value,
    pub handler: fn(&Connection, &Value) -> AppResult<Value>,
}

pub fn find(name: &str) -> Option<Tool> {
    all().into_iter().find(|tool| tool.name == name)
}

fn list_schemas() -> Vec<Value> {
    all()
        .into_iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.schema,
            })
        })
        .collect()
}

/// The `tools/list` payload.
pub fn list() -> Vec<Value> {
    list_schemas()
}

// --- schema helpers ---------------------------------------------------------

fn string(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn integer(description: &str) -> Value {
    json!({ "type": "integer", "description": description })
}

fn boolean(description: &str) -> Value {
    json!({ "type": "boolean", "description": description })
}

fn object(properties: Value) -> Value {
    json!({ "type": "object", "properties": properties })
}

/// A tool whose only argument is an `input` object.
fn input_object(schema: Value) -> Value {
    json!({ "type": "object", "properties": { "input": schema }, "required": ["input"] })
}

fn id_object() -> Value {
    json!({ "type": "object", "properties": { "id": string("The record id.") }, "required": ["id"] })
}

fn line_item_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "description": string("Line text."),
            "quantity": string("Decimal quantity as text, e.g. \"2\" or \"1.5\"."),
            "unit": string("Unit label, e.g. \"hour\"."),
            "unitPriceMinor": integer("Unit price in minor units (cents)."),
            "discountType": string("\"percent\" or \"fixed\"."),
            "discountValue": string(
                "Decimal discount as text. A percentage (e.g. \"10\" is 10%) when the type is \
                 \"percent\", or an amount in minor units when the type is \"fixed\"."
            ),
            "taxRateId": string("Tax rate id from ledgerly_tax_rates_list, or omit."),
            "taxInclusive": boolean("Whether unitPriceMinor already includes tax.")
        },
        "required": ["description", "quantity", "unitPriceMinor"]
    })
}

fn document_input_schema(date_field: &str) -> Value {
    json!({
        "type": "object",
        "properties": {
            "clientId": string("Client id from ledgerly_clients_list."),
            "issueDate": string("Issue date, YYYY-MM-DD."),
            date_field: string("Date, YYYY-MM-DD."),
            "currency": string("ISO currency code, e.g. USD. Defaults to the business currency."),
            "poReference": string("Optional reference."),
            "discountType": string("\"percent\" or \"fixed\"."),
            "discountValue": string(
                "Decimal discount as text. A percentage (e.g. \"10\" is 10%) when the type is \
                 \"percent\", or an amount in minor units when the type is \"fixed\"."
            ),
            "notes": string("Notes shown on the document."),
            "terms": string("Terms shown on the document."),
            "paymentInstructions": string("Payment instructions shown on the document."),
            "items": { "type": "array", "items": line_item_schema() }
        },
        "required": ["issueDate", "items"]
    })
}

// --- argument helpers -------------------------------------------------------

fn to_value<T: Serialize>(value: T) -> AppResult<Value> {
    serde_json::to_value(value).map_err(|error| AppError::Validation(error.to_string()))
}

fn arg_input<T: DeserializeOwned>(args: &Value) -> AppResult<T> {
    let value = args.get("input").cloned().unwrap_or(Value::Null);
    serde_json::from_value(value)
        .map_err(|error| AppError::Validation(format!("invalid 'input': {error}")))
}

fn required_str(args: &Value, key: &str) -> AppResult<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::Validation(format!("missing text argument '{key}'")))
}

fn optional_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

fn optional_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
}

// --- the registry -----------------------------------------------------------

pub fn all() -> Vec<Tool> {
    vec![
        // --- read ---------------------------------------------------------
        Tool {
            name: "ledgerly_app_info",
            description: "Ledgerly's version and where its data lives. Read-only.",
            write: false,
            schema: object(json!({})),
            handler: app_info,
        },
        Tool {
            name: "ledgerly_business_get",
            description: "The business profile: name, address, tax id, default currency and invoice wording. Read-only.",
            write: false,
            schema: object(json!({})),
            handler: business_get,
        },
        Tool {
            name: "ledgerly_settings_all",
            description: "Every stored key/value setting. Read-only.",
            write: false,
            schema: object(json!({})),
            handler: settings_all,
        },
        Tool {
            name: "ledgerly_settings_get",
            description: "One setting by key. Read-only.",
            write: false,
            schema: json!({ "type": "object", "properties": { "key": string("Setting key.") }, "required": ["key"] }),
            handler: settings_get,
        },
        Tool {
            name: "ledgerly_clients_list",
            description: "List or search clients. Read-only.",
            write: false,
            schema: object(json!({
                "query": string("Optional search over name, company and email."),
                "includeArchived": boolean("Include archived clients. Default false.")
            })),
            handler: clients_list,
        },
        Tool {
            name: "ledgerly_clients_get",
            description: "One client by id. Read-only.",
            write: false,
            schema: id_object(),
            handler: clients_get,
        },
        Tool {
            name: "ledgerly_catalog_list",
            description: "List or search catalog items (products and services). Read-only.",
            write: false,
            schema: object(json!({
                "query": string("Optional search over name and SKU."),
                "includeArchived": boolean("Include archived items. Default false.")
            })),
            handler: catalog_list,
        },
        Tool {
            name: "ledgerly_tax_rates_list",
            description: "List tax rates. Read-only.",
            write: false,
            schema: object(json!({
                "includeArchived": boolean("Include archived rates. Default false.")
            })),
            handler: tax_rates_list,
        },
        Tool {
            name: "ledgerly_numbering_get",
            description: "The document-numbering scheme for a kind (\"invoice\" or \"estimate\"). Read-only.",
            write: false,
            schema: json!({ "type": "object", "properties": { "kind": string("\"invoice\" or \"estimate\".") }, "required": ["kind"] }),
            handler: numbering_get,
        },
        Tool {
            name: "ledgerly_invoices_list",
            description: "List or filter invoices. Read-only.",
            write: false,
            schema: object(json!({
                "query": string("Optional search over number and client."),
                "status": string("draft, sent, unpaid, overdue, paid or void."),
                "clientId": string("Restrict to one client."),
                "age": string("Aging bucket: not_due, 1-30, 31-60, 61-90 or 90plus."),
                "issuedMonth": string("Issue month, YYYY-MM.")
            })),
            handler: invoices_list,
        },
        Tool {
            name: "ledgerly_invoices_get",
            description: "One invoice with its line items, totals, payments and balance. Read-only.",
            write: false,
            schema: id_object(),
            handler: invoices_get,
        },
        Tool {
            name: "ledgerly_invoices_totals",
            description: "Compute totals for a proposed invoice without saving it. Read-only.",
            write: false,
            schema: input_object(document_input_schema("dueDate")),
            handler: invoices_totals,
        },
        Tool {
            name: "ledgerly_estimates_list",
            description: "List or filter estimates. Read-only.",
            write: false,
            schema: object(json!({
                "query": string("Optional search over number and client."),
                "status": string("draft, sent, accepted, declined, expired or converted."),
                "clientId": string("Restrict to one client.")
            })),
            handler: estimates_list,
        },
        Tool {
            name: "ledgerly_estimates_get",
            description: "One estimate with its line items and totals. Read-only.",
            write: false,
            schema: id_object(),
            handler: estimates_get,
        },
        Tool {
            name: "ledgerly_payments_list",
            description: "Payments recorded against an invoice. Read-only.",
            write: false,
            schema: json!({ "type": "object", "properties": { "invoiceId": string("Invoice id.") }, "required": ["invoiceId"] }),
            handler: payments_list,
        },
        Tool {
            name: "ledgerly_reports_revenue",
            description: "Revenue by period between two dates (inclusive). Read-only.",
            write: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "from": string("Start date, YYYY-MM-DD."),
                    "to": string("End date, YYYY-MM-DD."),
                    "granularity": string("\"day\", \"month\" or \"year\". Default month.")
                },
                "required": ["from", "to"]
            }),
            handler: reports_revenue,
        },
        Tool {
            name: "ledgerly_reports_revenue_by_client",
            description: "Revenue per client between two dates (inclusive). Read-only.",
            write: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "from": string("Start date, YYYY-MM-DD."),
                    "to": string("End date, YYYY-MM-DD.")
                },
                "required": ["from", "to"]
            }),
            handler: reports_revenue_by_client,
        },
        Tool {
            name: "ledgerly_reports_aged_receivables",
            description: "Outstanding balances grouped by how long they have been due. Read-only.",
            write: false,
            schema: object(json!({})),
            handler: reports_aged_receivables,
        },
        Tool {
            name: "ledgerly_reports_tax",
            description: "Tax collected per rate between two dates (inclusive). Read-only.",
            write: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "from": string("Start date, YYYY-MM-DD."),
                    "to": string("End date, YYYY-MM-DD.")
                },
                "required": ["from", "to"]
            }),
            handler: reports_tax,
        },
        Tool {
            name: "ledgerly_reports_payments_by_method",
            description: "Payments grouped by method between two dates (inclusive). Read-only.",
            write: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "from": string("Start date, YYYY-MM-DD."),
                    "to": string("End date, YYYY-MM-DD.")
                },
                "required": ["from", "to"]
            }),
            handler: reports_payments_by_method,
        },
        // --- write --------------------------------------------------------
        Tool {
            name: "ledgerly_clients_create",
            description: "Create a client. Changes data.",
            write: true,
            schema: input_object(json!({
                "type": "object",
                "properties": {
                    "displayName": string("Client name. Required."),
                    "company": string("Company, if different from the name."),
                    "contactPerson": string("Contact person."),
                    "email": string("Email."),
                    "phone": string("Phone."),
                    "addressLine1": string("Address line 1."),
                    "addressLine2": string("Address line 2."),
                    "city": string("City."),
                    "region": string("Region / state."),
                    "postalCode": string("Postal code."),
                    "country": string("Country."),
                    "taxId": string("Tax / VAT id."),
                    "currency": string("Preferred currency, e.g. USD."),
                    "notes": string("Notes.")
                },
                "required": ["displayName"]
            })),
            handler: clients_create,
        },
        Tool {
            name: "ledgerly_clients_update",
            description: "Replace a client's details. Provide the full record. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Client id."),
                    "input": { "type": "object", "description": "Same fields as ledgerly_clients_create." }
                },
                "required": ["id", "input"]
            }),
            handler: clients_update,
        },
        Tool {
            name: "ledgerly_clients_set_archived",
            description: "Archive or unarchive a client. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Client id."),
                    "archived": boolean("True to archive, false to restore. Default true.")
                },
                "required": ["id"]
            }),
            handler: clients_set_archived,
        },
        Tool {
            name: "ledgerly_catalog_create",
            description: "Create a catalog item. Changes data.",
            write: true,
            schema: input_object(json!({
                "type": "object",
                "properties": {
                    "name": string("Item name. Required."),
                    "description": string("Description."),
                    "unit": string("Unit, e.g. hour."),
                    "unitPriceMinor": integer("Unit price in minor units (cents)."),
                    "currency": string("Currency, e.g. USD."),
                    "sku": string("SKU.")
                },
                "required": ["name"]
            })),
            handler: catalog_create,
        },
        Tool {
            name: "ledgerly_catalog_update",
            description: "Replace a catalog item. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Catalog item id."),
                    "input": { "type": "object", "description": "Same fields as ledgerly_catalog_create." }
                },
                "required": ["id", "input"]
            }),
            handler: catalog_update,
        },
        Tool {
            name: "ledgerly_tax_rates_create",
            description: "Create a tax rate. Changes data.",
            write: true,
            schema: input_object(json!({
                "type": "object",
                "properties": {
                    "name": string("Rate name, e.g. VAT."),
                    "rateBp": integer("Rate in basis points, so 20% is 2000.")
                },
                "required": ["name", "rateBp"]
            })),
            handler: tax_rates_create,
        },
        Tool {
            name: "ledgerly_tax_rates_update",
            description: "Replace a tax rate. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Tax rate id."),
                    "input": { "type": "object", "description": "Same fields as ledgerly_tax_rates_create." }
                },
                "required": ["id", "input"]
            }),
            handler: tax_rates_update,
        },
        Tool {
            name: "ledgerly_business_update",
            description: "Update the business profile. Provide the full record. Changes data.",
            write: true,
            schema: input_object(json!({
                "type": "object",
                "properties": {
                    "displayName": string("Business name."),
                    "legalName": string("Legal name."),
                    "email": string("Email."),
                    "phone": string("Phone."),
                    "website": string("Website."),
                    "addressLine1": string("Address line 1."),
                    "addressLine2": string("Address line 2."),
                    "city": string("City."),
                    "region": string("Region / state."),
                    "postalCode": string("Postal code."),
                    "country": string("Country."),
                    "taxId": string("Tax / VAT id."),
                    "defaultCurrency": string("Default currency, e.g. USD."),
                    "defaultTerms": string("Default terms."),
                    "defaultNotes": string("Default notes."),
                    "defaultPaymentInstructions": string("Default payment instructions.")
                }
            })),
            handler: business_update,
        },
        Tool {
            name: "ledgerly_invoices_create",
            description: "Create an invoice. Changes data.",
            write: true,
            schema: input_object(document_input_schema("dueDate")),
            handler: invoices_create,
        },
        Tool {
            name: "ledgerly_invoices_update",
            description: "Replace an invoice's details and items. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Invoice id."),
                    "input": {
                        "type": "object",
                        "description": "Same fields as ledgerly_invoices_create. The number is not changed."
                    }
                },
                "required": ["id", "input"]
            }),
            handler: invoices_update,
        },
        Tool {
            name: "ledgerly_invoices_duplicate",
            description: "Duplicate an invoice as a new draft. Changes data.",
            write: true,
            schema: id_object(),
            handler: invoices_duplicate,
        },
        Tool {
            name: "ledgerly_invoices_set_sent",
            description: "Record that an invoice was sent. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Invoice id."),
                    "sent": boolean("True to mark sent, false to clear.")
                },
                "required": ["id", "sent"]
            }),
            handler: invoices_set_sent,
        },
        Tool {
            name: "ledgerly_invoices_set_chased",
            description: "Record that a payment reminder was chased. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Invoice id."),
                    "chased": boolean("True to mark chased, false to clear.")
                },
                "required": ["id", "chased"]
            }),
            handler: invoices_set_chased,
        },
        Tool {
            name: "ledgerly_estimates_create",
            description: "Create an estimate. Changes data.",
            write: true,
            schema: input_object(document_input_schema("validUntil")),
            handler: estimates_create,
        },
        Tool {
            name: "ledgerly_estimates_update",
            description: "Replace an estimate's details and items. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Estimate id."),
                    "input": { "type": "object", "description": "Same fields as ledgerly_estimates_create." }
                },
                "required": ["id", "input"]
            }),
            handler: estimates_update,
        },
        Tool {
            name: "ledgerly_estimates_set_status",
            description: "Set an estimate's status: draft, sent, accepted, declined or expired. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Estimate id."),
                    "status": string("draft, sent, accepted, declined or expired.")
                },
                "required": ["id", "status"]
            }),
            handler: estimates_set_status,
        },
        Tool {
            name: "ledgerly_estimates_convert",
            description: "Create a linked invoice from an estimate. Changes data.",
            write: true,
            schema: id_object(),
            handler: estimates_convert,
        },
        Tool {
            name: "ledgerly_payments_create",
            description: "Record a payment against an invoice. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "invoiceId": string("Invoice id."),
                    "input": {
                        "type": "object",
                        "properties": {
                            "paidOn": string("Payment date, YYYY-MM-DD."),
                            "amountMinor": integer("Amount in minor units (cents)."),
                            "method": string("Method, e.g. bank."),
                            "reference": string("Reference."),
                            "note": string("Note.")
                        },
                        "required": ["paidOn", "amountMinor"]
                    }
                },
                "required": ["invoiceId", "input"]
            }),
            handler: payments_create,
        },
        Tool {
            name: "ledgerly_payments_update",
            description: "Replace a payment. Changes data.",
            write: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": string("Payment id."),
                    "input": { "type": "object", "description": "Same fields as the payment input on ledgerly_payments_create." }
                },
                "required": ["id", "input"]
            }),
            handler: payments_update,
        },
        Tool {
            name: "ledgerly_backup_now",
            description: "Write a .ledgerly backup, to the given path or the backup folder. Changes data.",
            write: true,
            schema: object(json!({ "path": string("Optional destination file path.") })),
            handler: backup_now,
        },
    ]
}

// --- read handlers ----------------------------------------------------------

fn app_info(_conn: &Connection, _args: &Value) -> AppResult<Value> {
    Ok(json!({
        "name": "ledgerly",
        "version": env!("CARGO_PKG_VERSION"),
        "dataDir": crate::resolve_data_dir().display().to_string(),
    }))
}

fn business_get(conn: &Connection, _args: &Value) -> AppResult<Value> {
    to_value(business::get(conn)?)
}

fn settings_all(conn: &Connection, _args: &Value) -> AppResult<Value> {
    to_value(settings::all(conn)?)
}

fn settings_get(conn: &Connection, args: &Value) -> AppResult<Value> {
    let key = required_str(args, "key")?;
    to_value(settings::get(conn, &key)?)
}

fn clients_list(conn: &Connection, args: &Value) -> AppResult<Value> {
    let query = optional_str(args, "query");
    let include_archived = optional_bool(args, "includeArchived").unwrap_or(false);
    to_value(clients::list(conn, query.as_deref(), include_archived)?)
}

fn clients_get(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    to_value(clients::get(conn, &id)?)
}

fn catalog_list(conn: &Connection, args: &Value) -> AppResult<Value> {
    let query = optional_str(args, "query");
    let include_archived = optional_bool(args, "includeArchived").unwrap_or(false);
    to_value(catalog::list(conn, query.as_deref(), include_archived)?)
}

fn tax_rates_list(conn: &Connection, args: &Value) -> AppResult<Value> {
    let include_archived = optional_bool(args, "includeArchived").unwrap_or(false);
    to_value(tax_rates::list(conn, include_archived)?)
}

fn numbering_get(conn: &Connection, args: &Value) -> AppResult<Value> {
    let kind = required_str(args, "kind")?;
    to_value(numbering::get(conn, &kind)?)
}

fn invoices_list(conn: &Connection, args: &Value) -> AppResult<Value> {
    let query = optional_str(args, "query");
    let status = optional_str(args, "status");
    let client_id = optional_str(args, "clientId");
    let age = optional_str(args, "age");
    let issued_month = optional_str(args, "issuedMonth");

    to_value(invoices::list(
        conn,
        &invoices::ListFilter {
            query: query.as_deref(),
            status: status.as_deref(),
            client_id: client_id.as_deref(),
            age: age.as_deref(),
            issued_month: issued_month.as_deref(),
        },
    )?)
}

fn invoices_get(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    to_value(invoices::get(conn, &id)?)
}

fn invoices_totals(conn: &Connection, args: &Value) -> AppResult<Value> {
    let input: invoices::InvoiceInput = arg_input(args)?;
    to_value(invoices::preview_totals(conn, &input)?)
}

fn estimates_list(conn: &Connection, args: &Value) -> AppResult<Value> {
    let query = optional_str(args, "query");
    let status = optional_str(args, "status");
    let client_id = optional_str(args, "clientId");
    to_value(estimates::list(
        conn,
        query.as_deref(),
        status.as_deref(),
        client_id.as_deref(),
    )?)
}

fn estimates_get(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    to_value(estimates::get(conn, &id)?)
}

fn payments_list(conn: &Connection, args: &Value) -> AppResult<Value> {
    let invoice_id = required_str(args, "invoiceId")?;
    to_value(payments::list(conn, &invoice_id)?)
}

fn reports_revenue(conn: &Connection, args: &Value) -> AppResult<Value> {
    let from = required_str(args, "from")?;
    let to = required_str(args, "to")?;
    let granularity = optional_str(args, "granularity").unwrap_or_else(|| "month".to_string());
    to_value(reports::revenue_by_period(
        conn,
        &from,
        &to,
        reports::Granularity::parse(&granularity),
    )?)
}

fn reports_revenue_by_client(conn: &Connection, args: &Value) -> AppResult<Value> {
    let from = required_str(args, "from")?;
    let to = required_str(args, "to")?;
    to_value(reports::revenue_by_client(conn, &from, &to)?)
}

fn reports_aged_receivables(conn: &Connection, _args: &Value) -> AppResult<Value> {
    to_value(reports::aged_receivables(conn)?)
}

fn reports_tax(conn: &Connection, args: &Value) -> AppResult<Value> {
    let from = required_str(args, "from")?;
    let to = required_str(args, "to")?;
    to_value(reports::tax_collected(conn, &from, &to)?)
}

fn reports_payments_by_method(conn: &Connection, args: &Value) -> AppResult<Value> {
    let from = required_str(args, "from")?;
    let to = required_str(args, "to")?;
    to_value(reports::payments_by_method(conn, &from, &to)?)
}

// --- write handlers ---------------------------------------------------------

fn clients_create(conn: &Connection, args: &Value) -> AppResult<Value> {
    let input: clients::ClientInput = arg_input(args)?;
    to_value(clients::create(conn, &input)?)
}

fn clients_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let input: clients::ClientInput = arg_input(args)?;
    to_value(clients::update(conn, &id, &input)?)
}

fn clients_set_archived(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let archived = optional_bool(args, "archived").unwrap_or(true);
    to_value(clients::set_archived(conn, &id, archived)?)
}

fn catalog_create(conn: &Connection, args: &Value) -> AppResult<Value> {
    let input: catalog::CatalogItemInput = arg_input(args)?;
    to_value(catalog::create(conn, &input)?)
}

fn catalog_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let input: catalog::CatalogItemInput = arg_input(args)?;
    to_value(catalog::update(conn, &id, &input)?)
}

fn tax_rates_create(conn: &Connection, args: &Value) -> AppResult<Value> {
    let input: tax_rates::TaxRateInput = arg_input(args)?;
    to_value(tax_rates::create(conn, &input)?)
}

fn tax_rates_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let input: tax_rates::TaxRateInput = arg_input(args)?;
    to_value(tax_rates::update(conn, &id, &input)?)
}

fn business_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let input: business::BusinessInput = arg_input(args)?;
    to_value(business::update(conn, &input)?)
}

/// Fill a blank currency from the business default, which is what the UI does.
fn with_default_currency(conn: &Connection, currency: &mut String) -> AppResult<()> {
    if currency.trim().is_empty() {
        *currency = business::get(conn)?.default_currency;
    }
    Ok(())
}

fn invoices_create(conn: &Connection, args: &Value) -> AppResult<Value> {
    let mut input: invoices::InvoiceInput = arg_input(args)?;
    with_default_currency(conn, &mut input.currency)?;
    to_value(invoices::create(conn, &input)?)
}

fn invoices_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let mut input: invoices::InvoiceInput = arg_input(args)?;
    with_default_currency(conn, &mut input.currency)?;
    to_value(invoices::update(conn, &id, &input)?)
}

fn invoices_duplicate(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    to_value(invoices::duplicate(conn, &id)?)
}

fn invoices_set_sent(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let sent = optional_bool(args, "sent").unwrap_or(true);
    to_value(invoices::set_sent(conn, &id, sent)?)
}

fn invoices_set_chased(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let chased = optional_bool(args, "chased").unwrap_or(true);
    to_value(invoices::set_chased(conn, &id, chased)?)
}

fn estimates_create(conn: &Connection, args: &Value) -> AppResult<Value> {
    let mut input: estimates::EstimateInput = arg_input(args)?;
    with_default_currency(conn, &mut input.currency)?;
    to_value(estimates::create(conn, &input)?)
}

fn estimates_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let mut input: estimates::EstimateInput = arg_input(args)?;
    with_default_currency(conn, &mut input.currency)?;
    to_value(estimates::update(conn, &id, &input)?)
}

fn estimates_set_status(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let status = required_str(args, "status")?;
    to_value(estimates::set_status(conn, &id, &status)?)
}

fn estimates_convert(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    to_value(estimates::convert(conn, &id)?)
}

fn payments_create(conn: &Connection, args: &Value) -> AppResult<Value> {
    let invoice_id = required_str(args, "invoiceId")?;
    let input: payments::PaymentInput = arg_input(args)?;
    to_value(payments::create(conn, &invoice_id, &input)?)
}

fn payments_update(conn: &Connection, args: &Value) -> AppResult<Value> {
    let id = required_str(args, "id")?;
    let input: payments::PaymentInput = arg_input(args)?;
    to_value(payments::update(conn, &id, &input)?)
}

fn backup_now(conn: &Connection, args: &Value) -> AppResult<Value> {
    let data_dir = crate::resolve_data_dir();
    let destination = match optional_str(args, "path").filter(|path| !path.trim().is_empty()) {
        Some(path) => PathBuf::from(path),
        None => {
            let folder = backup::default_folder(&data_dir);
            std::fs::create_dir_all(&folder)?;
            backup::suggested_path(conn, &folder)?
        }
    };

    backup::create_backup(conn, &destination)?;
    Ok(Value::String(destination.to_string_lossy().to_string()))
}
