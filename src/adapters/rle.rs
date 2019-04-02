//! Implementations of `NetworkAdapter` for types that wrap variable length
//! values

use super::prelude::*;
use num::{FromPrimitive, ToPrimitive};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::iter::IntoIterator;
use std::marker::PhantomData;
use std::ops::Deref;

/// A wrapper around a given value (of type `T`) which can be converted
/// to or from big endian bytes by prefixing the data with an integer (of type
/// `S`).
pub struct RLE<T, S = u16> {
    inner: T,
    phantom: PhantomData<S>,
}

impl<T, S> RLE<T, S> {
    /// Create a new run-length encoded wrapper for the given value
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            phantom: PhantomData::default(),
        }
    }

    /// Unwrap the contained value
    pub fn unwrap(self) -> T {
        self.inner
    }

    /// Get a reference to the contained value
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
}

impl<T, S> NetworkAdapter for RLE<Vec<T>, S>
where
    T: NetworkAdapter,
    S: NetworkAdapter + ToPrimitive + FromPrimitive + Display,
{
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        // decode length
        let len = S::get_be(bytes)?;

        // attempt to convert length to usize
        if let Some(len) = len.to_usize() {
            // decode remaining items and convert to appropriate collection
            (0..len)
                .map(|_| T::get_be(bytes))
                .collect::<Result<Vec<T>>>()
                .map(|inner| Self {
                    inner,
                    phantom: PhantomData::default(),
                })
        } else {
            Err(Error::InvalidData(format!(
                "cannot cast length to usize: {}",
                len
            )))
        }
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        // attempt to convert length from a usize
        if let Some(len) = S::from_usize(self.inner.len()) {
            // encode length and then each element
            len.put_be(bytes)?;
            self.inner.into_iter().try_for_each(|i| i.put_be(bytes))
        } else {
            Err(Error::InvalidData(format!(
                "cannot cast length from usize: {}",
                self.inner.len()
            )))
        }
    }
}

// TODO: bug here? length in chars is used by game instead of length in bytes
impl<S> NetworkAdapter for RLE<String, S>
where
    S: NetworkAdapter + ToPrimitive + FromPrimitive + Display,
{
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        let bytes = RLE::<Vec<_>, S>::get_be(bytes)?.unwrap();
        String::from_utf8(bytes)
            .map(|inner| Self {
                inner,
                phantom: PhantomData::default(),
            })
            .map_err(|e| Error::Other(e.into()))
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        RLE::<Vec<_>, S>::new(self.inner.into_bytes()).put_be(bytes)
    }
}

impl<T, S> Deref for RLE<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, S> AsRef<T> for RLE<T, S> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: Debug, S> Debug for RLE<T, S> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self.inner)
    }
}

impl<T: Display, S> Display for RLE<T, S> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.inner)
    }
}

impl<T: PartialEq, S, S2> PartialEq<RLE<T, S2>> for RLE<T, S> {
    fn eq(&self, other: &RLE<T, S2>) -> bool {
        self.inner == other.inner
    }
}

impl<T: Clone, S> Clone for RLE<T, S> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::io::Cursor;
    use std::mem::size_of;

    #[test]
    fn check_rle_size() {
        assert_eq!(size_of::<RLE<Vec<u32>>>(), size_of::<Vec<u32>>());
    }

    #[test]
    fn test_rle_vec() {
        let mut buf = vec![];
        RLE::<Vec<u8>>::new(vec![1, 2, 3, 4, 5])
            .put_be(&mut buf)
            .expect("encoding error");
        assert_eq!(buf, vec![0, 5, 1, 2, 3, 4, 5]);

        let output = RLE::<Vec<u8>>::get_be(&mut Cursor::new(&buf)).expect("decoding error");
        assert_eq!(output.unwrap(), vec![1, 2, 3, 4, 5]);

        let large = (0..300).collect::<Vec<u16>>();
        assert_matches!(
            RLE::<_, u8>::new(large).put_be(&mut buf),
            Err(Error::InvalidData(_))
        );
    }

    #[test]
    fn test_rle_string() {
        let mut buf = vec![];
        RLE::<String>::new("hello world".to_owned())
            .put_be(&mut buf)
            .expect("encoding error");

        let expected_encoded = {
            let mut b = vec![0, 11];
            b.extend_from_slice(b"hello world");
            b
        };

        assert_eq!(buf, expected_encoded);

        let output = RLE::<String>::get_be(&mut Cursor::new(&buf)).expect("decoding error");
        assert_eq!(output.unwrap(), "hello world");

        let large = "abc".repeat(100);
        assert_matches!(
            RLE::<String, u8>::new(large).put_be(&mut buf),
            Err(Error::InvalidData(_))
        )
    }
}
