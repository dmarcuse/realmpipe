//! Mappings for game IDs, objects, etc

use crate::net::packets::InternalPacketId;
use crypto::rc4::Rc4;
use failure_derive::Fail;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result::Result as StdResult;

/// The required length for the binary RC4 keys
const RC4_LEN: usize = 26;

/// Mappings extracted from the official ROTMG client needed to properly proxy
/// traffic
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mappings {
    /// The unified RC4 key for network communication
    binary_rc4: [u8; RC4_LEN],

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
    pub fn new(hex_rc4: String, game_packet_ids: HashMap<u8, InternalPacketId>) -> Result {
        // convert and validate RC4 key
        let binary_rc4 = match hex::decode(&hex_rc4) {
            Err(e) => return Err(Error::InvalidRC4Key(hex_rc4)),
            Ok(ref b) if b.len() != RC4_LEN => return Err(Error::InvalidRC4Key(hex_rc4)),
            Ok(b) => {
                let mut arr = [0u8; RC4_LEN];
                arr.copy_from_slice(&b);
                arr
            }
        };

        // convert packet ids
        let mut internal_packet_ids = HashMap::with_capacity(game_packet_ids.len());

        for (game_id, internal_id) in &game_packet_ids {
            if let Some(old) = internal_packet_ids.insert(*internal_id, *game_id) {
                // error - packets must be mapped as a one-to-one correspondence!
                let name = internal_id.get_name();
                let msg = format!(
                    "Duplicate packet mapping for {}: {} and {}",
                    name, old, game_id
                );

                return Err(Error::InvalidPacketMappings(msg));
            }
        }

        Ok(Mappings {
            binary_rc4,
            game_packet_ids,
            internal_packet_ids,
        })
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

    /// Get the two RC4 ciphers
    pub fn get_ciphers(&self) -> (Rc4, Rc4) {
        let (key0, key1) = self.binary_rc4.split_at(RC4_LEN / 2);
        (Rc4::new(key0), Rc4::new(key1))
    }
}
