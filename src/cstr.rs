//! Implementation and utilities for a generically encoded [`std::ffi::CStr`] equivalent type.
//!
//! See also the [`CStr<E>`] type.

#[cfg(feature = "alloc")]
use crate::cstring::CString;
use crate::encoding::{AlwaysValid, Encoding, NullTerminable, ValidateError};
use crate::str::Str;
use crate::utils::RangeOpen;
#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
use core::borrow::Borrow;
use core::error::Error;
use core::ffi::c_char;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::RangeBounds;
use core::ops::{Bound, Deref, Index};
use core::slice::SliceIndex;
use core::{fmt, ptr};

/// Error encountered when creating a [`CStr`] with no terminating null byte.
#[non_exhaustive]
#[derive(Debug)]
pub struct MissingNull;

impl fmt::Display for MissingNull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slice missing null byte")
    }
}

impl Error for MissingNull {}

/// Error encountered while creating a [`CStr`] from bytes until a null byte is encountered
#[derive(Debug, PartialEq)]
pub enum FromBytesTilNulError {
    /// The input isn't valid for the desired encoding
    Invalid(ValidateError),
    /// The input doesn't contain any null bytes
    MissingNull,
}

impl fmt::Display for FromBytesTilNulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error creating `CStr` til null: ")?;
        match self {
            FromBytesTilNulError::Invalid(_) => write!(f, "validation failed"),
            FromBytesTilNulError::MissingNull => write!(f, "missing null byte"),
        }
    }
}

impl Error for FromBytesTilNulError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FromBytesTilNulError::Invalid(validate) => Some(validate),
            FromBytesTilNulError::MissingNull => None,
        }
    }
}

/// Error encountered while creating a [`CStr`] from bytes with a single terminating null byte
#[derive(Debug, PartialEq)]
pub enum FromBytesWithNulError {
    /// The input isn't valid for the desired encoding
    Invalid(ValidateError),
    /// The input contains a null byte not in the final position
    HasNull {
        /// The index of the located null byte
        idx: usize,
    },
    /// The input doesn't contain any null bytes
    MissingNull,
}

impl fmt::Display for FromBytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error creating `CStr` with null: ")?;
        match self {
            FromBytesWithNulError::Invalid(_) => write!(f, "validation failed"),
            FromBytesWithNulError::HasNull { .. } => write!(f, "null byte not at end of slice"),
            FromBytesWithNulError::MissingNull => write!(f, "missing null byte"),
        }
    }
}

impl Error for FromBytesWithNulError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FromBytesWithNulError::Invalid(validate) => Some(validate),
            FromBytesWithNulError::HasNull { .. } => None,
            FromBytesWithNulError::MissingNull => None,
        }
    }
}

/// Error encountered while creating a [`CStr`] from an [`AlwaysValid`] encoding.
#[derive(Debug, PartialEq)]
pub enum FromBytesWithNulValidError {
    /// The input contains a null byte not in the final position
    HasNull {
        /// The index of the located null byte
        idx: usize,
    },
    /// The input doesn't contain any null bytes
    MissingNull,
}

impl fmt::Display for FromBytesWithNulValidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error creating `CStr` with null: ")?;
        match self {
            FromBytesWithNulValidError::HasNull { .. } => {
                write!(f, "null byte not at end of slice")
            }
            FromBytesWithNulValidError::MissingNull => write!(f, "missing null byte"),
        }
    }
}

impl Error for FromBytesWithNulValidError {}

/// A C-string slice, representing an encoded string with a single null (or zero) byte at the end.
/// This is normally represented in C as a `char*`, and is the most common form of string value
/// there.
///
/// Not all encodings are valid for use in C strings, as some encodings require the use of internal
/// null bytes to represent many characters. The [`NullTerminable`] trait is implemented by all
/// encodings that may be used in C strings.
///
/// Unlike the [`std::ffi::CStr`] type, this type implements `Deref` to [`Str<E>`]. This is because
/// while the `std` CStr isn't guaranteed to be in any particular encoding, and may not be a valid
/// `str`, this type always contains correctly encoded data.
#[repr(transparent)]
pub struct CStr<E>(PhantomData<E>, [u8]);

impl<E: Encoding + NullTerminable> CStr<E> {
    /// Create a `CStr` from a byte slice without checking whether it is valid for the current
    /// encoding, or whether it ends with a terminating null byte.
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid for the current encoding, contain a single null byte at the
    /// end.
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &CStr<E> {
        debug_assert!(E::validate(&bytes[..bytes.len() - 1]).is_ok());
        debug_assert_eq!(*bytes.last().unwrap(), 0);
        let ptr = ptr::from_ref(bytes) as *const CStr<E>;
        // SAFETY: `Str` is `repr(transparent)` containing a [u8].
        //         Provided bytes have precondition of being valid encoding
        unsafe { &*ptr }
    }

    /// Create a `CStr` from a mutable byte slice without checking whether it is valid for the
    /// current encoding, or whether it ends with a terminating null byte.
    ///
    /// # Safety
    ///
    /// The bytes passed must be valid for the current encoding, contain a single null byte at the
    /// end.
    pub unsafe fn from_bytes_with_nul_unchecked_mut(bytes: &mut [u8]) -> &mut CStr<E> {
        debug_assert!(E::validate(&bytes[..bytes.len() - 1]).is_ok());
        debug_assert_eq!(*bytes.last().unwrap(), 0);
        let ptr = ptr::from_mut(bytes) as *mut CStr<E>;
        // SAFETY: `Str` is `repr(transparent)` containing a [u8].
        //         Provided bytes have precondition of being valid encoding
        unsafe { &mut *ptr }
    }

    /// Create a `CStr` from a byte slice, ending at the first null byte. If there are no null bytes
    /// in the slice, or the data up till the first null isn't valid in the current encoding,
    /// then an error will be returned.
    ///
    /// Data *past* the first null byte isn't validated, and a successful return doesn't mean that
    /// data is valid for the current encoding.
    pub fn from_bytes_til_nul(bytes: &[u8]) -> Result<&CStr<E>, FromBytesTilNulError> {
        let nul = bytes
            .iter()
            .position(|b| *b == 0)
            .ok_or(FromBytesTilNulError::MissingNull)?;
        E::validate(&bytes[..nul]).map_err(FromBytesTilNulError::Invalid)?;
        // SAFETY: End position is the location of first null byte, prior bytes have been validated
        //         for the encoding.
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(&bytes[..=nul]) })
    }

    /// Create a `CStr` from a mutable byte slice, ending at the first null byte. If there are no
    /// null bytes in the slice, or the data up till the first null isn't valid in the current
    /// encoding, then an error will be returned.
    ///
    /// Data *past* the first null byte isn't validated, and a successful return doesn't mean that
    /// data is valid for the current encoding.
    pub fn from_bytes_til_nul_mut(bytes: &mut [u8]) -> Result<&mut CStr<E>, FromBytesTilNulError> {
        let nul = bytes
            .iter()
            .position(|b| *b == 0)
            .ok_or(FromBytesTilNulError::MissingNull)?;
        E::validate(&bytes[..nul]).map_err(FromBytesTilNulError::Invalid)?;
        // SAFETY: End position is the location of first null byte, prior bytes have been validated
        //         for the encoding.
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked_mut(&mut bytes[..=nul]) })
    }

    /// Create a `CStr` from a byte slice, with a single null byte at the end. If there is no null
    /// byte, or there are null bytes at any other position in the slice, an error is returned.
    /// An error will also be returned if the data isn't valid in the current encoding.
    pub fn from_bytes_with_nul(bytes: &[u8]) -> Result<&CStr<E>, FromBytesWithNulError> {
        let end_nul = bytes.last().map(|b| *b == 0).unwrap_or(false);
        if !end_nul {
            return Err(FromBytesWithNulError::MissingNull);
        }
        let slice = &bytes[..bytes.len() - 1];
        let internal_nul = slice.iter().position(|b| *b == 0);
        if let Some(idx) = internal_nul {
            return Err(FromBytesWithNulError::HasNull { idx });
        }
        E::validate(slice).map_err(FromBytesWithNulError::Invalid)?;
        // SAFETY: End position validated to be null and only null, prior bytes have been validated
        //         for the encoding.
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(bytes) })
    }

    /// Create a `CStr` from a mutable byte slice, with a single null byte at the end. If there is
    /// no null byte, or there are null bytes at any other position in the slice, an error is
    /// returned. An error will also be returned if the data isn't valid in the current encoding.
    pub fn from_bytes_with_nul_mut(
        bytes: &mut [u8],
    ) -> Result<&mut CStr<E>, FromBytesWithNulError> {
        let end_nul = bytes.last().map(|b| *b == 0).unwrap_or(false);
        if !end_nul {
            return Err(FromBytesWithNulError::MissingNull);
        }
        let slice = &bytes[..bytes.len() - 1];
        let internal_nul = slice.iter().position(|b| *b == 0);
        if let Some(idx) = internal_nul {
            return Err(FromBytesWithNulError::HasNull { idx });
        }
        E::validate(slice).map_err(FromBytesWithNulError::Invalid)?;
        // SAFETY: End position validated to be null and only null, prior bytes have been validated
        //         for the encoding.
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked_mut(bytes) })
    }

    /// Get a pointer suitable for passing to native C code. The returned point may be either `i8`
    /// or `u8` depending on the target platform.
    ///
    /// The returned pointer lives for as long as the `CStr`. See [`std::ffi::CStr::as_ptr`] for
    /// further details and warnings on lifetime considerations.
    pub fn as_ptr(&self) -> *const c_char {
        ptr::from_ref(&self.1).cast()
    }

    /// Get the underlying bytes for this string, including the terminating null byte.
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.1
    }

    unsafe fn as_bytes_with_nul_mut(&mut self) -> &mut [u8] {
        &mut self.1
    }

    fn check_bounds<R>(&self, idx: &R) -> Option<()>
    where
        R: RangeOpen<usize>,
    {
        let start = idx.start_bound();
        let start_idx = match start {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        };

        let end_idx = self.as_bytes().len();

        if !self.is_char_boundary(start_idx) || !self.is_char_boundary(end_idx) {
            None
        } else {
            Some(())
        }
    }

    /// Return a subslice of this `CStr`. This is a non-panicking alternative to indexing, returning
    /// [`None`] whenever indexing would panic.
    ///
    /// Unlike with `Str`, indexing a `CStr` may only be done with open-ended ranges, EG `idx..` or
    /// `..`. Otherwise it would be possible to create a `CStr` that didn't end with a terminating
    /// null byte.
    pub fn get<R>(&self, idx: R) -> Option<&CStr<E>>
    where
        R: RangeOpen<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        // SAFETY: The provided range has been validated as landing on character boundaries.
        //         Our internal bytes are guaranteed valid for the encoding.
        //         Final byte is already guaranteed to be null
        Some(unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul().get(idx)?) })
    }

    /// Return a mutable subslice of this `CStr`. This is a non-panicking alternative to indexing,
    /// returning [`None`] whenever indexing would panic.
    ///
    /// Unlike with `Str`, indexing a `CStr` may only be done with open-ended ranges, EG `idx..` or
    /// `..`. Otherwise it would be possible to create a `CStr` that didn't end with a terminating
    /// null byte.
    pub fn get_mut<R>(&mut self, idx: R) -> Option<&mut CStr<E>>
    where
        R: RangeOpen<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        // SAFETY: The provided range has been validated as landing on character boundaries.
        //         Our internal bytes are guaranteed valid for the encoding.
        //         Final byte is already guaranteed to be null
        Some(unsafe { CStr::from_bytes_with_nul_unchecked_mut(self.1.get_mut(idx)?) })
    }

    /// Convert this `CStr` into a [`Str`]. Unlike the equivalent std method, this is infallible,
    /// because our `CStr` is encoding-specific instead of arbitrary null-terminated bytes.
    ///
    /// Note that this method should rarely be needed, as `CStr` implements `Deref` into [`Str`].
    pub fn as_str(&self) -> &Str<E> {
        // This is the impl of `Deref` - no using `Str` methods.
        let bytes = self.as_bytes_with_nul();
        // SAFETY: Our internal bytes are guaranteed valid for the encoding.
        unsafe { Str::from_bytes_unchecked(&bytes[..bytes.len() - 1]) }
    }

    /// Convert this `CStr` into a mutable [`Str`]. This method is unsafe because it is possible to
    /// write null bytes into the string via methods such as [`Str::copy_from`].
    ///
    /// # Safety
    ///
    /// The returned reference must not be used to write null bytes into the C string.
    pub unsafe fn as_str_mut(&mut self) -> &mut Str<E> {
        let bytes = self.as_bytes_with_nul_mut();
        let len = bytes.len();
        Str::from_bytes_unchecked_mut(&mut bytes[..len - 1])
    }

    /// Copy the data of another C-string into this C-string. Due to the limitations of slicing C
    /// strings only till the end, the [`CStr::copy_range`] method is provided as it is most often
    /// more useful than this one.
    pub fn copy_from(&mut self, other: &CStr<E>) {
        if self.len() != other.len() {
            panic!(
                "Source string length ({}) doesn't match destination C string length ({})",
                self.len(),
                other.len(),
            );
        }
        self.1.copy_from_slice(other.as_bytes());
    }

    /// Copy the data from one C string into this one, taking the data from the first range and
    /// inserting it into the second range. This method panics if any of the range ends don't fall
    /// on a character boundary. This is the more powerful variant of [`CStr::copy_from`]`.
    pub fn copy_range<R1, R2>(&mut self, other: &CStr<E>, src_range: R1, dest_range: R2)
    where
        R1: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]> + Clone,
        R2: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]> + Clone,
    {
        #[cfg(feature = "alloc")]
        use core::fmt::Write;

        #[cfg(feature = "alloc")]
        fn bounds_to_range<W: Write>(f: &mut W, range: &impl RangeBounds<usize>) -> fmt::Result {
            match range.start_bound() {
                Bound::Included(i) => write!(f, "{}..", i)?,
                Bound::Excluded(e) => write!(f, "{}..", e + 1)?,
                Bound::Unbounded => (),
            }
            match range.end_bound() {
                Bound::Included(i) => write!(f, "={}", i)?,
                Bound::Excluded(e) => write!(f, "{}", e)?,
                Bound::Unbounded => (),
            }
            Ok(())
        }

        #[cfg(feature = "alloc")]
        let self_len = self.len();

        let dest = self.1.get_mut(src_range.clone()).unwrap_or_else(|| {
            #[cfg(feature = "alloc")]
            let str = {
                use alloc::string::String;
                let mut str = String::new();
                write!(&mut str, "Source string range (").unwrap();
                bounds_to_range(&mut str, &src_range).unwrap();
                write!(
                    &mut str,
                    ") out of bounds for C string length ({})",
                    self_len
                )
                .unwrap();
                str
            };
            #[cfg(not(feature = "alloc"))]
            let str = "Source string range out of bounds for C string length";
            panic!("{}", str)
        });

        let src = other.1.get(dest_range.clone()).unwrap_or_else(|| {
            #[cfg(feature = "alloc")]
            let str = {
                use alloc::string::String;
                let mut str = String::new();
                write!(&mut str, "Destination string range (").unwrap();
                bounds_to_range(&mut str, &dest_range).unwrap();
                write!(
                    &mut str,
                    ") out of bounds for C string length ({})",
                    other.len()
                )
                .unwrap();
                str
            };
            #[cfg(not(feature = "alloc"))]
            let str = "Destination string range out of bounds for C string length";
            panic!("{}", str)
        });

        if dest.len() != src.len() {
            panic!(
                "Source range length ({}) doesn't match destination range length ({})",
                src.len(),
                dest.len(),
            );
        }

        dest.copy_from_slice(src)
    }

    /// Split this string at an index, returning the two substrings on either side. This method
    /// panics if the index doesn't lie on a character boundary. The right-side substring is
    /// returned as a `CStr`, as it retains the trailing null.
    pub fn split_at(&self, idx: usize) -> Option<(&Str<E>, &CStr<E>)> {
        if self.is_char_boundary(idx) && idx < self.len() {
            let (start, end) = self.1.split_at(idx);
            // SAFETY: Index is a character boundary. Internal data guaranteed valid.
            let start = unsafe { Str::from_bytes_unchecked(start) };
            // SAFETY: Index is a character boundary. Trailing data guaranteed a valid C string.
            let end = unsafe { CStr::from_bytes_with_nul_unchecked(end) };
            Some((start, end))
        } else {
            None
        }
    }

    /// Split this string mutably at an index, returning the two substrings on either side. This
    /// method panics if the index doesn't lie on a character boundary. The right-side substring is
    /// returned as a `CStr`, as it retains the trailing null.
    pub fn split_at_mut(&mut self, idx: usize) -> Option<(&mut Str<E>, &mut CStr<E>)> {
        if self.is_char_boundary(idx) && idx < self.len() {
            let (start, end) = self.1.split_at_mut(idx);
            // SAFETY: Index is a character boundary. Internal data guaranteed valid.
            let start = unsafe { Str::from_bytes_unchecked_mut(start) };
            // SAFETY: Index is a character boundary. Trailing data guaranteed a valid C string.
            let end = unsafe { CStr::from_bytes_with_nul_unchecked_mut(end) };
            Some((start, end))
        } else {
            None
        }
    }
}

impl<E: NullTerminable + AlwaysValid> CStr<E> {
    /// Create a `CStr` from a byte slice, ending at the first null byte. See
    /// [`CStr::from_bytes_til_nul`]
    ///
    /// This method is provided for encodings that have no invalid byte patterns, meaning encoding
    /// validity checking is skipped.
    pub fn from_bytes_til_nul_valid(bytes: &[u8]) -> Result<&CStr<E>, MissingNull> {
        let nul_pos = bytes.iter().position(|b| *b == 0).ok_or(MissingNull)?;
        // SAFETY: Encoding has no invalid byte patterns. Data contains no internal nulls.
        Ok(unsafe { Self::from_bytes_with_nul_unchecked(&bytes[..=nul_pos]) })
    }

    /// Create a `CStr` from a mutable byte slice, ending at the first null byte. See
    /// [`CStr::from_bytes_til_nul_mut`]
    ///
    /// This method is provided for encodings that have no invalid byte patterns, meaning encoding
    /// validity checking is skipped.
    pub fn from_bytes_til_nul_valid_mut(bytes: &mut [u8]) -> Result<&mut CStr<E>, MissingNull> {
        let nul_pos = bytes.iter().position(|b| *b == 0).ok_or(MissingNull)?;
        // SAFETY: Encoding has no invalid byte patterns. Data contains no internal nulls.
        Ok(unsafe { Self::from_bytes_with_nul_unchecked_mut(&mut bytes[..=nul_pos]) })
    }

    /// Create a `CStr` from a byte slice, with a single null byte at the end. See
    /// [`CStr::from_bytes_with_nul`]
    ///
    /// This method is provided for encodings that have no invalid byte patterns, meaning encoding
    /// validity checking is skipped.
    pub fn from_bytes_with_nul_valid(bytes: &[u8]) -> Result<&CStr<E>, FromBytesWithNulValidError> {
        let end_nul = bytes.last().map(|b| *b == 0).unwrap_or(false);
        if !end_nul {
            return Err(FromBytesWithNulValidError::MissingNull);
        }
        let slice = &bytes[..bytes.len() - 1];
        let internal_nul = slice.iter().position(|b| *b == 0);
        if let Some(idx) = internal_nul {
            return Err(FromBytesWithNulValidError::HasNull { idx });
        }
        // SAFETY: End position validated to be null and only null, all bytes are valid for this
        //         encoding.
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(bytes) })
    }

    /// Create a `CStr` from a mutable byte slice, with a single null byte at the end. See
    /// [`CStr::from_bytes_with_nul_mut`]
    /// This method is provided for encodings that have no invalid byte patterns, meaning encoding
    /// validity checking is skipped.
    pub fn from_bytes_with_nul_valid_mut(
        bytes: &mut [u8],
    ) -> Result<&mut CStr<E>, FromBytesWithNulValidError> {
        let end_nul = bytes.last().map(|b| *b == 0).unwrap_or(false);
        if !end_nul {
            return Err(FromBytesWithNulValidError::MissingNull);
        }
        let slice = &bytes[..bytes.len() - 1];
        let internal_nul = slice.iter().position(|b| *b == 0);
        if let Some(idx) = internal_nul {
            return Err(FromBytesWithNulValidError::HasNull { idx });
        }
        // SAFETY: End position validated to be null and only null, all bytes are valid for this
        //         encoding.
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked_mut(bytes) })
    }
}

impl<E: NullTerminable> fmt::Debug for CStr<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c")?;
        <Str<E> as fmt::Debug>::fmt(self, f)
    }
}

impl<E: NullTerminable> Default for &CStr<E> {
    fn default() -> Self {
        // SAFETY: Empty string slice can never be invalid. Obviously there is a single null byte.
        unsafe { CStr::from_bytes_with_nul_unchecked(&[0]) }
    }
}

impl<E: NullTerminable> PartialEq for CStr<E> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<E: NullTerminable> Eq for CStr<E> {}

impl<E: NullTerminable> Hash for CStr<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

#[cfg(feature = "alloc")]
impl<E: NullTerminable> ToOwned for CStr<E> {
    type Owned = CString<E>;

    fn to_owned(&self) -> Self::Owned {
        // SAFETY: Internal bytes are guaranteed valid for encoding and to contain no null bytes.
        unsafe { CString::from_vec_unchecked(self.as_bytes().to_vec()) }
    }
}

impl<E, R> Index<R> for CStr<E>
where
    E: NullTerminable,
    R: RangeOpen<usize> + SliceIndex<[u8], Output = [u8]>,
{
    type Output = CStr<E>;

    fn index(&self, index: R) -> &Self::Output {
        self.get(index)
            .expect("Attempted to slice C-string at non-character boundary")
    }
}

impl<E: NullTerminable> Deref for CStr<E> {
    type Target = Str<E>;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<E: NullTerminable> AsRef<Str<E>> for CStr<E> {
    fn as_ref(&self) -> &Str<E> {
        self
    }
}

impl<E: NullTerminable> Borrow<Str<E>> for CStr<E> {
    fn borrow(&self) -> &Str<E> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{Ascii, Utf8};

    #[test]
    fn test_from_bytes_with_nul() {
        assert!(CStr::<Ascii>::from_bytes_with_nul(b"Hello World!\0").is_ok());
        assert_eq!(
            CStr::<Ascii>::from_bytes_with_nul(b"Hello World!"),
            Err(FromBytesWithNulError::MissingNull)
        );
        assert_eq!(
            CStr::<Ascii>::from_bytes_with_nul(b"Hello\0World!\0"),
            Err(FromBytesWithNulError::HasNull { idx: 5 })
        );
        assert!(matches!(
            CStr::<Utf8>::from_bytes_with_nul(b"Hello\xD8World!\0"),
            Err(FromBytesWithNulError::Invalid(_))
        ));
    }

    #[test]
    fn test_from_bytes_til_nul() {
        let base = CStr::<Ascii>::from_bytes_til_nul(b"Hello World!\0").unwrap();
        assert_eq!(
            CStr::<Ascii>::from_bytes_til_nul(b"Hello World!\0Goodbye"),
            Ok(base),
        );
        assert_eq!(
            CStr::<Ascii>::from_bytes_til_nul(b"Hello World!"),
            Err(FromBytesTilNulError::MissingNull),
        );
        assert!(matches!(
            CStr::<Utf8>::from_bytes_til_nul(b"Hello\x86World!\0"),
            Err(FromBytesTilNulError::Invalid(_))
        ));
        assert_eq!(
            CStr::<Utf8>::from_bytes_til_nul(b"Hello World!\0\x86"),
            CStr::from_bytes_til_nul(b"Hello World!\0"),
        );
    }

    #[test]
    fn test_bytes_with_nul() {
        let c = CStr::<Utf8>::from_bytes_til_nul(b"Hello World!\0").unwrap();

        assert_eq!(c.as_bytes(), b"Hello World!");
        assert_eq!(c.as_bytes_with_nul(), b"Hello World!\0");
    }
}
