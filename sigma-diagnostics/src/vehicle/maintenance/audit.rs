//! Compare a prescribed maintenance schedule against a bike's maintenance log.
//!
//! The audit runs only when the schedule versions match (the caller enforces the
//! block); a version mismatch produces a blocked audit that names the reason.

use serde::Serialize;
use sigma_racer_telemetry::protocol::now_iso;
use sigma_racer_telemetry::{
    parse_ts_millis, MaintenanceItem, MaintenanceReport, MaintenanceSchedule,
};

use super::wingman_db::ErrorRow;

const MS_PER_DAY: i64 = 86_400_000;
/// Fraction of an interval remaining at which an item is flagged "due soon".
const DUE_SOON_FRACTION: f64 = 0.1;

/// Per-item audit verdict, ordered so a worse status sorts first in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ItemStatus {
    /// Past its interval — service now.
    Overdue,
    /// No record of this service ever being performed.
    NeverDone,
    /// Within the final slice of its interval.
    DueSoon,
    /// Recently serviced, not yet due.
    Ok,
    /// No recurrence bound, or usage counters unavailable to judge it.
    Unknown,
}

impl ItemStatus {
    /// Short human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Overdue => "Overdue",
            Self::NeverDone => "Never done",
            Self::DueSoon => "Due soon",
            Self::Ok => "OK",
            Self::Unknown => "Unknown",
        }
    }

    /// Severity rank for combining multiple interval bounds (higher = worse).
    fn rank(self) -> u8 {
        match self {
            Self::Overdue | Self::NeverDone => 4,
            Self::DueSoon => 2,
            Self::Ok => 1,
            Self::Unknown => 0,
        }
    }
}

/// One row of the audit.
#[derive(Debug, Clone, Serialize)]
pub struct AuditItem {
    pub id: String,
    pub name: String,
    pub status: ItemStatus,
    /// Human-readable explanation ("Due at 12,000 km — 550 km remaining").
    pub detail: String,
    pub last_performed_at: Option<String>,
    pub last_odometer_km: Option<f64>,
}

/// The full audit outcome.
#[derive(Debug, Clone, Serialize)]
pub struct MaintenanceAudit {
    /// Latest schedule version from the updates service.
    pub schedule_version: String,
    /// Schedule version the bike is provisioned with.
    pub bike_schedule_version: String,
    pub versions_match: bool,
    pub odometer_km: f64,
    pub engine_hours: Option<f64>,
    /// RFC 3339 time the audit was generated.
    pub generated_at: String,
    /// Per-item verdicts (empty when blocked).
    pub items: Vec<AuditItem>,
    /// Recent error history read from the pulled database (newest first).
    #[serde(default)]
    pub errors: Vec<ErrorRow>,
    /// Set when the audit could not run (version mismatch).
    pub blocked_reason: Option<String>,
}

impl MaintenanceAudit {
    /// Worst status across all items, for a one-line headline.
    pub fn worst_status(&self) -> Option<ItemStatus> {
        self.items.iter().map(|i| i.status).max_by_key(|s| s.rank())
    }

    /// Count of items at a given status.
    pub fn count(&self, status: ItemStatus) -> usize {
        self.items.iter().filter(|i| i.status == status).count()
    }
}

/// Audit `report` against `schedule`. `now_ms` is the current wall clock in
/// milliseconds since the Unix epoch (injected for testability).
pub fn audit(
    schedule: &MaintenanceSchedule,
    report: &MaintenanceReport,
    now_ms: i64,
) -> MaintenanceAudit {
    let versions_match = schedule.version == report.schedule_version;
    let bike_version = if report.schedule_version.is_empty() {
        "(unprovisioned)".to_string()
    } else {
        report.schedule_version.clone()
    };

    if !versions_match {
        return MaintenanceAudit {
            schedule_version: schedule.version.clone(),
            bike_schedule_version: bike_version,
            versions_match: false,
            odometer_km: report.odometer_km,
            engine_hours: report.engine_hours,
            generated_at: now_iso(),
            items: Vec::new(),
            errors: Vec::new(),
            blocked_reason: Some(format!(
                "Bike is on maintenance schedule {}, latest is {}. Update the bike's schedule before auditing.",
                if report.schedule_version.is_empty() {
                    "(unprovisioned)"
                } else {
                    report.schedule_version.as_str()
                },
                schedule.version
            )),
        };
    }

    let items = schedule
        .items
        .iter()
        .map(|item| audit_item(item, report, now_ms))
        .collect();

    MaintenanceAudit {
        schedule_version: schedule.version.clone(),
        bike_schedule_version: bike_version,
        versions_match: true,
        odometer_km: report.odometer_km,
        engine_hours: report.engine_hours,
        generated_at: now_iso(),
        items,
        errors: Vec::new(),
        blocked_reason: None,
    }
}

/// Audit a single schedule item against the log's most recent matching service.
fn audit_item(item: &MaintenanceItem, report: &MaintenanceReport, now_ms: i64) -> AuditItem {
    // "Last done" = the highest-odometer entry for this item (odometer is monotonic).
    let last = report
        .logs
        .iter()
        .filter(|e| e.item_id == item.id)
        .max_by(|a, b| a.odometer_km.total_cmp(&b.odometer_km));

    let Some(last) = last else {
        let status = if item.interval.is_informational() {
            ItemStatus::Unknown
        } else {
            ItemStatus::NeverDone
        };
        return AuditItem {
            id: item.id.clone(),
            name: item.name.clone(),
            status,
            detail: "No record of this service.".into(),
            last_performed_at: None,
            last_odometer_km: None,
        };
    };

    let mut status = ItemStatus::Unknown;
    let mut details: Vec<String> = Vec::new();

    // Distance bound.
    if let Some(every_km) = item.interval.every_km {
        let due_km = last.odometer_km + every_km;
        let remaining = due_km - report.odometer_km;
        status = worse(status, threshold_status(remaining, every_km));
        details.push(format!(
            "Due at {:.0} km ({}).",
            due_km,
            remaining_phrase(remaining, "km")
        ));
    }

    // Calendar bound.
    if let Some(every_days) = item.interval.every_days {
        if let Some(last_ms) = parse_ts_millis(&last.performed_at) {
            let elapsed_days = (now_ms - last_ms) as f64 / MS_PER_DAY as f64;
            let remaining = every_days as f64 - elapsed_days;
            status = worse(status, threshold_status(remaining, every_days as f64));
            details.push(format!(
                "Due after {} days ({}).",
                every_days,
                remaining_phrase(remaining, "days")
            ));
        }
    }

    // Engine-hours bound (only when the bike reports hours).
    if let (Some(every_hours), Some(current_hours)) =
        (item.interval.every_engine_hours, report.engine_hours)
    {
        let base = last.engine_hours.unwrap_or(0.0);
        let due_hours = base + every_hours;
        let remaining = due_hours - current_hours;
        status = worse(status, threshold_status(remaining, every_hours));
        details.push(format!(
            "Due at {:.0} h ({}).",
            due_hours,
            remaining_phrase(remaining, "h")
        ));
    }

    let prefix = format!(
        "Last done at {:.0} km on {}. ",
        last.odometer_km,
        date_only(&last.performed_at)
    );

    AuditItem {
        id: item.id.clone(),
        name: item.name.clone(),
        status,
        detail: format!("{prefix}{}", details.join(" ")),
        last_performed_at: Some(last.performed_at.clone()),
        last_odometer_km: Some(last.odometer_km),
    }
}

/// Classify remaining headroom against an interval size.
fn threshold_status(remaining: f64, interval: f64) -> ItemStatus {
    if remaining <= 0.0 {
        ItemStatus::Overdue
    } else if interval > 0.0 && remaining <= interval * DUE_SOON_FRACTION {
        ItemStatus::DueSoon
    } else {
        ItemStatus::Ok
    }
}

/// Keep the worse of two statuses.
fn worse(a: ItemStatus, b: ItemStatus) -> ItemStatus {
    if b.rank() > a.rank() { b } else { a }
}

/// "550 km remaining" / "overdue by 120 km".
fn remaining_phrase(remaining: f64, unit: &str) -> String {
    if remaining < 0.0 {
        format!("overdue by {:.0} {unit}", -remaining)
    } else {
        format!("{remaining:.0} {unit} remaining")
    }
}

/// Trim an RFC 3339 timestamp to its date for display.
fn date_only(ts: &str) -> &str {
    ts.split('T').next().unwrap_or(ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigma_racer_telemetry::{
        MaintenanceInterval, MaintenanceItem, MaintenanceLogEntry, MaintenanceReport,
        MaintenanceSchedule,
    };

    fn schedule() -> MaintenanceSchedule {
        MaintenanceSchedule {
            version: "2026.1".into(),
            model: "sigma-racer".into(),
            published: String::new(),
            items: vec![
                MaintenanceItem {
                    id: "engine-oil".into(),
                    name: "Engine oil".into(),
                    interval: MaintenanceInterval {
                        every_km: Some(6_000.0),
                        ..Default::default()
                    },
                },
                MaintenanceItem {
                    id: "brake-fluid".into(),
                    name: "Brake fluid".into(),
                    interval: MaintenanceInterval {
                        every_days: Some(730),
                        ..Default::default()
                    },
                },
            ],
        }
    }

    #[test]
    fn blocks_on_version_mismatch() {
        let report = MaintenanceReport {
            schedule_version: "2025.4".into(),
            odometer_km: 10_000.0,
            engine_hours: None,
            logs: vec![],
        };
        let a = audit(&schedule(), &report, 0);
        assert!(!a.versions_match);
        assert!(a.blocked_reason.is_some());
        assert!(a.items.is_empty());
    }

    #[test]
    fn flags_overdue_never_done_and_ok() {
        // Oil done at 6,000 km; now at 12,500 km with a 6,000 km interval → due at
        // 12,000, so 500 km overdue. Brake fluid never recorded.
        let report = MaintenanceReport {
            schedule_version: "2026.1".into(),
            odometer_km: 12_500.0,
            engine_hours: None,
            logs: vec![MaintenanceLogEntry {
                item_id: "engine-oil".into(),
                performed_at: "2025-09-01T00:00:00.000Z".into(),
                odometer_km: 6_000.0,
                engine_hours: None,
                note: None,
            }],
        };
        let a = audit(&schedule(), &report, 0);
        assert!(a.versions_match);
        let oil = a.items.iter().find(|i| i.id == "engine-oil").unwrap();
        assert_eq!(oil.status, ItemStatus::Overdue);
        let brake = a.items.iter().find(|i| i.id == "brake-fluid").unwrap();
        assert_eq!(brake.status, ItemStatus::NeverDone);
        // Overdue and NeverDone are equally severe ("service now").
        assert!(matches!(
            a.worst_status(),
            Some(ItemStatus::Overdue | ItemStatus::NeverDone)
        ));
    }

    #[test]
    fn ok_when_recently_serviced() {
        let report = MaintenanceReport {
            schedule_version: "2026.1".into(),
            odometer_km: 6_500.0,
            engine_hours: None,
            logs: vec![MaintenanceLogEntry {
                item_id: "engine-oil".into(),
                performed_at: "2025-09-01T00:00:00.000Z".into(),
                odometer_km: 6_000.0,
                engine_hours: None,
                note: None,
            }],
        };
        let a = audit(&schedule(), &report, 0);
        let oil = a.items.iter().find(|i| i.id == "engine-oil").unwrap();
        assert_eq!(oil.status, ItemStatus::Ok);
    }
}
