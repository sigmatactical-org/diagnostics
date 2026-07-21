//! Mechanic application state.

use crate::config::SessionConfig;
use can_viewer::AppState as CanViewerAppState;
use can_viewer::InitialFiles;
use parking_lot::Mutex;
use sigma_diagnostics::{MaintenanceAudit, VehicleSession};
use std::ops::Deref;
use std::sync::Arc;

/// Desktop Mechanic state: shared diagnostics domain + vehicle session + config.
pub struct AppState {
    /// Reuse can-viewer app state (DiagnosticsState + session helpers for analysis).
    pub analysis: Arc<CanViewerAppState>,
    pub vehicle: VehicleSession,
    pub mechanic_session: Mutex<SessionConfig>,
    /// Most recent maintenance audit, cached so "Save report" persists exactly
    /// what the tab shows without re-querying the bike.
    pub maintenance_audit: Mutex<Option<MaintenanceAudit>>,
}

impl AppState {
    /// Shared state seeded with command-line files.
    pub fn new(initial: InitialFiles) -> Self {
        let mechanic_session = SessionConfig::load();
        Self {
            analysis: Arc::new(CanViewerAppState::with_initial_files(initial)),
            vehicle: VehicleSession::new(),
            mechanic_session: Mutex::new(mechanic_session),
            maintenance_audit: Mutex::new(None),
        }
    }
}

impl Deref for AppState {
    type Target = CanViewerAppState;

    fn deref(&self) -> &Self::Target {
        &self.analysis
    }
}
