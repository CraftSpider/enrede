//! Implementation and utilities for a dynamically encoded [`std::String`](std::string::String)
//! equivalent type.

use crate::encoding::{ArrayLike, Enc, Utf8};
use crate::errors::EncodingMismatch;
use crate::string::{InvalidChar, OwnValidateError};
use crate::{EncStr, Encoding, Str, String};
use alloc::borrow::{Cow, ToOwned};
use alloc::string::String as StdString;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::fmt::Formatter;
use core::ops::{Deref, DerefMut};

mod chunks;

use chunks::EncodedChunks;

/// Implementation of a dynamically encoded [`std::String`](std::string::String) type. This type is
/// similar to the standard library [`String`](std::string::String) type in many ways, but instead
/// of having a fixed UTF-8 encoding scheme, it uses an encoding determined at runtime.
///
/// `EncString` only implements `==` between instances with the same encoding. To compare strings of
/// different encoding by characters, use `a.chars().eq(b.chars())`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EncString(Enc, Vec<u8>);

impl EncString {
    /// Create a new, empty `EncString`
    pub const fn new(encoding: Enc) -> EncString {
        EncString(encoding, Vec::new())
    }

    /// Create an empty string with a pre-allocated capacity for `len` bytes.
    pub fn with_capacity(encoding: Enc, len: usize) -> EncString {
        EncString(encoding, Vec::with_capacity(len))
    }

    /// Create a `String` from bytes without checking whether it is valid for the current encoding.
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid for the current encoding.
    pub unsafe fn from_bytes_unchecked(encoding: Enc, bytes: Vec<u8>) -> EncString {
        debug_assert!(encoding.validate(&bytes).is_ok());
        EncString(encoding, bytes)
    }

    /// Create a `String` from bytes, validating the encoding and returning an [`OwnValidateError`]
    /// if it is not a valid string in the current encoding.
    pub fn from_bytes(encoding: Enc, bytes: Vec<u8>) -> Result<EncString, OwnValidateError> {
        match encoding.validate(&bytes) {
            // SAFETY: Bytes have been validated, they are guaranteed valid for the encoding
            Ok(_) => Ok(unsafe { EncString::from_bytes_unchecked(encoding, bytes) }),
            Err(err) => Err(OwnValidateError::new(err, bytes)),
        }
    }

    /// Attempt to convert bytes into a [`Str<E>`]. If any bytes are invalid for the current
    /// encoding, a new `String` will instead be allocated that replaces the invalid bytes with the
    /// replacement character for the encoding.
    pub fn from_bytes_lossy(encoding: Enc, bytes: &[u8]) -> Cow<'_, EncStr> {
        let mut chunks = EncodedChunks::new(encoding, bytes);

        let first_valid = if let Some(chunk) = chunks.next() {
            let valid = chunk.valid();
            if chunk.invalid().is_empty() {
                debug_assert_eq!(valid.len(), bytes.len());
                return Cow::Borrowed(valid);
            }
            valid
        } else {
            return Cow::Borrowed(EncStr::empty(encoding));
        };

        let mut res = EncString::with_capacity(encoding, bytes.len());
        res.push_str(first_valid);
        res.push(encoding.replacement());

        for chunk in chunks {
            res.push_str(chunk.valid());
            if !chunk.invalid().is_empty() {
                res.push(encoding.replacement());
            }
        }

        Cow::Owned(res)
    }

    /// Convert this `String` into a vector of its contained bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.1
    }

    /// Add a new character to this string. This method panics if the provided character isn't valid
    /// for the current encoding.
    pub fn push(&mut self, c: char) {
        self.try_push(c).unwrap_or_else(|_| {
            panic!(
                "Invalid character {:?} for encoding {}",
                c,
                self.0.shorthand()
            )
        });
    }

    /// Add a new character to this string. This method returns [`InvalidChar`] if the provided
    /// character isn't valid for the current encoding.
    pub fn try_push(&mut self, c: char) -> Result<(), InvalidChar> {
        self.1
            .extend(self.0.encode_char(c).ok_or(InvalidChar)?.slice());
        Ok(())
    }

    /// Extend this `String` with the contents of the provided [`Str`].
    pub fn push_str(&mut self, str: &EncStr) {
        if str.encoding() == self.0 {
            self.1.extend(str.as_bytes());
        } else {
            panic!(
                "Invalid encoding for string push: found {}, expected {}",
                str.encoding().shorthand(),
                self.0.shorthand()
            )
        }
    }

    /// Attempt to convert this into a [`String`] with a specified encoding. Returns `Err` if the
    /// encoding of this string doesn't match the desired encoding.
    pub fn downcast<E: Encoding>(self) -> Result<String<E>, Self> {
        if self.encoding() == Enc::of::<E>() {
            Ok(unsafe { String::from_bytes_unchecked(self.into_bytes()) })
        } else {
            Err(self)
        }
    }

    /// Convert this into a [`String`] with the specified encoding, without checking if the encodings
    /// match.
    ///
    /// # Safety
    ///
    /// This string's encoding must be the same as the provided `E`.
    pub unsafe fn downcast_unchecked<E: Encoding>(self) -> String<E> {
        unsafe { String::from_bytes_unchecked(self.into_bytes()) }
    }

    // UTF-8 methods

    /// Convert an [`std::String`](std::string::String) directly into an [`EncString`]
    pub fn from_std(value: StdString) -> Self {
        // SAFETY: `StdString` is UTF-8 by its validity guarantees.
        unsafe { EncString::from_bytes_unchecked(Enc::Utf8, value.into_bytes()) }
    }

    /// Convert a [`EncString`] directly into an [`std::String`](std::string::String), if the
    /// backing encoding is UTF-8.
    pub fn into_std(self) -> Option<StdString> {
        if self.0 == Enc::Utf8 {
            // SAFETY: `EncString` is UTF-8 by its validity guarantees since encoding matches.
            Some(unsafe { StdString::from_utf8_unchecked(self.into_bytes()) })
        } else {
            None
        }
    }
}

impl fmt::Debug for EncString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <EncStr as fmt::Debug>::fmt(self, f)
    }
}

impl fmt::Display for EncString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <EncStr as fmt::Display>::fmt(self, f)
    }
}

impl Deref for EncString {
    type Target = EncStr;

    fn deref(&self) -> &Self::Target {
        unsafe { EncStr::from_bytes_unchecked(self.0, &self.1) }
    }
}

impl DerefMut for EncString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { EncStr::from_bytes_unchecked_mut(self.0, &mut self.1) }
    }
}

impl AsRef<EncStr> for EncString {
    fn as_ref(&self) -> &EncStr {
        self
    }
}

impl AsMut<EncStr> for EncString {
    fn as_mut(&mut self) -> &mut EncStr {
        self
    }
}

impl Borrow<EncStr> for EncString {
    fn borrow(&self) -> &EncStr {
        self
    }
}

impl BorrowMut<EncStr> for EncString {
    fn borrow_mut(&mut self) -> &mut EncStr {
        self
    }
}

impl From<&EncStr> for EncString {
    fn from(value: &EncStr) -> Self {
        EncStr::to_owned(value)
    }
}

impl From<&str> for EncString {
    fn from(value: &str) -> Self {
        EncStr::from_std(value).to_owned()
    }
}

impl From<StdString> for EncString {
    fn from(value: StdString) -> Self {
        Self::from_std(value)
    }
}

impl<E: Encoding> From<&Str<E>> for EncString {
    fn from(value: &Str<E>) -> Self {
        <&EncStr>::from(value).to_owned()
    }
}

impl<E: Encoding> From<String<E>> for EncString {
    fn from(value: String<E>) -> Self {
        unsafe { EncString::from_bytes_unchecked(Enc::of::<E>(), value.into_bytes()) }
    }
}

impl<E: Encoding> TryFrom<EncString> for String<E> {
    type Error = EncodingMismatch;

    fn try_from(value: EncString) -> Result<Self, Self::Error> {
        value.downcast().map_err(|val| EncodingMismatch {
            found: val.encoding(),
            expected: Enc::of::<E>(),
        })
    }
}

impl TryFrom<EncString> for StdString {
    type Error = EncodingMismatch;

    fn try_from(value: EncString) -> Result<Self, Self::Error> {
        value
            .downcast::<Utf8>()
            .map(String::into_std)
            .map_err(|val| EncodingMismatch {
                found: val.encoding(),
                expected: Enc::Utf8,
            })
    }
}
