use serde::Serialize;

use super::{
    AttributeAssignmentInfo, AttributeDefaultInfo, AttributeDefinitionInfo, BitTimingInfo,
    ExtendedMultiplexingInfo, MessageInfo, NodeInfo, SignalValueDescriptions,
};

/// Full DBC structure for display/editing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DbcInfo {
    pub version: Option<String>,
    pub bit_timing: Option<BitTimingInfo>,
    pub comment: Option<String>,
    pub nodes: Vec<NodeInfo>,
    pub messages: Vec<MessageInfo>,
    pub value_descriptions: Vec<SignalValueDescriptions>,
    pub attribute_definitions: Vec<AttributeDefinitionInfo>,
    pub attribute_defaults: Vec<AttributeDefaultInfo>,
    pub attribute_values: Vec<AttributeAssignmentInfo>,
    pub extended_multiplexing: Vec<ExtendedMultiplexingInfo>,
}
