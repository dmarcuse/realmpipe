use crate::mappings::Mappings;
use crate::packets::{Downcast, Packet, PacketData};
use crate::proxy::raw::{RawPacket, Result as PacketResult};
use log::warn;

/// A wrapper around a `RawPacket`, which may be automatically converted to
/// a concrete packet type when necessary.
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

    /// Consume this packet and return the raw packet
    pub fn into_raw(self) -> RawPacket {
        self.raw
    }

    /// Get the mappings used by this `AutoPacket`
    pub fn get_mappings(&self) -> &Mappings {
        self.mappings
    }

    /// Get this packet as a `Packet`
    pub fn get_any(&mut self) -> Option<&Packet> {
        let id = self.mappings.get_internal_id(self.raw.game_id());

        if let Some(id) = id {
            if self.decoded.is_none() {
                // decode the packet
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

            // by this point, we have a packet result
            if self.decoded.as_ref().unwrap().is_ok() {
                // we have a packet, unwrap and downcast it
                Some(self.decoded.as_ref().unwrap().as_ref().unwrap())
            } else {
                None
            }
        } else {
            None
        }
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

        // decode (if necessary) and downcast the packet
        self.get_any().and_then(|p| p.downcast_ref())
    }
}
