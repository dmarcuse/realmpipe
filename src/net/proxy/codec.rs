//! A codec to frame and encrypt/decrypt ROTMG packets

use super::raw::RawPacket;
use crate::mappings::Mappings;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crypto::rc4::Rc4;
use crypto::symmetriccipher::SynchronousStreamCipher;
use failure_derive::Fail;
use num::ToPrimitive;
use std::convert::From;
use std::io::{Cursor, Error as IoError};
use std::sync::Arc;
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

    /// The packet was too long to be encoded
    #[fail(display = "Packet was too long ({})", _0)]
    TooLong(usize),
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
            // we don't have enough bytes to even know the packet size
            return Ok(None);
        }

        let packet_size = {
            let mut cursor = Cursor::new(&buf);
            cursor.get_u32_be() as usize
        };

        if buf.len() < 4 + packet_size {
            // we haven't received the full packet yet
            return Ok(None);
        }

        // extract the encrypted packet
        let mut decrypted = vec![0u8; packet_size];
        self.recv_rc4
            .process(&buf[4..packet_size + 4], &mut decrypted);

        // we have the decrypted packet, yield it
        Ok(Some(RawPacket::new(decrypted.into())))
    }
}

impl Encoder for Codec {
    type Item = RawPacket;
    type Error = CodecError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // convert the packet back into bytes
        let item = item.into_bytes();

        if let Some(packet_size) = item.len().to_u32() {
            // reserve some space to store the packet
            dst.reserve(4 + (packet_size as usize));

            // write the packet length
            dst.put_u32_be(packet_size);

            // encrypt the packet contents
            let mut encrypted = vec![0u8; packet_size as usize];
            self.send_rc4.process(&item, &mut encrypted);

            // write the packet contents
            dst.extend_from_slice(&encrypted);

            Ok(())
        } else {
            Err(CodecError::TooLong(item.len()))
        }
    }
}
