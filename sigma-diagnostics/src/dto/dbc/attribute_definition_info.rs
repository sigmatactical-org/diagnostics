use serde::Serialize;

use super::AttributeValueType;

/// Attribute definition (BA_DEF_).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttributeDefinitionInfo {
    pub name: String,
    pub object_type: String, // "network", "node", "message", "signal"
    pub value_type: AttributeValueType,
}
