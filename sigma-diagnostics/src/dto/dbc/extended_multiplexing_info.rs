use serde::Serialize;

/// Extended multiplexing entry (SG_MUL_VAL_).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtendedMultiplexingInfo {
    pub message_id: u32,
    pub signal_name: String,
    pub multiplexer_signal: String,
    pub ranges: Vec<(u64, u64)>,
}
