use serde::Serialize;

/// Attribute value type with constraints.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AttributeValueType {
    #[serde(rename = "int")]
    Int { min: i64, max: i64 },
    #[serde(rename = "hex")]
    Hex { min: i64, max: i64 },
    #[serde(rename = "float")]
    Float { min: f64, max: f64 },
    #[serde(rename = "string")]
    String,
    #[serde(rename = "enum")]
    Enum { values: Vec<String> },
}
