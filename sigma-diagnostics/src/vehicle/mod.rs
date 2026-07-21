//! Vehicle-facing diagnostics for shop tools (Mechanic) over SocketCAN.
//!
//! Protocol PDUs for maintenance/settings writes are defined with Wingman firmware;
//! this module ships stubs and M7 draft decode for live diagnosis.

mod ai;
mod diagnosis;
mod logs;
mod m7;
mod maintenance;
mod ota;
mod recording;
mod session;
mod settings;
mod transport;

pub use ai::{
    build_diagnosis_advisor, AiConfig, DiagnosisAdvisor, DiagnosisReading, ModelAdvisor,
    ReadingSeverity, ReadingSource, RuleBasedAdvisor,
};
pub use diagnosis::{AnomalyRow, DiagnosisSnapshot, VitalSignal};
pub use logs::{request_log_export, LogExportRequest};
pub use m7::{load_m7_draft_dbc, M7_DRAFT_DBC, M7_DRAFT_DBC_NAME};
pub use maintenance::{
    audit, default_records_dir, fetch_latest_schedule, perform_maintenance_audit,
    read_wingman_db, render_markdown, save_maintenance_record, AuditItem, ErrorRow, ItemStatus,
    MaintenanceAction, MaintenanceAudit, MaintenanceScheduleConfig, MaintenanceService,
    SavedRecord, StubMaintenanceService, WingmanData,
};
pub use ota::{fetch_channel_latest, ChannelRelease, OtaConfig};
pub use recording::{default_sessions_dir, new_session_path, TelemetryRecorder, TelemetryReplayer};
pub use session::{VehicleLinkConfig, VehicleSession, VehicleSessionStatus, DEFAULT_WIFI_PORT};
pub use settings::{SettingsService, StubSettingsService, VehicleSetting};
pub use transport::VehicleTransport;
