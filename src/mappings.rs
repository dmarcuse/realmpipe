//! Mappings for game IDs, objects, etc

use crate::net::packets::InternalPacketId;
use failure_derive::Fail;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result::Result as StdResult;

/// Mappings extracted from the official ROTMG client needed to properly proxy
/// traffic
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mappings {
    /// The unified RC4 key for network communication
    unified_rc4: String,

    /// Map of game packet IDs to internal packet IDs
    game_packet_ids: HashMap<u8, InternalPacketId>,

    /// Map of internal packet IDs to game packet IDs
    internal_packet_ids: HashMap<InternalPacketId, u8>,
}

/// An error constructing mappings
#[derive(Debug, Clone, Fail)]
pub enum Error {
    /// Caused by an invalid RC4 key
    #[fail(display = "RC4 key is invalid: {}", _0)]
    InvalidRC4Key(String),

    /// Caused by packet mappings that are not strictly one-to-one
    #[fail(display = "Invalid packet mappings: {}", _0)]
    InvalidPacketMappings(String),
}

/// A result wrapping either successfully constructed mappings, or an error
pub type Result = StdResult<Mappings, Error>;

impl Mappings {
    /// Create a new set of mappings
    pub fn new(unified_rc4: String, game_packet_ids: HashMap<u8, InternalPacketId>) -> Result {
        if unified_rc4.len() != 52 {
            Err(Error::InvalidRC4Key(unified_rc4))
        } else {
            let mut internal_packet_ids = HashMap::with_capacity(game_packet_ids.len());

            for (game_id, internal_id) in &game_packet_ids {
                if let Some(old) = internal_packet_ids.insert(*internal_id, *game_id) {
                    let name = internal_id.get_name();
                    let msg = format!(
                        "Duplicate packet mapping for {}: {} and {}",
                        name, old, game_id
                    );

                    return Err(Error::InvalidPacketMappings(msg));
                }
            }

            Ok(Mappings {
                unified_rc4,
                game_packet_ids,
                internal_packet_ids,
            })
        }
    }

    /// Get the RC4 keys for encrypting packets
    pub fn get_keys(&self) -> (&str, &str) {
        let half = self.unified_rc4.len() / 2;
        (&self.unified_rc4[..half], &self.unified_rc4[half..])
    }

    /// Map a game packet ID to an internal packet ID
    pub fn get_internal_id(&self, game_id: u8) -> Option<InternalPacketId> {
        self.game_packet_ids.get(&game_id).map(|i| i.clone())
    }

    /// Map an internal packet ID to a game packet ID
    pub fn get_game_id(&self, internal_id: InternalPacketId) -> Option<u8> {
        self.internal_packet_ids
            .get(&internal_id)
            .map(|i| i.clone())
    }
}
