use std::time::Duration;

use sigma_racer_telemetry::MaintenanceSchedule;

use super::MaintenanceScheduleConfig;

/// Fetch the latest prescribed maintenance schedule from the updates service.
pub fn fetch_latest_schedule(
    cfg: &MaintenanceScheduleConfig,
) -> Result<MaintenanceSchedule, String> {
    let body = ureq::get(&cfg.latest_url())
        .config()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .call()
        .map_err(|e| format!("Maintenance schedule fetch failed: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Maintenance schedule response: {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("Maintenance schedule JSON: {e}"))
}
