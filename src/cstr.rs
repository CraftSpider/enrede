#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
use core::borrow::Borrow;
use core::ffi::c_char;
use core::marker::PhantomData;
use core::ops::{Bound, Deref, Index};
use core::ptr;
use core::slice::SliceIndex;

#[cfg(feature = "alloc")]
use crate::cstring::CString;
use crate::encoding::{Encoding, NullTerminable, ValidateError};
use crate::str::Str;
use crate::utils::RangeOpen;

#[derive(Debug, PartialEq)]
pub enum FromBytesTilNulError {
    Invalid(ValidateError),
    MissingNull,
}

#[derive(Debug, PartialEq)]
pub enum FromBytesWithNulError {
    Invalid(ValidateError),
    HasNull { idx: usize },
    MissingNull,
}

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

    pub unsafe fn from_bytes_with_nul_unchecked_mut(bytes: &mut [u8]) -> &mut CStr<E> {
        debug_assert!(E::validate(&bytes[..bytes.len() - 1]).is_ok());
        debug_assert_eq!(*bytes.last().unwrap(), 0);
        let ptr = ptr::from_mut(bytes) as *mut CStr<E>;
        // SAFETY: `Str` is `repr(transparent)` containing a [u8].
        //         Provided bytes have precondition of being valid encoding
        unsafe { &mut *ptr }
    }

    pub fn from_bytes_til_nul(bytes: &[u8]) -> Result<&CStr<E>, FromBytesTilNulError> {
        let nul = bytes
            .iter()
            .position(|b| *b == 0)
            .ok_or(FromBytesTilNulError::MissingNull)?;
        E::validate(&bytes[..nul]).map_err(FromBytesTilNulError::Invalid)?;
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(&bytes[..=nul]) })
    }

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
        Ok(unsafe { CStr::from_bytes_with_nul_unchecked(bytes) })
    }

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

    pub fn get<R>(&self, idx: R) -> Option<&CStr<E>>
    where
        R: RangeOpen<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        Some(unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul().get(idx)?) })
    }

    pub fn get_mut<R>(&mut self, idx: R) -> Option<&mut CStr<E>>
    where
        R: RangeOpen<usize> + SliceIndex<[u8], Output = [u8]>,
    {
        self.check_bounds(&idx)?;
        Some(unsafe { CStr::from_bytes_with_nul_unchecked_mut(self.1.get_mut(idx)?) })
    }

    /// Convert this `CStr` into a [`Str`]. Unlike the equivalent std method, this is infallible,
    /// because our `CStr` is encoding-specific instead of arbitrary null-terminated bytes.
    pub fn as_str(&self) -> &Str<E> {
        unsafe { Str::from_bytes_unchecked(self.as_bytes()) }
    }
}

impl<E: NullTerminable> Default for &CStr<E> {
    fn default() -> Self {
        unsafe { CStr::from_bytes_with_nul_unchecked(&[0]) }
    }
}

impl<E: NullTerminable> PartialEq for CStr<E> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

#[cfg(feature = "alloc")]
impl<E: NullTerminable> ToOwned for CStr<E> {
    type Owned = CString<E>;

    fn to_owned(&self) -> Self::Owned {
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
            CStr::<Ascii>::from_bytes_with_nul(b"Hello\0World!"),
            Err(FromBytesWithNulError::HasNull { idx: 5 })
        );
        assert!(matches!(
            CStr::<Utf8>::from_bytes_with_nul(b"Hello\xD8World!"),
            Err(FromBytesWithNulError::Invalid(_))
        ));
    }
}
