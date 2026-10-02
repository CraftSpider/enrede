//! Implementation and utilities for a dynamically encoded [`str`] equivalent type.
//!
//! See also the [`EncStr`] type.

#[cfg(feature = "alloc")]
use crate::encoding::RecodeCause;
use crate::encoding::{Enc, ValidateError};
use crate::errors::{EncodingMismatch, RecodeError, RecodeIntoError};
#[cfg(feature = "alloc")]
use crate::estring::EncString;
use crate::{Encoding, Str};
#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
#[cfg(feature = "alloc")]
use alloc::vec;
use bytemuck::cast_slice;
use core::cmp::Ordering;
use core::fmt::{Formatter, Write};
use core::hash::{Hash, Hasher};
use core::ops::{Bound, Index, IndexMut, RangeBounds};
use core::slice::SliceIndex;
use core::{fmt, ptr, slice};

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
///
/// ## Caveats
///
/// - The implementation of `EncStr` is valid under tree borrows, but not stacked borrows. If you
/// don't know what this means, you probably don't have to worry about it.
///
/// This is because EncStr relies on the header pattern - creating a reference with a smaller
/// accessible range then converting it back to the longer range later. This has been deemed a
/// desirable pattern, so is likely to be possible in any future borrow model that is chosen.
///
/// - `Box<EncStr>` shouldn't be used.
///
/// Related to the header pattern above, the box will think that the backing allocation is
/// zero-sized. This won't lead to frees with mismatched layout, but no free at all, leaking the
/// backing memory. As such, no safe methods exist to create such a type.
pub struct EncStr([()]);

const PACK_SIZE: usize = Enc::max_discrim().next_power_of_two();
const PACK_OFFSET: u32 = PACK_SIZE.ilog2();

impl EncStr {
    /// Create a new, empty [`EncStr`] of the specified encoding.
    pub fn empty(encoding: Enc) -> &'static EncStr {
        unsafe { Self::from_bytes_unchecked(encoding, &[]) }
    }

    /// Create an `EncStr` from a mutable byte slice, validating the encoding and returning a
    /// [`ValidateError`] if it is not a valid string in the provided encoding.
    pub unsafe fn from_bytes_unchecked(encoding: Enc, bytes: &[u8]) -> &EncStr {
        debug_assert!(encoding.validate(&bytes).is_ok());
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

    /// Create an `EncStr` from a byte slice, validating the encoding and returning a
    /// [`ValidateError`] if it is not a valid string in the provided encoding.
    pub unsafe fn from_bytes_unchecked_mut(encoding: Enc, bytes: &mut [u8]) -> &mut EncStr {
        debug_assert!(encoding.validate(&bytes).is_ok());
        assert!(
            bytes.len() <= (usize::MAX >> PACK_OFFSET),
            "String too long - dynamically encoded strings must have enough free high bits to fit {} possible encodings.",
            PACK_SIZE,
        );
        // Length of slice:
        let len = ((encoding as u8 as usize) << (usize::BITS - PACK_OFFSET)) + bytes.len();
        let slice = unsafe { slice::from_raw_parts_mut(bytes.as_mut_ptr().cast::<()>(), len) };
        let ptr = ptr::from_ref(slice) as *mut EncStr;
        unsafe { &mut *ptr }
    }

    /// Create an `EncStr` from a byte slice, validating the encoding and returning a [`ValidateError`]
    /// if it is not a valid string in the provided encoding.
    pub fn from_bytes(encoding: Enc, bytes: &[u8]) -> Result<&EncStr, ValidateError> {
        encoding.validate(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(encoding, bytes) })
    }

    /// Create an `EncStr` from a mutable byte slice, validating the encoding and returning a
    /// [`ValidateError`] if it is not a valid string in the provided encoding.
    pub fn from_bytes_mut(encoding: Enc, bytes: &[u8]) -> Result<&EncStr, ValidateError> {
        encoding.validate(bytes)?;
        Ok(unsafe { Self::from_bytes_unchecked(encoding, bytes) })
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

    /// Get the underlying bytes for this string mutably. This method is unsafe because it is
    /// possible to write invalid bytes for the encoding into the slice.
    ///
    /// # Safety
    ///
    /// The returned reference must not be used to write invalid data into the string.
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(ptr::from_mut(&mut self.0).cast::<u8>(), self.len()) }
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
            Bound::Included(i) => *i + 1,
            Bound::Excluded(i) => *i,
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

    /// Return a mutable subslice of this `EncStr`. This is a non-panicking alternative to indexing,
    /// returning [`None`] whenever indexing would panic.
    pub fn get_mut<R>(&mut self, idx: R) -> Option<&mut Self>
    where
        R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        // SAFETY: The provided range has been validated as landing on character boundaries.
        //         Our internal bytes are guaranteed valid for the encoding.
        Some(unsafe {
            EncStr::from_bytes_unchecked_mut(self.encoding(), self.as_bytes_mut().get_mut(idx)?)
        })
    }

    /// Return a mutable subslice of this `EncStr`, without bound checks.
    ///
    /// # Safety
    ///
    /// - The caller must ensure the range indices are in-bounds of the string byte length
    /// - The caller must ensure neither the range indices do not fall in the middle of a character
    pub unsafe fn get_unchecked_mut<R>(&mut self, idx: R) -> &mut Self
    where
        R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        // SAFETY: Delegated to caller
        unsafe {
            EncStr::from_bytes_unchecked_mut(
                self.encoding(),
                self.as_bytes_mut().get_unchecked_mut(idx),
            )
        }
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

    /// Return an iterator over the [`char`]s of this string slice. See [`str::chars`] for caveats
    /// about this method.
    pub fn chars(&self) -> Chars<'_> {
        Chars::new(self)
    }

    /// Return an iterator over the [`char`]s of this string slice and their positions. See
    /// [`str::char_indices`] for caveats about this method.
    pub fn char_indices(&self) -> CharIndices<'_> {
        CharIndices::new(self)
    }

    /// Split this string at an index, returning the two substrings on either side. This method
    /// returns `None` if the index doesn't lie on a character boundary.
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

    /// Split this string mutably at an index, returning the two substrings on either side. This
    /// method returns `None` if the index doesn't lie on a character boundary.
    pub fn split_at_mut(&mut self, idx: usize) -> Option<(&mut EncStr, &mut EncStr)> {
        if self.is_char_boundary(idx) && idx < self.len() {
            let encoding = self.encoding();
            // SAFETY: We won't be writing through this slice, only converting it back into a &mut EncStr
            let (start, end) = unsafe { self.as_bytes_mut().split_at_mut(idx) };
            // SAFETY: Index is a character boundary. Internal data guaranteed valid.
            let start = unsafe { EncStr::from_bytes_unchecked_mut(encoding, start) };
            // SAFETY: Index is a character boundary. Internal data guaranteed valid.
            let end = unsafe { EncStr::from_bytes_unchecked_mut(encoding, end) };
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
    ) -> Result<&'a EncStr, RecodeIntoError<'a, Self>> {
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

    // UTF-8 specific methods

    /// Equivalent to [`EncStr::from_bytes_unchecked`] but for UTF-8 specifically
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid UTF-8.
    pub unsafe fn from_utf8_unchecked(str: &[u8]) -> &Self {
        // SAFETY: Precondition that input is valid UTF-8
        Self::from_bytes_unchecked(Enc::Utf8, str)
    }

    /// Equivalent to [`EncStr::from_bytes`] but for UTF-8 specifically
    pub fn from_utf8(str: &[u8]) -> Result<&Self, ValidateError> {
        Self::from_bytes(Enc::Utf8, str)
    }

    /// Convert a [`str`] directly into a UTF-8 [`EncStr`].
    pub fn from_std(value: &str) -> &EncStr {
        // SAFETY: `&str` is UTF-8 by its validity guarantees.
        unsafe { Self::from_bytes_unchecked(Enc::Utf8, value.as_bytes()) }
    }

    /// Convert an [`EncStr`] into a [`str`], if the backing encoding is UTF-8.
    pub fn as_std(&self) -> Option<&str> {
        if self.encoding() == Enc::Utf8 {
            // SAFETY: `&EncStr` is UTF-8 here by our validity guarantees.
            Some(unsafe { core::str::from_utf8_unchecked(self.as_bytes()) })
        } else {
            None
        }
    }

    // UTF-16 specific methods

    /// Equivalent to [`EncStr::from_bytes_unchecked`] but for UTF-16 specifically
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid UTF-16 in little endian.
    pub unsafe fn from_utf16_unchecked(str: &[u16]) -> &Self {
        // SAFETY: Precondition that input is valid UTF-16
        Self::from_bytes_unchecked(Enc::Utf16LE, cast_slice(str))
    }

    /// Equivalent to [`EncStr::from_bytes`] but for UTF-16 specifically.
    pub fn from_utf16(str: &[u16]) -> Result<&Self, ValidateError> {
        Self::from_bytes(Enc::Utf16LE, cast_slice(str))
    }

    // UTF-32 specific methods

    const UTF32: Enc = if cfg!(target_endian = "little") {
        Enc::Utf32LE
    } else {
        Enc::Utf32BE
    };

    /// Equivalent to [`EncStr::from_bytes_unchecked`] but for UTF-32 specifically
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid UTF-32 in the native endian.
    pub unsafe fn from_utf32_unchecked(str: &[u32]) -> &Self {
        // SAFETY: Precondition that input is valid UTF-32
        Self::from_bytes_unchecked(Self::UTF32, cast_slice(str))
    }

    /// Equivalent to [`EncStr::from_bytes`] but for UTF-32 specifically
    pub fn from_utf32(str: &[u32]) -> Result<&Self, ValidateError> {
        Self::from_bytes(Self::UTF32, cast_slice(str))
    }

    /// Convert a [`&[char]`] directly into a [`EncStr`]
    pub fn from_chars(str: &[char]) -> &Self {
        // SAFETY: Utf32 encoding is exactly equivalent to `char` encoding.
        unsafe { Self::from_bytes_unchecked(Self::UTF32, cast_slice(str)) }
    }

    /// Attempt to convert an [`EncStr`] directly into a [`&[char]`]. This will fail if the `EncStr`
    /// is not sufficiently aligned for a `char`, or the encoding isn't UTF32.
    pub fn try_chars(&self) -> Option<&[char]> {
        let len = self.as_bytes().len();
        let ptr = ptr::from_ref(&self.0);
        if self.encoding() != Self::UTF32 || (ptr.cast::<()>() as usize) % align_of::<char>() != 0 {
            None
        } else {
            // SAFETY: We have guaranteed correct alignment and encoding, and Utf32 encoding is
            //         exactly equivalent to `char` encoding.
            Some(unsafe { slice::from_raw_parts(ptr.cast(), len / 4) })
        }
    }
}

impl fmt::Debug for EncStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"")?;
        for c in self.chars() {
            f.write_char(c)?;
        }
        write!(f, "\"{}", self.encoding().shorthand())
    }
}

impl fmt::Display for EncStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for c in self.chars() {
            f.write_char(c)?;
        }
        Ok(())
    }
}

impl<R> Index<R> for EncStr
where
    R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
{
    type Output = EncStr;

    fn index(&self, index: R) -> &Self::Output {
        self.get(index)
            .expect("Attempted to slice string at non-character boundary")
    }
}

impl<R> IndexMut<R> for EncStr
where
    R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
{
    fn index_mut(&mut self, index: R) -> &mut Self::Output {
        self.get_mut(index)
            .expect("Attempted to slice string at non-character boundary")
    }
}

impl PartialEq for EncStr {
    fn eq(&self, other: &Self) -> bool {
        self.encoding() == other.encoding() && self.as_bytes() == other.as_bytes()
    }
}

impl Eq for EncStr {}

impl Hash for EncStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.encoding().hash(state);
        self.as_bytes().hash(state);
    }
}

impl AsRef<[u8]> for EncStr {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(feature = "alloc")]
impl ToOwned for EncStr {
    type Owned = EncString;

    fn to_owned(&self) -> Self::Owned {
        let bytes = self.as_bytes().to_vec();
        unsafe { EncString::from_bytes_unchecked(self.encoding(), bytes) }
    }
}

// Conversions

impl<'a, E: Encoding> From<&'a Str<E>> for &'a EncStr {
    fn from(value: &'a Str<E>) -> Self {
        unsafe { EncStr::from_bytes_unchecked(Enc::of::<E>(), value.as_bytes()) }
    }
}

impl<'a, E: Encoding> TryFrom<&'a EncStr> for &'a Str<E> {
    type Error = EncodingMismatch;

    fn try_from(value: &'a EncStr) -> Result<Self, Self::Error> {
        value.downcast().ok_or_else(|| EncodingMismatch {
            found: value.encoding(),
            expected: Enc::of::<E>(),
        })
    }
}

impl<'a> TryFrom<&'a EncStr> for &'a str {
    type Error = EncodingMismatch;

    fn try_from(value: &'a EncStr) -> Result<Self, Self::Error> {
        value
            .downcast()
            .map(Str::as_std)
            .ok_or_else(|| EncodingMismatch {
                found: value.encoding(),
                expected: Enc::Utf8,
            })
    }
}

impl<'a> From<&'a str> for &'a EncStr {
    fn from(value: &'a str) -> Self {
        EncStr::from_std(value)
    }
}

impl<'a> From<&'a [char]> for &'a EncStr {
    fn from(value: &'a [char]) -> Self {
        EncStr::from_chars(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_len() {
        let a = EncStr::from_std("a");
        assert_eq!(a.len(), 1);

        let b = EncStr::from_std("abc123");
        assert_eq!(b.len(), 6);
    }

    #[test]
    fn test_encoding() {
        let a = EncStr::from_std("abc");
        assert_eq!(a.encoding(), Enc::Utf8);

        let b = EncStr::from_chars(&['a', 'b', 'c']);
        assert_eq!(
            b.encoding(),
            if cfg!(target_endian = "little") {
                Enc::Utf32LE
            } else {
                Enc::Utf32BE
            }
        );
    }

    #[test]
    fn test_get() {
        let s = EncStr::from_std("abcd");
        assert_eq!(s.get(0..1), Some(EncStr::from_std("a")));
        assert_eq!(s.get(3..4), Some(EncStr::from_std("d")));

        assert_eq!(s.get(0..2), Some(EncStr::from_std("ab")));
        assert_eq!(s.get(2..4), Some(EncStr::from_std("cd")));

        assert_eq!(s.get(0..0), Some(EncStr::empty(Enc::Utf8)));
        assert_eq!(s.get(0..100), None);

        let s = EncStr::from_std("€𐐷b");
        assert_eq!(s.get(0..3), Some(EncStr::from_std("€")));
        assert_eq!(s.get(3..7), Some(EncStr::from_std("𐐷")));
        assert_eq!(s.get(7..8), Some(EncStr::from_std("b")));

        assert_eq!(s.get(0..7), Some(EncStr::from_std("€𐐷")));
        assert_eq!(s.get(3..8), Some(EncStr::from_std("𐐷b")));

        assert_eq!(s.get(0..1), None);
        assert_eq!(s.get(1..3), None);
        assert_eq!(s.get(1..5), None);
        assert_eq!(s.get(0..100), None);
        assert_eq!(s.get(1..1), None);
    }

    #[test]
    fn test_empty() {
        #[cfg(feature = "ascii")]
        {
            let s = EncStr::empty(Enc::Ascii);
            assert_eq!(s.len(), 0);
        }

        let s = EncStr::empty(Enc::Utf8);
        assert_eq!(s.len(), 0);

        let s = EncStr::empty(Enc::Utf32BE);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_std_roundtrip() {
        let s = "Hello World! 😀";
        let e = EncStr::from_std(s);
        assert_eq!(e.as_std(), Some("Hello World! 😀"));
    }

    #[test]
    fn test_chars() {
        let str = Str::from_std("Abc𐐷d");
        assert_eq!(&str.chars().collect::<Vec<_>>(), &['A', 'b', 'c', '𐐷', 'd']);

        let str = Str::from_utf16(&[
            b'A' as u16,
            b'b' as u16,
            b'c' as u16,
            0xD801,
            0xDC37,
            b'd' as u16,
        ])
        .unwrap();
        assert_eq!(&str.chars().collect::<Vec<_>>(), &['A', 'b', 'c', '𐐷', 'd']);

        let str = EncStr::from_chars(&['A', 'b', 'c', '𐐷', 'd']);
        assert_eq!(&str.chars().collect::<Vec<_>>(), &['A', 'b', 'c', '𐐷', 'd']);
    }
}
