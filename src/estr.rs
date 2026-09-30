//! Implementation and utilities for a dynamically encoded [`str`] equivalent type.
//!
//! See also the [`EncStr`] type.

use crate::encoding::{Enc, RecodeCause, ValidateError};
#[cfg(feature = "alloc")]
use crate::estring::EncString;
use crate::{encoding, Encoding, Str};
use alloc::borrow::ToOwned;
#[cfg(feature = "alloc")]
use alloc::vec;
use core::cmp::Ordering;
use core::error::Error;
use core::fmt::Formatter;
use core::ops::{Bound, RangeBounds};
use core::slice::SliceIndex;
use core::{fmt, ptr, slice};

mod iter;

pub use iter::*;

/// Error encountered while re-encoding an [`EncStr`] or [`CEncStr`](crate::CEncStr) into another
/// format
#[derive(Clone, Debug, PartialEq)]
pub struct RecodeError {
    valid_up_to: usize,
    char: char,
    char_len: u8,
}

impl RecodeError {
    /// The length of valid data in the input before the error was encountered. Calling
    /// [`recode`](EncStr::recode) again on the input sliced down to this length will succeed.
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// The character encountered that caused re-encoding to fail. This character most likely isn't
    /// supported by the new encoding.
    pub fn char(&self) -> char {
        self.char
    }

    /// The length of the character in the input encoding. Skipping this many bytes forwards from
    /// [`valid_up_to`](Self::valid_up_to) and trying again will avoid this particular error
    /// character (though recoding may fail again immediately due to another invalid character).
    pub fn char_len(&self) -> usize {
        self.char_len as usize
    }
}

/// Error encountered while re-encoding a [`Str`](Str) or [`CStr`](crate::CStr) into another
/// format in a pre-allocated buffer
#[derive(Clone, PartialEq)]
pub struct RecodeIntoError<'a> {
    input_used: usize,
    str: &'a EncStr,
    cause: RecodeCause,
}

impl<'a> RecodeIntoError<'a> {
    fn from_recode(err: encoding::RecodeError, str: &'a EncStr) -> Self {
        RecodeIntoError {
            input_used: err.input_used(),
            str,
            cause: err.cause().clone(),
        }
    }

    /// The length of valid data in the input before the error was encountered. Calling
    /// [`recode_into`](Str::recode_into) again on the input sliced down to this length will succeed.
    pub fn valid_up_to(&self) -> usize {
        self.input_used
    }

    /// The portion of the buffer with valid data written into it, as a [`Str`] in the desired
    /// encoding.
    pub fn output_valid(&self) -> &'a EncStr {
        self.str
    }

    /// The reason encoding stopped. See [`RecodeCause`] for more details on possible reasons.
    pub fn cause(&self) -> &RecodeCause {
        &self.cause
    }
}

impl fmt::Debug for RecodeIntoError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecodeIntoError")
            .field("input_used", &self.input_used)
            .field("str", &self.str)
            .field("cause", &self.cause)
            .finish()
    }
}

impl fmt::Display for RecodeIntoError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while recoding `Str` into buffer: ")?;
        self.cause.write_cause(f)
    }
}

impl Error for RecodeIntoError<'_> {}

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
pub struct EncStr([()]);
// #[derive(Copy, Clone, PartialEq, Eq, Hash)]
// pub struct EncStr<'a> {
//     enc: Enc,
//     ptr: &'a [u8],
// }

const PACK_SIZE: usize = Enc::max_discrim().next_power_of_two();
const PACK_OFFSET: u32 = PACK_SIZE.ilog2();

impl EncStr {
    /// Create a new, empty [`EncStr`] of the specified encoding.
    pub fn empty(encoding: Enc) -> &'static EncStr {
        unsafe { Self::from_bytes_unchecked(encoding, &[]) }
    }

    /// Create an `EncStr` from a byte slice without checking whether it is valid for the provided
    /// encoding.
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid for the provided encoding.
    pub fn from_bytes(encoding: Enc, bytes: &[u8]) -> Result<&EncStr, ValidateError> {
        encoding.validate(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(encoding, bytes) })
    }

    /// Create an `EncStr` from a byte slice, validating the encoding and returning a
    /// [`ValidateError`] if it is not a valid string in the provided encoding.
    pub unsafe fn from_bytes_unchecked(encoding: Enc, bytes: &[u8]) -> &EncStr {
        assert!(
            bytes.len() <= (usize::MAX >> PACK_OFFSET),
            "String too long - dynamically encoded strings must have enough free high bits to fit {} possible encodings.",
            PACK_SIZE,
        );
        // Length of slice:
        let len = ((encoding as u8 as usize) << (usize::BITS - PACK_OFFSET)) + bytes.len();
        let slice = unsafe { slice::from_raw_parts(bytes.as_ptr().cast::<()>(), len) };
        let ptr = ptr::from_ref(slice) as *const EncStr;
        unsafe { &*ptr }
    }

    /// Get the dynamic encoding of this string
    pub fn encoding(&self) -> Enc {
        unsafe { Enc::try_from(self.0.len() >> (usize::BITS - PACK_OFFSET)).unwrap_unchecked() }
    }

    /// Get the length of this string in bytes
    pub fn len(&self) -> usize {
        self.0.len() & (usize::MAX >> PACK_OFFSET)
    }

    /// Whether this string is empty - IE is a zero-length slice.
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    /// Get the underlying bytes for this string
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(ptr::from_ref(&self.0).cast::<u8>(), self.len()) }
    }

    /// Return an iterator over the [`char`]s of this string slice. See [`str::chars`] for caveats
    /// about this method.
    pub fn chars(&self) -> Chars<'_> {
        Chars::new(self)
    }

    fn check_bounds<R>(&self, idx: &R) -> Option<()>
    where
        R: RangeBounds<usize>,
    {
        let start = idx.start_bound();
        let end = idx.end_bound();

        let start_idx = match start {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        };

        let end_idx = match end {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i - 1,
            Bound::Unbounded => self.as_bytes().len(),
        };

        if !self.is_char_boundary(start_idx) || !self.is_char_boundary(end_idx) {
            None
        } else {
            Some(())
        }
    }

    /// Return a subslice of this `EncStr`. This is a non-panicking alternative to indexing,
    /// returning [`None`] whenever indexing would panic.
    pub fn get<R>(&self, idx: R) -> Option<&Self>
    where
        R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        // SAFETY: The provided range has been validated as landing on character boundaries.
        //         Our internal bytes are guaranteed valid for the encoding.
        Some(unsafe { EncStr::from_bytes_unchecked(self.encoding(), self.as_bytes().get(idx)?) })
    }

    /// Return a subslice of this `EncStr`, without bound checks.
    ///
    /// # Safety
    ///
    /// - The caller must ensure the range indices are in-bounds of the string byte length
    /// - The caller must ensure neither the range indices do not fall in the middle of a character
    pub unsafe fn get_unchecked<R>(&self, idx: R) -> &Self
    where
        R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        // SAFETY: Delegated to caller
        unsafe { EncStr::from_bytes_unchecked(self.encoding(), self.as_bytes().get_unchecked(idx)) }
    }

    /// Check whether the byte at `idx` is on a character boundary - IE is the first byte in a code
    /// point or the end of the string.
    ///
    /// The start and end of the string are considered boundaries, indexes greater than `self.len()`
    /// are considered not boundaries.
    pub fn is_char_boundary(&self, idx: usize) -> bool {
        match idx.cmp(&self.len()) {
            Ordering::Equal => true,
            Ordering::Greater => false,
            Ordering::Less => unsafe { self.encoding().char_bound_unchecked(self, idx) },
        }
    }

    /// Return an iterator over the [`char`]s of this string slice and their positions. See
    /// [`str::char_indices`] for caveats about this method.
    pub fn char_indices(&self) -> CharIndices<'_> {
        CharIndices::new(self)
    }

    /// Split this string at an index, returning the two substrings on either side. This method
    /// panics if the index doesn't lie on a character boundary.
    pub fn split_at(&self, idx: usize) -> Option<(&EncStr, &EncStr)> {
        if self.is_char_boundary(idx) && idx < self.len() {
            let (start, end) = self.as_bytes().split_at(idx);
            // SAFETY: Index is a character boundary. Internal data guaranteed valid.
            let start = unsafe { EncStr::from_bytes_unchecked(self.encoding(), start) };
            // SAFETY: Index is a character boundary. Internal data guaranteed valid.
            let end = unsafe { EncStr::from_bytes_unchecked(self.encoding(), end) };
            Some((start, end))
        } else {
            None
        }
    }

    /// Get this `Str` in a different [`Encoding`]. This method writes the new string into the
    /// provided buffer, and returns the portion of the buffer containing the string as a new `Str`.
    pub fn recode_into<'a>(
        &self,
        encoding: Enc,
        buffer: &'a mut [u8],
    ) -> Result<&'a EncStr, RecodeIntoError<'a>> {
        encoding
            .recode(self, buffer)
            .map(|len| {
                // SAFETY: Value written into `out` by `recode` is guaranteed valid in encoding
                //         E2.
                unsafe { EncStr::from_bytes_unchecked(encoding, &buffer[..len]) }
            })
            .map_err(|err| {
                // SAFETY: Value written into `out` by `recode` is guaranteed valid in encoding
                //         E2, up to output_valid.
                let str = unsafe {
                    EncStr::from_bytes_unchecked(encoding, &buffer[..err.output_valid()])
                };
                RecodeIntoError::from_recode(err, str)
            })
    }

    /// Get this `EncStr` in a different [`Encoding`]. This method allocates a new [`EncString`]
    /// with the desired encoding, and returns an error if the source string contains any characters
    /// that cannot be represented in the destination encoding.
    #[cfg(feature = "alloc")]
    pub fn recode(&self, encoding: Enc) -> Result<EncString, RecodeError> {
        let mut ptr = self;
        let mut total_len = 0;
        let mut out = vec![0; self.as_bytes().len()];
        loop {
            match encoding.recode(ptr, &mut out[total_len..]) {
                Ok(len) => {
                    out.truncate(total_len + len);
                    // SAFETY: Value written into `out` by `recode` is guaranteed valid in encoding
                    //         E2.
                    return Ok(unsafe { EncString::from_bytes_unchecked(encoding, out) });
                }
                Err(e) => match e.cause() {
                    RecodeCause::NeedSpace { .. } => {
                        out.resize(out.len() + self.as_bytes().len(), 0);
                        ptr = ptr.get(e.input_used()..).unwrap();
                        total_len += e.output_valid();
                    }
                    &RecodeCause::InvalidChar { char, len } => {
                        return Err(RecodeError {
                            valid_up_to: e.input_used(),
                            char,
                            char_len: len as u8,
                        });
                    }
                },
            }
        }
    }

    /// Get this `Str` in a different [`Encoding`]. This method allocates a new [`String`] with the
    /// desired encoding, replacing any characters that can't be represented in the destination
    /// encoding with the encoding's replacement character.
    #[cfg(feature = "alloc")]
    pub fn recode_lossy(&self, encoding: Enc) -> EncString {
        let mut ptr = self;
        let mut total_len = 0;
        let mut out = vec![0; self.as_bytes().len()];
        loop {
            match encoding.recode(ptr, &mut out[total_len..]) {
                Ok(len) => {
                    out.truncate(total_len + len);
                    // SAFETY: Value written into `out` by `recode` is guaranteed valid in encoding
                    //         E2.
                    return unsafe { EncString::from_bytes_unchecked(encoding, out) };
                }
                Err(e) => match e.cause() {
                    RecodeCause::NeedSpace { .. } => {
                        out.resize(out.len() + self.as_bytes().len(), 0);
                        ptr = ptr.get(e.input_used()..).unwrap();
                        total_len += e.output_valid();
                    }
                    &RecodeCause::InvalidChar { char: _, len } => {
                        let replace_len = encoding.char_len(encoding.replacement());
                        out.resize(out.len() + replace_len, 0);
                        encoding
                            .encode(
                                encoding.replacement(),
                                &mut out[total_len + e.output_valid()..],
                            )
                            .unwrap();
                        ptr = ptr.get(e.input_used() + len..).unwrap();
                        total_len += e.output_valid() + replace_len;
                    }
                },
            }
        }
    }

    /// Attempt to convert this into a [`Str`] with a specified encoding. Returns `None` if the
    /// encoding of this string doesn't match the desired encoding.
    pub fn downcast<E: Encoding>(&self) -> Option<&'_ Str<E>> {
        if self.encoding() == Enc::of::<E>() {
            Some(unsafe { Str::from_bytes_unchecked(self.as_bytes()) })
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
    pub unsafe fn downcast_unchecked<E: Encoding>(&self) -> &'_ Str<E> {
        unsafe { Str::from_bytes_unchecked(self.as_bytes()) }
    }
}

impl PartialEq for EncStr {
    fn eq(&self, other: &Self) -> bool {
        self.encoding() == other.encoding() && self.as_bytes() == other.as_bytes()
    }
}

impl Eq for EncStr {}

impl<'a, E: Encoding> From<&'a Str<E>> for &'a EncStr {
    fn from(value: &'a Str<E>) -> Self {
        unsafe { EncStr::from_bytes_unchecked(Enc::of::<E>(), value.as_bytes()) }
    }
}

impl<'a, E: Encoding> TryFrom<&'a EncStr> for &'a Str<E> {
    type Error = EncodingMismatch;

    fn try_from(value: &'a EncStr) -> Result<Self, Self::Error> {
        value.downcast().ok_or(EncodingMismatch)
    }
}

impl fmt::Debug for EncStr {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[cfg(feature = "alloc")]
impl ToOwned for EncStr {
    type Owned = EncString;

    fn to_owned(&self) -> Self::Owned {
        todo!()
    }
}

/// Error returned when an operation is performed on a [`EncStr`] that requires one encoding but
/// a different one is found.
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct EncodingMismatch;
