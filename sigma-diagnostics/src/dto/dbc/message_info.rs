use serde::Serialize;

use super::SignalInfo;

/// Message definition from DBC.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MessageInfo {
    pub id: u32,
    pub is_extended: bool,
    pub name: String,
    pub dlc: u8,
    pub sender: String,
    pub signals: Vec<SignalInfo>,
    /// Comment from CM_ BO_ entry
    pub comment: Option<String>,
}
