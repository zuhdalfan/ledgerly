use crate::backup::{self, BackupInfo, BackupSettings, BackupSettingsInput};
use crate::db::business::{self, Business, BusinessInput};
use crate::db::catalog::{self, CatalogItem, CatalogItemInput};
use crate::db::clients::{self, Client, ClientInput};
use crate::db::estimates::{self, Estimate, EstimateInput};
use crate::db::invoices::{self, Invoice, InvoiceInput};
use crate::db::numbering::{self, Numbering, NumberingInput};
use crate::db::payments::{self, Payment, PaymentInput};
use crate::db::reports::{
    self, AgedRow, ClientRevenueRow, Granularity, MethodRow, PeriodRow, TaxRow,
};
use crate::db::settings::{self, Setting};
use crate::db::tax_rates::{self, TaxRate, TaxRateInput};
use crate::domain::totals::InvoiceTotals;
use crate::error::{AppError, AppResult};
use crate::export;
use crate::licensing::{self, LicenseStatus};
use crate::pdf;
use crate::security::{self, LockSettings, LockSettingsInput};
use crate::AppState;
use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

/// Gate for Pro-only work. Free (and anything unrecognised) is denied.
fn ensure(state: &AppState, conn: &Connection, feature: &str) -> AppResult<()> {
    if state.entitlements(conn)?.allows(feature) {
        return Ok(());
    }

    Err(AppError::Validation(format!(
        "{} is a Ledgerly Pro feature.",
        feature_label(feature)
    )))
}

/// Everything that touches user data passes through this while the lock is on,
/// so the check cannot be bypassed by talking to the IPC layer directly.
fn require_unlocked(state: &AppState) -> AppResult<()> {
    if state.is_unlocked() {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Ledgerly is locked. Unlock to continue.".to_string(),
        ))
    }
}

fn feature_label(feature: &str) -> &'static str {
    match feature {
        "reports" => "Reports",
        "pdf_templates" => "PDF templates other than Classic",
        "branding" => "Custom branding",
        "scheduled_backups" => "Scheduled backups",
        "batch_export" => "Batch export",
        "multi_currency" => "Multi-currency",
        _ => "That feature",
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub data_dir: String,
    pub database_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(locked: bool) -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        AppState::new(conn, PathBuf::from("."), locked)
    }

    #[test]
    fn a_locked_app_refuses_data_commands() {
        let locked = state(true);
        assert!(require_unlocked(&locked).is_err());

        locked.set_unlocked(true);
        assert!(require_unlocked(&locked).is_ok());

        locked.set_unlocked(false);
        assert!(
            require_unlocked(&locked).is_err(),
            "locking again must re-arm"
        );
    }

    #[test]
    fn an_unlocked_app_allows_them() {
        assert!(require_unlocked(&state(false)).is_ok());
    }

    #[test]
    fn pro_features_are_labelled_for_the_user() {
        assert_eq!(feature_label("reports"), "Reports");
        assert_eq!(feature_label("scheduled_backups"), "Scheduled backups");
        assert_eq!(feature_label("something_new"), "That feature");
    }
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
    require_unlocked(&state)?;
    Ok(settings::all(&conn)?)
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(settings::get(&conn, &key)?)
}

#[tauri::command]
pub fn settings_set(state: State<'_, AppState>, key: String, value: String) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    settings::set(&conn, &key, &value)?;
    Ok(())
}

#[tauri::command]
pub fn business_get(state: State<'_, AppState>) -> AppResult<Business> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(business::get(&conn)?)
}

#[tauri::command]
pub fn business_update(state: State<'_, AppState>, input: BusinessInput) -> AppResult<Business> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(business::update(&conn, &input)?)
}

#[tauri::command]
pub fn clients_list(
    state: State<'_, AppState>,
    query: Option<String>,
    include_archived: Option<bool>,
) -> AppResult<Vec<Client>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(clients::list(
        &conn,
        query.as_deref(),
        include_archived.unwrap_or(false),
    )?)
}

#[tauri::command]
pub fn clients_get(state: State<'_, AppState>, id: String) -> AppResult<Option<Client>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(clients::get(&conn, &id)?)
}

#[tauri::command]
pub fn clients_create(state: State<'_, AppState>, input: ClientInput) -> AppResult<Client> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(clients::create(&conn, &input)?)
}

#[tauri::command]
pub fn clients_update(
    state: State<'_, AppState>,
    id: String,
    input: ClientInput,
) -> AppResult<Client> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(clients::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn clients_set_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> AppResult<Client> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(clients::set_archived(&conn, &id, archived)?)
}

#[tauri::command]
pub fn catalog_list(
    state: State<'_, AppState>,
    query: Option<String>,
    include_archived: Option<bool>,
) -> AppResult<Vec<CatalogItem>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(catalog::list(
        &conn,
        query.as_deref(),
        include_archived.unwrap_or(false),
    )?)
}

#[tauri::command]
pub fn catalog_get(state: State<'_, AppState>, id: String) -> AppResult<Option<CatalogItem>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(catalog::get(&conn, &id)?)
}

#[tauri::command]
pub fn catalog_create(
    state: State<'_, AppState>,
    input: CatalogItemInput,
) -> AppResult<CatalogItem> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(catalog::create(&conn, &input)?)
}

#[tauri::command]
pub fn catalog_update(
    state: State<'_, AppState>,
    id: String,
    input: CatalogItemInput,
) -> AppResult<CatalogItem> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(catalog::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn catalog_set_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> AppResult<CatalogItem> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(catalog::set_archived(&conn, &id, archived)?)
}

#[tauri::command]
pub fn tax_rates_list(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> AppResult<Vec<TaxRate>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(tax_rates::list(&conn, include_archived.unwrap_or(false))?)
}

#[tauri::command]
pub fn tax_rates_create(state: State<'_, AppState>, input: TaxRateInput) -> AppResult<TaxRate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(tax_rates::create(&conn, &input)?)
}

#[tauri::command]
pub fn tax_rates_update(
    state: State<'_, AppState>,
    id: String,
    input: TaxRateInput,
) -> AppResult<TaxRate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(tax_rates::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn tax_rates_set_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> AppResult<TaxRate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(tax_rates::set_archived(&conn, &id, archived)?)
}

#[tauri::command]
pub fn numbering_get(state: State<'_, AppState>, kind: String) -> AppResult<Numbering> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(numbering::get(&conn, &kind)?)
}

#[tauri::command]
pub fn numbering_update(
    state: State<'_, AppState>,
    kind: String,
    input: NumberingInput,
) -> AppResult<Numbering> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(numbering::update(&conn, &kind, &input)?)
}

#[tauri::command]
pub fn invoices_list(
    state: State<'_, AppState>,
    query: Option<String>,
    status: Option<String>,
) -> AppResult<Vec<Invoice>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::list(&conn, query.as_deref(), status.as_deref())?)
}

#[tauri::command]
pub fn invoices_get(state: State<'_, AppState>, id: String) -> AppResult<Option<Invoice>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::get(&conn, &id)?)
}

#[tauri::command]
pub fn invoices_totals(
    state: State<'_, AppState>,
    input: InvoiceInput,
) -> AppResult<InvoiceTotals> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::preview_totals(&conn, &input)?)
}

#[tauri::command]
pub fn invoices_create(state: State<'_, AppState>, input: InvoiceInput) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::create(&conn, &input)?)
}

#[tauri::command]
pub fn invoices_update(
    state: State<'_, AppState>,
    id: String,
    input: InvoiceInput,
) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn invoices_duplicate(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::duplicate(&conn, &id)?)
}

#[tauri::command]
pub fn invoices_set_void(
    state: State<'_, AppState>,
    id: String,
    voided: bool,
) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
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
    let template_name = template.as_deref().unwrap_or("classic").to_string();

    let (invoice, business, client, entitlements, locale) = {
        let conn = state.db.lock().unwrap();
        require_unlocked(&state)?;
        let entitlements = state.entitlements(&conn)?;
        // Document language is independent of the app's UI language.
        let locale = settings::get(&conn, "pdf.locale")?.unwrap_or_else(|| "en".to_string());
        if template_name != "classic" && !entitlements.allows("pdf_templates") {
            return Err(AppError::Validation(
                "PDF templates other than Classic are a Ledgerly Pro feature.".to_string(),
            ));
        }

        let invoice = invoices::get(&conn, &id)?
            .ok_or_else(|| AppError::Pdf("invoice not found".to_string()))?;
        let business = business::get(&conn)?;
        let client = match invoice.client_id.as_deref() {
            Some(client_id) => clients::get(&conn, client_id)?,
            None => None,
        };
        (invoice, business, client, entitlements, locale)
    };

    let bytes = pdf::render_invoice(
        &invoice,
        &business,
        client.as_ref(),
        pdf::PdfOptions {
            template: pdf::Template::parse(&template_name),
            // Free invoices carry a small footer; Pro removes it.
            promo_footer: !entitlements.allows("branding"),
            locale: pdf::DocumentLocale::parse(&locale),
        },
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

#[tauri::command]
pub fn invoices_set_sent(state: State<'_, AppState>, id: String, sent: bool) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(invoices::set_sent(&conn, &id, sent)?)
}

#[tauri::command]
pub fn estimates_list(
    state: State<'_, AppState>,
    query: Option<String>,
    status: Option<String>,
) -> AppResult<Vec<Estimate>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(estimates::list(&conn, query.as_deref(), status.as_deref())?)
}

#[tauri::command]
pub fn estimates_get(state: State<'_, AppState>, id: String) -> AppResult<Option<Estimate>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(estimates::get(&conn, &id)?)
}

#[tauri::command]
pub fn estimates_totals(
    state: State<'_, AppState>,
    input: EstimateInput,
) -> AppResult<InvoiceTotals> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(estimates::preview_totals(&conn, &input)?)
}

#[tauri::command]
pub fn estimates_create(state: State<'_, AppState>, input: EstimateInput) -> AppResult<Estimate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(estimates::create(&conn, &input)?)
}

#[tauri::command]
pub fn estimates_update(
    state: State<'_, AppState>,
    id: String,
    input: EstimateInput,
) -> AppResult<Estimate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(estimates::update(&conn, &id, &input)?)
}

#[tauri::command]
pub fn estimates_duplicate(state: State<'_, AppState>, id: String) -> AppResult<Estimate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(estimates::duplicate(&conn, &id)?)
}

#[tauri::command]
pub fn estimates_set_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
) -> AppResult<Estimate> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    estimates::set_status(&conn, &id, &status)
}

/// Create an invoice from an estimate and link them.
#[tauri::command]
pub fn estimates_convert(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    estimates::convert(&conn, &id)
}

#[tauri::command]
pub fn payments_list(state: State<'_, AppState>, invoice_id: String) -> AppResult<Vec<Payment>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(payments::list(&conn, &invoice_id)?)
}

#[tauri::command]
pub fn payments_create(
    state: State<'_, AppState>,
    invoice_id: String,
    input: PaymentInput,
) -> AppResult<Payment> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    payments::create(&conn, &invoice_id, &input)
}

#[tauri::command]
pub fn payments_update(
    state: State<'_, AppState>,
    id: String,
    input: PaymentInput,
) -> AppResult<Payment> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    payments::update(&conn, &id, &input)
}

#[tauri::command]
pub fn payments_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    payments::delete(&conn, &id)
}

#[tauri::command]
pub fn reports_revenue(
    state: State<'_, AppState>,
    from: String,
    to: String,
    granularity: Option<String>,
) -> AppResult<Vec<PeriodRow>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    ensure(&state, &conn, "reports")?;
    Ok(reports::revenue_by_period(
        &conn,
        &from,
        &to,
        Granularity::parse(granularity.as_deref().unwrap_or("month")),
    )?)
}

#[tauri::command]
pub fn reports_revenue_by_client(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<ClientRevenueRow>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    ensure(&state, &conn, "reports")?;
    Ok(reports::revenue_by_client(&conn, &from, &to)?)
}

#[tauri::command]
pub fn reports_aged_receivables(state: State<'_, AppState>) -> AppResult<Vec<AgedRow>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    ensure(&state, &conn, "reports")?;
    Ok(reports::aged_receivables(&conn)?)
}

#[tauri::command]
pub fn reports_tax(state: State<'_, AppState>, from: String, to: String) -> AppResult<Vec<TaxRow>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    ensure(&state, &conn, "reports")?;
    Ok(reports::tax_collected(&conn, &from, &to)?)
}

#[tauri::command]
pub fn reports_payments_by_method(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> AppResult<Vec<MethodRow>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    ensure(&state, &conn, "reports")?;
    Ok(reports::payments_by_method(&conn, &from, &to)?)
}

/// Write a report the frontend already holds to a CSV file.
#[tauri::command]
pub fn export_csv(path: String, headers: Vec<String>, rows: Vec<Vec<String>>) -> AppResult<String> {
    export::write_csv(&path, &headers, &rows)?;
    Ok(path)
}

/// Write the whole database to a single JSON file.
#[tauri::command]
pub fn export_json(state: State<'_, AppState>, path: String) -> AppResult<String> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    export::export_json(&conn, &path)?;
    Ok(path)
}

/// Write a `.ledgerly` backup. With no path, it goes to the backup folder.
#[tauri::command]
pub fn backup_now(state: State<'_, AppState>, path: Option<String>) -> AppResult<String> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;

    let destination = match path.filter(|value| !value.trim().is_empty()) {
        Some(value) => PathBuf::from(value),
        None => {
            let folder = backup::default_folder(&state.data_dir);
            std::fs::create_dir_all(&folder)?;
            backup::suggested_path(&conn, &folder)?
        }
    };

    backup::create_backup(&conn, &destination)?;
    Ok(destination.to_string_lossy().to_string())
}

/// Replace the current data with a backup. A safety copy of the current data is
/// taken first and its path returned.
#[tauri::command]
pub fn backup_restore(state: State<'_, AppState>, path: String) -> AppResult<String> {
    let database = state.database_path();
    require_unlocked(&state)?;
    let mut guard = state.db.lock().unwrap();

    let stamp: String = guard.query_row(
        "SELECT strftime('%Y%m%d-%H%M%S', 'now', 'localtime')",
        [],
        |row| row.get(0),
    )?;
    let safety = state.data_dir.join(format!("pre-restore-{stamp}.sqlite"));

    // Consistent copy of what we are about to replace.
    backup::snapshot_database(&guard, &safety)?;

    // Close the live connection so the file can be swapped underneath it.
    let previous = std::mem::replace(&mut *guard, Connection::open_in_memory()?);
    drop(previous);

    let result = backup::restore_database(Path::new(&path), &database);

    // Always leave the app with a working connection, then report any failure.
    *guard = crate::db::open(&database)?;

    if let Ok(manifest) = &result {
        eprintln!(
            "restored backup with schema version {}",
            manifest.schema_version
        );
    }
    result?;

    Ok(safety.to_string_lossy().to_string())
}

#[tauri::command]
pub fn backup_list(state: State<'_, AppState>) -> AppResult<Vec<BackupInfo>> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    let settings = backup::load_settings(&conn, &state.data_dir)?;
    backup::list_backups(Path::new(&settings.folder))
}

#[tauri::command]
pub fn backup_settings_get(state: State<'_, AppState>) -> AppResult<BackupSettings> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    Ok(backup::load_settings(&conn, &state.data_dir)?)
}

#[tauri::command]
pub fn backup_settings_set(
    state: State<'_, AppState>,
    input: BackupSettingsInput,
) -> AppResult<BackupSettings> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    if input.enabled {
        ensure(&state, &conn, "scheduled_backups")?;
    }
    Ok(backup::save_settings(&conn, &state.data_dir, &input)?)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityStatus {
    pub locked: bool,
    pub settings: LockSettings,
    pub min_passphrase_length: usize,
}

fn security_status_for(state: &AppState, conn: &Connection) -> AppResult<SecurityStatus> {
    Ok(SecurityStatus {
        locked: !state.is_unlocked(),
        settings: security::settings(conn)?,
        min_passphrase_length: security::MIN_PASSPHRASE_LEN,
    })
}

/// Safe to call while locked: the lock screen needs it.
#[tauri::command]
pub fn security_status(state: State<'_, AppState>) -> AppResult<SecurityStatus> {
    let conn = state.db.lock().unwrap();
    security_status_for(&state, &conn)
}

#[tauri::command]
pub fn security_unlock(
    state: State<'_, AppState>,
    passphrase: String,
) -> AppResult<SecurityStatus> {
    let conn = state.db.lock().unwrap();

    if security::settings(&conn)?.enabled && !security::verify(&conn, &passphrase)? {
        return Err(AppError::Validation(
            "That passphrase is not correct.".to_string(),
        ));
    }

    state.set_unlocked(true);
    security_status_for(&state, &conn)
}

#[tauri::command]
pub fn security_lock(state: State<'_, AppState>) -> AppResult<SecurityStatus> {
    let conn = state.db.lock().unwrap();
    state.set_unlocked(false);
    security_status_for(&state, &conn)
}

#[tauri::command]
pub fn security_enable(
    state: State<'_, AppState>,
    input: LockSettingsInput,
) -> AppResult<SecurityStatus> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;

    security::enable(&conn, &input.passphrase, input.auto_lock_minutes)?;
    state.set_unlocked(true);
    security_status_for(&state, &conn)
}

#[tauri::command]
pub fn security_disable(
    state: State<'_, AppState>,
    passphrase: String,
) -> AppResult<SecurityStatus> {
    let conn = state.db.lock().unwrap();
    // Verifying the current passphrase is itself the permission check.
    security::disable(&conn, &passphrase)?;
    security_status_for(&state, &conn)
}

#[tauri::command]
pub fn security_set_auto_lock(
    state: State<'_, AppState>,
    minutes: i64,
) -> AppResult<SecurityStatus> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;

    security::set_auto_lock(&conn, minutes)?;
    security_status_for(&state, &conn)
}

#[tauri::command]
pub fn license_status(state: State<'_, AppState>) -> AppResult<LicenseStatus> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    licensing::status(&conn)
}

#[tauri::command]
pub fn license_request_code(state: State<'_, AppState>) -> AppResult<String> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    licensing::request_code_text(&conn)
}

#[tauri::command]
pub fn license_save_request_file(state: State<'_, AppState>, path: String) -> AppResult<String> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    let text = licensing::request_code_text(&conn)?;
    std::fs::write(&path, format!("{text}\n"))?;
    Ok(path)
}

#[tauri::command]
pub fn license_import_key(state: State<'_, AppState>, text: String) -> AppResult<LicenseStatus> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    let status = licensing::activate(&conn, &text)?;
    state.forget_entitlements();
    Ok(status)
}

#[tauri::command]
pub fn license_import_file(state: State<'_, AppState>, path: String) -> AppResult<LicenseStatus> {
    let text = std::fs::read_to_string(&path)?;
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    let status = licensing::activate(&conn, &text)?;
    state.forget_entitlements();
    Ok(status)
}

#[tauri::command]
pub fn license_deactivate(state: State<'_, AppState>) -> AppResult<LicenseStatus> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    let status = licensing::deactivate(&conn)?;
    state.forget_entitlements();
    Ok(status)
}

#[tauri::command]
pub fn license_start_trial(state: State<'_, AppState>) -> AppResult<LicenseStatus> {
    let conn = state.db.lock().unwrap();
    require_unlocked(&state)?;
    let status = licensing::start_trial(&conn, licensing::DEFAULT_TRIAL_DAYS)?;
    state.forget_entitlements();
    Ok(status)
}
