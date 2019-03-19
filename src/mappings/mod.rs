//! Mappings for game IDs, objects, etc

use serde::{Deserialize, Serialize};

/// Mappings extracted from the official ROTMG client needed to properly proxy
/// traffic
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mappings {
    /// The unified RC4 key for network communication
    pub(crate) unified_rc4: String,
}
