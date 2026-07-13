use serde::{Deserialize, Serialize};

/// Serializable CAN frame for frontend communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFrameDto {
    pub timestamp: f64,
    pub channel: String,
    pub can_id: u32,
    pub is_extended: bool,
    pub is_fd: bool,
    pub brs: bool,
    pub esi: bool,
    pub dlc: u8,
    pub data: Vec<u8>,
}

impl CanFrameDto {
    /// Helper to extract CAN ID as u32 from embedded_can::Id.
    #[cfg(target_os = "linux")]
    fn id_to_u32(id: embedded_can::Id) -> u32 {
        match id {
            embedded_can::Id::Standard(id) => id.as_raw() as u32,
            embedded_can::Id::Extended(id) => id.as_raw(),
        }
    }

    /// Create from any socketcan frame type (classic CAN or CAN FD).
    /// Uses embedded_can::Frame trait for frame access.
    /// Returns None for error and remote frames.
    #[cfg(target_os = "linux")]
    pub fn from_any_frame(
        frame: &socketcan::CanAnyFrame,
        timestamp: f64,
        channel: &str,
    ) -> Option<Self> {
        // Use embedded_can::Frame trait for generic frame access
        use embedded_can::Frame;
        // Note: is_brs() and is_esi() are inherent methods on CanFdFrame

        match frame {
            socketcan::CanAnyFrame::Normal(data_frame) => Some(Self {
                timestamp,
                channel: channel.to_string(),
                can_id: Self::id_to_u32(data_frame.id()),
                is_extended: data_frame.is_extended(),
                is_fd: false,
                brs: false,
                esi: false,
                dlc: data_frame.dlc() as u8,
                data: data_frame.data().to_vec(),
            }),
            socketcan::CanAnyFrame::Remote(_) => None, // Remote frames not supported (deprecated in CAN FD)
            socketcan::CanAnyFrame::Fd(fd_frame) => Some(Self {
                timestamp,
                channel: channel.to_string(),
                can_id: Self::id_to_u32(fd_frame.id()),
                is_extended: fd_frame.is_extended(),
                is_fd: true,
                brs: fd_frame.is_brs(),
                esi: fd_frame.is_esi(),
                dlc: fd_frame.dlc() as u8,
                data: fd_frame.data().to_vec(),
            }),
            socketcan::CanAnyFrame::Error(_) => None, // Skip error frames
        }
    }

    /// Create from MDF4 channel data (classic CAN).
    pub fn from_mdf4(timestamp: f64, channel: String, can_id: u32, dlc: u8, data: Vec<u8>) -> Self {
        let is_fd = data.len() > 8 || dlc > 8;
        Self {
            timestamp,
            channel,
            can_id,
            is_extended: can_id > 0x7FF,
            is_fd,
            brs: false, // Not available in basic MDF4 data
            esi: false,
            dlc,
            data,
        }
    }

    /// Create from MDF4 channel data with CAN FD flags.
    #[allow(dead_code)]
    pub fn from_mdf4_fd(
        timestamp: f64,
        channel: String,
        can_id: u32,
        dlc: u8,
        data: Vec<u8>,
        brs: bool,
        esi: bool,
    ) -> Self {
        Self {
            timestamp,
            channel,
            can_id,
            is_extended: can_id > 0x7FF,
            is_fd: true,
            brs,
            esi,
            dlc,
            data,
        }
    }
}
