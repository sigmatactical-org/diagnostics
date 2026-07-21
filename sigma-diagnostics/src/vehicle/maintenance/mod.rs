//! Maintenance: reset actions (protocol pending with Wingman), the prescribed
//! schedule fetched from the updates service, and the audit that compares that
//! schedule against the bike's maintenance log.

mod audit;
mod fetch;
mod maintenance_action;
mod maintenance_service;
mod orchestrate;
mod records;
mod report_md;
mod schedule_config;
mod stub_maintenance_service;
mod wingman_db;

pub use audit::{audit, AuditItem, ItemStatus, MaintenanceAudit};
pub use fetch::fetch_latest_schedule;
pub use maintenance_action::MaintenanceAction;
pub use maintenance_service::MaintenanceService;
pub use orchestrate::perform_maintenance_audit;
pub use records::{default_records_dir, save_maintenance_record, SavedRecord};
pub use report_md::render_markdown;
pub use schedule_config::MaintenanceScheduleConfig;
pub use stub_maintenance_service::StubMaintenanceService;
pub use wingman_db::{read_wingman_db, ErrorRow, WingmanData};
