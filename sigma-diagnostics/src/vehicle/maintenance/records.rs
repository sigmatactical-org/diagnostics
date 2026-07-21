//! Persist maintenance reports to the mechanic's records directory.
//!
//! Records live alongside recorded telemetry sessions under the mechanic's
//! config dir. Each saved report is a Markdown document plus a JSON sidecar of
//! the structured audit, so records are both human- and machine-readable.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::audit::MaintenanceAudit;
use super::report_md::render_markdown;

/// Where saved maintenance reports live: `~/.config/sigma-racer-mechanic/records`.
pub fn default_records_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".config")
        .join("sigma-racer-mechanic")
        .join("records")
}

/// A saved record: the Markdown path and its JSON sidecar path.
#[derive(Debug, Clone)]
pub struct SavedRecord {
    pub markdown_path: PathBuf,
    pub json_path: PathBuf,
}

/// Render `audit` to Markdown and save it (plus a JSON sidecar) to the records
/// directory. Returns the written paths.
pub fn save_maintenance_record(audit: &MaintenanceAudit) -> Result<SavedRecord, String> {
    let dir = default_records_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("records dir {}: {e}", dir.display()))?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let markdown_path = dir.join(format!("maintenance-{stamp}.md"));
    let json_path = dir.join(format!("maintenance-{stamp}.json"));

    let md = render_markdown(audit);
    fs::write(&markdown_path, md).map_err(|e| format!("write {}: {e}", markdown_path.display()))?;

    let json = serde_json::to_string_pretty(audit).map_err(|e| e.to_string())?;
    fs::write(&json_path, json + "\n")
        .map_err(|e| format!("write {}: {e}", json_path.display()))?;

    Ok(SavedRecord {
        markdown_path,
        json_path,
    })
}
