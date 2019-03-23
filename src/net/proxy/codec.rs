//! A codec to frame and encrypt/decrypt ROTMG packets

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

impl Codec {
    pub fn new_client(mappings: &Mappings) -> Self {
        let (recv_rc4, send_rc4) = mappings.get_ciphers();
        Self { recv_rc4, send_rc4 }
    }

    pub fn new_server(mappings: &Mappings) -> Self {
        let (send_rc4, recv_rc4) = mappings.get_ciphers();
        Self { recv_rc4, send_rc4 }
    }
}

#[derive(Debug, Fail)]
pub enum DecodeError {
    #[fail(display = "IO error: {}", _0)]
    IoError(IoError),
}

impl From<IoError> for DecodeError {
    fn from(e: IoError) -> Self {
        DecodeError::IoError(e)
    }
}

impl Decoder for Codec {
    type Item = Bytes;
    type Error = DecodeError;

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
        Ok(Some(decrypted.into()))
    }
}

/// An error that occurred while writing a packet
#[derive(Debug, Fail)]
pub enum EncodeError {
    /// A low level IO error
    #[fail(display = "IO error: {}", _0)]
    IoError(IoError),

    #[fail(display = "Packet was too long ({})", _0)]
    TooLong(usize),
}

impl From<IoError> for EncodeError {
    fn from(e: IoError) -> Self {
        EncodeError::IoError(e)
    }
}

impl Encoder for Codec {
    type Item = Bytes;
    type Error = EncodeError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
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
            Err(EncodeError::TooLong(item.len()))
        }
    }
}
