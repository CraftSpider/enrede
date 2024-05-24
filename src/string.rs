//! Implementation and utilities for a generically encoded [`std::String`](std::string::String)
//! equivalent type.

use core::fmt;
use std::borrow::{Borrow, BorrowMut, Cow};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::string::String as StdString;

use super::encoding::{Encoding, Utf8, ValidateError};
use super::str::Str;

mod chunks;

use chunks::EncodedChunks;

/// An error returned when you attempt to perform operations using a character not supported in a
/// specific encoding.
#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidChar;

/// Implementation of a generically encoded [`std::String`](std::string::String) type. This type is
/// similar to the standard library [`String`](std::string::String) type in many ways, but instead
/// of having a fixed UTF-8 encoding scheme, it uses an encoding determined by the generic `E` it
/// is provided.
///
/// `String` only implements `==` between instances with the same encoding. To compare strings of
/// different encoding by characters, use `a.chars().eq(b.chars())`.
#[derive(Clone)]
pub struct String<E>(PhantomData<E>, Vec<u8>);

impl<E: Encoding> String<E> {
    /// Create a new, empty `String`
    pub const fn new() -> String<E> {
        String(PhantomData, Vec::new())
    }

    /// Create an empty string with a pre-allocated capacity for `len` bytes.
    pub fn with_capacity(len: usize) -> String<E> {
        String(PhantomData, Vec::with_capacity(len))
    }

    /// Create a `String` from bytes without checking whether it is valid for the current encoding.
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> String<E> {
        String(PhantomData, bytes)
    }

    /// Create a `String` from bytes, validating the encoding and returning a [`ValidateError`] if
    /// it is not a valid string in the current encoding.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<String<E>, ValidateError> {
        E::validate(&bytes)?;
        Ok(unsafe { String::from_bytes_unchecked(bytes) })
    }

    /// Attempt to convert bytes into a [`Str<E>`]. If any bytes are invalid for the current
    /// encoding, a new `String` will instead be allocated that replaces the invalid bytes with the
    /// replacement character for the encoding.
    pub fn from_bytes_lossy(bytes: &[u8]) -> Cow<'_, Str<E>> {
        let mut chunks = EncodedChunks::new(bytes);

        let first_valid = if let Some(chunk) = chunks.next() {
            let valid = chunk.valid();
            if chunk.invalid().is_empty() {
                debug_assert_eq!(valid.len(), bytes.len());
                return Cow::Borrowed(valid);
            }
            valid
        } else {
            return Cow::Borrowed(<&Str<E>>::default());
        };

        let mut res = String::with_capacity(bytes.len());
        res.push_str(first_valid);
        res.push(E::REPLACEMENT);

        for chunk in chunks {
            res.push_str(chunk.valid());
            if !chunk.invalid().is_empty() {
                res.push(E::REPLACEMENT);
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
                "Invalid character '{:?}' for encoding {}",
                c,
                E::shorthand()
            )
        });
    }

    /// Add a new character to this string. This method returns [`InvalidChar`] if the provided
    /// character isn't valid for the current encoding.
    pub fn try_push(&mut self, c: char) -> Result<(), InvalidChar> {
        self.1.extend(E::encode_char(c).ok_or(InvalidChar)?);
        Ok(())
    }

    /// Extend this `String` with the contents of the provided [`Str`].
    pub fn push_str(&mut self, str: &Str<E>) {
        self.1.extend(str.as_bytes());
    }
}

impl String<Utf8> {
    /// Convert an [`std::String`](std::string::String) directly into a [`String<Utf8>`]
    pub fn from_std(value: StdString) -> Self {
        unsafe { String::from_bytes_unchecked(value.into_bytes()) }
    }

    /// Convert a [`String<Utf8>`] directly into a [`std::String`](std::string::String)
    pub fn into_std(self) -> StdString {
        unsafe { StdString::from_utf8_unchecked(self.into_bytes()) }
    }
}

impl<E: Encoding> fmt::Debug for String<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Str<E> as fmt::Debug>::fmt(self, f)
    }
}

impl<E: Encoding> fmt::Display for String<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Str<E> as fmt::Display>::fmt(self, f)
    }
}

impl<E: Encoding> Default for String<E> {
    fn default() -> Self {
        String::new()
    }
}

impl<E: Encoding> PartialEq for String<E> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<E: Encoding> Eq for String<E> {}

impl<E: Encoding> Deref for String<E> {
    type Target = Str<E>;

    fn deref(&self) -> &Self::Target {
        unsafe { Str::from_bytes_unchecked(&self.1) }
    }
}

impl<E: Encoding> DerefMut for String<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Str::from_bytes_unchecked_mut(&mut self.1) }
    }
}

impl<E: Encoding> AsRef<Str<E>> for String<E> {
    fn as_ref(&self) -> &Str<E> {
        self
    }
}

impl<E: Encoding> AsMut<Str<E>> for String<E> {
    fn as_mut(&mut self) -> &mut Str<E> {
        self
    }
}

impl<E: Encoding> Borrow<Str<E>> for String<E> {
    fn borrow(&self) -> &Str<E> {
        self
    }
}

impl<E: Encoding> BorrowMut<Str<E>> for String<E> {
    fn borrow_mut(&mut self) -> &mut Str<E> {
        self
    }
}

impl<E: Encoding> FromIterator<char> for String<E> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        iter.into_iter().fold(String::new(), |mut acc, c| {
            acc.push(c);
            acc
        })
    }
}

// Encoding-specific implementations

impl From<&str> for String<Utf8> {
    fn from(value: &str) -> Self {
        Str::from_std(value).to_owned()
    }
}

impl From<StdString> for String<Utf8> {
    fn from(value: StdString) -> Self {
        Self::from_std(value)
    }
}

impl From<String<Utf8>> for StdString {
    fn from(value: String<Utf8>) -> Self {
        value.into_std()
    }
}
