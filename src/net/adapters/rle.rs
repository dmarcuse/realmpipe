//! Implementations of `NetworkAdapter` for types that wrap collections and
//! are preceded by a value indicating the number of items stored.

use super::prelude::*;
use num::{FromPrimitive, ToPrimitive};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;
use std::ops::Deref;

/// A wrapper around a given collection (of type `C`) which can be converted
/// to or from big endian bytes by prefixing the data with an integer (of type
/// `S`).
pub struct RLE<S, C> {
    collection: C,
    phantom: PhantomData<S>,
}

impl<S, C> RLE<S, C> {
    /// Create a new run length encoded collection from the given collection
    pub fn new(collection: C) -> Self {
        Self {
            collection,
            phantom: PhantomData::default(),
        }
    }

    /// Unwrap the contained collection
    pub fn unwrap(self) -> C {
        self.collection
    }
}

impl<S, C> NetworkAdapter for RLE<S, C>
where
    S: NetworkAdapter + ToPrimitive + FromPrimitive + Display,
    C: IntoIterator + FromIterator<<C as IntoIterator>::Item> + 'static,
    C::Item: NetworkAdapter,
    C::IntoIter: ExactSizeIterator,
{
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        // decode length
        let len = S::get_be(bytes)?;

        // attempt to convert length to usize
        if let Some(len) = len.to_usize() {
            // decode remaining items and convert to appropriate collection
            (0..len)
                .map(|_| C::Item::get_be(bytes))
                .collect::<Result<C>>()
                .map(|collection| Self {
                    collection,
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
        // convert collection to an iterator
        let mut iter = self.collection.into_iter();

        // attempt to convert length from a usize
        if let Some(len) = S::from_usize(iter.len()) {
            // encode length and then each element
            len.put_be(bytes)?;
            iter.try_for_each(|i| i.put_be(bytes))
        } else {
            Err(Error::InvalidData(format!(
                "cannot cast length from usize: {}",
                iter.len()
            )))
        }
    }
}

impl<S, C> Deref for RLE<S, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.collection
    }
}

impl<S, C> Borrow<C> for RLE<S, C> {
    fn borrow(&self) -> &C {
        &self.collection
    }
}

impl<S, C: Debug> Debug for RLE<S, C> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self.collection)
    }
}

impl<S, S2, C: PartialEq> PartialEq<RLE<S2, C>> for RLE<S, C> {
    fn eq(&self, other: &RLE<S2, C>) -> bool {
        self.collection == other.collection
    }
}

impl<S, C: Clone> Clone for RLE<S, C> {
    fn clone(&self) -> Self {
        Self::new(self.collection.clone())
    }
}

/// A wrapper around a `String` which can be converted to or from big endian
/// bytes by prefixing the data with an integer (of type `S`)
pub struct RLEString<S> {
    string: String,
    phantom: PhantomData<S>,
}

impl<S> RLEString<S> {
    /// Create a new run length encoded string wrapping the given string
    pub fn new(string: String) -> Self {
        Self {
            string,
            phantom: PhantomData::default(),
        }
    }

    /// Unwrap the stored string
    pub fn unwrap(self) -> String {
        self.string
    }
}

// TODO: bug here? length in chars is used by game instead of length in bytes
impl<S> NetworkAdapter for RLEString<S>
where
    S: NetworkAdapter + ToPrimitive + FromPrimitive + Display,
{
    fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
        let bytes = RLE::<S, Vec<u8>>::get_be(bytes)?.unwrap();
        String::from_utf8(bytes)
            .map(|string| Self {
                string,
                phantom: PhantomData::default(),
            })
            .map_err(|e| Error::Other(e.into()))
    }

    fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
        RLE::<S, _>::new(self.string.into_bytes()).put_be(bytes)
    }
}

impl<S> Deref for RLEString<S> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl<S> Borrow<String> for RLEString<S> {
    fn borrow(&self) -> &String {
        &self.string
    }
}

impl<S> Debug for RLEString<S> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self.string)
    }
}

impl<S, S2> PartialEq<RLEString<S2>> for RLEString<S> {
    fn eq(&self, other: &RLEString<S2>) -> bool {
        self.string == other.string
    }
}

impl<S> Clone for RLEString<S> {
    fn clone(&self) -> Self {
        Self::new(self.string.clone())
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
        assert_eq!(size_of::<RLE<u16, Vec<u32>>>(), size_of::<Vec<u32>>());
    }

    #[test]
    fn test_rle_vec() {
        let mut buf = vec![];
        RLE::<u16, Vec<u8>>::new(vec![1, 2, 3, 4, 5])
            .put_be(&mut buf)
            .expect("encoding error");
        assert_eq!(buf, vec![0, 5, 1, 2, 3, 4, 5]);

        let output = RLE::<u16, Vec<u8>>::get_be(&mut Cursor::new(&buf)).expect("decoding error");
        assert_eq!(output.unwrap(), vec![1, 2, 3, 4, 5]);

        let large = (0..300).collect::<Vec<u16>>();
        assert_matches!(
            RLE::<u8, _>::new(large).put_be(&mut buf),
            Err(Error::InvalidData(_))
        );
    }

    #[test]
    fn test_rle_string() {
        let mut buf = vec![];
        RLEString::<u16>::new("hello world".to_owned())
            .put_be(&mut buf)
            .expect("encoding error");

        let mut expected_encoded = {
            let mut b = vec![0, 11];
            b.extend_from_slice(b"hello world");
            b
        };

        assert_eq!(buf, expected_encoded);

        let output = RLEString::<u16>::get_be(&mut Cursor::new(&buf)).expect("decoding error");
        assert_eq!(output.unwrap(), "hello world");

        let large = "abc".repeat(100);
        assert_matches!(
            RLEString::<u8>::new(large).put_be(&mut buf),
            Err(Error::InvalidData(_))
        )
    }
}
