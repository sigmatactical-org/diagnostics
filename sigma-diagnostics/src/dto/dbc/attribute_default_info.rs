use serde::Serialize;

use super::AttributeValueInfo;

/// Attribute default value (BA_DEF_DEF_).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttributeDefaultInfo {
    pub name: String,
    pub value: AttributeValueInfo,
}
