//! Implementation and utilities for a dynamically encoded [`str`] equivalent type.
//!
//! See also the [`EncStr`] type.

use crate::encoding::Enc;
use crate::{Encoding, Str};

mod iter;

pub use iter::*;

/// Implementation of a dynamically encoded [`str`] type. This type is similar to the standard
/// library [`str`] type in many ways, but instead of having a fixed UTF-8 encoding scheme, it uses
/// an encoding determined at runtime.
///
/// `DynStr` implements `==` to only be true between instances with the same encoding. To compare
/// strings of different encoding by characters, use `a.chars().eq(b.chars())`.
///
/// See [`Str`] for a version with compile-time chosen encoding.
///
/// ## Invariant
///
/// Rust libraries may assume that a `DynStr` is valid for the [`Encoding`] matching its [`Enc`].
///
/// Constructing non-`Enc` string slices is not immediate UB, but any function called on it may
/// assume that it is valid.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EncStr<'a> {
    encoding: Enc,
    ptr: &'a [u8],
}

impl<'a> EncStr<'a> {
    /// Create a new, empty [`EncStr`] of the specified encoding.
    pub fn empty(encoding: Enc) -> EncStr<'a> {
        EncStr { encoding, ptr: &[] }
    }

    /// Get the dynamic encoding of this string
    pub fn encoding(&self) -> Enc {
        self.encoding
    }

    /// Get the length of this string in bytes
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Whether this string is empty - IE is a zero-length slice.
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    /// Get the underlying bytes for this string
    pub fn as_bytes(&self) -> &'a [u8] {
        self.ptr
    }

    /// Return an iterator over the [`char`]s of this string slice. See [`str::chars`] for caveats
    /// about this method.
    pub fn chars(self) -> Chars<'a> {
        Chars::new(self)
    }

    /// Return an iterator over the [`char`]s of this string slice and their positions. See
    /// [`str::char_indices`] for caveats about this method.
    pub fn char_indices(self) -> CharIndices<'a> {
        CharIndices::new(self)
    }

    /// Attempt to convert this into a [`Str`] with a specified encoding. Returns `None` if the
    /// encoding of this string doesn't match the desired encoding.
    pub fn downcast<E: Encoding>(&self) -> Option<&'a Str<E>> {
        if self.encoding == Enc::of::<E>() {
            Some(unsafe { Str::from_bytes_unchecked(self.ptr) })
        } else {
            None
        }
    }

    /// Convert this into a [`Str`] with the specified encoding, without checking if the encodings
    /// match.
    ///
    /// # Safety
    ///
    /// This string's encoding must be the same as the provided `E`.
    pub unsafe fn downcast_unchecked<E: Encoding>(&self) -> &'a Str<E> {
        unsafe { Str::from_bytes_unchecked(self.ptr) }
    }
}

impl<'a, E: Encoding> From<&'a Str<E>> for EncStr<'a> {
    fn from(value: &'a Str<E>) -> Self {
        EncStr {
            encoding: Enc::of::<E>(),
            ptr: value.as_bytes(),
        }
    }
}

impl<'a, E: Encoding> TryFrom<EncStr<'a>> for &'a Str<E> {
    type Error = EncodingMismatch;

    fn try_from(value: EncStr<'a>) -> Result<Self, Self::Error> {
        value.downcast().ok_or(EncodingMismatch)
    }
}

/// Error returned when an operation is performed on a [`EncStr`] that requires one encoding but
/// a different one is found.
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct EncodingMismatch;
