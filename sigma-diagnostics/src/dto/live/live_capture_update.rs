use serde::{Deserialize, Serialize};

use super::{CaptureStatsDto, StatsHtml};

/// Periodic update sent during live capture (legacy HTML rendering).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCaptureUpdate {
    pub stats: CaptureStatsDto,
    /// Pre-rendered HTML for message monitor table body.
    pub messages_html: String,
    /// Pre-rendered HTML for signal monitor container.
    pub signals_html: String,
    /// Pre-rendered HTML for frame stream table body.
    pub frames_html: String,
    /// Pre-rendered HTML for error monitor table body.
    pub errors_html: String,
    /// Pre-formatted stats strings.
    pub stats_html: StatsHtml,
    /// Badge counts for tabs.
    pub message_count: u32,
    pub signal_count: u32,
    pub frame_count: usize,
    pub error_count: u32,
}
