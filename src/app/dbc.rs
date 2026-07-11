//! DBC editor tab controller.

use super::helpers::{parse_hex_id, pick_open_file, pick_save_file, set_status, vec_model};
use crate::dbc_export::empty_dbc_info;
use crate::dto::{DbcInfo, MessageInfo, NodeInfo, SignalInfo};
use crate::services::{get_dbc_info, get_dbc_path, load_dbc, save_dbc_info};
use crate::state::AppState;
use crate::{
    DbcAttributeRow as UiDbcAttributeRow, DbcMessageRow as UiDbcMessageRow,
    DbcNodeRow as UiDbcNodeRow, DbcSignalRow as UiDbcSignalRow,
    DbcValueDescRow as UiDbcValueDescRow, SigmaCanViewer,
};
use parking_lot::Mutex;
use slint::Weak;
use std::sync::Arc;

pub struct DbcController {
    state: Arc<AppState>,
    ui: Weak<SigmaCanViewer>,
    edit_state: Mutex<DbcInfo>,
    path: Mutex<Option<String>>,
    dirty: Mutex<bool>,
    selected_message: Mutex<i32>,
    selected_signal: Mutex<i32>,
    selected_node: Mutex<i32>,
}

impl DbcController {
    pub fn new(state: Arc<AppState>, ui: Weak<SigmaCanViewer>) -> Self {
        Self {
            state,
            ui,
            edit_state: Mutex::new(empty_dbc_info()),
            path: Mutex::new(None),
            dirty: Mutex::new(false),
            selected_message: Mutex::new(-1),
            selected_signal: Mutex::new(-1),
            selected_node: Mutex::new(-1),
        }
    }

    pub fn wire(self: Arc<Self>, ui: &SigmaCanViewer) {
        if let Ok(Some(info)) = get_dbc_info(&self.state) {
            *self.edit_state.lock() = info;
            *self.path.lock() = get_dbc_path(&self.state);
            self.refresh_ui();
        }

        let this = self.clone();
        ui.on_new_dbc({
            let t = this.clone();
            move || t.new_dbc()
        });
        ui.on_dbc_open({
            let t = this.clone();
            move || t.open_dbc()
        });
        ui.on_save_dbc({
            let t = this.clone();
            move || t.save_dbc()
        });
        ui.on_save_dbc_as({
            let t = this.clone();
            move || t.save_dbc_as()
        });
        ui.on_add_message({
            let t = this.clone();
            move || t.add_message()
        });
        ui.on_delete_message({
            let t = this.clone();
            move || t.delete_message()
        });
        ui.on_add_signal({
            let t = this.clone();
            move || t.add_signal()
        });
        ui.on_delete_signal({
            let t = this.clone();
            move || t.delete_signal()
        });
        ui.on_add_node({
            let t = this.clone();
            move || t.add_node()
        });
        ui.on_delete_node({
            let t = this.clone();
            move || t.delete_node()
        });
        ui.on_message_selected({
            let t = this.clone();
            move |i| t.select_message(i)
        });
        ui.on_signal_selected({
            let t = this.clone();
            move |i| t.select_signal(i)
        });
        ui.on_node_selected({
            let t = this.clone();
            move |i| t.select_node(i)
        });
    }

    fn with_ui<F: FnOnce(&SigmaCanViewer)>(&self, f: F) {
        if let Some(ui) = self.ui.upgrade() {
            f(&ui);
        }
    }

    fn mark_dirty(&self) {
        *self.dirty.lock() = true;
        self.with_ui(|ui| ui.set_dbc_dirty(true));
    }

    fn refresh_ui(&self) {
        let info = self.edit_state.lock();
        let path = self.path.lock().clone();
        let dirty = *self.dirty.lock();

        let messages: Vec<UiDbcMessageRow> = info.messages.iter().map(message_row).collect();
        let signals: Vec<UiDbcSignalRow> =
            selected_message_signals(&info, *self.selected_message.lock());
        let nodes: Vec<UiDbcNodeRow> = info.nodes.iter().map(node_row).collect();
        let attributes: Vec<UiDbcAttributeRow> = info
            .attribute_definitions
            .iter()
            .map(|a| UiDbcAttributeRow {
                name: a.name.clone().into(),
                object_type: a.object_type.clone().into(),
                value_summary: attribute_value_summary(&a.value_type).into(),
            })
            .collect();
        let value_descriptions: Vec<UiDbcValueDescRow> = info
            .value_descriptions
            .iter()
            .map(|vd| UiDbcValueDescRow {
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
            ui.set_dbc_path(path.unwrap_or_default().into());
            ui.set_dbc_dirty(dirty);
            ui.set_dbc_version(info.version.clone().unwrap_or_default().into());
            ui.set_dbc_comment(info.comment.clone().unwrap_or_default().into());
            ui.set_dbc_messages(vec_model(&messages));
            ui.set_dbc_signals(vec_model(&signals));
            ui.set_dbc_nodes(vec_model(&nodes));
            ui.set_dbc_attributes(vec_model(&attributes));
            ui.set_dbc_value_descriptions(vec_model(&value_descriptions));
            ui.set_dbc_selected_message(*self.selected_message.lock());
            ui.set_dbc_selected_signal(*self.selected_signal.lock());
            ui.set_dbc_selected_node(*self.selected_node.lock());
        });
    }

    fn apply_edits_from_ui(&self) {
        let mut info = self.edit_state.lock();
        self.with_ui(|ui| {
            info.version = Some(ui.get_dbc_version().to_string());
            info.comment = Some(ui.get_dbc_comment().to_string());

            let panel = ui.get_dbc_editor_panel();
            if panel == 0 {
                apply_message_edits(&mut info, ui, *self.selected_message.lock());
            } else if panel == 1 {
                apply_signal_edits(
                    &mut info,
                    ui,
                    *self.selected_message.lock(),
                    *self.selected_signal.lock(),
                );
            } else if panel == 2 {
                apply_node_edits(&mut info, ui, *self.selected_node.lock());
            }
        });
        self.mark_dirty();
    }

    fn new_dbc(&self) {
        *self.edit_state.lock() = empty_dbc_info();
        *self.path.lock() = None;
        *self.dirty.lock() = false;
        *self.selected_message.lock() = -1;
        *self.selected_signal.lock() = -1;
        *self.selected_node.lock() = -1;
        let _ = crate::services::clear_dbc(&self.state);
        self.refresh_ui();
        self.with_ui(|ui| set_status(ui, "New DBC created"));
    }

    fn open_dbc(&self) {
        if let Some(path) = pick_open_file("Open DBC", &[("DBC", &["dbc"])]) {
            match load_dbc(&path, &self.state) {
                Ok(msg) => {
                    if let Ok(Some(info)) = get_dbc_info(&self.state) {
                        *self.edit_state.lock() = info;
                    }
                    *self.path.lock() = Some(path);
                    *self.dirty.lock() = false;
                    self.refresh_ui();
                    self.with_ui(|ui| set_status(ui, &msg));
                }
                Err(e) => self.with_ui(|ui| set_status(ui, &e)),
            }
        }
    }

    fn save_dbc(&self) {
        if let Some(path) = self.path.lock().clone() {
            self.save_to_path(&path);
        } else {
            self.save_dbc_as();
        }
    }

    fn save_dbc_as(&self) {
        if let Some(path) = pick_save_file("Save DBC", &[("DBC", &["dbc"])]) {
            self.save_to_path(&path);
        }
    }

    fn save_to_path(&self, path: &str) {
        self.apply_edits_from_ui();
        let info = self.edit_state.lock().clone();
        match save_dbc_info(path, &info, &self.state) {
            Ok(()) => {
                *self.path.lock() = Some(path.to_string());
                *self.dirty.lock() = false;
                self.refresh_ui();
                self.with_ui(|ui| set_status(ui, &format!("Saved {path}")));
            }
            Err(e) => self.with_ui(|ui| set_status(ui, &e)),
        }
    }

    fn add_message(&self) {
        self.apply_edits_from_ui();
        let mut info = self.edit_state.lock();
        let id = next_message_id(&info);
        info.messages.push(MessageInfo {
            id,
            is_extended: false,
            name: format!("Message_{id}"),
            dlc: 8,
            sender: "Vector__XXX".to_string(),
            signals: Vec::new(),
            comment: None,
        });
        drop(info);
        *self.selected_message.lock() = (self.edit_state.lock().messages.len() as i32) - 1;
        *self.selected_signal.lock() = -1;
        self.mark_dirty();
        self.refresh_ui();
        self.select_message(*self.selected_message.lock());
    }

    fn delete_message(&self) {
        let idx = *self.selected_message.lock();
        if idx < 0 {
            return;
        }
        self.apply_edits_from_ui();
        let mut info = self.edit_state.lock();
        if let Ok(i) = usize::try_from(idx) {
            if i < info.messages.len() {
                info.messages.remove(i);
            }
        }
        *self.selected_message.lock() = -1;
        *self.selected_signal.lock() = -1;
        self.mark_dirty();
        self.refresh_ui();
    }

    fn add_signal(&self) {
        let msg_idx = *self.selected_message.lock();
        if msg_idx < 0 {
            self.with_ui(|ui| set_status(ui, "Select a message first"));
            return;
        }
        self.apply_edits_from_ui();
        let mut info = self.edit_state.lock();
        if let Some(msg) = info.messages.get_mut(msg_idx as usize) {
            msg.signals.push(SignalInfo {
                name: format!("Signal_{}", msg.signals.len()),
                start_bit: 0,
                length: 8,
                byte_order: "little_endian".to_string(),
                is_signed: false,
                factor: 1.0,
                offset: 0.0,
                min: 0.0,
                max: 0.0,
                unit: String::new(),
                receivers: vec!["Vector__XXX".to_string()],
                is_multiplexer: false,
                multiplexer_value: None,
                comment: None,
            });
            *self.selected_signal.lock() = (msg.signals.len() as i32) - 1;
        }
        drop(info);
        self.mark_dirty();
        self.refresh_ui();
        self.select_signal(*self.selected_signal.lock());
    }

    fn delete_signal(&self) {
        let msg_idx = *self.selected_message.lock();
        let sig_idx = *self.selected_signal.lock();
        if msg_idx < 0 || sig_idx < 0 {
            return;
        }
        self.apply_edits_from_ui();
        let mut info = self.edit_state.lock();
        if let Some(msg) = info.messages.get_mut(msg_idx as usize) {
            if let Ok(i) = usize::try_from(sig_idx) {
                if i < msg.signals.len() {
                    msg.signals.remove(i);
                }
            }
        }
        *self.selected_signal.lock() = -1;
        self.mark_dirty();
        self.refresh_ui();
    }

    fn add_node(&self) {
        self.apply_edits_from_ui();
        let mut info = self.edit_state.lock();
        let name = format!("Node_{}", info.nodes.len());
        info.nodes.push(NodeInfo {
            name,
            comment: None,
        });
        *self.selected_node.lock() = (info.nodes.len() as i32) - 1;
        drop(info);
        self.mark_dirty();
        self.refresh_ui();
        self.select_node(*self.selected_node.lock());
    }

    fn delete_node(&self) {
        let idx = *self.selected_node.lock();
        if idx < 0 {
            return;
        }
        self.apply_edits_from_ui();
        let mut info = self.edit_state.lock();
        if let Ok(i) = usize::try_from(idx) {
            if i < info.nodes.len() {
                info.nodes.remove(i);
            }
        }
        *self.selected_node.lock() = -1;
        self.mark_dirty();
        self.refresh_ui();
    }

    fn select_message(&self, index: i32) {
        self.apply_edits_from_ui();
        *self.selected_message.lock() = index;
        *self.selected_signal.lock() = -1;
        let info = self.edit_state.lock();
        self.with_ui(|ui| {
            ui.set_dbc_selected_message(index);
            ui.set_dbc_selected_signal(-1);
            if let Some(msg) = info.messages.get(index as usize) {
                ui.set_dbc_edit_message_id(format!("0x{:X}", msg.id).into());
                ui.set_dbc_edit_message_name(msg.name.clone().into());
                ui.set_dbc_edit_message_dlc(msg.dlc.to_string().into());
                ui.set_dbc_edit_message_sender(msg.sender.clone().into());
            }
            let signals: Vec<UiDbcSignalRow> = selected_message_signals(&info, index);
            ui.set_dbc_signals(vec_model(&signals));
        });
    }

    fn select_signal(&self, index: i32) {
        self.apply_edits_from_ui();
        *self.selected_signal.lock() = index;
        let msg_idx = *self.selected_message.lock();
        let info = self.edit_state.lock();
        self.with_ui(|ui| {
            ui.set_dbc_selected_signal(index);
            if let Some(msg) = info.messages.get(msg_idx as usize) {
                if let Some(sig) = msg.signals.get(index as usize) {
                    ui.set_dbc_edit_signal_name(sig.name.clone().into());
                    ui.set_dbc_edit_signal_start(sig.start_bit.to_string().into());
                    ui.set_dbc_edit_signal_length(sig.length.to_string().into());
                    ui.set_dbc_edit_signal_factor(sig.factor.to_string().into());
                    ui.set_dbc_edit_signal_offset(sig.offset.to_string().into());
                    ui.set_dbc_edit_signal_unit(sig.unit.clone().into());
                }
            }
        });
    }

    fn select_node(&self, index: i32) {
        self.apply_edits_from_ui();
        *self.selected_node.lock() = index;
        let info = self.edit_state.lock();
        self.with_ui(|ui| {
            ui.set_dbc_selected_node(index);
            if let Some(node) = info.nodes.get(index as usize) {
                ui.set_dbc_edit_node_name(node.name.clone().into());
                ui.set_dbc_edit_node_comment(node.comment.clone().unwrap_or_default().into());
            }
        });
    }
}

fn message_row(m: &MessageInfo) -> UiDbcMessageRow {
    UiDbcMessageRow {
        id: format!("0x{:X}", m.id).into(),
        name: m.name.clone().into(),
        dlc: m.dlc.to_string().into(),
        sender: m.sender.clone().into(),
        signal_count: m.signals.len().to_string().into(),
    }
}

fn node_row(n: &NodeInfo) -> UiDbcNodeRow {
    UiDbcNodeRow {
        name: n.name.clone().into(),
        comment: n.comment.clone().unwrap_or_default().into(),
    }
}

fn selected_message_signals(info: &DbcInfo, msg_idx: i32) -> Vec<UiDbcSignalRow> {
    info.messages
        .get(msg_idx as usize)
        .map(|m| {
            m.signals
                .iter()
                .map(|s| UiDbcSignalRow {
                    name: s.name.clone().into(),
                    start_bit: s.start_bit.to_string().into(),
                    length: s.length.to_string().into(),
                    factor: s.factor.to_string().into(),
                    unit: s.unit.clone().into(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn next_message_id(info: &DbcInfo) -> u32 {
    info.messages.iter().map(|m| m.id).max().unwrap_or(255) + 1
}

fn apply_message_edits(info: &mut DbcInfo, ui: &SigmaCanViewer, idx: i32) {
    if idx < 0 {
        return;
    }
    if let Some(msg) = info.messages.get_mut(idx as usize) {
        if let Some(id) = parse_hex_id(ui.get_dbc_edit_message_id().as_ref()) {
            msg.id = id;
            msg.is_extended = id > 0x7FF;
        }
        msg.name = ui.get_dbc_edit_message_name().to_string();
        msg.dlc = ui.get_dbc_edit_message_dlc().parse().unwrap_or(8);
        msg.sender = ui.get_dbc_edit_message_sender().to_string();
    }
}

fn apply_signal_edits(info: &mut DbcInfo, ui: &SigmaCanViewer, msg_idx: i32, sig_idx: i32) {
    if msg_idx < 0 || sig_idx < 0 {
        return;
    }
    if let Some(msg) = info.messages.get_mut(msg_idx as usize) {
        if let Some(sig) = msg.signals.get_mut(sig_idx as usize) {
            sig.name = ui.get_dbc_edit_signal_name().to_string();
            sig.start_bit = ui.get_dbc_edit_signal_start().parse().unwrap_or(0);
            sig.length = ui.get_dbc_edit_signal_length().parse().unwrap_or(8);
            sig.factor = ui.get_dbc_edit_signal_factor().parse().unwrap_or(1.0);
            sig.offset = ui.get_dbc_edit_signal_offset().parse().unwrap_or(0.0);
            sig.unit = ui.get_dbc_edit_signal_unit().to_string();
        }
    }
}

fn attribute_value_summary(value_type: &crate::dto::AttributeValueType) -> String {
    match value_type {
        crate::dto::AttributeValueType::Int { min, max } => format!("INT {min}..{max}"),
        crate::dto::AttributeValueType::Hex { min, max } => format!("HEX {min}..{max}"),
        crate::dto::AttributeValueType::Float { min, max } => format!("FLOAT {min}..{max}"),
        crate::dto::AttributeValueType::String => "STRING".to_string(),
        crate::dto::AttributeValueType::Enum { values } => format!("ENUM ({})", values.len()),
    }
}

fn apply_node_edits(info: &mut DbcInfo, ui: &SigmaCanViewer, idx: i32) {
    if idx < 0 {
        return;
    }
    if let Some(node) = info.nodes.get_mut(idx as usize) {
        node.name = ui.get_dbc_edit_node_name().to_string();
        let comment = ui.get_dbc_edit_node_comment().to_string();
        node.comment = if comment.is_empty() {
            None
        } else {
            Some(comment)
        };
    }
}
