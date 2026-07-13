use serde::Serialize;

/// Attribute value (can be int, float, or string).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AttributeValueInfo {
    Int(i64),
    Float(f64),
    String(String),
}
