use serde::Serialize;

/// Single value description entry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueDescriptionEntry {
    pub value: i64,
    pub description: String,
}
