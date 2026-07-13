/// Decoded signal row for side panel.
#[derive(Debug, Clone, Default)]
pub struct SignalTableRow {
    pub signal_name: String,
    pub value: String,
    pub unit: String,
    pub raw_value: String,
    pub description: String,
}
