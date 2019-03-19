//! Mappings for game IDs, objects, etc

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mappings {
    /// The unified RC4 key for network communication
    pub unified_rc4: String,
}
