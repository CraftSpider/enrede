use crate::encoding::{ArrayLike, Enc};
use crate::string::{InvalidChar, OwnValidateError};
use crate::EncStr;
use alloc::vec::Vec;

mod chunks;

use chunks::EncodedChunks;

/// Similar to Cow but supports custom conversion for sized types.
pub enum MaybeOwned<'a> {
    /// Owned value
    Owned(EncString),
    /// Borrowed value
    Borrowed(EncStr<'a>),
}

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
    pub fn from_bytes_lossy(encoding: Enc, bytes: &[u8]) -> MaybeOwned<'_> {
        let mut chunks = EncodedChunks::new(encoding, bytes);

        let first_valid = if let Some(chunk) = chunks.next() {
            let valid = chunk.valid();
            if chunk.invalid().is_empty() {
                debug_assert_eq!(valid.len(), bytes.len());
                return MaybeOwned::Borrowed(valid);
            }
            valid
        } else {
            return MaybeOwned::Borrowed(EncStr::empty(encoding));
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

        MaybeOwned::Owned(res)
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
    pub fn push_str(&mut self, str: EncStr<'_>) {
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

    pub fn deref(&self) -> EncStr<'_> {
        unsafe { EncStr::from_bytes_unchecked(self.0, &self.1) }
    }
}

// Unfortunately, we can't impl Deref or DerefMut for now
