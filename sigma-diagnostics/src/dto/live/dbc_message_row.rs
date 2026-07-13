/// DBC message list row.
#[derive(Debug, Clone, Default)]
pub struct DbcMessageRow {
    pub id: String,
    pub name: String,
    pub dlc: String,
    pub sender: String,
    pub signal_count: String,
}
