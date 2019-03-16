use std::io::{Read, Result as IoResult, Write};
use std::mem::size_of;

/// An adapter used to de/serialize bytes from ROTMG networking
pub trait Adapter: Sized {
    fn read(from: &mut dyn Read) -> IoResult<Self>;

    fn write(self, to: &mut dyn Write) -> IoResult<()>;
}

/// Define adapters for given types which can be converted to/from big endian already
macro_rules! simple_adapter {
    ($($type:ty),*) => {
        $(
            impl Adapter for $type {
                fn read(from: &mut dyn Read) -> IoResult<Self> {
                    let mut bytes = [0u8; size_of::<Self>()];
                    from.read_exact(&mut bytes[..])?;
                    Ok(Self::from_be_bytes(bytes))
                }

                fn write(self, to: &mut dyn Write) -> IoResult<()> {
                    to.write_all(&self.to_be_bytes()[..])
                }
            }
        )*
    };
}

simple_adapter! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128
}

impl Adapter for bool {
    fn read(from: &mut dyn Read) -> IoResult<Self> {
        Adapter::read(from).map(|b: u8| b != 0)
    }

    fn write(self, to: &mut dyn Write) -> IoResult<()> {
        Adapter::write(self as u8, to)
    }
}

impl Adapter for f32 {
    fn read(from: &mut dyn Read) -> IoResult<Self> {
        Adapter::read(from).map(Self::from_bits)
    }

    fn write(self, to: &mut dyn Write) -> IoResult<()> {
        Adapter::write(self.to_bits(), to)
    }
}

impl Adapter for f64 {
    fn read(from: &mut dyn Read) -> IoResult<Self> {
        Adapter::read(from).map(Self::from_bits)
    }

    fn write(self, to: &mut dyn Write) -> IoResult<()> {
        Adapter::write(self.to_bits(), to)
    }
}

impl Adapter for String {
    fn read(from: &mut dyn Read) -> IoResult<Self> {
        let size: u16 = Adapter::read(from)?;
        let mut bytes = vec![0u8; size as usize];
        from.read_exact(&mut bytes[..])?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn write(self, to: &mut dyn Write) -> IoResult<()> {
        Adapter::write(self.len() as u16, to)?;
        to.write_all(self.as_bytes())
    }
}
