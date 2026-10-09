mod commands;
mod db;
mod domain;
mod error;
mod pdf;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn database_path(&self) -> PathBuf {
        db_path(&self.data_dir)
    }
}

fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("ledgerly.db")
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
            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
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
            commands::open_in_viewer,
            commands::invoices_set_sent,
            commands::payments_list,
            commands::payments_create,
            commands::payments_update,
            commands::payments_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
