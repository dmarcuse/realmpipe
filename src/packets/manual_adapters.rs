//! Manually implemented packet adapters

use super::server::Pic;
use crate::adapters::prelude::*;

impl NetworkAdapter for Pic {
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        let w = u32::get_be(bytes)?;
        let h = u32::get_be(bytes)?;

        let reqd_bytes = (w as usize) * (h as usize) * 4;

        if bytes.remaining() >= reqd_bytes {
            let mut bitmap_data = vec![0u8; reqd_bytes];
            bytes.copy_to_slice(&mut bitmap_data[..]);
            Ok(Self { w, h, bitmap_data })
        } else {
            Err(Error::InsufficientData {
                remaining: bytes.remaining(),
                needed: reqd_bytes,
            })
        }
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        self.w.put_be(bytes)?;
        self.h.put_be(bytes)?;
        bytes.put_slice(&self.bitmap_data[..]);
        Ok(())
    }
}
