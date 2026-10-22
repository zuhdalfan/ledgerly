//! Tray icon and close-to-tray.
//!
//! Closing the window quits unless the user has opted in. When they have, the
//! tray is the only way back to the app and the only honest way out: Quit exits
//! the process, and hiding locks the app when the lock is on.

use rusqlite::Connection;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::db::settings;
use crate::security;
use crate::AppState;

pub const TRAY_ID: &str = "ledgerly-tray";

/// Close hides instead of quitting.
pub const KEY_TRAY: &str = "ui.tray";
/// Lock when hiding, when the app lock is enabled. On unless explicitly off.
pub const KEY_LOCK_ON_HIDE: &str = "security.lockOnHide";

pub fn tray_enabled(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(settings::get(conn, KEY_TRAY)?.as_deref() == Some("1"))
}

pub fn lock_on_hide(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(settings::get(conn, KEY_LOCK_ON_HIDE)?.as_deref() != Some("0"))
}

/// Does a close request hide rather than quit? Quit always wins, so the tray's
/// Quit cannot be swallowed by the same handler that hides the window.
pub fn hide_on_close(tray_enabled: bool, quitting: bool) -> bool {
    tray_enabled && !quitting
}

fn lock_available(conn: &Connection) -> bool {
    security::is_enabled(conn).unwrap_or(false)
}

/// The tray menu. Rebuilt rather than mutated so the Lock now item can follow
/// the lock state without keeping a handle to it in shared state.
fn menu<R: Runtime>(app: &AppHandle<R>, lock_available: bool) -> tauri::Result<Menu<R>> {
    let open = MenuItem::with_id(app, "open", "Open Ledgerly", true, None::<&str>)?;
    let lock = MenuItem::with_id(app, "lock", "Lock now", lock_available, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Ledgerly", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[
            &open,
            &lock,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )
}

/// Build the tray if it is not already there. Removing and rebuilding on every
/// settings change would make the icon blink; `apply` only acts on a change.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if app.tray_by_id(TRAY_ID).is_some() {
        return Ok(());
    }

    let lock_available = app
        .try_state::<AppState>()
        .map(|state| {
            let conn = state.db.lock().unwrap();
            lock_available(&conn) && state.is_unlocked()
        })
        .unwrap_or(false);

    let menu = menu(app, lock_available)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Ledgerly")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "lock" => lock_now(app),
            "quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let focused = app
                    .get_webview_window("main")
                    .map(|window| {
                        window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false)
                    })
                    .unwrap_or(false);

                if focused {
                    hide_and_lock(app);
                } else {
                    show_main_window(app);
                }
            }
        });

    // The bundle icon is already the torn-paper mark, so it is the honest
    // icon at menu-bar size without shipping a second asset that could drift.
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    refresh_lock_item(app);
    Ok(())
}

pub fn remove<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.remove_tray_by_id(TRAY_ID);
}

/// Turn the tray on or off and report the state that actually holds.
pub fn apply<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> tauri::Result<()> {
    if enabled {
        build(app)?;
    } else {
        remove(app);
    }

    if let Some(state) = app.try_state::<AppState>() {
        state.set_tray_enabled(enabled);
    }
    Ok(())
}

/// Enable the Lock now item only when there is something to lock: the app lock
/// is on and the app is currently unlocked.
pub fn refresh_lock_item<R: Runtime>(app: &AppHandle<R>) {
    let enabled = app
        .try_state::<AppState>()
        .map(|state| {
            let conn = state.db.lock().unwrap();
            lock_available(&conn) && state.is_unlocked()
        })
        .unwrap_or(false);

    if app.tray_by_id(TRAY_ID).is_none() {
        return;
    }

    // Menus belong to the UI thread on macOS and Windows, and this is called
    // from command handlers and the tray callbacks alike.
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = handle.tray_by_id(TRAY_ID) {
            if let Ok(menu) = menu(&handle, enabled) {
                let _ = tray.set_menu(Some(menu));
            }
        }
    });
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }

    // Showing the window is the catch-up point after sleep: if the local date
    // moved while we were resident, the daily check is due now.
    crate::reminders::check_due(app);
}

/// Hide the window and, when the lock is on and lock-on-hide is enabled, lock
/// the app first. The shell listens for `security:changed`, so it paints the
/// gate the next time it is shown.
pub fn hide_and_lock<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        let should_lock = {
            let conn = state.db.lock().unwrap();
            lock_available(&conn) && lock_on_hide(&conn).unwrap_or(true)
        };

        if should_lock && state.is_unlocked() {
            state.set_unlocked(false);
            let _ = app.emit("security:changed", ());
            refresh_lock_item(app);
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn lock_now<R: Runtime>(app: &AppHandle<R>) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let lock_available = {
        let conn = state.db.lock().unwrap();
        lock_available(&conn)
    };
    if !lock_available {
        return;
    }

    state.set_unlocked(false);
    let _ = app.emit("security:changed", ());
    refresh_lock_item(app);
}

pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        state.set_quitting(true);
    }
    app.exit(0);
}

/// Ask the app to hide itself. Used by the window close handler.
pub fn hide_for_close<R: Runtime>(app: &AppHandle<R>) {
    hide_and_lock(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    #[test]
    fn closing_quits_unless_tray_mode_is_on() {
        assert!(!hide_on_close(false, false), "the default stays a quit");
        assert!(hide_on_close(true, false));
    }

    #[test]
    fn quit_always_exits_even_in_tray_mode() {
        assert!(!hide_on_close(true, true));
    }

    #[test]
    fn tray_mode_is_off_by_default() {
        let t = temp_db();
        assert!(!tray_enabled(&t.conn).unwrap());
    }

    #[test]
    fn locking_on_hide_is_on_by_default() {
        let t = temp_db();
        assert!(lock_on_hide(&t.conn).unwrap());
    }

    #[test]
    fn the_setting_is_read_back() {
        let t = temp_db();
        settings::set(&t.conn, KEY_TRAY, "1").unwrap();
        settings::set(&t.conn, KEY_LOCK_ON_HIDE, "0").unwrap();

        assert!(tray_enabled(&t.conn).unwrap());
        assert!(!lock_on_hide(&t.conn).unwrap());
    }
}
