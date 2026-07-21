//! Mechanic Slint controllers.

mod ai;
mod analysis;
mod vehicle;

use crate::state::AppState;
use crate::SigmaRacerMechanic;
use slint::{ComponentHandle, Timer, TimerMode};
use std::rc::Rc;
use std::sync::Arc;

/// Run the Mechanic window (vehicle tabs + embedded analysis tabs).
pub fn run(state: Arc<AppState>) -> Result<(), slint::PlatformError> {
    let ui = SigmaRacerMechanic::new()?;
    ui.set_version_text(env!("CARGO_PKG_VERSION").into());
    ui.set_status_text("Ready".into());

    let analysis = Rc::new(analysis::AnalysisController::new(
        state.clone(),
        ui.as_weak(),
    ));
    analysis::AnalysisController::wire(analysis.clone(), &ui);
    analysis.populate_about();
    analysis.load_initial();

    let vehicle = Rc::new(vehicle::VehicleController::new(state.clone(), ui.as_weak()));
    vehicle::VehicleController::wire(vehicle.clone(), &ui);
    vehicle.refresh_interfaces();
    vehicle.refresh_settings();
    vehicle.init_ota_labels();
    vehicle.init_maintenance();

    let ai = Rc::new(ai::AiController::new(state.clone(), ui.as_weak()));
    ai::AiController::wire(ai.clone(), &ui);
    ai.init();

    let ui_weak = ui.as_weak();
    let vehicle_poll = vehicle.clone();
    let analysis_poll = analysis.clone();
    let sync_timer = Timer::default();
    sync_timer.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(200),
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            vehicle_poll.poll_diagnosis_into(&ui);
            analysis_poll.poll_live_into(&ui);
        },
    );
    std::mem::forget(sync_timer);

    ui.run()
}
