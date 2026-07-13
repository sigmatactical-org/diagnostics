use serde::Serialize;

use super::ValueDescriptionEntry;

/// Value descriptions for a signal (VAL_).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SignalValueDescriptions {
    pub message_id: u32,
    pub signal_name: String,
    pub descriptions: Vec<ValueDescriptionEntry>,
}
