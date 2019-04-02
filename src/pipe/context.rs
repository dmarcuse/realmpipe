use crate::packets::Packet;

/// Context for a received packet
pub struct PacketContext {
    pub(crate) cancelled: bool,
    pub(crate) extra: Vec<Packet>,
}

impl PacketContext {
    /// Request that the given packet be cancelled, preventing it from being
    /// sent to the other side of the connection. The packet will be cancelled
    /// if any plugin calls this method, even if none of the other plugins do.
    /// However, any remaining plugin callbacks will still be called for
    /// cancelled packets.
    pub fn cancel_packet(&mut self) {
        self.cancelled = true;
    }

    /// Send the given packet to the appropriate side of the connection. The
    /// packet will not trigger plugin callbacks, and will be sent directly.
    /// If an error occurs encoding the packet, the error will be emitted as a
    /// warning, and the packet will be skipped.
    pub fn send_packet(&mut self, packet: Packet) {
        self.extra.push(packet);
    }
}

impl Default for PacketContext {
    fn default() -> Self {
        Self {
            cancelled: false,
            extra: Vec::with_capacity(0),
        }
    }
}
