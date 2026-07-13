/// Table row for MDF4 / filtered frames.
#[derive(Debug, Clone, Default)]
pub struct FrameTableRow {
    pub index: i32,
    pub timestamp: String,
    pub can_id: String,
    pub channel: String,
    pub dlc: String,
    pub data_hex: String,
    pub message_name: String,
}
