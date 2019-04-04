use serde::{Deserialize, Serialize};

/// Persistent configuration for realmpipe
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The currently-used client version
    client_version: Option<String>,

    /// Whether to automatically check for game client updates
    update_check: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_version: None,
            update_check: true,
        }
    }
}
