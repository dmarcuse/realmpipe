//! Implementations of `NetworkAdapter` for more complicated types

use super::prelude::*;

/// Will only decode data if >0 bytes are remaining in the buffer, and will
/// only encode data when `Some(T)` variant is passed.
impl<T: NetworkAdapter> NetworkAdapter for Option<T> {
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        if bytes.remaining() == 0 {
            Ok(None)
        } else {
            NetworkAdapter::get_be(bytes).map(Some)
        }
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        match self {
            Some(v) => v.put_be(bytes),
            None => Ok(()),
        }
    }
}
