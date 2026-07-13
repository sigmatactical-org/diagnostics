//! DTOs for the live-capture views: statistics plus the pre-rendered rows and
//! HTML the frontend and Slint UI bind to.

mod capture_stats_dto;
mod dbc_message_row;
mod dbc_node_row;
mod dbc_signal_row;
mod frame_table_row;
mod live_capture_display;
mod live_capture_update;
mod live_error_row;
mod live_frame_row;
mod live_message_row;
mod live_signal_row;
mod signal_table_row;
mod stats_html;

pub use capture_stats_dto::CaptureStatsDto;
pub use dbc_message_row::DbcMessageRow;
pub use dbc_node_row::DbcNodeRow;
pub use dbc_signal_row::DbcSignalRow;
pub use frame_table_row::FrameTableRow;
pub use live_capture_display::LiveCaptureDisplay;
pub use live_capture_update::LiveCaptureUpdate;
pub use live_error_row::LiveErrorRow;
pub use live_frame_row::LiveFrameRow;
pub use live_message_row::LiveMessageRow;
pub use live_signal_row::LiveSignalRow;
pub use signal_table_row::SignalTableRow;
pub use stats_html::StatsHtml;
