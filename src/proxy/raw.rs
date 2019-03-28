//! Intermediary representation of packets

use crate::adapters::Error as AdapterError;
use crate::mappings::Mappings;
use crate::packets::{InternalPacketId, Packet};
use bytes::{Bytes, IntoBuf};
use failure_derive::Fail;
use std::result::Result as StdResult;

/// A "raw" packet, which has been received and decrypted, but not yet parsed
/// into a `Packet`. This intermediary representation allows for more efficient
/// and robust communication: A packet can be received and identified, but only
/// fully deserialized into a `Packet` if necessary, reducing overhead.
/// Additionally, if a packet cannot be successfully deserialized for any
/// reason, it may still be relayed as a `RawPacket`, allowing for basic fault
/// tolerance.
#[derive(Debug, Clone)]
pub struct RawPacket {
    bytes: Bytes,
}

/// An error converting a `RawPacket` to or from a `Packet`
#[derive(Debug, Fail)]
pub enum Error {
    /// The conversion failed because there was no mapping for the raw packet ID
    /// to an internal packet ID
    #[fail(display = "No mapping for game packet ID {}", _0)]
    UnmappedGameId(u8),

    /// The conversion failed because there was no mapping for the internal
    /// packet ID to a game packet ID
    #[fail(display = "No mapping for internal packet ID {:?}", _0)]
    UnmappedInternalId(InternalPacketId),

    /// The `NetworkAdapter` failed to convert the packet
    #[fail(display = "Adapter error: {}", _0)]
    AdapterError(AdapterError),
}

/// The result of a conversion of a `RawPacket` to or from a `Packet`
pub type Result<T> = StdResult<T, Error>;

impl RawPacket {
    /// Create a new raw packet from the given bytes. The first four bytes of
    /// the buffer are the big-endian integer length of the packet. THe fifth
    /// byte is the game packet ID. Any remaining bytes represent the contents
    /// of the packet, which should be in their decrypted form.
    pub(crate) fn new(bytes: Bytes) -> RawPacket {
        debug_assert!(bytes.len() >= 5, "packet must be at least 5 bytes");
        Self { bytes }
    }

    /// Convert this packet back into the underlying `Bytes`. Check the `new`
    /// method for details on the structure.
    pub(crate) fn into_bytes(self) -> Bytes {
        self.bytes
    }

    /// Get the game ID representing this packet type
    pub fn game_id(&self) -> u8 {
        self.bytes[4]
    }

    /// Get the decrypted binary contents of this packet
    pub fn contents(&self) -> Bytes {
        self.bytes.slice_from(5)
    }

    /// Attempt to convert this raw packet into a deserialized packet using
    /// the given `mappings`.
    pub fn to_packet(&self, mappings: &Mappings) -> Result<Packet> {
        let game_id = self.game_id();

        if let Some(id) = mappings.get_internal_id(game_id) {
            Packet::from_bytes(id, &mut self.contents().into_buf()).map_err(Error::AdapterError)
        } else {
            Err(Error::UnmappedGameId(game_id))
        }
    }

    /// Attempt to convert the given `packet` into a `RawPacket` using the given
    /// `mappings`
    pub fn from_packet(packet: Packet, mappings: &Mappings) -> Result<RawPacket> {
        let internal_id = packet.get_internal_id();

        if let Some(game_id) = mappings.get_game_id(internal_id) {
            // reserve 4 bytes for the size
            let mut buf = vec![0u8; 4];

            // store the game id
            buf.push(game_id);

            // encode packet
            packet.to_bytes(&mut buf).map_err(Error::AdapterError)?;

            // store packet length
            let len = buf.len() as u32;
            (&mut buf[0..4]).copy_from_slice(&len.to_be_bytes()[..]);

            // convert it into a RawPacket
            Ok(RawPacket::new(buf.into()))
        } else {
            Err(Error::UnmappedInternalId(internal_id))
        }
    }
}
