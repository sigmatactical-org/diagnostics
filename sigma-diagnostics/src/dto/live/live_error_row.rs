/// Row for live error monitor (Slint UI).
#[derive(Debug, Clone, Default)]
pub struct LiveErrorRow {
    pub timestamp: String,
    pub channel: String,
    pub error_type: String,
    pub details: String,
    pub count: String,
}
