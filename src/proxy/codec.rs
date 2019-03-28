//! A codec to frame and encrypt/decrypt ROTMG packets

use super::raw::RawPacket;
use crate::mappings::Mappings;
use crate::rc4::Rc4;
use bytes::{Buf, BytesMut};
use failure_derive::Fail;
use std::convert::From;
use std::io::{Cursor, Error as IoError};
use tokio::codec::{Decoder, Encoder};

/// The codec for framing and encrypting/decrypting ROTMG packets. This struct
/// stores the RC4 cipher states for the sending and receiving functionality.
pub struct Codec {
    recv_rc4: Rc4,
    send_rc4: Rc4,
}

/// An error that occurred while writing a packet
#[derive(Debug, Fail)]
pub enum CodecError {
    /// A low level IO error
    #[fail(display = "IO error: {}", _0)]
    IoError(IoError),
}

impl From<IoError> for CodecError {
    fn from(e: IoError) -> Self {
        CodecError::IoError(e)
    }
}

impl Codec {
    /// Construct a new codec for communicating ith the game client.
    pub fn new_client(mappings: &Mappings) -> Self {
        let (recv_rc4, send_rc4) = mappings.get_ciphers();
        Self { recv_rc4, send_rc4 }
    }

    /// Construct a new client for communicating with the game server.
    pub fn new_server(mappings: &Mappings) -> Self {
        let (send_rc4, recv_rc4) = mappings.get_ciphers();
        Self { recv_rc4, send_rc4 }
    }
}

impl Decoder for Codec {
    type Item = RawPacket;
    type Error = CodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.len() < 4 {
            // we need more bytes to determine the packet size
            return Ok(None);
        }

        // get the total length of the packet
        let packet_size = {
            let mut cursor = Cursor::new(&buf);
            cursor.get_u32_be() as usize
        };

        // todo: turn this into a CodecError?
        debug_assert!(packet_size >= 5, "invalid packet size: {}", packet_size);

        // we haven't received the full packet yet
        if buf.len() < packet_size {
            return Ok(None);
        }

        // full packet has been received
        // remove the entire packet from the buffer
        let mut packet = buf.split_to(packet_size);

        // decrypt the packet contents
        self.recv_rc4.process(&mut packet[5..]);

        // we have the decrypted packet, yield it
        Ok(Some(RawPacket::new(packet.freeze())))
    }
}

impl Encoder for Codec {
    type Item = RawPacket;
    type Error = CodecError;

    fn encode(&mut self, packet: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // convert the packet back into bytes
        let packet = packet.into_bytes();

        // make the packet mutable so we can encrypt it
        let mut packet = BytesMut::from(packet);

        // encrypt the packet contents
        self.send_rc4.process(&mut packet[5..]);

        // finally, write the packet
        dst.extend_from_slice(&packet[..]);
        Ok(())
    }
}
