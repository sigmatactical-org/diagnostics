//! Data Transfer Objects for frontend communication.
//!
//! These types are serializable versions of internal types, used for
//! communication between the Rust backend and the JavaScript frontend.
//!
//! They are grouped by domain into submodules, but every type is re-exported
//! flat here so callers use a single `dto::TypeName` path regardless of which
//! submodule defines it:
//!
//! - [`dbc`] — a parsed DBC database for display and editing.
//! - [`frame`] — CAN frames, decoded signals, and bus errors.
//! - [`live`] — live-capture statistics and pre-rendered UI rows.

pub mod dbc;
pub mod frame;
pub mod live;

pub use dbc::{
    AttributeAssignmentInfo, AttributeDefaultInfo, AttributeDefinitionInfo, AttributeTargetInfo,
    AttributeValueInfo, AttributeValueType, BitTimingInfo, DbcInfo, ExtendedMultiplexingInfo,
    MessageInfo, NodeInfo, SignalInfo, SignalValueDescriptions, ValueDescriptionEntry,
};
pub use frame::{CanBpfFilter, CanErrorDto, CanFrameDto, DecodeResponse, DecodedSignalDto};
pub use live::{
    CaptureStatsDto, DbcMessageRow, DbcNodeRow, DbcSignalRow, FrameTableRow, LiveCaptureDisplay,
    LiveCaptureUpdate, LiveErrorRow, LiveFrameRow, LiveMessageRow, LiveSignalRow, SignalTableRow,
    StatsHtml,
};
