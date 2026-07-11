//! DBC file loading, decoding, and editing services.

use crate::dbc_export::export_dbc_info_to_string;
use crate::decode::{decode_frame, DecodeResult};
use crate::dto::{
    AttributeAssignmentInfo, AttributeDefaultInfo, AttributeDefinitionInfo, AttributeTargetInfo,
    AttributeValueInfo, AttributeValueType, BitTimingInfo, CanFrameDto, DbcInfo, DecodeResponse,
    ExtendedMultiplexingInfo, MessageInfo, NodeInfo, SignalInfo, SignalValueDescriptions,
    ValueDescriptionEntry,
};
use crate::state::AppState;
use dbc_rs::Dbc;

/// Load and parse a DBC file from disk.
pub fn load_dbc(path: &str, state: &AppState) -> Result<String, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read DBC: {e}"))?;

    let dbc = Dbc::parse(&content).map_err(|e| format!("Failed to parse DBC: {e:?}"))?;
    let msg_count = dbc.messages().len();

    state.set_dbc(dbc);
    *state.dbc_path.lock() = Some(path.to_string());

    if let Err(e) = state.session.lock().set_dbc_path(Some(path.to_string())) {
        log::warn!("Failed to save session: {e}");
    }

    Ok(format!("Loaded {msg_count} messages"))
}

/// Clear the loaded DBC.
pub fn clear_dbc(state: &AppState) -> Result<(), String> {
    state.clear_dbc();
    *state.dbc_path.lock() = None;

    if let Err(e) = state.session.lock().set_dbc_path(None) {
        log::warn!("Failed to save session: {e}");
    }

    Ok(())
}

/// Get path to the currently loaded DBC file.
pub fn get_dbc_path(state: &AppState) -> Option<String> {
    state.dbc_path.lock().clone()
}

/// Decode a single CAN frame.
pub fn decode_single_frame(frame: &CanFrameDto, state: &AppState) -> DecodeResponse {
    let dbc_guard = state.dbc.lock();
    let Some(ref dbc) = *dbc_guard else {
        return DecodeResponse {
            signals: Vec::new(),
            errors: Vec::new(),
        };
    };

    match decode_frame(frame, dbc) {
        DecodeResult::Signals(signals) => DecodeResponse {
            signals,
            errors: Vec::new(),
        },
        DecodeResult::Error(err) => DecodeResponse {
            signals: Vec::new(),
            errors: vec![err],
        },
    }
}

/// Decode multiple CAN frames.
pub fn decode_frames(frames: &[CanFrameDto], state: &AppState) -> DecodeResponse {
    let dbc_guard = state.dbc.lock();
    let Some(ref dbc) = *dbc_guard else {
        return DecodeResponse {
            signals: Vec::new(),
            errors: Vec::new(),
        };
    };

    let mut signals = Vec::new();
    let mut errors = Vec::new();

    for frame in frames {
        match decode_frame(frame, dbc) {
            DecodeResult::Signals(sigs) => signals.extend(sigs),
            DecodeResult::Error(err) => errors.push(err),
        }
    }

    DecodeResponse { signals, errors }
}

/// Save DBC content to a file after validation.
pub fn save_dbc_content(path: &str, content: &str, state: &AppState) -> Result<(), String> {
    let dbc = Dbc::parse(content).map_err(|e| format!("Invalid DBC content: {e:?}"))?;

    std::fs::write(path, content).map_err(|e| format!("Failed to write DBC: {e}"))?;

    state.set_dbc(dbc);
    *state.dbc_path.lock() = Some(path.to_string());

    if let Err(e) = state.session.lock().set_dbc_path(Some(path.to_string())) {
        log::warn!("Failed to save session: {e}");
    }

    Ok(())
}

/// Save a [`DbcInfo`] structure to disk.
pub fn save_dbc_info(path: &str, info: &DbcInfo, state: &AppState) -> Result<(), String> {
    let content = export_dbc_info_to_string(info);
    save_dbc_content(path, &content, state)
}

/// Update in-memory DBC from content string.
pub fn update_dbc_content(content: &str, state: &AppState) -> Result<String, String> {
    let dbc = Dbc::parse(content).map_err(|e| format!("Failed to parse DBC: {e:?}"))?;
    let msg_count = dbc.messages().len();

    state.set_dbc(dbc);
    Ok(format!("Updated DBC with {msg_count} messages"))
}

/// Update in-memory DBC from a [`DbcInfo`] structure.
pub fn update_dbc_info(info: &DbcInfo, state: &AppState) -> Result<String, String> {
    let content = export_dbc_info_to_string(info);
    update_dbc_content(&content, state)
}

/// Get information about the loaded DBC.
pub fn get_dbc_info(state: &AppState) -> Result<Option<DbcInfo>, String> {
    let dbc_guard = state.dbc.lock();
    let Some(ref dbc) = *dbc_guard else {
        return Ok(None);
    };

    let nodes: Vec<NodeInfo> = dbc
        .nodes()
        .iter_nodes()
        .map(|node| NodeInfo {
            name: node.name().to_string(),
            comment: node.comment().map(|s| s.to_string()),
        })
        .collect();

    let messages: Vec<MessageInfo> = dbc
        .messages()
        .iter()
        .map(|msg| {
            let signals: Vec<SignalInfo> = msg
                .signals()
                .iter()
                .map(|sig| {
                    let byte_order = match sig.byte_order() {
                        dbc_rs::ByteOrder::BigEndian => "big_endian",
                        dbc_rs::ByteOrder::LittleEndian => "little_endian",
                    };
                    let receivers: Vec<String> =
                        sig.receivers().iter().map(|r| r.to_string()).collect();
                    SignalInfo {
                        name: sig.name().to_string(),
                        start_bit: sig.start_bit() as u32,
                        length: sig.length() as u32,
                        byte_order: byte_order.to_string(),
                        is_signed: !sig.is_unsigned(),
                        factor: sig.factor(),
                        offset: sig.offset(),
                        min: sig.min(),
                        max: sig.max(),
                        unit: sig.unit().unwrap_or("").to_string(),
                        receivers,
                        is_multiplexer: sig.is_multiplexer_switch(),
                        multiplexer_value: sig.multiplexer_switch_value(),
                        comment: sig.comment().map(|s| s.to_string()),
                    }
                })
                .collect();

            MessageInfo {
                id: msg.id(),
                is_extended: msg.is_extended(),
                name: msg.name().to_string(),
                dlc: msg.dlc(),
                sender: msg.sender().to_string(),
                signals,
                comment: msg.comment().map(|s| s.to_string()),
            }
        })
        .collect();

    let mut value_descriptions: Vec<SignalValueDescriptions> = Vec::new();
    for msg in dbc.messages().iter() {
        for sig in msg.signals().iter() {
            if let Some(val_descs) = dbc.value_descriptions_for_signal(msg.id(), sig.name()) {
                let descriptions: Vec<ValueDescriptionEntry> = val_descs
                    .iter()
                    .map(|(val, desc)| ValueDescriptionEntry {
                        value: val as i64,
                        description: desc.to_string(),
                    })
                    .collect();
                if !descriptions.is_empty() {
                    value_descriptions.push(SignalValueDescriptions {
                        message_id: msg.id(),
                        signal_name: sig.name().to_string(),
                        descriptions,
                    });
                }
            }
        }
    }

    let attribute_definitions: Vec<AttributeDefinitionInfo> = dbc
        .attribute_definitions()
        .iter()
        .map(|def| {
            let object_type = match def.object_type() {
                dbc_rs::AttributeObjectType::Network => "network",
                dbc_rs::AttributeObjectType::Node => "node",
                dbc_rs::AttributeObjectType::Message => "message",
                dbc_rs::AttributeObjectType::Signal => "signal",
            };
            let value_type = match def.value_type() {
                dbc_rs::AttributeValueType::Int { min, max } => AttributeValueType::Int {
                    min: *min,
                    max: *max,
                },
                dbc_rs::AttributeValueType::Hex { min, max } => AttributeValueType::Hex {
                    min: *min,
                    max: *max,
                },
                dbc_rs::AttributeValueType::Float { min, max } => AttributeValueType::Float {
                    min: *min,
                    max: *max,
                },
                dbc_rs::AttributeValueType::String => AttributeValueType::String,
                dbc_rs::AttributeValueType::Enum { values } => AttributeValueType::Enum {
                    values: values.iter().map(|v| v.to_string()).collect(),
                },
            };
            AttributeDefinitionInfo {
                name: def.name().to_string(),
                object_type: object_type.to_string(),
                value_type,
            }
        })
        .collect();

    let attribute_defaults: Vec<AttributeDefaultInfo> = dbc
        .attribute_defaults()
        .iter()
        .map(|(name, value)| AttributeDefaultInfo {
            name: name.to_string(),
            value: convert_attribute_value(value),
        })
        .collect();

    let attribute_values: Vec<AttributeAssignmentInfo> = dbc
        .attribute_values()
        .iter()
        .map(|((name, target), value)| {
            let target_info = match target {
                dbc_rs::AttributeTarget::Network => AttributeTargetInfo::Network,
                dbc_rs::AttributeTarget::Node(node_name) => AttributeTargetInfo::Node {
                    node_name: node_name.to_string(),
                },
                dbc_rs::AttributeTarget::Message(msg_id) => AttributeTargetInfo::Message {
                    message_id: *msg_id,
                },
                dbc_rs::AttributeTarget::Signal(msg_id, sig_name) => AttributeTargetInfo::Signal {
                    message_id: *msg_id,
                    signal_name: sig_name.to_string(),
                },
            };
            AttributeAssignmentInfo {
                name: name.to_string(),
                target: target_info,
                value: convert_attribute_value(value),
            }
        })
        .collect();

    let extended_multiplexing: Vec<ExtendedMultiplexingInfo> = dbc
        .extended_multiplexing()
        .iter()
        .map(|em| ExtendedMultiplexingInfo {
            message_id: em.message_id(),
            signal_name: em.signal_name().to_string(),
            multiplexer_signal: em.multiplexer_switch().to_string(),
            ranges: em.value_ranges().to_vec(),
        })
        .collect();

    let bit_timing = dbc.bit_timing().and_then(|bt| {
        bt.baudrate().map(|baudrate| BitTimingInfo {
            baudrate,
            btr1: bt.btr1().unwrap_or(0),
            btr2: bt.btr2().unwrap_or(0),
        })
    });

    Ok(Some(DbcInfo {
        version: dbc.version().map(|v| v.to_string()),
        bit_timing,
        comment: dbc.comment().map(|s| s.to_string()),
        nodes,
        messages,
        value_descriptions,
        attribute_definitions,
        attribute_defaults,
        attribute_values,
        extended_multiplexing,
    }))
}

fn convert_attribute_value(value: &dbc_rs::AttributeValue) -> AttributeValueInfo {
    match value {
        dbc_rs::AttributeValue::Int(i) => AttributeValueInfo::Int(*i),
        dbc_rs::AttributeValue::Float(f) => AttributeValueInfo::Float(*f),
        dbc_rs::AttributeValue::String(s) => AttributeValueInfo::String(s.to_string()),
    }
}

/// Get the DBC file format specification text.
pub fn get_dbc_specification() -> String {
    dbc_rs::SPECIFICATION.to_string()
}
