mod backup;
mod commands;
mod db;
mod domain;
mod error;
mod export;
pub mod mcp;
mod pdf;
mod security;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
    /// False while the app lock is engaged; every data command checks this.
    unlocked: Mutex<bool>,
}

impl AppState {
    pub fn new(db: Connection, data_dir: PathBuf, locked: bool) -> Self {
        Self {
            db: Mutex::new(db),
            data_dir,
            unlocked: Mutex::new(!locked),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        *self.unlocked.lock().unwrap()
    }

    pub fn set_unlocked(&self, value: bool) {
        *self.unlocked.lock().unwrap() = value;
    }

    pub fn database_path(&self) -> PathBuf {
        db_path(&self.data_dir)
    }
}

fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("ledgerly.db")
}

/// Where the app keeps its data, resolved the same way Tauri resolves
/// `app_data_dir()` for the `com.ledgerly.desktop` bundle id.
///
/// The MCP server is a separate process with no Tauri handle, so it repeats
/// this calculation to find the one database the two of them share.
/// `LEDGERLY_DATA_DIR` overrides it, which is handy for tests and for anyone
/// who has moved the folder by hand.
pub fn resolve_data_dir() -> PathBuf {
    match std::env::var_os("LEDGERLY_DATA_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => data_base_dir().join("com.ledgerly.desktop"),
    }
}

#[cfg(target_os = "windows")]
fn data_base_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(target_os = "macos")]
fn data_base_dir() -> PathBuf {
    home_dir().join("Library").join("Application Support")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn data_base_dir() -> PathBuf {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => home_dir().join(".local").join("share"),
    }
}

#[cfg(not(target_os = "windows"))]
fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::open(&db_path(&data_dir))?;

            // Rotating automatic backup, at most once a day when enabled. A
            // failure here must never stop the app from starting.
            if let Err(error) = backup::auto_backup(&conn, &data_dir) {
                eprintln!("automatic backup skipped: {error}");
            }

            // If a passphrase is set, start locked.
            let locked = security::is_enabled(&conn).unwrap_or(false);
            app.manage(AppState::new(conn, data_dir, locked));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::mcp_info,
            commands::settings_all,
            commands::settings_get,
            commands::settings_set,
            commands::business_get,
            commands::business_update,
            commands::clients_list,
            commands::clients_get,
            commands::clients_create,
            commands::clients_update,
            commands::clients_set_archived,
            commands::catalog_list,
            commands::catalog_get,
            commands::catalog_create,
            commands::catalog_update,
            commands::catalog_set_archived,
            commands::tax_rates_list,
            commands::tax_rates_create,
            commands::tax_rates_update,
            commands::tax_rates_set_archived,
            commands::numbering_get,
            commands::numbering_update,
            commands::invoices_list,
            commands::invoices_get,
            commands::invoices_totals,
            commands::invoices_create,
            commands::invoices_update,
            commands::invoices_duplicate,
            commands::invoices_set_void,
            commands::pdf_export_invoice,
            commands::pdf_export_estimate,
            commands::pdf_export_statement,
            commands::open_in_viewer,
            commands::invoices_set_sent,
            commands::invoices_set_chased,
            commands::payments_list,
            commands::payments_create,
            commands::payments_update,
            commands::payments_delete,
            commands::estimates_list,
            commands::estimates_get,
            commands::estimates_totals,
            commands::estimates_create,
            commands::estimates_update,
            commands::estimates_duplicate,
            commands::estimates_set_status,
            commands::estimates_convert,
            commands::reports_revenue,
            commands::reports_revenue_by_client,
            commands::reports_aged_receivables,
            commands::reports_tax,
            commands::reports_payments_by_method,
            commands::export_csv,
            commands::export_json,
            commands::backup_now,
            commands::backup_restore,
            commands::backup_list,
            commands::backup_settings_get,
            commands::backup_settings_set,
            commands::security_status,
            commands::security_unlock,
            commands::security_lock,
            commands::security_enable,
            commands::security_disable,
            commands::security_set_auto_lock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod data_dir_tests {
    use super::*;

    #[test]
    fn the_default_data_dir_is_scoped_to_the_bundle_id() {
        assert!(resolve_data_dir()
            .to_string_lossy()
            .ends_with("com.ledgerly.desktop"));
    }
}
