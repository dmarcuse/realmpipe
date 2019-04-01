use crate::mappings::Mappings;
use crate::packets::{Downcast, Packet, PacketData};
use crate::proxy::raw::{RawPacket, Result as PacketResult};
use log::warn;

/// A wrapper around a `RawPacket`, which may be automatically downcast to a
/// `Packet` instance.
pub struct AutoPacket<'a> {
    raw: RawPacket,
    mappings: &'a Mappings,
    decoded: Option<PacketResult<Packet>>,
}

impl<'a> AutoPacket<'a> {
    /// Create a new `AutoPacket` wrapping the given `RawPacket`
    pub fn new(raw: RawPacket, mappings: &'a Mappings) -> Self {
        Self {
            raw,
            mappings,
            decoded: None,
        }
    }

    /// Get the underlying `RawPacket` instance
    pub fn get_raw(&self) -> &RawPacket {
        &self.raw
    }

    /// Get the mappings used by this `AutoPacket`
    pub fn get_mappings(&self) -> &Mappings {
        self.mappings
    }

    /// Attempt to downcast this packet into a concrete type
    pub fn downcast<'b, T>(&'b mut self) -> Option<&'b T>
    where
        T: PacketData + 'b,
        &'b Packet: Downcast<&'b T>,
    {
        // get the internal ID
        let id = self.mappings.get_internal_id(self.raw.game_id())?;

        // check that the ID matches the desired one
        if id != T::INTERNAL_ID {
            return None;
        }

        // check that we have a stored result
        if let None = self.decoded {
            // attempt to downcast it
            self.decoded = Some(self.raw.to_packet(self.mappings));

            // if the result was an error, log it
            if let Some(Err(e)) = &self.decoded {
                warn!(
                    "Error decoding packet of type {:?}: {:?}. Contents: {:#x?}",
                    id,
                    e,
                    self.raw.contents()
                )
            }
        }

        // by this point, we have a packet result and just need to handle it
        if self.decoded.as_ref().unwrap().is_ok() {
            // we have a packet, unwrap and downcast it
            self.decoded
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .downcast_ref()
        } else {
            // we have an error, just ignore it and return None
            None
        }
    }
}
