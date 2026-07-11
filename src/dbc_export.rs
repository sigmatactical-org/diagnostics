//! Serialize [`DbcInfo`] to DBC file format (ported from the legacy TS editor).

use crate::dto::{AttributeTargetInfo, AttributeValueInfo, AttributeValueType, DbcInfo};

/// Export a [`DbcInfo`] to DBC file content.
pub fn export_dbc_info_to_string(dbc: &DbcInfo) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "VERSION \"{}\"",
        dbc.version.as_deref().unwrap_or("")
    ));
    lines.push(String::new());
    lines.push("NS_ :".to_string());
    lines.push(String::new());

    if let Some(ref bt) = dbc.bit_timing {
        if bt.baudrate > 0 {
            if bt.btr1 > 0 || bt.btr2 > 0 {
                lines.push(format!("BS_: {} : {},{}", bt.baudrate, bt.btr1, bt.btr2));
            } else {
                lines.push(format!("BS_: {}", bt.baudrate));
            }
        } else {
            lines.push("BS_:".to_string());
        }
    } else {
        lines.push("BS_:".to_string());
    }
    lines.push(String::new());

    if dbc.nodes.is_empty() {
        lines.push("BU_:".to_string());
    } else {
        lines.push(format!(
            "BU_: {}",
            dbc.nodes
                .iter()
                .map(|n| n.name.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    lines.push(String::new());

    for msg in &dbc.messages {
        let msg_id = if msg.is_extended {
            msg.id | 0x8000_0000
        } else {
            msg.id
        };
        lines.push(format!(
            "BO_ {} {}: {} {}",
            msg_id, msg.name, msg.dlc, msg.sender
        ));

        for sig in &msg.signals {
            let byte_order = if sig.byte_order == "little_endian" {
                1
            } else {
                0
            };
            let value_type = if sig.is_signed { '-' } else { '+' };
            let mux_indicator = if sig.is_multiplexer {
                " M".to_string()
            } else if let Some(v) = sig.multiplexer_value {
                format!(" m{v}")
            } else {
                String::new()
            };
            let receivers = if sig.receivers.is_empty() {
                "Vector__XXX".to_string()
            } else {
                sig.receivers.join(",")
            };
            lines.push(format!(
                " SG_ {}{} : {}|{}@{}{} ({},{}) [{}|{}] \"{}\" {}",
                sig.name,
                mux_indicator,
                sig.start_bit,
                sig.length,
                byte_order,
                value_type,
                sig.factor,
                sig.offset,
                sig.min,
                sig.max,
                sig.unit,
                receivers
            ));
        }
        lines.push(String::new());
    }

    if let Some(ref comment) = dbc.comment {
        lines.push(format!("CM_ \"{}\" ;", escape_dbc_string(comment)));
    }
    for node in &dbc.nodes {
        if let Some(ref comment) = node.comment {
            lines.push(format!(
                "CM_ BU_ {} \"{}\" ;",
                node.name,
                escape_dbc_string(comment)
            ));
        }
    }
    for msg in &dbc.messages {
        let msg_id = if msg.is_extended {
            msg.id | 0x8000_0000
        } else {
            msg.id
        };
        if let Some(ref comment) = msg.comment {
            lines.push(format!(
                "CM_ BO_ {msg_id} \"{}\" ;",
                escape_dbc_string(comment)
            ));
        }
        for sig in &msg.signals {
            if let Some(ref comment) = sig.comment {
                lines.push(format!(
                    "CM_ SG_ {msg_id} {} \"{}\" ;",
                    sig.name,
                    escape_dbc_string(comment)
                ));
            }
        }
    }

    for def in &dbc.attribute_definitions {
        let object_type = attribute_object_keyword(&def.object_type);
        let value_type = attribute_value_type_string(&def.value_type);
        lines.push(format!(
            "BA_DEF_ {object_type}\"{}\" {value_type} ;",
            def.name
        ));
    }
    for def in &dbc.attribute_defaults {
        let value_str = attribute_value_string(&def.value);
        lines.push(format!("BA_DEF_DEF_ \"{}\" {value_str} ;", def.name));
    }
    for val in &dbc.attribute_values {
        let target_str = attribute_target_string(&val.target);
        let value_str = attribute_value_string(&val.value);
        lines.push(format!("BA_ \"{}\" {target_str}{value_str} ;", val.name));
    }
    for vd in &dbc.value_descriptions {
        if vd.descriptions.is_empty() {
            continue;
        }
        let entries = vd
            .descriptions
            .iter()
            .map(|d| format!("{} \"{}\"", d.value, escape_dbc_string(&d.description)))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!(
            "VAL_ {} {} {entries} ;",
            vd.message_id, vd.signal_name
        ));
    }
    for em in &dbc.extended_multiplexing {
        let ranges = em
            .ranges
            .iter()
            .map(|r| format!("{}-{}", r.0, r.1))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "SG_MUL_VAL_ {} {} {} {ranges} ;",
            em.message_id, em.signal_name, em.multiplexer_signal
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Create an empty DBC suitable for editing.
pub fn empty_dbc_info() -> DbcInfo {
    DbcInfo {
        version: None,
        bit_timing: None,
        comment: None,
        nodes: Vec::new(),
        messages: Vec::new(),
        value_descriptions: Vec::new(),
        attribute_definitions: Vec::new(),
        attribute_defaults: Vec::new(),
        attribute_values: Vec::new(),
        extended_multiplexing: Vec::new(),
    }
}

fn escape_dbc_string(s: &str) -> String {
    s.replace('\\', "")
        .replace('\n', " ")
        .replace('\r', "")
        .replace('"', "'")
}

fn attribute_object_keyword(object_type: &str) -> &'static str {
    match object_type {
        "node" => "BU_ ",
        "message" => "BO_ ",
        "signal" => "SG_ ",
        _ => "",
    }
}

fn attribute_value_type_string(value_type: &AttributeValueType) -> String {
    match value_type {
        AttributeValueType::Int { min, max } => format!("INT {min} {max}"),
        AttributeValueType::Hex { min, max } => format!("HEX {min} {max}"),
        AttributeValueType::Float { min, max } => format!("FLOAT {min} {max}"),
        AttributeValueType::String => "STRING".to_string(),
        AttributeValueType::Enum { values } => format!(
            "ENUM {}",
            values
                .iter()
                .map(|v| format!("\"{}\"", escape_dbc_string(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn attribute_target_string(target: &AttributeTargetInfo) -> String {
    match target {
        AttributeTargetInfo::Network => String::new(),
        AttributeTargetInfo::Node { node_name } => format!("BU_ {node_name} "),
        AttributeTargetInfo::Message { message_id } => format!("BO_ {message_id} "),
        AttributeTargetInfo::Signal {
            message_id,
            signal_name,
        } => format!("SG_ {message_id} {signal_name} "),
    }
}

fn attribute_value_string(value: &AttributeValueInfo) -> String {
    match value {
        AttributeValueInfo::Int(i) => i.to_string(),
        AttributeValueInfo::Float(f) => f.to_string(),
        AttributeValueInfo::String(s) => format!("\"{}\"", escape_dbc_string(s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::get_dbc_info;
    use crate::state::AppState;
    use dbc_rs::Dbc;

    #[test]
    fn round_trip_export_parse() {
        let sample = r#"VERSION ""

NS_ :

BS_:

BU_: ECM

BO_ 256 EngineData: 8 ECM
 SG_ RPM : 7|16@0+ (0.25,0) [0|8000] "rpm" ECM

"#;
        let state = AppState::default();
        state.set_dbc(Dbc::parse(sample).unwrap());
        let info = get_dbc_info(&state).unwrap().unwrap();
        let exported = export_dbc_info_to_string(&info);
        assert!(exported.contains("BO_ 256 EngineData"));
        assert!(exported.contains("SG_ RPM"));
    }
}
