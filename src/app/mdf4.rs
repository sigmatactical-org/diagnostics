//! MDF4 inspector tab controller.

use super::helpers::{
    format_can_id, format_data_hex, parse_can_ids, pick_open_file, pick_save_file, set_status,
    vec_model,
};
use crate::dto::{CanFrameDto, FrameTableRow};
use crate::services::{
    decode_single_frame, export_logs, filter_frames, get_dbc_path, load_dbc, load_mdf4,
    FilterConfig, MatchStatus,
};
use crate::state::AppState;
use crate::{FrameRow, SigmaCanViewer, SignalRow};
use parking_lot::Mutex;
use slint::{ModelRc, Weak};
use std::sync::Arc;

pub struct Mdf4Controller {
    state: Arc<AppState>,
    ui: Weak<SigmaCanViewer>,
    all_frames: Mutex<Vec<CanFrameDto>>,
    filtered_frames: Mutex<Vec<CanFrameDto>>,
}

impl Mdf4Controller {
    pub fn new(state: Arc<AppState>, ui: Weak<SigmaCanViewer>) -> Self {
        Self {
            state,
            ui,
            all_frames: Mutex::new(Vec::new()),
            filtered_frames: Mutex::new(Vec::new()),
        }
    }

    pub fn wire(self: Arc<Self>, ui: &SigmaCanViewer) {
        ui.on_open_mdf4({
            let this = self.clone();
            move || this.open_mdf4()
        });
        ui.on_mdf4_open_dbc({
            let this = self.clone();
            move || this.open_dbc()
        });
        ui.on_export_mdf4({
            let this = self.clone();
            move || this.export_mdf4()
        });
        ui.on_apply_filters({
            let this = self.clone();
            move || this.apply_filters()
        });
        ui.on_clear_filters({
            let this = self.clone();
            move || this.clear_filters()
        });
        ui.on_frame_selected({
            let this = self.clone();
            move |index| this.select_frame(index)
        });
    }

    pub fn load_initial_files(&self) {
        let initial = crate::services::get_initial_files(&self.state);
        if let Some(path) = initial.dbc_path {
            match load_dbc(&path, &self.state) {
                Ok(_) => self.refresh_dbc_status(),
                Err(e) => log::warn!("Startup DBC load failed ({path}): {e}"),
            }
        }
        if let Some(path) = initial.mdf4_path {
            self.load_mdf4_path(&path, true);
        }
    }

    fn with_ui<F: FnOnce(&SigmaCanViewer)>(&self, f: F) {
        if let Some(ui) = self.ui.upgrade() {
            f(&ui);
        }
    }

    fn refresh_dbc_status(&self) {
        self.with_ui(|ui| {
            if let Some(path) = get_dbc_path(&self.state) {
                let name = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path);
                ui.set_mdf4_dbc_status(name.to_string().into());
            } else {
                ui.set_mdf4_dbc_status("No DBC".into());
            }
        });
    }

    fn open_mdf4(&self) {
        if let Some(path) = pick_open_file("Open MDF4", &[("MDF4", &["mf4", "mdf"])]) {
            self.load_mdf4_path(&path, false);
        }
    }

    fn load_mdf4_path(&self, path: &str, startup: bool) {
        if !std::path::Path::new(path).is_file() {
            if startup {
                log::warn!("Startup MDF4 path not found: {path}");
            } else {
                self.with_ui(|ui| set_status(ui, &format!("File not found: {path}")));
            }
            return;
        }

        match load_mdf4(path, &self.state) {
            Ok((frames, _decoded)) => {
                *self.all_frames.lock() = frames;
                self.apply_filters();
                self.with_ui(|ui| {
                    ui.set_mdf4_path(path.into());
                    let count = self.all_frames.lock().len();
                    set_status(ui, &format!("Loaded {count} frames"));
                });
            }
            Err(e) => {
                if startup {
                    log::warn!("Startup MDF4 load failed ({path}): {e}");
                    self.with_ui(|ui| set_status(ui, "Ready"));
                } else {
                    self.with_ui(|ui| set_status(ui, &e));
                }
            }
        }
    }

    fn open_dbc(&self) {
        if let Some(path) = pick_open_file("Open DBC", &[("DBC", &["dbc"])]) {
            match load_dbc(&path, &self.state) {
                Ok(msg) => {
                    self.refresh_dbc_status();
                    self.apply_filters();
                    self.with_ui(|ui| set_status(ui, &msg));
                }
                Err(e) => self.with_ui(|ui| set_status(ui, &e)),
            }
        }
    }

    fn export_mdf4(&self) {
        let frames = self.filtered_frames.lock().clone();
        if frames.is_empty() {
            self.with_ui(|ui| set_status(ui, "No frames to export"));
            return;
        }
        if let Some(path) = pick_save_file("Export MDF4", &[("MDF4", &["mf4"])]) {
            match export_logs(&path, &frames) {
                Ok(count) => self.with_ui(|ui| {
                    set_status(ui, &format!("Exported {count} frames to {path}"));
                }),
                Err(e) => self.with_ui(|ui| set_status(ui, &e)),
            }
        }
    }

    fn apply_filters(&self) {
        let all = self.all_frames.lock().clone();
        self.with_ui(|ui| {
            let can_id_text = ui.get_mdf4_filter_can_id().to_string();
            let message_text = ui.get_mdf4_filter_message().to_string();
            let data_text = ui.get_mdf4_filter_data().to_string();

            let mut filters = FilterConfig::default();
            let can_ids = parse_can_ids(&can_id_text);
            if !can_ids.is_empty() {
                filters.can_ids = can_ids;
            }
            if !message_text.is_empty() {
                filters.messages = vec![message_text];
            }
            if !data_text.is_empty() {
                filters.data_pattern = Some(data_text);
            }
            filters.match_status = MatchStatus::All;

            let result = filter_frames(all, filters, &self.state);
            *self.filtered_frames.lock() = result.frames.clone();
            self.update_frame_table(&result.frames);
            ui.set_mdf4_frame_count(
                format!("{} / {} frames", result.filtered_count, result.total_count).into(),
            );
            ui.set_mdf4_selected_frame(-1);
            ui.set_mdf4_signals(ModelRc::new(slint::VecModel::default()));
        });
    }

    fn clear_filters(&self) {
        self.with_ui(|ui| {
            ui.set_mdf4_filter_can_id("".into());
            ui.set_mdf4_filter_message("".into());
            ui.set_mdf4_filter_data("".into());
        });
        self.apply_filters();
    }

    fn update_frame_table(&self, frames: &[CanFrameDto]) {
        let rows: Vec<FrameTableRow> = frames
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let message_name = self.message_name_for(f);
                FrameTableRow {
                    index: i as i32,
                    timestamp: format!("{:.3}", f.timestamp),
                    can_id: format_can_id(f.can_id, f.is_extended),
                    channel: f.channel.clone(),
                    dlc: f.dlc.to_string(),
                    data_hex: format_data_hex(&f.data),
                    message_name,
                }
            })
            .collect();

        let slint_rows: Vec<FrameRow> = rows
            .iter()
            .map(|r| FrameRow {
                index: r.index,
                timestamp: r.timestamp.clone().into(),
                can_id: r.can_id.clone().into(),
                channel: r.channel.clone().into(),
                dlc: r.dlc.clone().into(),
                data_hex: r.data_hex.clone().into(),
                message_name: r.message_name.clone().into(),
            })
            .collect();

        self.with_ui(|ui| {
            ui.set_mdf4_frames(vec_model(&slint_rows));
        });
    }

    fn message_name_for(&self, frame: &CanFrameDto) -> String {
        let fast = self.state.fast_dbc.lock();
        if let Some(ref fdbc) = *fast {
            let msg = if frame.is_extended {
                fdbc.get_extended(frame.can_id)
            } else {
                fdbc.get(frame.can_id)
            };
            return msg.map(|m| m.name().to_string()).unwrap_or_default();
        }
        String::new()
    }

    fn select_frame(&self, index: i32) {
        if index < 0 {
            return;
        }
        let frames = self.filtered_frames.lock();
        let Some(frame) = frames.get(index as usize) else {
            return;
        };

        let response = decode_single_frame(frame, &self.state);
        let signal_rows: Vec<SignalRow> = response
            .signals
            .iter()
            .map(|s| SignalRow {
                signal_name: s.signal_name.clone().into(),
                value: format!("{:.4}", s.value).into(),
                unit: s.unit.clone().into(),
                raw_value: s.raw_value.to_string().into(),
                description: s.description.clone().unwrap_or_default().into(),
            })
            .collect();

        self.with_ui(|ui| {
            ui.set_mdf4_selected_frame(index);
            ui.set_mdf4_signals(vec_model(&signal_rows));
        });
    }
}
