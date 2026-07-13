use serde::Serialize;

/// Node (ECU) definition from DBC.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeInfo {
    pub name: String,
    pub comment: Option<String>,
}
