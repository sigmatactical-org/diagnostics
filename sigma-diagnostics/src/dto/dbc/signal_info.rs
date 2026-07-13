use serde::Serialize;

/// Signal definition from DBC.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SignalInfo {
    pub name: String,
    pub start_bit: u32,
    pub length: u32,
    pub byte_order: String,
    pub is_signed: bool,
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub receivers: Vec<String>,
    pub is_multiplexer: bool,
    pub multiplexer_value: Option<u64>,
    /// Comment from CM_ SG_ entry
    pub comment: Option<String>,
}
