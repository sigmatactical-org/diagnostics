use serde::{Deserialize, Serialize};

use super::DecodedSignalDto;

/// Response from decode_frames command, including any errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeResponse {
    pub signals: Vec<DecodedSignalDto>,
    pub errors: Vec<String>,
}
