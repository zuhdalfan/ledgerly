//! The Ledgerly MCP server.
//!
//! A small Model Context Protocol server over stdio that lets a desktop AI
//! client drive the same SQLite database the app uses. It is deliberately
//! stdio-only: Ledgerly promises it never opens a network connection, and a
//! stdio server keeps that promise literally -- no socket, no port, and no HTTP
//! client anywhere in the dependency graph.
//!
//! The process is launched by the AI client (see `docs/AI.md`), not by the app.

mod tools;

use std::io::{BufRead, Write};

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::{db, resolve_data_dir, security};
use tools::Tool;

/// Settings the server honours. Kept here so the UI and the server cannot drift
/// on the key names or the on/off spelling.
pub const ENABLED_KEY: &str = "mcp.enabled";
pub const ALLOW_WRITE_KEY: &str = "mcp.allowWrite";

/// The MCP revision this server speaks when the client does not name one.
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// Whether the user has switched AI access on in Settings -> AI.
pub fn enabled(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(db::settings::get(conn, ENABLED_KEY)?.as_deref() == Some("true"))
}

/// Whether the user has also allowed the AI to change data.
pub fn allow_write(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(db::settings::get(conn, ALLOW_WRITE_KEY)?.as_deref() == Some("true"))
}

/// Read one JSON-RPC message per line from stdin and write one response per
/// request to stdout. Returns a process exit code.
pub fn run() -> i32 {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();

    // Open the database once. If it will not open we still speak the protocol,
    // so the client gets a readable error instead of silence. The directory may
    // not exist yet if the assistant is used before the app has ever run.
    let data_dir = resolve_data_dir();
    let _ = std::fs::create_dir_all(&data_dir);

    let open_error;
    let conn = match db::open(&data_dir.join("ledgerly.db")) {
        Ok(conn) => Some(conn),
        Err(error) => {
            open_error = error.to_string();
            eprintln!("ledgerly-mcp: could not open the database: {open_error}");
            None
        }
    };

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                let message = error_response(Value::Null, -32700, &error.to_string());
                let _ = write_message(&mut out, &message);
                continue;
            }
        };

        if let Some(response) = handle(conn.as_ref(), &request) {
            if write_message(&mut out, &response).is_err() {
                break;
            }
        }
    }

    0
}

fn write_message(out: &mut impl Write, message: &Value) -> std::io::Result<()> {
    serde_json::to_writer(&mut *out, message)?;
    out.write_all(b"\n")?;
    out.flush()
}

/// Dispatch one request. Returns `None` for notifications, which take no reply.
fn handle(conn: Option<&Connection>, request: &Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");

    match method {
        "initialize" => Some(result(id, initialize_result(request))),
        "ping" => Some(result(id, json!({}))),
        "tools/list" => Some(result(id, json!({ "tools": tools::list() }))),
        "tools/call" => Some(call_tool(conn, id, request)),
        other => {
            // Any other notification is acknowledged by ignoring it.
            if request.get("id").is_none() || other.starts_with("notifications/") {
                None
            } else {
                Some(error_response(
                    id,
                    -32601,
                    &format!("method not found: {other}"),
                ))
            }
        }
    }
}

fn initialize_result(request: &Value) -> Value {
    // Echo the client's protocol version so we work with whatever it asked for.
    let version = request
        .get("params")
        .and_then(|params| params.get("protocolVersion"))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_PROTOCOL_VERSION);

    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "ledgerly", "version": env!("CARGO_PKG_VERSION") },
        "instructions": "Ledgerly is a local, offline invoicing app. Its data lives on this \
                         machine only, and this server never uses the network.",
    })
}

fn call_tool(conn: Option<&Connection>, id: Value, request: &Value) -> Value {
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let Some(tool) = tools::find(name) else {
        return tool_error(id, &format!("unknown tool: {name}"));
    };

    let Some(conn) = conn else {
        return tool_error(
            id,
            "Ledgerly's database could not be opened. Start Ledgerly once, then try again.",
        );
    };

    if let Err(error) = gate(conn, &tool) {
        return tool_error(id, &error.to_string());
    }

    match (tool.handler)(conn, &arguments) {
        Ok(value) => result(
            id,
            json!({ "content": [ { "type": "text", "text": value.to_string() } ] }),
        ),
        Err(error) => tool_error(id, &error.to_string()),
    }
}

/// The consent gate, checked before every call. Mirrors the app's own lock: if
/// the user has not switched AI access on, or the lock is engaged and the
/// passphrase from the client config does not verify, or a write is attempted
/// while changes are off, the call is refused.
fn gate(conn: &Connection, tool: &Tool) -> AppResult<()> {
    if !enabled(conn)? {
        return Err(AppError::Validation(
            "AI access is off in Ledgerly. Turn it on in Settings > AI.".to_string(),
        ));
    }

    if security::is_enabled(conn)? {
        let passphrase = std::env::var("LEDGERLY_MCP_PASSPHRASE").unwrap_or_default();
        if passphrase.is_empty() || !security::verify(conn, &passphrase)? {
            return Err(AppError::Validation(
                "Ledgerly is locked. Unlock to continue.".to_string(),
            ));
        }
    }

    if tool.write && !allow_write(conn)? {
        return Err(AppError::Validation(
            "Changes are off. Turn on 'Allow changes' in Settings > AI.".to_string(),
        ));
    }

    Ok(())
}

fn result(id: Value, value: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": value })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// A tool failure is a normal result with `isError`, per MCP, so the model can
/// read it and recover rather than seeing the transport fail.
fn tool_error(id: Value, message: &str) -> Value {
    result(
        id,
        json!({
            "content": [ { "type": "text", "text": message } ],
            "isError": true
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    fn call(conn: &Connection, name: &str, arguments: Value) -> Value {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments }
        });
        handle(Some(conn), &request).expect("a request with an id gets a response")
    }

    fn text(response: &Value) -> String {
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    fn is_error(response: &Value) -> bool {
        response["result"]["isError"].as_bool().unwrap_or(false)
    }

    fn enable(conn: &Connection, key: &str, value: &str) {
        db::settings::set(conn, key, value).unwrap();
    }

    #[test]
    fn initialize_reports_the_server() {
        let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": { "protocolVersion": "2024-11-05" } });
        let response = handle(None, &request).unwrap();
        assert_eq!(response["result"]["serverInfo"]["name"], "ledgerly");
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    }

    #[test]
    fn notifications_get_no_reply() {
        let request = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(handle(None, &request).is_none());
    }

    #[test]
    fn tools_are_listed() {
        let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
        let response = handle(None, &request).unwrap();
        assert!(response["result"]["tools"].as_array().unwrap().len() > 10);
    }

    #[test]
    fn ai_access_is_off_by_default() {
        let t = temp_db();
        let response = call(&t.conn, "ledgerly_clients_list", json!({}));
        assert!(is_error(&response));
        assert!(text(&response).contains("off"));
    }

    #[test]
    fn a_read_tool_works_once_enabled() {
        let t = temp_db();
        enable(&t.conn, ENABLED_KEY, "true");

        let response = call(&t.conn, "ledgerly_clients_list", json!({}));
        assert!(!is_error(&response), "{}", text(&response));
        assert_eq!(text(&response), "[]");
    }

    #[test]
    fn writes_need_the_allow_write_setting() {
        let t = temp_db();
        enable(&t.conn, ENABLED_KEY, "true");

        let response = call(
            &t.conn,
            "ledgerly_clients_create",
            json!({ "input": { "displayName": "Acme" } }),
        );
        assert!(is_error(&response));
        assert!(text(&response).contains("Changes are off"));
    }

    #[test]
    fn a_write_works_when_allowed() {
        let t = temp_db();
        enable(&t.conn, ENABLED_KEY, "true");
        enable(&t.conn, ALLOW_WRITE_KEY, "true");

        let response = call(
            &t.conn,
            "ledgerly_clients_create",
            json!({ "input": { "displayName": "Acme" } }),
        );
        assert!(!is_error(&response), "{}", text(&response));
        assert!(text(&response).contains("Acme"));
    }

    #[test]
    fn the_lock_refuses_calls_without_the_passphrase() {
        let t = temp_db();
        enable(&t.conn, ENABLED_KEY, "true");
        security::enable(&t.conn, "a decent passphrase", 15).unwrap();

        let response = call(&t.conn, "ledgerly_clients_list", json!({}));
        assert!(is_error(&response));
        assert!(text(&response).contains("locked"));
    }

    #[test]
    fn an_unknown_tool_is_a_readable_error() {
        let t = temp_db();
        enable(&t.conn, ENABLED_KEY, "true");

        let response = call(&t.conn, "ledgerly_does_not_exist", json!({}));
        assert!(is_error(&response));
        assert!(text(&response).contains("unknown tool"));
    }
}
