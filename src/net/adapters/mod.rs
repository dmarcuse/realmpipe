//! Adapters for encoding and decoding data as bytes for transmission over the
//! network.

mod primitives;
mod rle;
mod complex;

use self::prelude::*;
pub use self::rle::{RLEString, RLE};
use failure::Fail;
use std::convert::From;

pub(in crate) mod prelude {
    pub use super::rle::{RLEString, RLE};
    pub use super::{Error, NetworkAdapter, Result};
    pub use bytes::{Buf, BufMut};
}

/// An error occuring when converting a type to or from big endian bytes
#[derive(Debug, Fail)]
pub enum Error {
    /// Insufficient data left in the buffer to decode the given type
    #[fail(
        display = "Not enough data left in buffer: need {} bytes, {} bytes remaining",
        needed, remaining
    )]
    InsufficientData {
        /// The number of bytes remaining in the buffer
        remaining: usize,

        /// The number of bytes needed - note that this is not necessarily
        /// the number of bytes needed to decode the entire object, just part
        /// of it.
        needed: usize,
    },

    /// The given data is invalid and cannot be encoded or decoded properly
    #[fail(display = "Invalid data: {}", _0)]
    InvalidData(String),

    /// A different type of error
    #[fail(display = "{}", _0)]
    Other(failure::Error),
}

/// The result of encoding or decoding data using a `NetworkAdapter`
pub type Result<T> = std::result::Result<T, Error>;

/// A trait providing functionality for converting a type to or from buffers of
/// bytes in big endian notation.
pub trait NetworkAdapter: Sized + 'static {
    /// Decode an instance from the given buffer. The amount of data remaining
    /// in the buffer should be checked and AdapterError::InsufficientData
    /// should be returned when appropriate.
    fn get_be(bytes: &mut dyn Buf) -> Result<Self>;

    /// Encode an instance to the given buffer. It may be assumed that the
    /// buffer will be large enough to store the entire encoded sequence, so no
    /// size checks are necessary.
    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()>;
}

impl From<failure::Error> for Error {
    fn from(f: failure::Error) -> Self {
        Error::Other(f)
    }
}
