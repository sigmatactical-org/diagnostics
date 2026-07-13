use serde::{Deserialize, Serialize};

/// Serializable decoded signal for frontend communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedSignalDto {
    pub timestamp: f64,
    pub message_name: String,
    pub signal_name: String,
    pub value: f64,
    pub raw_value: i64,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl DecodedSignalDto {
    /// Convert from dbc_rs::DecodedSignal, adding timestamp and message name.
    pub fn from_dbc_signal(
        sig: &dbc_rs::DecodedSignal<'_>,
        timestamp: f64,
        message_name: &str,
    ) -> Self {
        Self {
            timestamp,
            message_name: message_name.to_string(),
            signal_name: sig.name.to_string(),
            value: sig.value,
            raw_value: sig.raw_value,
            unit: sig.unit.unwrap_or("").to_string(),
            description: sig.description.map(|s| s.to_string()),
        }
    }
}
