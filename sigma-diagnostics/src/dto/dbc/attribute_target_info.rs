use serde::Serialize;

/// Attribute target specification.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AttributeTargetInfo {
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "node")]
    Node { node_name: String },
    #[serde(rename = "message")]
    Message { message_id: u32 },
    #[serde(rename = "signal")]
    Signal {
        message_id: u32,
        signal_name: String,
    },
}
