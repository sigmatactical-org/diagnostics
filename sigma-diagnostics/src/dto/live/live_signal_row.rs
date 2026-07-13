/// Row for live signal monitor (Slint UI).
#[derive(Debug, Clone, Default)]
pub struct LiveSignalRow {
    pub message_name: String,
    pub signal_name: String,
    pub value: String,
    pub unit: String,
    pub min_value: String,
    pub max_value: String,
}
