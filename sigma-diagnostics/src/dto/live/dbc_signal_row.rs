/// DBC signal list row.
#[derive(Debug, Clone, Default)]
pub struct DbcSignalRow {
    pub name: String,
    pub start_bit: String,
    pub length: String,
    pub factor: String,
    pub unit: String,
}
