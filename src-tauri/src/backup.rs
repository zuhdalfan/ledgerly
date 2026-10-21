//! Backup and restore.
//!
//! A `.ledgerly` file is a ZIP holding the SQLite database plus a manifest with
//! the schema version, so an older backup can be restored and re-migrated and a
//! newer one is politely refused.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::db::settings;
use crate::error::{AppError, AppResult};

pub const FORMAT_VERSION: u32 = 1;
pub const EXTENSION: &str = "ledgerly";

const MANIFEST_NAME: &str = "manifest.json";
const DATABASE_NAME: &str = "db.sqlite";

const KEY_ENABLED: &str = "backup.enabled";
const KEY_FOLDER: &str = "backup.folder";
const KEY_KEEP: &str = "backup.keep";
const KEY_LAST_RUN: &str = "backup.lastRun";

fn archive_error(error: zip::result::ZipError) -> AppError {
    AppError::Archive(error.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format_version: u32,
    pub schema_version: i64,
    pub application: String,
    pub application_version: String,
    pub created_at: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    /// Seconds since the epoch; the UI formats it.
    pub modified_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    pub enabled: bool,
    pub folder: String,
    pub keep: i64,
    pub last_run: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettingsInput {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub folder: String,
    #[serde(default = "default_keep")]
    pub keep: i64,
}

fn default_keep() -> i64 {
    7
}

fn temp_sibling(path: &Path) -> AppResult<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let stem = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "ledgerly".to_string());
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    Ok(parent.join(format!(".{stem}.tmp-{nanos}")))
}

fn sidecar(database: &Path, suffix: &str) -> PathBuf {
    let mut name = database.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Consistent copy of the live database. `VACUUM INTO` checkpoints the WAL, so
/// the result is a single self-contained file.
pub fn snapshot_database(conn: &Connection, destination: &Path) -> AppResult<()> {
    remove_if_present(destination)?;
    conn.execute(
        "VACUUM INTO ?1",
        rusqlite::params![destination.to_string_lossy().as_ref()],
    )?;
    Ok(())
}

/// Write a `.ledgerly` archive containing the database and its manifest.
pub fn create_backup(conn: &Connection, destination: &Path) -> AppResult<Manifest> {
    let staged = temp_sibling(destination)?;
    let result = (|| -> AppResult<Manifest> {
        snapshot_database(conn, &staged)?;

        let schema_version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let created_at: String = conn.query_row("SELECT datetime('now')", [], |row| row.get(0))?;

        let manifest = Manifest {
            format_version: FORMAT_VERSION,
            schema_version,
            application: "Ledgerly".to_string(),
            application_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at,
            database: DATABASE_NAME.to_string(),
        };

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let mut writer = ZipWriter::new(File::create(destination)?);

        writer
            .start_file(MANIFEST_NAME, options)
            .map_err(archive_error)?;
        let json =
            serde_json::to_vec_pretty(&manifest).map_err(|e| AppError::Archive(e.to_string()))?;
        writer.write_all(&json)?;

        writer
            .start_file(DATABASE_NAME, options)
            .map_err(archive_error)?;
        io::copy(&mut File::open(&staged)?, &mut writer)?;

        writer.finish().map_err(archive_error)?;

        Ok(manifest)
    })();

    let _ = fs::remove_file(&staged);
    result
}

/// Read and validate a backup's manifest.
pub fn read_manifest(archive_path: &Path) -> AppResult<Manifest> {
    let mut archive = ZipArchive::new(File::open(archive_path)?).map_err(archive_error)?;

    let manifest = {
        let mut entry = archive.by_name(MANIFEST_NAME).map_err(|_| {
            AppError::Validation("This file is not a Ledgerly backup (no manifest).".to_string())
        })?;
        let mut text = String::new();
        entry.read_to_string(&mut text)?;
        serde_json::from_str::<Manifest>(&text)
            .map_err(|_| AppError::Validation("The backup manifest is unreadable.".to_string()))?
    };

    if manifest.format_version > FORMAT_VERSION {
        return Err(AppError::Validation(format!(
            "This backup was made by a newer version of Ledgerly (format {}).",
            manifest.format_version
        )));
    }

    let supported = crate::db::migrations::MIGRATIONS.len() as i64;
    if manifest.schema_version > supported {
        return Err(AppError::Validation(
            "This backup was made by a newer version of Ledgerly and cannot be restored here."
                .to_string(),
        ));
    }

    Ok(manifest)
}

/// Replace `database` with the copy inside the archive.
///
/// The caller must have closed any open connection first, and must reopen
/// afterwards.
pub fn restore_database(archive_path: &Path, database: &Path) -> AppResult<Manifest> {
    let manifest = read_manifest(archive_path)?;

    // Extract first, so a corrupt or partial archive never touches the real file.
    let staged = temp_sibling(database)?;
    let extract = (|| -> AppResult<()> {
        let mut archive = ZipArchive::new(File::open(archive_path)?).map_err(archive_error)?;
        let mut entry = archive.by_name(&manifest.database).map_err(|_| {
            AppError::Validation("The backup does not contain a database.".to_string())
        })?;
        let mut out = File::create(&staged)?;
        io::copy(&mut entry, &mut out)?;
        out.flush()?;
        Ok(())
    })();

    if let Err(error) = extract {
        let _ = fs::remove_file(&staged);
        return Err(error);
    }

    // `fs::rename` replaces the destination on both Unix and Windows.
    fs::rename(&staged, database)?;

    // The old WAL belongs to the old file and must not be replayed.
    remove_if_present(&sidecar(database, "-wal"))?;
    remove_if_present(&sidecar(database, "-shm"))?;

    Ok(manifest)
}

pub fn list_backups(folder: &Path) -> AppResult<Vec<BackupInfo>> {
    let mut backups = Vec::new();
    if !folder.exists() {
        return Ok(backups);
    }

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some(EXTENSION) {
            continue;
        }
        let metadata = entry.metadata()?;
        backups.push(BackupInfo {
            name: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: path.to_string_lossy().to_string(),
            size_bytes: metadata.len(),
            modified_at: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs() as i64),
        });
    }

    // Names are timestamped, so this is newest-first.
    backups.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(backups)
}

/// Delete all but the newest `keep` backups. Returns how many were removed.
pub fn prune(folder: &Path, keep: usize) -> AppResult<usize> {
    if keep == 0 {
        return Ok(0);
    }

    let backups = list_backups(folder)?;
    let mut removed = 0;

    for info in backups.into_iter().skip(keep) {
        if fs::remove_file(&info.path).is_ok() {
            removed += 1;
        }
    }

    Ok(removed)
}

pub fn default_folder(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

/// A timestamped file name inside `folder`, e.g. `ledgerly-20261009-103000.ledgerly`.
pub fn suggested_path(conn: &Connection, folder: &Path) -> rusqlite::Result<PathBuf> {
    let stamp: String = conn.query_row(
        "SELECT strftime('%Y%m%d-%H%M%S', 'now', 'localtime')",
        [],
        |row| row.get(0),
    )?;
    Ok(folder.join(format!("ledgerly-{stamp}.{EXTENSION}")))
}

pub fn load_settings(conn: &Connection, data_dir: &Path) -> rusqlite::Result<BackupSettings> {
    let folder = settings::get(conn, KEY_FOLDER)?
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_folder(data_dir).to_string_lossy().to_string());

    let keep = settings::get(conn, KEY_KEEP)?
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or_else(default_keep);

    Ok(BackupSettings {
        enabled: settings::get(conn, KEY_ENABLED)?.as_deref() == Some("1"),
        folder,
        keep: keep.clamp(1, 365),
        last_run: settings::get(conn, KEY_LAST_RUN)?,
    })
}

pub fn save_settings(
    conn: &Connection,
    data_dir: &Path,
    input: &BackupSettingsInput,
) -> rusqlite::Result<BackupSettings> {
    let folder = if input.folder.trim().is_empty() {
        default_folder(data_dir).to_string_lossy().to_string()
    } else {
        input.folder.trim().to_string()
    };

    settings::set(conn, KEY_ENABLED, if input.enabled { "1" } else { "0" })?;
    settings::set(conn, KEY_FOLDER, &folder)?;
    settings::set(conn, KEY_KEEP, &input.keep.clamp(1, 365).to_string())?;

    load_settings(conn, data_dir)
}

/// Take an automatic backup if one is due (enabled, and none yet today), then
/// rotate. Returns the file written, if any.
pub fn auto_backup(conn: &Connection, data_dir: &Path) -> AppResult<Option<PathBuf>> {
    let settings = load_settings(conn, data_dir)?;
    if !settings.enabled {
        return Ok(None);
    }

    let today: String = conn.query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))?;
    if settings
        .last_run
        .as_deref()
        .is_some_and(|last| last.starts_with(&today))
    {
        return Ok(None);
    }

    let folder = PathBuf::from(&settings.folder);
    fs::create_dir_all(&folder)?;

    let destination = suggested_path(conn, &folder)?;
    create_backup(conn, &destination)?;
    prune(&folder, settings.keep as usize)?;
    settings::set(conn, KEY_LAST_RUN, &today)?;

    Ok(Some(destination))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, clients, tests_support::temp_db};

    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn temp_dir(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ledgerly-{name}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn client(conn: &Connection, name: &str) {
        clients::create(
            conn,
            &clients::ClientInput {
                display_name: name.to_string(),
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
    }

    #[test]
    fn backup_records_the_schema_version() {
        let t = temp_db();
        let dir = temp_dir("backup-manifest");
        let path = dir.0.join("test.ledgerly");

        let manifest = create_backup(&t.conn, &path).unwrap();
        assert_eq!(manifest.format_version, FORMAT_VERSION);
        assert_eq!(
            manifest.schema_version,
            crate::db::migrations::MIGRATIONS.len() as i64
        );

        let read_back = read_manifest(&path).unwrap();
        assert_eq!(read_back.schema_version, manifest.schema_version);
        assert_eq!(read_back.database, DATABASE_NAME);
    }

    #[test]
    fn restore_brings_back_the_backed_up_data() {
        let dir = temp_dir("backup-roundtrip");
        let db_path = dir.0.join("ledgerly.db");
        let archive = dir.0.join("snapshot.ledgerly");

        let conn = db::open(&db_path).unwrap();
        client(&conn, "Before backup");
        create_backup(&conn, &archive).unwrap();

        // Change the live database after the backup.
        client(&conn, "After backup");
        assert_eq!(clients::list(&conn, None, false).unwrap().len(), 2);

        drop(conn); // restore requires the connection to be closed

        let manifest = restore_database(&archive, &db_path).unwrap();
        assert_eq!(
            manifest.schema_version,
            crate::db::migrations::MIGRATIONS.len() as i64
        );

        let reopened = db::open(&db_path).unwrap();
        let names: Vec<String> = clients::list(&reopened, None, false)
            .unwrap()
            .into_iter()
            .map(|client| client.display_name)
            .collect();

        assert_eq!(
            names,
            vec!["Before backup"],
            "restore must undo later changes"
        );
    }

    #[test]
    fn a_newer_backup_is_refused() {
        let dir = temp_dir("backup-newer");
        let archive = dir.0.join("future.ledgerly");

        // Hand-build an archive claiming a much newer schema.
        let options = SimpleFileOptions::default();
        let mut writer = ZipWriter::new(File::create(&archive).unwrap());
        writer.start_file(MANIFEST_NAME, options).unwrap();
        write!(
            writer,
            "{{\"formatVersion\":{},\"schemaVersion\":9999,\"application\":\"Ledgerly\",\"applicationVersion\":\"9.0.0\",\"createdAt\":\"2030-01-01\",\"database\":\"db.sqlite\"}}",
            FORMAT_VERSION
        )
        .unwrap();
        writer.start_file(DATABASE_NAME, options).unwrap();
        writer.write_all(b"not a database").unwrap();
        writer.finish().unwrap();

        let error = read_manifest(&archive).unwrap_err().to_string();
        assert!(error.contains("newer version"), "unexpected error: {error}");
    }

    #[test]
    fn a_foreign_zip_is_refused() {
        let dir = temp_dir("backup-foreign");
        let archive = dir.0.join("not-a-backup.ledgerly");

        let mut writer = ZipWriter::new(File::create(&archive).unwrap());
        writer
            .start_file("readme.txt", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"hello").unwrap();
        writer.finish().unwrap();

        let error = read_manifest(&archive).unwrap_err().to_string();
        assert!(
            error.contains("not a Ledgerly backup"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn prune_keeps_only_the_newest_backups() {
        let t = temp_db();
        let dir = temp_dir("backup-prune");

        for stamp in ["20260101-090000", "20260102-090000", "20260103-090000"] {
            create_backup(&t.conn, &dir.0.join(format!("ledgerly-{stamp}.ledgerly"))).unwrap();
        }

        assert_eq!(list_backups(&dir.0).unwrap().len(), 3);
        assert_eq!(
            list_backups(&dir.0).unwrap()[0].name,
            "ledgerly-20260103-090000.ledgerly"
        );

        assert_eq!(prune(&dir.0, 2).unwrap(), 1);
        let remaining = list_backups(&dir.0).unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining
            .iter()
            .all(|info| info.name != "ledgerly-20260101-090000.ledgerly"));
    }

    #[test]
    fn automatic_backups_run_once_a_day_and_rotate() {
        let t = temp_db();
        let dir = temp_dir("backup-auto");

        // Disabled by default.
        assert!(auto_backup(&t.conn, &dir.0).unwrap().is_none());

        save_settings(
            &t.conn,
            &dir.0,
            &BackupSettingsInput {
                enabled: true,
                folder: dir.0.to_string_lossy().to_string(),
                keep: 2,
            },
        )
        .unwrap();

        let first = auto_backup(&t.conn, &dir.0).unwrap();
        assert!(first.is_some(), "first run should write a backup");

        // Same day: nothing more.
        assert!(auto_backup(&t.conn, &dir.0).unwrap().is_none());
        assert_eq!(list_backups(&dir.0).unwrap().len(), 1);

        let settings = load_settings(&t.conn, &dir.0).unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.keep, 2);
        assert!(settings.last_run.is_some());
    }
}
