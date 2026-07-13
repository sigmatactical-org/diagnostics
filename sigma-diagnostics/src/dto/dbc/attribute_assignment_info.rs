use serde::Serialize;

use super::{AttributeTargetInfo, AttributeValueInfo};

/// Attribute value assignment (BA_).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttributeAssignmentInfo {
    pub name: String,
    pub target: AttributeTargetInfo,
    pub value: AttributeValueInfo,
}
