//! Implementation and utilities for a generically encoded [`std::ffi::CString`] equivalent type.

use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::cstr::CStr;
use crate::encoding::{Encoding, NullTerminable, ValidateError};
#[cfg(feature = "alloc")]
pub use crate::err::RecodeError;
use crate::str::Str;
use crate::string::String;

/// The cause of an error while creating a [`CString`]
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum CStringErrorCause {
    /// The input wasn't valid for the desired encoding
    Invalid(ValidateError),
    /// The input contains a null byte not in the final position
    HasNull {
        /// The position of the null byte in the input
        idx: usize,
    },
}

/// An error encountered while creating a new [`CString`] from a container of bytes
#[derive(Debug, PartialEq)]
pub struct CStringError {
    bytes: Vec<u8>,
    cause: CStringErrorCause,
}

impl CStringError {
    /// Get the cause of this error
    pub fn cause(&self) -> &CStringErrorCause {
        &self.cause
    }

    /// Consume this error, returning the input bytes which generated the error in the first place.
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

/// An error encountered while converting a [`String`] into a [`CString`]
#[derive(Debug, PartialEq)]
pub struct NulError {
    bytes: Vec<u8>,
    nul_pos: usize,
}

impl NulError {
    /// Returns the position of the null byte in the input that caused [`CString::try_from`] to
    /// fail.
    pub fn nul_position(&self) -> usize {
        self.nul_pos
    }

    /// Consume this error, returning the input bytes which generated the error in the first place.
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

/// A type representing an owned, generically-encoded C-string. This means the string contains a
/// single trailing null byte, with no other null bytes internally.
///
/// This type is to [`CStr`] as [`String`] is to [`Str`] - it represents the owned form of C string,
/// while [`CStr`] represents the borrowed form.
pub struct CString<E>(PhantomData<E>, Vec<u8>);

impl<E: Encoding + NullTerminable> CString<E> {
    /// Create a C string from a byte vector, without checking for interior null
    /// bytes or valid encoding. This method is similar to [`CString::new`],
    /// but without validity checking.
    ///
    /// The trailing null byte will be appended by this method.
    ///
    /// # Safety
    ///
    /// The provided vector must contain no null bytes and be valid for the
    /// current encoding.
    pub unsafe fn from_vec_unchecked(mut bytes: Vec<u8>) -> CString<E> {
        bytes.push(0);
        CString(PhantomData, bytes)
    }

    /// Create a new C string from a container of bytes. The provided data should contain no null
    /// bytes.
    ///
    /// This function will consume and validate the provided data, checking that it contains no null
    /// bytes and is valid for the current encoding. If those checks pass, a single null byte is
    /// appended to the end.
    ///
    /// If you have a [`String<E>`], you should prefer the [`CString::try_from`] implementation.
    /// It is capable of skipping the encoding check and only validating the lack of null bytes.
    pub fn new<T>(bytes: T) -> Result<CString<E>, CStringError>
    where
        T: Into<Vec<u8>>,
    {
        let bytes = bytes.into();
        let nul_pos = bytes.iter().position(|b| *b == 0);
        if let Some(idx) = nul_pos {
            return Err(CStringError {
                bytes,
                cause: CStringErrorCause::HasNull { idx },
            });
        }
        // Can't use map_err due to moving `bytes`, sad :(
        if let Err(e) = E::validate(&bytes) {
            return Err(CStringError {
                bytes,
                cause: CStringErrorCause::Invalid(e),
            });
        }
        Ok(unsafe { Self::from_vec_unchecked(bytes) })
    }

    /// Convert this `CString` into a [`String`] by removing the trailing null. Unlike the
    /// equivalent `std` method, this is infallible because our `CString` is encoding-specific.
    pub fn into_string(self) -> String<E> {
        unsafe { String::from_bytes_unchecked(self.into_bytes()) }
    }

    /// Convert this `CString` into bytes, minus the trailing null byte
    pub fn into_bytes(mut self) -> Vec<u8> {
        self.1.pop();
        self.1
    }

    /// Convert this `CString` into bytes, including the trailing null byte
    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.1
    }
}

impl<E: NullTerminable> fmt::Debug for CString<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <CStr<E> as fmt::Debug>::fmt(self, f)
    }
}

impl<E: NullTerminable> Default for CString<E> {
    fn default() -> Self {
        unsafe { CString::from_vec_unchecked(Vec::new()) }
    }
}

impl<E: NullTerminable> PartialEq for CString<E> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<E: NullTerminable> Eq for CString<E> {}

impl<E: NullTerminable> Hash for CString<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

impl<E: NullTerminable> Deref for CString<E> {
    type Target = CStr<E>;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.1) }
    }
}

impl<E: NullTerminable> DerefMut for CString<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { CStr::from_bytes_with_nul_unchecked_mut(&mut self.1) }
    }
}

impl<E: NullTerminable> AsRef<CStr<E>> for CString<E> {
    fn as_ref(&self) -> &CStr<E> {
        self
    }
}

impl<E: NullTerminable> AsMut<CStr<E>> for CString<E> {
    fn as_mut(&mut self) -> &mut CStr<E> {
        self
    }
}

impl<E: NullTerminable> AsRef<Str<E>> for CString<E> {
    fn as_ref(&self) -> &Str<E> {
        self
    }
}

impl<E: NullTerminable> Borrow<CStr<E>> for CString<E> {
    fn borrow(&self) -> &CStr<E> {
        self
    }
}

impl<E: NullTerminable> BorrowMut<CStr<E>> for CString<E> {
    fn borrow_mut(&mut self) -> &mut CStr<E> {
        self
    }
}

impl<E: NullTerminable> TryFrom<String<E>> for CString<E> {
    type Error = NulError;

    fn try_from(value: String<E>) -> Result<Self, Self::Error> {
        // This can be slightly more efficient than `new` - we know the bytes are valid for `E`,
        // so only need to check for an inner null byte.
        let bytes = value.into_bytes();
        if let Some(nul_pos) = bytes.iter().position(|b| *b == 0) {
            return Err(NulError { bytes, nul_pos });
        }
        Ok(unsafe { CString::from_vec_unchecked(bytes) })
    }
}

impl<E: NullTerminable> TryFrom<alloc::ffi::CString> for CString<E> {
    type Error = ValidateError;

    fn try_from(value: alloc::ffi::CString) -> Result<Self, Self::Error> {
        let bytes = value.into_bytes();
        E::validate(&bytes)?;
        // SAFETY: An std CString is guaranteed to contain no internal null bytes
        //         Bytes have been validated
        Ok(unsafe { CString::from_vec_unchecked(bytes) })
    }
}
