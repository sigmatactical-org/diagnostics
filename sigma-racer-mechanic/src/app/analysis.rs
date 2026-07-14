//! Analysis tabs (MDF4 / Live / DBC / About) for Mechanic — uses shared AppState/services.

use crate::state::AppState;
use crate::{
    DbcAttributeRow, DbcCatalogRow, DbcMessageRow, DbcNodeRow, DbcSignalRow, DbcValueDescRow,
    FrameRow, LiveErrorRow, LiveFrameRow, LiveMessageRow, LiveSignalRow, SigmaRacerMechanic,
    SignalRow,
};
use can_viewer::dto::CanFrameDto;
use can_viewer::services::{
    decode_single_frame, export_logs, filter_frames, get_dbc_info, get_dbc_path,
    is_capture_running, list_can_interfaces, load_dbc, load_mdf4, start_capture, stop_capture,
    CaptureSession, FilterConfig, MatchStatus,
};
use parking_lot::Mutex;
use slint::{Model, ModelRc, SharedString, VecModel, Weak};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

/// Analysis tabs (MDF4 / Live / DBC / About) reusing can-viewer services.
pub struct AnalysisController {
    state: Arc<AppState>,
    ui: Weak<SigmaRacerMechanic>,
    all_frames: Mutex<Vec<CanFrameDto>>,
    filtered_frames: Mutex<Vec<CanFrameDto>>,
    live_capture: Mutex<Option<CaptureSession>>,
}

impl AnalysisController {
    /// Controller bound to the shared state and UI handle.
    pub fn new(state: Arc<AppState>, ui: Weak<SigmaRacerMechanic>) -> Self {
        Self {
            state,
            ui,
            all_frames: Mutex::new(Vec::new()),
            filtered_frames: Mutex::new(Vec::new()),
            live_capture: Mutex::new(None),
        }
    }

    /// Hook the analysis tab callbacks.
    pub fn wire(self: Rc<Self>, ui: &SigmaRacerMechanic) {
        ui.on_open_mdf4({
            let t = self.clone();
            move || {
                let t = t.clone();
                slint::Timer::single_shot(std::time::Duration::from_millis(1), move || {
                    if let Some(path) = Self::pick_open("Open MDF4", &[("MDF4", &["mf4", "mdf"])]) {
                        t.load_mdf4_path(&path);
                    }
                });
            }
        });
        ui.on_mdf4_open_dbc({
            let t = self.clone();
            move || {
                let t = t.clone();
                slint::Timer::single_shot(std::time::Duration::from_millis(1), move || {
                    if let Some(path) = Self::pick_open("Open DBC", &[("DBC", &["dbc"])]) {
                        match load_dbc(&path, &t.state.analysis) {
                            Ok(_) => {
                                t.refresh_dbc_status();
                                t.refresh_dbc_ui();
                                t.apply_filters();
                                t.with_ui(|ui| ui.set_status_text(format!("Loaded {path}").into()));
                            }
                            Err(e) => t.with_ui(|ui| ui.set_status_text(e.into())),
                        }
                    }
                });
            }
        });
        ui.on_export_mdf4({
            let t = self.clone();
            move || {
                let frames = t.filtered_frames.lock().clone();
                let ui = t.ui.clone();
                slint::Timer::single_shot(std::time::Duration::from_millis(1), move || {
                    if let Some(path) = Self::pick_save("Export MDF4", &[("MDF4", &["mf4"])]) {
                        match export_logs(&path, &frames) {
                            Ok(n) => {
                                if let Some(ui) = ui.upgrade() {
                                    ui.set_status_text(format!("Exported {n} frames").into());
                                }
                            }
                            Err(e) => {
                                if let Some(ui) = ui.upgrade() {
                                    ui.set_status_text(e.into());
                                }
                            }
                        }
                    }
                });
            }
        });
        ui.on_apply_filters({
            let t = self.clone();
            move || t.apply_filters()
        });
        ui.on_clear_filters({
            let t = self.clone();
            move || t.clear_filters()
        });
        ui.on_frame_selected({
            let t = self.clone();
            move |i| t.select_frame(i)
        });
        ui.on_refresh_interfaces_live({
            let t = self.clone();
            move || t.refresh_live_interfaces()
        });
        ui.on_start_capture({
            let t = self.clone();
            move || t.start_live()
        });
        ui.on_stop_capture({
            let t = self.clone();
            move || t.stop_live()
        });
        ui.on_export_capture({
            let t = self.clone();
            move || {
                t.with_ui(|ui| ui.set_status_text("Use MDF4 export after stopping capture".into()))
            }
        });
        ui.on_open_dbc_catalog({
            let t = self.clone();
            move || {
                t.with_ui(|ui| {
                    ui.set_dbc_catalog_open(true);
                    ui.set_dbc_catalog_status(
                        "Use Catalog from DBC tab — fetch via updates URL".into(),
                    );
                })
            }
        });
        ui.on_dbc_catalog_close({
            let t = self.clone();
            move || t.with_ui(|ui| ui.set_dbc_catalog_open(false))
        });
        ui.on_new_dbc({
            let t = self.clone();
            move || {
                let _ = can_viewer::services::clear_dbc(&t.state.analysis);
                t.refresh_dbc_ui();
                t.with_ui(|ui| ui.set_status_text("New DBC (in-memory cleared)".into()));
            }
        });
        ui.on_save_dbc({
            let t = self.clone();
            move || t.save_dbc()
        });
        ui.on_save_dbc_as({
            let t = self.clone();
            move || {
                let t = t.clone();
                slint::Timer::single_shot(std::time::Duration::from_millis(1), move || {
                    if let Some(path) = Self::pick_save("Save DBC", &[("DBC", &["dbc"])]) {
                        t.save_dbc_to(&path);
                    }
                });
            }
        });
        ui.on_begin_edit({
            let t = self.clone();
            move || t.with_ui(|ui| ui.set_dbc_editing(true))
        });
        ui.on_message_selected({
            let t = self.clone();
            move |i| t.select_dbc_message(i)
        });
    }

    fn with_ui<F: FnOnce(&SigmaRacerMechanic)>(&self, f: F) {
        if let Some(ui) = self.ui.upgrade() {
            f(&ui);
        }
    }

    /// Fill the About tab from build-time generated metadata.
    pub fn populate_about(&self) {
        self.with_ui(crate::about::populate);
    }

    /// Load files passed on the command line, if any.
    pub fn load_initial(&self) {
        self.refresh_live_interfaces();
        if let Some(path) = self.state.analysis.initial_files.lock().dbc_path.clone() {
            if let Err(e) = load_dbc(&path, &self.state.analysis) {
                log::warn!("Startup DBC load failed ({path}): {e}");
            }
        }
        self.refresh_dbc_status();
        self.refresh_dbc_ui();
        if let Some(path) = self.state.analysis.initial_files.lock().mdf4_path.clone() {
            self.load_mdf4_path(&path);
        }
    }

    fn pick_open(title: &str, exts: &[(&str, &[&str])]) -> Option<String> {
        let mut d = rfd::FileDialog::new().set_title(title);
        for (n, e) in exts {
            d = d.add_filter(*n, e);
        }
        d.pick_file().map(|p| p.display().to_string())
    }

    fn pick_save(title: &str, exts: &[(&str, &[&str])]) -> Option<String> {
        let mut d = rfd::FileDialog::new().set_title(title);
        for (n, e) in exts {
            d = d.add_filter(*n, e);
        }
        d.save_file().map(|p| p.display().to_string())
    }

    fn load_mdf4_path(&self, path: &str) {
        match load_mdf4(path, &self.state.analysis) {
            Ok((frames, _)) => {
                *self.all_frames.lock() = frames;
                self.with_ui(|ui| {
                    ui.set_mdf4_path(path.into());
                    let name = Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);
                    ui.set_mdf4_file_name(name.into());
                });
                self.apply_filters();
            }
            Err(e) => self.with_ui(|ui| ui.set_status_text(e.into())),
        }
    }

    fn refresh_dbc_status(&self) {
        self.with_ui(|ui| {
            if let Some(path) = get_dbc_path(&self.state.analysis) {
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path);
                ui.set_mdf4_dbc_status(name.into());
                ui.set_mdf4_dbc_file_name(name.into());
                ui.set_dbc_path(name.into());
            } else {
                ui.set_mdf4_dbc_status("No DBC".into());
                ui.set_mdf4_dbc_file_name("".into());
            }
        });
    }

    fn apply_filters(&self) {
        let all = self.all_frames.lock().clone();
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let mut filters = FilterConfig::default();
        let can_ids = parse_can_ids(&ui.get_mdf4_filter_can_id());
        if !can_ids.is_empty() {
            filters.can_ids = can_ids;
        }
        let message = ui.get_mdf4_filter_message().to_string();
        if !message.is_empty() {
            filters.messages = vec![message];
        }
        let data = ui.get_mdf4_filter_data().to_string();
        if !data.is_empty() {
            filters.data_pattern = Some(data);
        }
        filters.match_status = MatchStatus::All;
        let result = filter_frames(all, filters, &self.state.analysis);
        *self.filtered_frames.lock() = result.frames.clone();
        let rows: Vec<FrameRow> = result
            .frames
            .iter()
            .enumerate()
            .map(|(i, f)| FrameRow {
                index: i as i32,
                timestamp: format!("{:.3}", f.timestamp).into(),
                can_id: format_can_id(f.can_id, f.is_extended).into(),
                channel: f.channel.clone().into(),
                dlc: f.dlc.to_string().into(),
                data_hex: format_data_hex(&f.data).into(),
                message_name: "".into(),
            })
            .collect();
        ui.set_mdf4_frames(ModelRc::new(VecModel::from(rows)));
        ui.set_mdf4_frame_count(
            format!(
                "Loaded {} / {} frames",
                result.filtered_count, result.total_count
            )
            .into(),
        );
        ui.set_mdf4_selected_frame(-1);
        ui.set_mdf4_signals(ModelRc::new(VecModel::default()));
    }

    fn clear_filters(&self) {
        self.with_ui(|ui| {
            ui.set_mdf4_filter_can_id("".into());
            ui.set_mdf4_filter_message("".into());
            ui.set_mdf4_filter_data("".into());
        });
        self.apply_filters();
    }

    fn select_frame(&self, index: i32) {
        let frames = self.filtered_frames.lock();
        let Some(frame) = frames.get(index as usize) else {
            return;
        };
        let decoded = decode_single_frame(frame, &self.state.analysis.diag);
        let rows: Vec<SignalRow> = decoded
            .signals
            .into_iter()
            .map(|s| SignalRow {
                signal_name: s.signal_name.into(),
                value: format!("{:.4}", s.value).into(),
                unit: s.unit.into(),
                raw_value: s.raw_value.to_string().into(),
                description: s.description.unwrap_or_default().into(),
            })
            .collect();
        self.with_ui(|ui| {
            ui.set_mdf4_selected_frame(index);
            ui.set_mdf4_signals(ModelRc::new(VecModel::from(rows)));
        });
    }

    fn refresh_live_interfaces(&self) {
        let list = list_can_interfaces().unwrap_or_default();
        self.with_ui(|ui| {
            ui.set_live_interfaces(ModelRc::new(VecModel::from(
                list.iter()
                    .map(|s| SharedString::from(s.as_str()))
                    .collect::<Vec<_>>(),
            )));
        });
    }

    fn start_live(&self) {
        // Prefer not to clash with vehicle session capture.
        if self.state.vehicle.is_connected(&self.state.analysis.diag) {
            self.with_ui(|ui| {
                ui.set_status_text("Disconnect vehicle link before Live capture".into());
            });
            return;
        }
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let ifaces = ui.get_live_interfaces();
        let idx = ui.get_live_selected_interface() as usize;
        let iface = ifaces
            .row_data(idx)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "can0".into());
        let log = std::env::temp_dir().join("mechanic-live.mf4");
        match start_capture(
            &iface,
            &log.to_string_lossy(),
            false,
            None,
            &self.state.analysis,
        ) {
            Ok(session) => {
                *self.live_capture.lock() = Some(session);
                ui.set_live_capturing(true);
                ui.set_live_capture_status(format!("Capturing on {iface}").into());
            }
            Err(e) => ui.set_status_text(e.into()),
        }
    }

    fn stop_live(&self) {
        let _ = stop_capture(&self.state.analysis);
        *self.live_capture.lock() = None;
        self.with_ui(|ui| {
            ui.set_live_capturing(false);
            ui.set_live_capture_status("Stopped".into());
        });
    }

    /// Pull pending live-capture updates into the UI (called on a timer).
    pub fn poll_live_into(&self, ui: &SigmaRacerMechanic) {
        if !ui.get_live_capturing() {
            return;
        }
        if !is_capture_running(&self.state.analysis) {
            ui.set_live_capturing(false);
            ui.set_live_capture_status("Stopped".into());
            return;
        }
        let guard = self.live_capture.lock();
        let Some(session) = guard.as_ref() else {
            return;
        };
        let Some(display) = session.poll_update() else {
            return;
        };
        ui.set_live_stats_text(
            format!(
                "{} frames · {:.0}/s",
                display.stats.frame_count, display.stats.frame_rate
            )
            .into(),
        );
        can_viewer::apply_live_rows!(ui, &display);
    }

    fn refresh_dbc_ui(&self) {
        let Ok(Some(info)) = get_dbc_info(&self.state.analysis) else {
            self.with_ui(|ui| {
                ui.set_dbc_messages(ModelRc::new(VecModel::default()));
                ui.set_dbc_nodes(ModelRc::new(VecModel::default()));
            });
            return;
        };
        let messages: Vec<DbcMessageRow> = info
            .messages
            .iter()
            .map(|m| DbcMessageRow {
                id: format!("0x{:X}", m.id).into(),
                name: m.name.clone().into(),
                dlc: m.dlc.to_string().into(),
                sender: m.sender.clone().into(),
                signal_count: m.signals.len().to_string().into(),
            })
            .collect();
        let nodes: Vec<DbcNodeRow> = info
            .nodes
            .iter()
            .map(|n| DbcNodeRow {
                name: n.name.clone().into(),
                comment: n.comment.clone().unwrap_or_default().into(),
            })
            .collect();
        let attributes: Vec<DbcAttributeRow> = info
            .attribute_definitions
            .iter()
            .map(|a| DbcAttributeRow {
                name: a.name.clone().into(),
                object_type: a.object_type.clone().into(),
                value_summary: "".into(),
            })
            .collect();
        let value_descriptions: Vec<DbcValueDescRow> = info
            .value_descriptions
            .iter()
            .map(|vd| DbcValueDescRow {
                message_id: format!("0x{:X}", vd.message_id).into(),
                signal_name: vd.signal_name.clone().into(),
                entries: vd
                    .descriptions
                    .iter()
                    .map(|d| format!("{}=\"{}\"", d.value, d.description))
                    .collect::<Vec<_>>()
                    .join(", ")
                    .into(),
            })
            .collect();
        self.with_ui(|ui| {
            ui.set_dbc_version(info.version.clone().unwrap_or_default().into());
            ui.set_dbc_comment(info.comment.clone().unwrap_or_default().into());
            ui.set_dbc_messages(ModelRc::new(VecModel::from(messages)));
            ui.set_dbc_nodes(ModelRc::new(VecModel::from(nodes)));
            ui.set_dbc_attributes(ModelRc::new(VecModel::from(attributes)));
            ui.set_dbc_value_descriptions(ModelRc::new(VecModel::from(value_descriptions)));
            ui.set_dbc_dirty(false);
        });
        self.refresh_dbc_status();
    }

    fn select_dbc_message(&self, index: i32) {
        let Ok(Some(info)) = get_dbc_info(&self.state.analysis) else {
            return;
        };
        let Some(msg) = info.messages.get(index as usize) else {
            return;
        };
        let signals: Vec<DbcSignalRow> = msg
            .signals
            .iter()
            .map(|s| DbcSignalRow {
                name: s.name.clone().into(),
                start_bit: s.start_bit.to_string().into(),
                length: s.length.to_string().into(),
                byte_order: if s.byte_order.contains("big") {
                    "Motorola".into()
                } else {
                    "Intel".into()
                },
                factor: s.factor.to_string().into(),
                unit: s.unit.clone().into(),
            })
            .collect();
        self.with_ui(|ui| {
            ui.set_dbc_selected_message(index);
            ui.set_dbc_signals(ModelRc::new(VecModel::from(signals)));
            ui.set_dbc_edit_message_id(format!("0x{:X}", msg.id).into());
            ui.set_dbc_edit_message_name(msg.name.clone().into());
            ui.set_dbc_edit_message_dlc(msg.dlc.to_string().into());
            ui.set_dbc_edit_message_sender(msg.sender.clone().into());
        });
    }

    fn save_dbc(&self) {
        if let Some(path) = get_dbc_path(&self.state.analysis) {
            self.save_dbc_to(&path);
        } else {
            self.with_ui(|ui| {
                ui.set_status_text("No path — use Save As".into());
            });
        }
    }

    fn save_dbc_to(&self, path: &str) {
        let Ok(Some(info)) = get_dbc_info(&self.state.analysis) else {
            self.with_ui(|ui| ui.set_status_text("No DBC to save".into()));
            return;
        };
        match can_viewer::services::save_dbc_info(path, &info, &self.state.analysis) {
            Ok(()) => {
                self.refresh_dbc_status();
                self.with_ui(|ui| {
                    ui.set_dbc_dirty(false);
                    ui.set_dbc_editing(false);
                    ui.set_status_text(format!("Saved {path}").into());
                });
            }
            Err(e) => self.with_ui(|ui| ui.set_status_text(e.into())),
        }
    }
}

fn format_can_id(id: u32, extended: bool) -> String {
    if extended {
        format!("0x{id:08X}")
    } else {
        format!("0x{id:03X}")
    }
}

fn format_data_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_can_ids(text: &str) -> Vec<u32> {
    text.split([',', ' ', ';'])
        .filter_map(|s| {
            let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
            u32::from_str_radix(s, 16).ok()
        })
        .collect()
}

// Silence unused import if catalog row unused in stubs
#[allow(dead_code)]
fn _catalog_ty(_: DbcCatalogRow) {}
