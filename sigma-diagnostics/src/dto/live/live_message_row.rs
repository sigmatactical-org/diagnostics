/// Row for live message monitor (Slint UI).
#[derive(Debug, Clone, Default)]
pub struct LiveMessageRow {
    pub can_id: String,
    pub message_name: String,
    pub data_hex: String,
    pub count: String,
    pub rate: String,
}
