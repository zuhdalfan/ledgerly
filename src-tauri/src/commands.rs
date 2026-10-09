use crate::db::business::{self, Business, BusinessInput};
use crate::db::catalog::{self, CatalogItem, CatalogItemInput};
use crate::db::clients::{self, Client, ClientInput};
use crate::db::invoices::{self, Invoice, InvoiceInput};
use crate::db::numbering::{self, Numbering, NumberingInput};
use crate::db::settings::{self, Setting};
use crate::db::tax_rates::{self, TaxRate, TaxRateInput};
use crate::domain::totals::InvoiceTotals;
use crate::error::{AppError, AppResult};
use crate::pdf;
use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub data_dir: String,
    pub database_path: String,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state.data_dir.display().to_string(),
        database_path: state.database_path().display().to_string(),
    }
}

#[tauri::command]
pub fn settings_all(state: State<'_, AppState>) -> AppResult<Vec<Setting>> {
    let conn = state.db.lock().unwrap();
    Ok(settings::all(&conn)?)
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    let conn = state.db.lock().unwrap();
    Ok(settings::get(&conn, &key)?)
}

#[tauri::command]
pub fn settings_set(state: State<'_, AppState>, key: String, value: String) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    settings::set(&conn, &key, &value)?;
    Ok(())
}

#[tauri::command]
pub fn business_get(state: State<'_, AppState>) -> AppResult<Business> {
    let conn = state.db.lock().unwrap();
    Ok(business::get(&conn)?)
}

#[tauri::command]
pub fn business_update(state: State<'_, AppState>, input: BusinessInput) -> AppResult<Business> {
    let conn = state.db.lock().unwrap();
    Ok(business::update(&conn, &input)?)
}

#[tauri::command]
pub fn clients_list(
    state: State<'_, AppState>,
    query: Option<String>,
    include_archived: Option<bool>,
) -> AppResult<Vec<Client>> {
    let conn = state.db.lock().unwrap();
    Ok(clients::list(
        &conn,
        query.as_deref(),
        include_archived.unwrap_or(false),
    )?)
}

#[tauri::command]
pub fn clients_get(state: State<'_, AppState>, id: String) -> AppResult<Option<Client>> {
    let conn = state.db.lock().unwrap();
    Ok(clients::get(&conn, &id)?)
}

#[tauri::command]
pub fn clients_create(state: State<'_, AppState>, input: ClientInput) -> AppResult<Client> {
    let conn = state.db.lock().unwrap();
    Ok(clients::create(&conn, &input)?)
}

#[tauri::command]
pub fn clients_update(
    state: State<'_, AppState>,
    id: String,
    input: ClientInput,
) -> AppResult<Client> {
    let conn = state.db.lock().unwrap();
    Ok(clients::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn clients_set_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> AppResult<Client> {
    let conn = state.db.lock().unwrap();
    Ok(clients::set_archived(&conn, &id, archived)?)
}

#[tauri::command]
pub fn catalog_list(
    state: State<'_, AppState>,
    query: Option<String>,
    include_archived: Option<bool>,
) -> AppResult<Vec<CatalogItem>> {
    let conn = state.db.lock().unwrap();
    Ok(catalog::list(
        &conn,
        query.as_deref(),
        include_archived.unwrap_or(false),
    )?)
}

#[tauri::command]
pub fn catalog_get(state: State<'_, AppState>, id: String) -> AppResult<Option<CatalogItem>> {
    let conn = state.db.lock().unwrap();
    Ok(catalog::get(&conn, &id)?)
}

#[tauri::command]
pub fn catalog_create(
    state: State<'_, AppState>,
    input: CatalogItemInput,
) -> AppResult<CatalogItem> {
    let conn = state.db.lock().unwrap();
    Ok(catalog::create(&conn, &input)?)
}

#[tauri::command]
pub fn catalog_update(
    state: State<'_, AppState>,
    id: String,
    input: CatalogItemInput,
) -> AppResult<CatalogItem> {
    let conn = state.db.lock().unwrap();
    Ok(catalog::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn catalog_set_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> AppResult<CatalogItem> {
    let conn = state.db.lock().unwrap();
    Ok(catalog::set_archived(&conn, &id, archived)?)
}

#[tauri::command]
pub fn tax_rates_list(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> AppResult<Vec<TaxRate>> {
    let conn = state.db.lock().unwrap();
    Ok(tax_rates::list(&conn, include_archived.unwrap_or(false))?)
}

#[tauri::command]
pub fn tax_rates_create(state: State<'_, AppState>, input: TaxRateInput) -> AppResult<TaxRate> {
    let conn = state.db.lock().unwrap();
    Ok(tax_rates::create(&conn, &input)?)
}

#[tauri::command]
pub fn tax_rates_update(
    state: State<'_, AppState>,
    id: String,
    input: TaxRateInput,
) -> AppResult<TaxRate> {
    let conn = state.db.lock().unwrap();
    Ok(tax_rates::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn tax_rates_set_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> AppResult<TaxRate> {
    let conn = state.db.lock().unwrap();
    Ok(tax_rates::set_archived(&conn, &id, archived)?)
}

#[tauri::command]
pub fn numbering_get(state: State<'_, AppState>, kind: String) -> AppResult<Numbering> {
    let conn = state.db.lock().unwrap();
    Ok(numbering::get(&conn, &kind)?)
}

#[tauri::command]
pub fn numbering_update(
    state: State<'_, AppState>,
    kind: String,
    input: NumberingInput,
) -> AppResult<Numbering> {
    let conn = state.db.lock().unwrap();
    Ok(numbering::update(&conn, &kind, &input)?)
}

#[tauri::command]
pub fn invoices_list(
    state: State<'_, AppState>,
    query: Option<String>,
    status: Option<String>,
) -> AppResult<Vec<Invoice>> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::list(&conn, query.as_deref(), status.as_deref())?)
}

#[tauri::command]
pub fn invoices_get(state: State<'_, AppState>, id: String) -> AppResult<Option<Invoice>> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::get(&conn, &id)?)
}

#[tauri::command]
pub fn invoices_totals(
    state: State<'_, AppState>,
    input: InvoiceInput,
) -> AppResult<InvoiceTotals> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::preview_totals(&conn, &input)?)
}

#[tauri::command]
pub fn invoices_create(state: State<'_, AppState>, input: InvoiceInput) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::create(&conn, &input)?)
}

#[tauri::command]
pub fn invoices_update(
    state: State<'_, AppState>,
    id: String,
    input: InvoiceInput,
) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn invoices_duplicate(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::duplicate(&conn, &id)?)
}

#[tauri::command]
pub fn invoices_set_void(
    state: State<'_, AppState>,
    id: String,
    voided: bool,
) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    Ok(invoices::set_void(&conn, &id, voided)?)
}

/// Render an invoice to PDF and write it to `path` (chosen by the user in the
/// save dialog). Returns the path written.
#[tauri::command]
pub fn pdf_export_invoice(
    state: State<'_, AppState>,
    id: String,
    path: String,
    template: Option<String>,
) -> AppResult<String> {
    let (invoice, business, client) = {
        let conn = state.db.lock().unwrap();
        let invoice = invoices::get(&conn, &id)?
            .ok_or_else(|| AppError::Pdf("invoice not found".to_string()))?;
        let business = business::get(&conn)?;
        let client = match invoice.client_id.as_deref() {
            Some(client_id) => clients::get(&conn, client_id)?,
            None => None,
        };
        (invoice, business, client)
    };

    let bytes = pdf::render_invoice(
        &invoice,
        &business,
        client.as_ref(),
        pdf::Template::parse(template.as_deref().unwrap_or("classic")),
    )
    .map_err(AppError::Pdf)?;

    std::fs::write(&path, &bytes)?;
    Ok(path)
}

/// Open a file (usually the PDF just exported) in the OS default viewer.
#[tauri::command]
pub fn open_in_viewer(app: tauri::AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|error| AppError::Pdf(error.to_string()))?;
    Ok(())
}
