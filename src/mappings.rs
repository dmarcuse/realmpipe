//! Mappings for game IDs, objects, etc

use crate::net::packets::InternalPacketId;
use bimap::BiHashMap;
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

    /// The mappings between game packet IDs and internal packet IDs
    packet_mappings: BiHashMap<u8, InternalPacketId>,
}

/// An error constructing mappings
#[derive(Debug, Clone, Fail)]
pub enum Error {
    /// Caused by an invalid RC4 key
    #[fail(display = "RC4 key is invalid: {}", _0)]
    InvalidRC4Key(String),
}

/// A result wrapping either successfully constructed mappings, or an error
pub type Result = StdResult<Mappings, Error>;

impl Mappings {
    /// Create a new set of mappings
    ///
    /// # Arguments
    /// `hex_rc4` - the hex-encoded RC4 key to use to encrypt/decrypt packets
    /// `packet_mappings` - bidirectional mappings between game packet IDs and
    /// internal packet IDs.
    pub fn new(hex_rc4: String, packet_mappings: BiHashMap<u8, InternalPacketId>) -> Result {
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

        Ok(Self {
            binary_rc4,
            packet_mappings,
        })
    }

    /// Get the complete mapping table for packet IDs
    pub fn get_packet_mappings(&self) -> &BiHashMap<u8, InternalPacketId> {
        &self.packet_mappings
    }

    /// Map a game packet ID to an internal packet ID, if one is present
    pub fn get_internal_id(&self, game_id: u8) -> Option<InternalPacketId> {
        self.packet_mappings.get_by_left(&game_id).map(|&i| i)
    }

    /// Map an internal packet ID to a game packet ID, if one is present
    pub fn get_game_id(&self, internal_id: InternalPacketId) -> Option<u8> {
        self.packet_mappings.get_by_right(&internal_id).map(|&i| i)
    }

    /// Get the two RC4 ciphers
    pub fn get_ciphers(&self) -> (Rc4, Rc4) {
        let (key0, key1) = self.binary_rc4.split_at(RC4_LEN / 2);
        (Rc4::new(key0), Rc4::new(key1))
    }
}
