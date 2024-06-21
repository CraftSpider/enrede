//! The base for generic encoding support. This module provides the [`Encoding`] trait and its
//! various implementors, such as [`Utf8`].
//!
//! Generally, you want to interact with the crate through the [`Str<E>`] type, however, if you
//! want more low-level encoding operations, you can perform them directly through methods such
//! as [`Encoding::encode`].

use crate::str::Str;
use arrayvec::ArrayVec;
use core::slice;

mod ascii;
mod iso;
mod utf;
mod win;

pub use ascii::*;
pub use iso::*;
pub use utf::*;
pub use win::*;

mod sealed {
    pub trait Sealed: Sized {}
}
use sealed::Sealed;

#[doc(hidden)]
pub trait ArrayLike {
    fn slice(&self) -> &[u8];
}

impl<const N: usize> ArrayLike for ArrayVec<u8, N> {
    fn slice(&self) -> &[u8] {
        self
    }
}

impl<const N: usize> ArrayLike for [u8; N] {
    fn slice(&self) -> &[u8] {
        self
    }
}

impl ArrayLike for u8 {
    fn slice(&self) -> &[u8] {
        slice::from_ref(self)
    }
}

/// An arbitrary encoding. Examples include [`Utf8`], [`Ascii`], or [`Win1252`].
///
/// This trait is sealed, and multiple internal items are unstable, preventing downstream
/// implementations. If you want an encoding not currently supported, please open an issue.
pub trait Encoding: Default + Sealed {
    #[doc(hidden)]
    const REPLACEMENT: char;
    #[doc(hidden)]
    const MAX_LEN: usize;
    #[doc(hidden)]
    type Bytes: ArrayLike;

    #[doc(hidden)]
    fn shorthand() -> &'static str;

    /// Given a byte slice, determine whether it is valid for the current encoding.
    fn validate(bytes: &[u8]) -> Result<(), ValidateError>;

    /// Take a character and encode it directly into the provided buffer. If successful, returns the
    /// length of the buffer that was written.
    fn encode(char: char, out: &mut [u8]) -> Result<usize, EncodeError> {
        match Self::encode_char(char) {
            Some(a) => {
                let a = a.slice();
                if a.len() > out.len() {
                    Err(EncodeError::NeedSpace { len: a.len() })
                } else {
                    out[..a.len()].copy_from_slice(a);
                    Ok(a.len())
                }
            }
            None => Err(EncodeError::InvalidChar),
        }
    }

    /// Given a string in another encoding, re-encode it into this encoding character by character.
    /// On success, returns the length of the output that was written.
    fn recode<E: Encoding>(str: &Str<E>, out: &mut [u8]) -> Result<usize, RecodeError> {
        str.char_indices().try_fold(0, |out_pos, (idx, c)| {
            match Self::encode(c, &mut out[out_pos..]) {
                Ok(len) => Ok(out_pos + len),
                Err(e) => Err(RecodeError {
                    input_used: idx,
                    output_valid: out_pos,
                    cause: match e {
                        EncodeError::NeedSpace { len } => RecodeCause::NeedSpace { len },
                        EncodeError::InvalidChar => RecodeCause::InvalidChar {
                            char: c,
                            len: E::char_len(c),
                        },
                    },
                }),
            }
        })
    }

    #[doc(hidden)]
    fn encode_char(c: char) -> Option<Self::Bytes>;
    #[doc(hidden)]
    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>);

    #[doc(hidden)]
    fn char_bound(str: &Str<Self>, idx: usize) -> bool;
    #[doc(hidden)]
    fn char_len(c: char) -> usize;
}

/// An encoding that can be used in a C-string, meaning it may encode valid data with no internal
/// null bytes.
///
/// ## Requirements
///
/// - The encoding doesn't require null bytes to encode non-null text. This excludes formats
///   such as UTF-16, which needs internal null bytes to encode ASCII value.
/// - The format either doesn't map the null byte to a character, or maps it to the null character.
pub trait NullTerminable: Encoding {}

/// An encoding for which all bytes are always valid, meaning validation of a byte slice for this
/// encoding will never fail.
pub trait AlwaysValid: Encoding {}

/// An error encountered while validating a byte stream for a certain encoding.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidateError {
    valid_up_to: usize,
    error_len: Option<u8>,
}

impl ValidateError {
    /// The length of valid data in the byte stream before the error was encountered. Data up to
    /// this point may be passed to [`Str::from_bytes_unchecked`] soundly.
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// The length of the error, or None if it occurred at the end of the stream. If `Some`,
    /// decoding may skip this many bytes forward, replacing it with a substitution character,
    /// and continue decoding from that point. If `None`, all remaining data in the stream is
    /// invalid. If decoding chunked data, it may represent a cut-off character.
    pub fn error_len(&self) -> Option<usize> {
        self.error_len.map(|e| e as usize)
    }
}

/// An error while encoding a `char` directly into a buffer
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum EncodeError {
    /// The output is too small to hold the encoded character
    NeedSpace {
        /// Space required to encode the character
        len: usize,
    },
    /// The provided character isn't valid for the output encoding
    InvalidChar,
}

/// The cause of a recoding error.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum RecodeCause {
    /// The output is too small to hold the entire input
    NeedSpace {
        /// Space required to encode just one more character
        len: usize,
    },
    /// The input contained a character that isn't valid for the output encoding
    InvalidChar {
        /// The character encountered that isn't supported in the encoding
        char: char,
        /// The length of this character in the input
        len: usize,
    },
}

/// An error encountered while encoding a string into another format.
#[derive(Clone, Debug, PartialEq)]
pub struct RecodeError {
    input_used: usize,
    output_valid: usize,
    cause: RecodeCause,
}

impl RecodeError {
    /// The amount of input successfully consumed. Data up to this point in the input has been
    /// encoded into the output.
    pub fn input_used(&self) -> usize {
        self.input_used
    }

    /// The amount of output with valid data written into it. Data up to this point in the output
    /// may be passed to [`Str::from_bytes_unchecked`] soundly.
    pub fn output_valid(&self) -> usize {
        self.output_valid
    }

    /// The reason encoding stopped. See [`RecodeCause`] for more details on possible reasons.
    pub fn cause(&self) -> &RecodeCause {
        &self.cause
    }
}
