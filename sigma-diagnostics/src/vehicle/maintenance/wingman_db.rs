//! Read a SQLite database pulled whole from the bike.
//!
//! The bike ships its entire database (see `pull_database`); the shop tool opens
//! it locally and reads whatever it needs — the maintenance log + metadata for
//! the audit, and the error history for the record. New on-bike tables become
//! readable here with no protocol change.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use sigma_racer_telemetry::{MaintenanceLogEntry, MaintenanceReport};

/// One error-history row read back from the bike's database.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ErrorRow {
    pub ts: String,
    pub event: String,
    pub edge: String,
    pub severity: Option<String>,
    pub message: String,
    pub odometer_km: Option<f64>,
}

/// The datasets the shop tool reads out of a pulled database.
#[derive(Debug, Clone)]
pub struct WingmanData {
    pub report: MaintenanceReport,
    pub errors: Vec<ErrorRow>,
}

/// The magic header of every SQLite file.
const SQLITE_MAGIC: &[u8] = b"SQLite format 3\0";
/// Cap on error-history rows pulled into a report.
const ERROR_LIMIT: u32 = 200;

/// Parse a pulled database blob into the report (for the audit) and recent
/// error history (for the record). An empty blob means an unprovisioned bike.
pub fn read_wingman_db(bytes: &[u8]) -> Result<WingmanData, String> {
    if bytes.is_empty() {
        return Ok(WingmanData {
            report: unprovisioned(),
            errors: Vec::new(),
        });
    }
    if !bytes.starts_with(SQLITE_MAGIC) {
        // The relay sends a small JSON error line when the pull fails.
        return Err(format!(
            "bike returned no database: {}",
            String::from_utf8_lossy(&bytes[..bytes.len().min(200)]).trim()
        ));
    }

    // rusqlite opens by path, so stage the blob in a temp file.
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("wingman-pull-{stamp}.db"));
    fs::write(&path, bytes).map_err(|e| format!("stage pulled db: {e}"))?;
    let result = read_open(&path);
    let _ = fs::remove_file(&path);
    result
}

fn read_open(path: &std::path::Path) -> Result<WingmanData, String> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("open pulled db: {e}"))?;

    let schedule_version = meta_get(&conn, "schedule_version")?.unwrap_or_default();
    let odometer_km = meta_get(&conn, "odometer_km")?
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let engine_hours = meta_get(&conn, "engine_hours")?.and_then(|s| s.parse::<f64>().ok());

    let logs = read_logs(&conn)?;
    let errors = read_errors(&conn)?;

    Ok(WingmanData {
        report: MaintenanceReport {
            schedule_version,
            odometer_km,
            engine_hours,
            logs,
        },
        errors,
    })
}

fn read_logs(conn: &Connection) -> Result<Vec<MaintenanceLogEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT item_id, performed_at, odometer_km, engine_hours, note \
             FROM maintenance_log ORDER BY odometer_km ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
    let logs = stmt
        .query_map([], |r| {
            Ok(MaintenanceLogEntry {
                item_id: r.get(0)?,
                performed_at: r.get(1)?,
                odometer_km: r.get(2)?,
                engine_hours: r.get(3)?,
                note: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(logs)
}

fn read_errors(conn: &Connection) -> Result<Vec<ErrorRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT ts, event, edge, severity, message, odometer_km \
             FROM error_history ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let errors = stmt
        .query_map([ERROR_LIMIT], |r| {
            Ok(ErrorRow {
                ts: r.get(0)?,
                event: r.get(1)?,
                edge: r.get(2)?,
                severity: r.get(3)?,
                message: r.get(4)?,
                odometer_km: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(errors)
}

fn meta_get(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
        .optional()
        .map_err(|e| format!("read meta {key}: {e}"))
}

/// The report for an empty/unprovisioned pull.
fn unprovisioned() -> MaintenanceReport {
    MaintenanceReport {
        schedule_version: String::new(),
        odometer_km: 0.0,
        engine_hours: None,
        logs: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a database like the bike's, then return its raw bytes.
    fn make_db_bytes() -> Vec<u8> {
        let path = std::env::temp_dir().join(format!("wingman-mk-{}.db", std::process::id()));
        let _ = fs::remove_file(&path);
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE maintenance_log (id INTEGER PRIMARY KEY, item_id TEXT, performed_at TEXT, odometer_km REAL, engine_hours REAL, note TEXT);
                 CREATE TABLE error_history (id INTEGER PRIMARY KEY, ts TEXT, event TEXT, edge TEXT, severity TEXT, vss TEXT, message TEXT, odometer_km REAL);
                 INSERT INTO meta VALUES ('schedule_version','2026.1'),('odometer_km','12345'),('engine_hours','310.5');
                 INSERT INTO maintenance_log (item_id, performed_at, odometer_km, engine_hours, note) VALUES ('engine-oil','2025-09-01T00:00:00.000Z',6000,150,'shop A');
                 INSERT INTO error_history (ts, event, edge, severity, vss, message, odometer_km) VALUES ('2026-02-01T00:00:00.000Z','coolant_overheat','raised','CRITICAL','Vehicle.OBD.CoolantTemperature','Coolant 118 C',9000);",
            )
            .unwrap();
        }
        let bytes = fs::read(&path).unwrap();
        let _ = fs::remove_file(&path);
        bytes
    }

    #[test]
    fn reads_report_and_errors() {
        let data = read_wingman_db(&make_db_bytes()).unwrap();
        assert_eq!(data.report.schedule_version, "2026.1");
        assert_eq!(data.report.odometer_km, 12_345.0);
        assert_eq!(data.report.engine_hours, Some(310.5));
        assert_eq!(data.report.logs.len(), 1);
        assert_eq!(data.errors.len(), 1);
        assert_eq!(data.errors[0].event, "coolant_overheat");
        assert_eq!(data.errors[0].severity.as_deref(), Some("CRITICAL"));
    }

    #[test]
    fn empty_pull_is_unprovisioned() {
        let data = read_wingman_db(&[]).unwrap();
        assert!(data.report.schedule_version.is_empty());
        assert!(data.report.logs.is_empty());
        assert!(data.errors.is_empty());
    }

    #[test]
    fn non_sqlite_bytes_error() {
        let err = read_wingman_db(b"{\"error\":\"boom\"}").unwrap_err();
        assert!(err.contains("boom"), "got: {err}");
    }
}
