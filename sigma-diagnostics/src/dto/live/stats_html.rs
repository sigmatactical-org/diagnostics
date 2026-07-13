use serde::{Deserialize, Serialize};

/// Pre-rendered stats strings for frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsHtml {
    pub message_count: String,
    pub frame_count: String,
    pub frame_rate: String,
    pub elapsed: String,
}
