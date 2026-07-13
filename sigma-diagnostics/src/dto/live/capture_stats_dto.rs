use serde::{Deserialize, Serialize};

/// Capture statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptureStatsDto {
    pub frame_count: u64,
    pub message_count: u32,
    pub signal_count: u32,
    pub frame_rate: f64,
    pub elapsed_secs: f64,
    pub capture_file: Option<String>,
}
