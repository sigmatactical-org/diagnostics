/// Shop-side configuration for fetching the prescribed maintenance schedule
/// from the updates service. The schedule is owned and distributed by that
/// service (like the OTA catalog), so the app never hard-codes intervals.
#[derive(Debug, Clone)]
pub struct MaintenanceScheduleConfig {
    pub base_url: String,
    pub model: String,
}

impl MaintenanceScheduleConfig {
    /// Read the schedule endpoint from env, sharing the updates-service base URL
    /// with the OTA/DBC clients.
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("SIGMA_UPDATES_URL")
                .unwrap_or_else(|_| "http://updates.sigma.localtest.me:30080".into())
                .trim_end_matches('/')
                .to_owned(),
            model: std::env::var("SIGMA_VEHICLE_MODEL")
                .unwrap_or_else(|_| "sigma-racer".into()),
        }
    }

    /// Endpoint for the model's latest maintenance schedule.
    pub fn latest_url(&self) -> String {
        format!("{}/v1/maintenance/{}/latest", self.base_url, self.model)
    }
}
