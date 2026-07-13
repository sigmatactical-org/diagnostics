/// Row for live frame stream (Slint UI).
#[derive(Debug, Clone, Default)]
pub struct LiveFrameRow {
    pub timestamp: String,
    pub can_id: String,
    pub dlc: String,
    pub data_hex: String,
    pub flags: String,
}
