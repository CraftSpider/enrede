//! Implementation and utilities for a generically encoded [`str`] equivalent type.
//!
//! See also the [`Str<E>`] type.

#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
#[cfg(feature = "alloc")]
use alloc::vec;
use bytemuck::must_cast_slice as cast_slice;
use core::cmp::Ordering;
use core::fmt::Write;
use core::marker::PhantomData;
use core::ops::{Bound, Index, RangeBounds};
use core::slice::SliceIndex;
use core::{fmt, mem, ptr, slice};

#[cfg(feature = "alloc")]
use crate::encoding::RecodeCause;
use crate::encoding::{Encoding, Utf16, Utf32, Utf8, ValidateError};
#[cfg(feature = "alloc")]
pub use crate::err::RecodeError;
#[cfg(feature = "alloc")]
use crate::string::String;

mod iter;

pub use iter::{CharIndices, Chars};

/// Implementation of a generically encoded [`str`] type. This type is similar to the standard
/// library [`str`] type in many ways, but instead of having a fixed UTF-8 encoding scheme, it uses
/// an encoding determined by the generic `E` it is provided.
///
/// `Str` only implements `==` between instances with the same encoding. To compare strings of
/// different encoding by characters, use `a.chars().eq(b.chars())`.
///
/// ## Invariant
///
/// Rust libraries may assume that a `Str<E>` is valid for the [`Encoding`] `E`.
///
/// Constructing non-`E` string slices is not immediate UB, but any function called on it may assume
/// that it is valid.
#[repr(transparent)]
pub struct Str<E>(PhantomData<E>, [u8]);

impl<E: Encoding> Str<E> {
    /// Create a `Str` from a byte slice without checking whether it is valid for the current
    /// encoding.
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid for the current encoding.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Str<E> {
        debug_assert!(E::validate(bytes).is_ok());
        let ptr = ptr::from_ref(bytes) as *const Str<E>;
        // SAFETY: `Str` is `repr(transparent)` containing a [u8].
        //         Provided bytes have precondition of being valid encoding
        unsafe { &*ptr }
    }

    /// Create a `Str` from a mutable byte slice without checking whether it is valid for the
    /// current encoding.
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid for the current encoding.
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Str<E> {
        debug_assert!(E::validate(bytes).is_ok());
        let ptr = ptr::from_mut(bytes) as *mut Str<E>;
        // SAFETY: `Str` is `repr(transparent)` containing a [u8].
        //         Provided bytes have precondition of being valid encoding
        unsafe { &mut *ptr }
    }

    /// Create a `Str` from a byte slice, validating the encoding and returning a [`ValidateError`]
    /// if it is not a valid string in the current encoding.
    pub fn from_bytes(bytes: &[u8]) -> Result<&Str<E>, ValidateError> {
        E::validate(bytes)?;
        // SAFETY: Bytes have been validated, they are guaranteed valid for the encoding
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Create a `Str` from a mutable byte slice, validating the encoding and returning a
    /// [`ValidateError`] if it is not a valid string in the current encoding.
    pub fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut Str<E>, ValidateError> {
        E::validate(bytes)?;
        // SAFETY: Bytes have been validated, they are guaranteed valid for the encoding
        Ok(unsafe { Self::from_bytes_unchecked_mut(bytes) })
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
    pub fn as_bytes(&self) -> &[u8] {
        &self.1
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

    /// Return a subslice of this `Str`. This is a non-panicking alternative to indexing, returning
    /// [`None`] whenever indexing would panic.
    pub fn get<R>(&self, idx: R) -> Option<&Self>
    where
        R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        // SAFETY: The provided range has been validated as landing on character boundaries.
        //         Our internal bytes are guaranteed valid for the encoding.
        Some(unsafe { Str::from_bytes_unchecked(self.as_bytes().get(idx)?) })
    }

    /// Return a mutable subslice of this `Str`. This is a non-panicking alternative to indexing,
    /// returning [`None`] whenever indexing would panic.
    pub fn get_mut<R>(&mut self, idx: R) -> Option<&mut Self>
    where
        R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        // SAFETY: The provided range has been validated as landing on character boundaries.
        //         Our internal bytes are guaranteed valid for the encoding.
        Some(unsafe { Str::from_bytes_unchecked_mut(self.1.get_mut(idx)?) })
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
            Ordering::Less => E::char_bound(self, idx),
        }
    }

    /// Returns `true` if the given pattern is a prefix of this string slice, `false` otherwise.
    pub fn starts_with(&self, other: &Self) -> bool {
        self.as_bytes().starts_with(other.as_bytes())
    }

    /// Returns `true` if the given pattern is a suffix of this string slice, `false` otherwise.
    pub fn ends_with(&self, other: &Self) -> bool {
        self.as_bytes().ends_with(other.as_bytes())
    }

    /// Return an iterator over the [`char`]s of this string slice. See [`str::chars`] for caveats
    /// about this method.
    pub fn chars(&self) -> Chars<'_, E> {
        Chars::new(self)
    }

    /// Return an iterator over the [`char`]s of this string slice and their positions. See
    /// [`str::char_indices`] for caveats about this method.
    pub fn char_indices(&self) -> CharIndices<'_, E> {
        CharIndices::new(self)
    }

    /// Get this `Str` in a different [`Encoding`]. This method allocates a new [`String`] with the
    /// desired encoding, and returns an error if the source string contains any characters that
    /// cannot be represented in the destination encoding.
    #[cfg(feature = "alloc")]
    pub fn recode<E2: Encoding>(&self) -> Result<String<E2>, RecodeError> {
        let mut ptr = self;
        let mut total_len = 0;
        let mut out = vec![0; self.1.len()];
        loop {
            match E2::recode(ptr, &mut out[total_len..]) {
                Ok(len) => {
                    out.truncate(total_len + len);
                    // SAFETY: Value written into `out` by `recode` is guaranteed valid in encoding
                    //         E2.
                    return Ok(unsafe { String::<E2>::from_bytes_unchecked(out) });
                }
                Err(e) => match e.cause() {
                    RecodeCause::NeedSpace { .. } => {
                        out.resize(out.len() + self.1.len(), 0);
                        ptr = &ptr[e.input_used()..];
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
    pub fn recode_lossy<E2: Encoding>(&self) -> String<E2> {
        let mut ptr = self;
        let mut total_len = 0;
        let mut out = vec![0; self.1.len()];
        loop {
            match E2::recode(ptr, &mut out[total_len..]) {
                Ok(len) => {
                    out.truncate(total_len + len);
                    // SAFETY: Value written into `out` by `recode` is guaranteed valid in encoding
                    //         E2.
                    return unsafe { String::from_bytes_unchecked(out) };
                }
                Err(e) => match e.cause() {
                    RecodeCause::NeedSpace { .. } => {
                        out.resize(out.len() + self.1.len(), 0);
                        ptr = &ptr[e.input_used()..];
                        total_len += e.output_valid();
                    }
                    &RecodeCause::InvalidChar { char: _, len } => {
                        let replace_len = E2::char_len(E2::REPLACEMENT);
                        out.resize(out.len() + replace_len, 0);
                        E2::encode(E2::REPLACEMENT, &mut out[total_len + e.output_valid()..])
                            .unwrap();
                        ptr = &ptr[e.input_used() + len..];
                        total_len += e.output_valid() + replace_len;
                    }
                },
            }
        }
    }
}

impl Str<Utf8> {
    /// Equivalent to [`Str::from_bytes_unchecked`] but for UTF-8 specifically
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid UTF-8.
    pub unsafe fn from_utf8_unchecked(str: &[u8]) -> &Self {
        // SAFETY: Precondition that input is valid UTF-8
        Self::from_bytes_unchecked(str)
    }

    /// Equivalent to [`Str::from_bytes`] but for UTF-8 specifically
    pub fn from_utf8(str: &[u8]) -> Result<&Self, ValidateError> {
        Self::from_bytes(str)
    }

    /// Convert a [`str`] directly into a [`Str<Utf8>`].
    pub fn from_std(value: &str) -> &Str<Utf8> {
        // SAFETY: `&str` is UTF-8 by its validity guarantees.
        unsafe { Self::from_bytes_unchecked(value.as_bytes()) }
    }

    /// Convert a [`Str<Utf8>`] directly into a [`str`]
    pub fn as_std(&self) -> &str {
        // SAFETY: `&Str` is UTF-8 by our validity guarantees.
        unsafe { core::str::from_utf8_unchecked(&self.1) }
    }
}

impl Str<Utf16> {
    /// Equivalent to [`Str::from_bytes_unchecked`] but for UTF-16 specifically
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid UTF-16.
    pub unsafe fn from_utf16_unchecked(str: &[u16]) -> &Self {
        // SAFETY: Precondition that input is valid UTF-16
        Self::from_bytes_unchecked(cast_slice(str))
    }

    /// Equivalent to [`Str::from_bytes`] but for UTF-16 specifically
    pub fn from_utf16(str: &[u16]) -> Result<&Self, ValidateError> {
        Self::from_bytes(cast_slice(str))
    }
}

impl Str<Utf32> {
    /// Equivalent to [`Str::from_bytes_unchecked`] but for UTF-32 specifically
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid UTF-32.
    pub unsafe fn from_utf32_unchecked(str: &[u32]) -> &Self {
        // SAFETY: Precondition that input is valid UTF-32
        Self::from_bytes_unchecked(cast_slice(str))
    }

    /// Equivalent to [`Str::from_bytes`] but for UTF-32 specifically
    pub fn from_utf32(str: &[u32]) -> Result<&Self, ValidateError> {
        Self::from_bytes(cast_slice(str))
    }

    /// Convert a [`&[char]`] directly into a [`Str<Utf32>`]
    pub fn from_chars(str: &[char]) -> &Self {
        // SAFETY: Utf32 encoding is exactly equivalent to `char` encoding.
        unsafe { Self::from_bytes_unchecked(cast_slice(str)) }
    }

    /// Attemp to convert a [`Str<Utf32>`] directly into a [`&[char]`]. This will fail if the `Str`
    /// is not sufficiently aligned for a `char`.
    pub fn try_chars(&self) -> Option<&[char]> {
        let len = self.1.len();
        let ptr = ptr::from_ref(&self.1);
        if (ptr.cast::<()>() as usize) % mem::align_of::<char>() != 0 {
            None
        } else {
            // SAFETY: We have guaranteed correct alignment, and Utf32 encoding is exactly
            //         equivalent to `char` encoding.
            Some(unsafe { slice::from_raw_parts(ptr.cast(), len / 4) })
        }
    }
}

impl<E: Encoding> fmt::Debug for Str<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\"", E::shorthand())?;
        for c in self.chars() {
            f.write_char(c)?;
        }
        write!(f, "\"")
    }
}

impl<E: Encoding> fmt::Display for Str<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.chars() {
            f.write_char(c)?;
        }
        Ok(())
    }
}

impl<E: Encoding> Default for &Str<E> {
    fn default() -> Self {
        // SAFETY: Empty string slice can never be invalid
        unsafe { Str::from_bytes_unchecked(&[]) }
    }
}

#[cfg(feature = "alloc")]
impl<E: Encoding> ToOwned for Str<E> {
    type Owned = String<E>;

    fn to_owned(&self) -> Self::Owned {
        let bytes = self.as_bytes().to_vec();
        // SAFETY: Our internal bytes are guaranteed valid for our encoding
        unsafe { String::from_bytes_unchecked(bytes) }
    }
}

impl<E, R> Index<R> for Str<E>
where
    E: Encoding,
    R: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>,
{
    type Output = Str<E>;

    fn index(&self, index: R) -> &Self::Output {
        self.get(index)
            .expect("Attempted to slice string at non-character boundary")
    }
}

impl<E: Encoding> PartialEq for Str<E> {
    fn eq(&self, other: &Str<E>) -> bool {
        self.1 == other.1
    }
}

impl<E: Encoding> Eq for Str<E> {}

// Encoding-specific implementations

impl<'a> From<&'a Str<Utf8>> for &'a str {
    fn from(value: &'a Str<Utf8>) -> Self {
        value.as_std()
    }
}

impl<'a> From<&'a str> for &'a Str<Utf8> {
    fn from(value: &'a str) -> Self {
        Str::from_std(value)
    }
}

impl<'a> From<&'a [char]> for &'a Str<Utf32> {
    fn from(value: &'a [char]) -> Self {
        Str::from_chars(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use crate::encoding::{Ascii, Win1252};

    #[test]
    fn test_chars() {
        let str = Str::from_std("Abc𐐷d");
        assert_eq!(&str.chars().collect::<Vec<_>>(), &['A', 'b', 'c', '𐐷', 'd'],);

        let str = Str::<Utf16>::from_utf16(&[
            b'A' as u16,
            b'b' as u16,
            b'c' as u16,
            0xD801,
            0xDC37,
            b'd' as u16,
        ])
        .unwrap();
        assert_eq!(&str.chars().collect::<Vec<_>>(), &['A', 'b', 'c', '𐐷', 'd'],);

        let str = Str::from_chars(&['A', 'b', 'c', '𐐷', 'd']);
        assert_eq!(&str.chars().collect::<Vec<_>>(), &['A', 'b', 'c', '𐐷', 'd'],);
    }

    #[test]
    fn test_char_indices() {
        let str = Str::from_std("Abc𐐷d");
        assert_eq!(
            &str.char_indices().collect::<Vec<_>>(),
            &[(0, 'A'), (1, 'b'), (2, 'c'), (3, '𐐷'), (7, 'd')],
        );

        let str = Str::<Utf16>::from_utf16(&[
            b'A' as u16,
            b'b' as u16,
            b'c' as u16,
            0xD801,
            0xDC37,
            b'd' as u16,
        ])
        .unwrap();
        assert_eq!(
            &str.char_indices().collect::<Vec<_>>(),
            &[(0, 'A'), (2, 'b'), (4, 'c'), (6, '𐐷'), (10, 'd')],
        );

        let str = Str::from_chars(&['A', 'b', 'c', '𐐷', 'd']);
        assert_eq!(
            &str.char_indices().collect::<Vec<_>>(),
            &[(0, 'A'), (4, 'b'), (8, 'c'), (12, '𐐷'), (16, 'd')],
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_recode_small_to_large() {
        let a = Str::from_std("Hello World!");
        let b = a.recode::<Utf32>().unwrap();

        assert_eq!(
            &*b,
            Str::from_chars(&['H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd', '!']),
        );

        let a = Str::from_std("A𐐷b");
        let b = a.recode::<Utf16>().unwrap();

        assert_eq!(
            &*b,
            Str::from_utf16(&[b'A' as u16, 0xD801, 0xDC37, b'b' as u16]).unwrap()
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_recode_invalid_chars() {
        let a = Str::from_std("A𐐷b");
        let b = a.recode::<Ascii>();

        assert_eq!(
            b,
            Err(RecodeError {
                valid_up_to: 1,
                char: '𐐷',
                char_len: 4,
            })
        );

        let a = Str::from_std("€𐐷b");
        let b = a.recode::<Win1252>();

        assert_eq!(
            b,
            Err(RecodeError {
                valid_up_to: 3,
                char: '𐐷',
                char_len: 4,
            })
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_recode_lossy_invalid_chars() {
        let a = Str::from_std("A𐐷b");
        let b = a.recode_lossy::<Ascii>();

        assert_eq!(&*b, Str::from_bytes(b"A\x1Ab").unwrap());

        let a = Str::from_std("€𐐷b");
        let b = a.recode_lossy::<Win1252>();

        assert_eq!(&*b, Str::from_bytes(b"\x80\x1Ab").unwrap());
    }
}
