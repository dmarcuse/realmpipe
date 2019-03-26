//! Implementations of `NetworkAdapter` for primitive stdlib types

use super::prelude::*;
use std::mem::size_of;

macro_rules! auto_adapters {
    ($($type:ty),* $(,)?) => {
        $(
            impl NetworkAdapter for $type {
                fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
                    if bytes.remaining() < size_of::<Self>() {
                        return Err(Error::InsufficientData {
                            remaining: bytes.remaining(),
                            needed: size_of::<Self>()
                        })
                    }

                    let mut raw = [0u8; size_of::<Self>()];
                    bytes.copy_to_slice(&mut raw[..]);
                    Ok(Self::from_be_bytes(raw))
                }

                fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
                    bytes.put_slice(&self.to_be_bytes());
                    Ok(())
                }
            }
        )*
    }
}

auto_adapters! {
    u8, u16, u32, u64,
    i8, i16, i32, i64,
}

impl NetworkAdapter for f32 {
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        u32::get_be(bytes).map(Self::from_bits)
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        self.to_bits().put_be(bytes)
    }
}

impl NetworkAdapter for f64 {
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        u64::get_be(bytes).map(Self::from_bits)
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        self.to_bits().put_be(bytes)
    }
}

impl NetworkAdapter for bool {
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        u8::get_be(bytes).map(|b| b != 0)
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        (self as u8).put_be(bytes)
    }
}
