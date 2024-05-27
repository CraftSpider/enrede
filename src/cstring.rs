//! Implementation and utilities for a generically encoded [`std::ffi::CString`] equivalent type.

use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::fmt;
use core::hash::{Hash, Hasher};

use crate::cstr::CStr;
use crate::encoding::{Encoding, NullTerminable, ValidateError};
#[cfg(feature = "alloc")]
pub use crate::err::RecodeError;
use crate::str::Str;
use crate::string::String;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum CStringErrorCause {
    Invalid(ValidateError),
    HasNull { idx: usize },
}

#[derive(Debug, PartialEq)]
pub struct CStringError {
    bytes: Vec<u8>,
    cause: CStringErrorCause,
}

impl CStringError {
    pub fn cause(&self) -> &CStringErrorCause {
        &self.cause
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Debug, PartialEq)]
pub struct NulError {
    bytes: Vec<u8>,
    nul_pos: usize,
}

impl NulError {
    pub fn nul_position(&self) -> usize {
        self.nul_pos
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

pub struct CString<E>(PhantomData<E>, Vec<u8>);

impl<E: Encoding + NullTerminable> CString<E> {
    pub fn new<T>(bytes: T) -> Result<CString<E>, CStringError>
    where
        T: Into<Vec<u8>>,
    {
        let mut bytes = bytes.into();
        let nul_pos = bytes.iter().position(|b| *b == 0);
        if let Some(idx) = nul_pos {
            return Err(CStringError {
                bytes,
                cause: CStringErrorCause::HasNull { idx },
            });
        }
        if let Err(e) = E::validate(&bytes) {
            return Err(CStringError {
                bytes,
                cause: CStringErrorCause::Invalid(e),
            });
        }
        bytes.push(0);
        Ok(CString(PhantomData, bytes))
    }

    pub unsafe fn from_vec_unchecked(mut bytes: Vec<u8>) -> CString<E> {
        bytes.push(0);
        CString(PhantomData, bytes)
    }

    pub fn into_string(self) -> String<E> {
        unsafe { String::from_bytes_unchecked(self.into_bytes()) }
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        self.1.pop();
        self.1
    }

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
            return Err(NulError { bytes, nul_pos })
        }
        Ok(unsafe { CString::from_vec_unchecked(bytes) })
    }
}
