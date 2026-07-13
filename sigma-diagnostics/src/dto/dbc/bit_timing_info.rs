use serde::Serialize;

/// Bit timing configuration (BS_ section).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BitTimingInfo {
    pub baudrate: u32,
    pub btr1: u32,
    pub btr2: u32,
}
