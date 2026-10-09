use crate::db::settings::{self, Setting};
use crate::error::AppResult;
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
