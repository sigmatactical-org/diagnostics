use super::{
    CaptureStatsDto, LiveErrorRow, LiveFrameRow, LiveMessageRow, LiveSignalRow, StatsHtml,
};

/// Structured live capture data for Slint binding.
#[derive(Debug, Clone, Default)]
pub struct LiveCaptureDisplay {
    pub stats: CaptureStatsDto,
    pub messages: Vec<LiveMessageRow>,
    pub signals: Vec<LiveSignalRow>,
    pub frames: Vec<LiveFrameRow>,
    pub errors: Vec<LiveErrorRow>,
    pub stats_html: StatsHtml,
    pub message_count: u32,
    pub signal_count: u32,
    pub frame_count: usize,
    pub error_count: u32,
}
