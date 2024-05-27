use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::cstr::CStr;
use crate::encoding::{Encoding, NullTerminable, ValidateError};
#[cfg(feature = "alloc")]
pub use crate::err::RecodeError;
use crate::string::String;

pub enum NewErrorCause {
    Invalid(ValidateError),
    HasNull { idx: usize },
}

pub struct NewError {
    bytes: Vec<u8>,
    cause: NewErrorCause,
}

pub struct CString<E>(PhantomData<E>, Vec<u8>);

impl<E: Encoding + NullTerminable> CString<E> {
    pub fn new<T>(bytes: T) -> Result<CString<E>, NewError>
    where
        T: Into<Vec<u8>>,
    {
        let mut bytes = bytes.into();
        let nul_pos = bytes.iter().position(|b| *b == 0);
        if let Some(idx) = nul_pos {
            return Err(NewError {
                bytes,
                cause: NewErrorCause::HasNull { idx },
            });
        }
        if let Err(e) = E::validate(&bytes) {
            return Err(NewError {
                bytes,
                cause: NewErrorCause::Invalid(e),
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
