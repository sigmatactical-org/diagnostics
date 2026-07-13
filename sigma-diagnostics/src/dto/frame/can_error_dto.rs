use serde::{Deserialize, Serialize};

/// CAN bus error frame for frontend communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanErrorDto {
    pub timestamp: f64,
    pub channel: String,
    pub error_type: String,
    pub details: String,
}

#[cfg(target_os = "linux")]
impl CanErrorDto {
    /// Create from socketcan error frame.
    pub fn from_error_frame(
        frame: socketcan::CanErrorFrame,
        timestamp: f64,
        channel: &str,
    ) -> Self {
        use socketcan::CanError;

        // Convert frame to CanError using the From trait
        let error: CanError = frame.into();

        let (error_type, details) = match error {
            CanError::TransmitTimeout => ("TX Timeout", "Transmit timeout".to_string()),
            CanError::LostArbitration(bit) => ("Lost Arbitration", format!("at bit {}", bit)),
            CanError::ControllerProblem(err) => ("Controller", format!("{:?}", err)),
            CanError::ProtocolViolation { vtype, location } => (
                "Protocol Violation",
                format!("{:?} at {:?}", vtype, location),
            ),
            CanError::TransceiverError => ("Transceiver", "Transceiver error".to_string()),
            CanError::NoAck => ("No ACK", "No acknowledgment received".to_string()),
            CanError::BusOff => ("Bus Off", "Controller is bus-off".to_string()),
            CanError::BusError => ("Bus Error", "Bus error occurred".to_string()),
            CanError::Restarted => ("Restarted", "Controller restarted".to_string()),
            CanError::DecodingFailure(msg) => ("Decode Error", msg.to_string()),
            CanError::Unknown(code) => ("Unknown", format!("Error code: 0x{:08X}", code)),
        };

        Self {
            timestamp,
            channel: channel.to_string(),
            error_type: error_type.to_string(),
            details,
        }
    }
}
