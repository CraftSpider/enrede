//! Implementation and utilities for a generically encoded [`std::ffi::CStr`] equivalent type.
//!
//! See also the [`CStr<E>`] type.

#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
use core::borrow::Borrow;
use core::ffi::c_char;
use core::marker::PhantomData;
use core::ops::{Bound, Deref, Index};
use core::slice::SliceIndex;
use core::{fmt, ptr};
use core::hash::{Hash, Hasher};

#[cfg(feature = "alloc")]
use crate::cstring::CString;
use crate::encoding::{Encoding, NullTerminable, ValidateError};
use crate::str::Str;
use crate::utils::RangeOpen;

/// Error encountered while creating a [`CStr`] from bytes until a null byte is encountered
#[derive(Debug, PartialEq)]
pub enum FromBytesTilNulError {
    /// The input isn't valid for the desired encoding
    Invalid(ValidateError),
    /// The input doesn't contain any null bytes
    MissingNull,
}

/// Error encountered while creating a [`CStr`] from bytes with a single terminating null byte
#[derive(Debug, PartialEq)]
pub enum FromBytesWithNulError {
    /// The input isn't valid for the desired encoding
    Invalid(ValidateError),
    /// The input contains a null byte not in the final position
    HasNull {
        /// The index of the located null byte
        idx: usize
    },
    /// The input doesn't contain any null bytes
    MissingNull,
}

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
    pub fn as_str(&self) -> &Str<E> {
        // This is the impl of `Deref` - no using `Str` methods.
        let bytes = self.as_bytes_with_nul();
        // SAFETY: Our internal bytes are guaranteed valid for the encoding.
        unsafe { Str::from_bytes_unchecked(&bytes[..bytes.len() - 1]) }
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