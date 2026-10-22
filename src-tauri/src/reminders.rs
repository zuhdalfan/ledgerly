//! Daily overdue reminders.
//!
//! At most one OS notification per local day, only while the app is resident in
//! the tray, unlocked, and there is at least one invoice past its due date. No
//! polling: the check rides launch, unlock, a single midnight timer and the
//! window being shown.

use std::time::Duration;

use rusqlite::Connection;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::{NotificationExt, PermissionState};

use crate::db::settings;
use crate::AppState;

/// Send the daily overdue notification.
pub const KEY_OVERDUE: &str = "notify.overdue";
/// Local date of the last check, so the check runs once a day.
pub const KEY_LAST_RUN: &str = "notify.lastRun";
/// What the OS last said: `granted` or `denied`. Shown in Settings.
pub const KEY_PERMISSION: &str = "notify.permission";

const TITLE: &str = "Ledgerly";

pub fn enabled(conn: &Connection) -> rusqlite::Result<bool> {
    // On by default: it can do nothing until tray mode is on and the OS allows
    // notifications, so making the user find a second switch adds no safety.
    Ok(settings::get(conn, KEY_OVERDUE)?.as_deref() != Some("0"))
}

pub fn permission(conn: &Connection) -> rusqlite::Result<Option<String>> {
    settings::get(conn, KEY_PERMISSION)
}

/// One check per local date.
pub fn should_check(last_run: Option<&str>, today: &str) -> bool {
    last_run != Some(today)
}

/// Invoices that were issued, are past due, and still carry a balance.
///
/// Drafts were never issued, voided invoices are not owed, and a future due
/// date is not overdue yet. `status = 'sent'` is the stored state; part-paid
/// invoices keep it, so they are included while they still owe money.
pub fn count_overdue(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*)
           FROM invoice i
          WHERE i.status = 'sent'
            AND i.voided_at IS NULL
            AND i.due_date IS NOT NULL
            AND date(i.due_date) < date('now', 'localtime')
            AND i.total_minor
                - (SELECT IFNULL(SUM(p.amount_minor), 0)
                     FROM payment p WHERE p.invoice_id = i.id) > 0",
        [],
        |row| row.get(0),
    )
}

/// Short and factual. Never a client name, an amount or an invoice number: a
/// notification is visible without unlocking the app.
pub fn notification_body(locale: &str, count: i64) -> String {
    if locale == "id" {
        // Indonesian does not inflect plurals.
        format!("{count} faktur sudah jatuh tempo — Buka Ledgerly untuk melihatnya.")
    } else if count == 1 {
        "1 invoice is past due — Open Ledgerly to see it.".to_string()
    } else {
        format!("{count} invoices are past due — Open Ledgerly to see them.")
    }
}

/// Seconds until the next local midnight, measured on the local wall clock so
/// the UTC offset cancels out. `None` means the database could not answer.
pub fn seconds_until_midnight(conn: &Connection) -> Option<u64> {
    let seconds: i64 = conn
        .query_row(
            "SELECT CAST(strftime('%s', datetime('now', 'localtime', 'start of day', '+1 day')) AS INTEGER)
                  - CAST(strftime('%s', datetime('now', 'localtime')) AS INTEGER)",
            [],
            |row| row.get(0),
        )
        .ok()?;

    Some(seconds.clamp(60, 86_400) as u64)
}

fn today(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))
        .ok()
}

fn ui_locale(conn: &Connection) -> String {
    settings::get(conn, "ui.locale")
        .ok()
        .flatten()
        .filter(|locale| locale == "id")
        .unwrap_or_else(|| "en".to_string())
}

/// In-app "send a test" copy. Deliberately says nothing about invoices, so the
/// test is safe to fire at any time, including with no data at all.
pub fn test_body(locale: &str) -> String {
    if locale == "id" {
        "Ini notifikasi uji. Notifikasi Ledgerly berfungsi.".to_string()
    } else {
        "This is a test notification. Ledgerly notifications are working.".to_string()
    }
}

/// Send a notification on demand, bypassing the daily gate and the overdue
/// query. Used by the Settings test button so a user can check OS delivery
/// without waiting for a due date or clearing a marker by hand.
pub fn send_test<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let locale = app
        .try_state::<AppState>()
        .map(|state| {
            let conn = state.db.lock().unwrap();
            ui_locale(&conn)
        })
        .unwrap_or_else(|| "en".to_string());

    app.notification()
        .builder()
        .title(TITLE)
        .body(test_body(&locale))
        .show()
        .map_err(|error| error.to_string())
}

/// Run the daily check if it is due. Returns the count when a notification was
/// attempted, so callers (and tests) can tell "nothing due" from "sent".
pub fn check_due<R: Runtime>(app: &AppHandle<R>) -> Option<i64> {
    let state = app.try_state::<AppState>()?;
    if !state.is_unlocked() {
        // A locked app announces nothing; the check waits for an unlock.
        return None;
    }

    let conn = state.db.lock().unwrap();

    if !enabled(&conn).unwrap_or(false) || !crate::tray::tray_enabled(&conn).unwrap_or(false) {
        return None;
    }

    let today = today(&conn)?;
    let last_run = settings::get(&conn, KEY_LAST_RUN).ok().flatten();
    if !should_check(last_run.as_deref(), &today) {
        return None;
    }

    let count = count_overdue(&conn).ok()?;

    // Record the check even when nothing is due, so the query runs once a day.
    let _ = settings::set(&conn, KEY_LAST_RUN, &today);

    if count <= 0 {
        return None;
    }

    let body = notification_body(&ui_locale(&conn), count);
    drop(conn);

    let outcome = app.notification().builder().title(TITLE).body(body).show();

    let state = app.try_state::<AppState>()?;
    let conn = state.db.lock().unwrap();
    let permission = match app.notification().permission_state() {
        Ok(PermissionState::Granted) => "granted",
        Ok(PermissionState::Denied) => "denied",
        // Prompt states only exist on mobile; they are not a grant yet.
        Ok(_) => "prompt",
        Err(_) => "granted",
    };
    let _ = settings::set(&conn, KEY_PERMISSION, permission);

    if let Err(error) = outcome {
        eprintln!("overdue notification skipped: {error}");
    }

    Some(count)
}

/// One timer, rescheduled for the next local midnight. Sleep is a single thread
/// that wakes once a day; it is not a poll and it touches no network.
pub fn spawn_midnight_timer<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || loop {
        let wait = app
            .try_state::<AppState>()
            .and_then(|state| {
                let conn = state.db.lock().unwrap();
                seconds_until_midnight(&conn)
            })
            .unwrap_or(60);

        std::thread::sleep(Duration::from_secs(wait));
        let _ = check_due(&app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    fn invoice(conn: &Connection, id: &str, status: &str, due: &str, total: i64) {
        conn.execute(
            "INSERT INTO invoice (id, number, status, issue_date, due_date, total_minor)
             VALUES (?1, ?2, ?3, date('now', 'localtime', '-30 day'), ?4, ?5)",
            rusqlite::params![id, format!("INV-{id}"), status, due, total],
        )
        .unwrap();
    }

    fn payment(conn: &Connection, invoice_id: &str, amount: i64) {
        conn.execute(
            "INSERT INTO payment (id, invoice_id, paid_on, amount_minor)
             VALUES (?1, ?2, date('now', 'localtime'), ?3)",
            rusqlite::params![format!("pay-{invoice_id}"), invoice_id, amount],
        )
        .unwrap();
    }

    #[test]
    fn only_issued_unpaid_past_due_invoices_count() {
        let t = temp_db();

        // Counted: sent, past due, unpaid.
        invoice(&t.conn, "a", "sent", "2020-01-01", 1_000);
        // Counted: part-paid still owes money.
        invoice(&t.conn, "b", "sent", "2020-01-01", 1_000);
        payment(&t.conn, "b", 400);
        // Excluded: paid in full.
        invoice(&t.conn, "c", "sent", "2020-01-01", 1_000);
        payment(&t.conn, "c", 1_000);
        // Excluded: draft was never issued.
        invoice(&t.conn, "d", "draft", "2020-01-01", 1_000);
        // Excluded: voided.
        invoice(&t.conn, "e", "void", "2020-01-01", 1_000);
        // Excluded: due today, not past due.
        invoice(
            &t.conn,
            "f",
            "sent",
            today(&t.conn).unwrap().as_str(),
            1_000,
        );

        assert_eq!(count_overdue(&t.conn).unwrap(), 2);
    }

    #[test]
    fn a_voided_at_marker_excludes_an_invoice() {
        let t = temp_db();
        invoice(&t.conn, "a", "sent", "2020-01-01", 1_000);
        t.conn
            .execute(
                "UPDATE invoice SET voided_at = datetime('now') WHERE id = 'a'",
                [],
            )
            .unwrap();

        assert_eq!(count_overdue(&t.conn).unwrap(), 0);
    }

    #[test]
    fn the_check_runs_once_per_local_date() {
        assert!(should_check(None, "2026-10-22"));
        assert!(!should_check(Some("2026-10-22"), "2026-10-22"));
        assert!(should_check(Some("2026-10-21"), "2026-10-22"));
    }

    #[test]
    fn the_notification_never_names_a_client_or_an_amount() {
        assert_eq!(
            notification_body("en", 1),
            "1 invoice is past due — Open Ledgerly to see it."
        );
        assert_eq!(
            notification_body("en", 3),
            "3 invoices are past due — Open Ledgerly to see them."
        );
        for body in [notification_body("en", 2), notification_body("id", 2)] {
            assert!(!body.contains('$'));
            assert!(!body.contains("INV-"));
        }
    }

    #[test]
    fn reminders_are_on_by_default() {
        let t = temp_db();
        assert!(enabled(&t.conn).unwrap());

        settings::set(&t.conn, KEY_OVERDUE, "0").unwrap();
        assert!(!enabled(&t.conn).unwrap());
    }

    #[test]
    fn the_test_notification_makes_no_claims_about_data() {
        for body in [test_body("en"), test_body("id")] {
            assert!(!body.contains("invoice is past due"));
            assert!(!body.contains("faktur sudah jatuh tempo"));
        }
        assert!(!test_body("en").contains('$'));
    }

    #[test]
    fn the_midnight_timer_is_under_a_day_and_not_immediate() {
        let t = temp_db();
        let wait = seconds_until_midnight(&t.conn).expect("sqlite should answer");
        assert!((60..=86_400).contains(&wait));
    }
}
