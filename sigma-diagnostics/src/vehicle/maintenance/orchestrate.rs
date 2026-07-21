//! The end-to-end maintenance audit: fetch the latest schedule from the updates
//! service, query the bike for its schedule version and log, and (only if the
//! versions match) compare them. A mismatch returns a blocked audit naming the
//! reason, so the shop knows the bike needs a schedule update first.

use std::time::{SystemTime, UNIX_EPOCH};

use sigma_racer_telemetry::pull_database;

use super::audit::{audit, MaintenanceAudit};
use super::fetch::fetch_latest_schedule;
use super::schedule_config::MaintenanceScheduleConfig;
use super::wingman_db::read_wingman_db;

/// Run the full audit against the bike reachable at `host:port` (the relay).
///
/// Steps 3–6 of the maintenance flow: fetch the latest schedule from the updates
/// service, pull the bike's whole database, block on schedule-version mismatch,
/// else compare the schedule against the bike's log. The pulled error history is
/// attached to the result for the record.
pub fn perform_maintenance_audit(
    schedule_cfg: &MaintenanceScheduleConfig,
    host: &str,
    port: u16,
) -> Result<MaintenanceAudit, String> {
    let schedule = fetch_latest_schedule(schedule_cfg)?;
    let bytes = pull_database(host, port)?;
    let data = read_wingman_db(&bytes)?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let mut result = audit(&schedule, &data.report, now_ms);
    result.errors = data.errors;
    Ok(result)
}
