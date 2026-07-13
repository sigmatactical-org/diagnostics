use serde::{Deserialize, Serialize};

/// Kernel-level CAN filter (BPF) for socket filtering.
///
/// Filters are applied at the kernel level before frames reach userspace,
/// providing efficient hardware-accelerated filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanBpfFilter {
    /// CAN ID to match
    pub can_id: u32,
    /// Mask for matching (1 bits = must match, 0 bits = don't care)
    pub mask: u32,
    /// If true, filter matches extended (29-bit) IDs
    pub is_extended: bool,
    /// If true, invert the filter (reject matching frames)
    pub inverted: bool,
}
