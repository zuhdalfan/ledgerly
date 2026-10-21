//! File export: reports to CSV and the whole database to JSON.
//!
//! Everything is written locally; nothing leaves the machine.

use rusqlite::types::ValueRef;
use rusqlite::Connection;
use serde_json::{json, Map, Value};
use std::fs;

use crate::error::AppResult;

/// Every table, in dependency order, for a full data export.
const TABLES: &[&str] = &[
    "setting",
    "business",
    "client",
    "catalog_item",
    "tax_rate",
    "numbering",
    "invoice",
    "invoice_item",
    "payment",
    "estimate",
    "estimate_item",
];

/// Quote a cell only when it needs it, doubling embedded quotes.
fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Write a report as CSV. A BOM is included so Excel detects UTF-8.
pub fn write_csv(path: &str, headers: &[String], rows: &[Vec<String>]) -> AppResult<()> {
    let mut out = String::from('\u{FEFF}');

    out.push_str(
        &headers
            .iter()
            .map(|header| csv_cell(header))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push_str("\r\n");

    for row in rows {
        out.push_str(
            &row.iter()
                .map(|cell| csv_cell(cell))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push_str("\r\n");
    }

    fs::write(path, out)?;
    Ok(())
}

fn dump_table(conn: &Connection, table: &str) -> rusqlite::Result<Vec<Value>> {
    let mut stmt = conn.prepare(&format!("SELECT * FROM {table}"))?;
    let names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect();

    let mut rows = stmt.query([])?;
    let mut out = Vec::new();

    while let Some(row) = rows.next()? {
        let mut object = Map::new();
        for (index, name) in names.iter().enumerate() {
            let value = match row.get_ref(index)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(number) => json!(number),
                ValueRef::Real(number) => json!(number),
                ValueRef::Text(bytes) => Value::String(String::from_utf8_lossy(bytes).into_owned()),
                ValueRef::Blob(_) => Value::Null,
            };
            object.insert(name.clone(), value);
        }
        out.push(Value::Object(object));
    }

    Ok(out)
}

/// Export every table as JSON — the portable, tool-readable copy of the database.
pub fn export_json(conn: &Connection, path: &str) -> AppResult<()> {
    let mut tables = Map::new();
    for table in TABLES {
        tables.insert((*table).to_string(), Value::Array(dump_table(conn, table)?));
    }

    let exported_at: String = conn.query_row("SELECT datetime('now')", [], |row| row.get(0))?;

    let document = json!({
        "application": "Ledgerly",
        "applicationVersion": env!("CARGO_PKG_VERSION"),
        "exportedAt": exported_at,
        "tables": tables,
    });

    let text = serde_json::to_string_pretty(&document)
        .map_err(|error| crate::error::AppError::Validation(error.to_string()))?;
    fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::clients;
    use crate::db::tests_support::temp_db;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("ledgerly-{name}-{nanos}"))
    }

    #[test]
    fn csv_quotes_only_when_needed() {
        assert_eq!(csv_cell("plain"), "plain");
        assert_eq!(csv_cell("has,comma"), "\"has,comma\"");
        assert_eq!(csv_cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_cell("two\nlines"), "\"two\nlines\"");
    }

    #[test]
    fn writes_a_csv_file_with_a_bom_and_crlf() {
        let path = temp_path("report.csv");
        write_csv(
            path.to_str().unwrap(),
            &["Period".to_string(), "Amount".to_string()],
            &[
                vec!["2026-01".to_string(), "1000.00".to_string()],
                vec!["2026-02".to_string(), "1,250.50".to_string()],
            ],
        )
        .unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert!(text.starts_with('\u{FEFF}'));
        assert!(text.contains("Period,Amount\r\n"));
        assert!(text.contains("\"1,250.50\""));
    }

    #[test]
    fn exports_every_table_as_json() {
        let t = temp_db();
        clients::create(
            &t.conn,
            &clients::ClientInput {
                display_name: "Acme".to_string(),
                company: None,
                contact_person: None,
                email: None,
                phone: None,
                address_line1: None,
                address_line2: None,
                city: None,
                region: None,
                postal_code: None,
                country: None,
                tax_id: None,
                currency: None,
                notes: None,
                locale: None,
            },
        )
        .unwrap();

        let path = temp_path("export.json");
        export_json(&t.conn, path.to_str().unwrap()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        let document: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(document["application"], "Ledgerly");

        let tables = &document["tables"];
        for table in TABLES {
            assert!(tables.get(*table).is_some(), "missing table {table}");
        }

        let clients_json = tables["client"].as_array().unwrap();
        assert_eq!(clients_json.len(), 1);
        assert_eq!(clients_json[0]["display_name"], "Acme");
        // Empty columns survive as nulls.
        assert!(clients_json[0]["email"].is_null());
    }
}
