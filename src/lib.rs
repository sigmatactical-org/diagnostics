//! Sigma Tactical CAN Viewer library.
//!
//! Native Slint desktop application for MDF4 inspection, SocketCAN capture,
//! and DBC editing.

slint::include_modules!();

pub mod app;
pub mod config;
pub mod dbc_export;
pub mod decode;
pub mod dto;
pub mod live_capture;
pub mod services;
pub mod state;

pub use state::{AppState, InitialFiles};

pub use dto::{CanBpfFilter, CanFrameDto, DecodedSignalDto};

pub use services::filter::{
    build_message_cache_from_dbc, calculate_frame_stats, detect_dlc, filter_frames,
    filter_frames_with_cache, get_message_counts, match_data_pattern, parse_data_pattern,
    DbcMessageCache, DbcMessageInfo, DlcDetectionResult, FilterConfig, FilterResult, FrameStats,
    MatchStatus, MessageCount,
};

pub use config::SessionConfig;
pub use services::mdf::parse_can_dataframe;
