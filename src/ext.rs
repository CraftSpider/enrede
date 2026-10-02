//! Extensions to standard library types for working with different encodings.

use crate::encoding::Enc;
use crate::{EncStr, EncString, Encoding, Str, String};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::{format, io};

/// Extension to the operations on a [`Path`], providing encoding-specific methods
pub trait PathExt {
    /// Attempt to get the contents of this path as a [`Str`] in the specified encoding.
    ///
    /// Note that paths in Rust are always stored as a superset of UTF-8 containing potentially
    /// invalid characters.
    fn to_encoding<E: Encoding>(&self) -> Option<&Str<E>>;

    /// Attempt to get the contents of this path as a [`Str`] in the specified encoding, replacing
    /// invalid characters with replacement characters if needed. Only allocates if the string
    /// isn't already valid in the specified encoding.
    fn to_encoding_lossy<E: Encoding>(&self) -> Cow<'_, Str<E>>;

    /// Attempt to get the contents of this path as an [`EncStr`] in the provided encoding.
    ///
    /// Note that paths in Rust are always stored as a superset of UTF-8 containing potentially
    /// invalid characters.
    fn to_encoding_dyn(&self, encoding: Enc) -> Option<&EncStr>;

    /// Attempt to get the contents of this path as an [`EncStr`] in the specified encoding,
    /// replacing invalid characters with replacement characters if needed. Only allocates if the
    /// string isn't already valid in the specified encoding.
    fn to_encoding_lossy_dyn(&self, encoding: Enc) -> Cow<'_, EncStr>;
}

impl PathExt for Path {
    fn to_encoding<E: Encoding>(&self) -> Option<&Str<E>> {
        Str::from_bytes(self.as_os_str().as_encoded_bytes()).ok()
    }

    fn to_encoding_lossy<E: Encoding>(&self) -> Cow<'_, Str<E>> {
        String::from_bytes_lossy(self.as_os_str().as_encoded_bytes())
    }

    fn to_encoding_dyn(&self, encoding: Enc) -> Option<&EncStr> {
        EncStr::from_bytes(encoding, self.as_os_str().as_encoded_bytes()).ok()
    }

    fn to_encoding_lossy_dyn(&self, encoding: Enc) -> Cow<'_, EncStr> {
        EncString::from_bytes_lossy(encoding, self.as_os_str().as_encoded_bytes())
    }
}

/// Extension to the operations on readable types, providing encoding-specific read methods.
pub trait ReadEnc: Read {
    /// Read all the bytes of the file into a `String<E>` of the specified encoding.
    fn read_to_encoding<E: Encoding>(&mut self) -> Result<String<E>, io::Error>;

    /// Read all the bytes of the file into an `EncString` of the provided encoding.
    fn read_to_encoding_dyn(&mut self, enc: Enc) -> Result<EncString, io::Error>;
}

impl<R: Read> ReadEnc for R {
    fn read_to_encoding<E: Encoding>(&mut self) -> Result<String<E>, io::Error> {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;
        String::<E>::from_bytes(buf).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("stream did not contain valid {}", E::shorthand()),
            )
        })
    }

    fn read_to_encoding_dyn(&mut self, enc: Enc) -> Result<EncString, io::Error> {
        let mut buf = Vec::new();
        self.read_to_end(&mut buf)?;
        EncString::from_bytes(enc, buf).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("stream did not contain valid {}", enc.shorthand()),
            )
        })
    }
}
