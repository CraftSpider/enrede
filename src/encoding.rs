//! The base for generic encoding support. This module provides the [`Encoding`] trait and its
//! various implementors, such as [`Utf8`].
//!
//! Generally, you want to interact with the crate through the [`Str<E>`] type, however, if you
//! want more low-level encoding operations, you can perform them directly through methods such
//! as [`Encoding::encode`].

use crate::str::Str;
use crate::utils::IntoAV;
use arrayvec::ArrayVec;
use core::error::Error;
use core::{fmt, slice};

mod ascii;
mod iso;
mod jis;
mod mac;
mod utf;
mod win;

pub use ascii::*;
pub use iso::*;
pub use jis::*;
pub use mac::*;
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

    #[doc(hidden)]
    fn dyn_enc() -> Enc;

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

    /// Determine whether the provided index is a character boundary in the provided string.
    /// Implementations may generally assume idx is in-bounds, though this is not a safety
    /// precondition.
    #[doc(hidden)]
    fn char_bound(str: &Str<Self>, idx: usize) -> bool;

    /// Get the length of the given character in this encoding. Characters not supported by the
    /// encoding are expected to return 0 currently, this may change in the future.
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

macro_rules! enc {
    ($($encoding:ident),* $(,)?) => {
        /// Enumeration of all supported string encodings. Supports most encoding operations
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        #[repr(u8)]
        pub enum Enc {
            $(
            #[doc = concat!("See [`", stringify!($encoding), "`]")]
            $encoding,
            )*
        }

        impl Enc {
            /// Get the [`Enc`] value for the provided encoding
            pub fn of<E: Encoding>() -> Enc {
                E::dyn_enc()
            }

            #[cfg(test)]
            pub(crate) fn all() -> impl Iterator<Item = Enc> {
                [$(Self::$encoding),*].into_iter()
            }

            #[cfg(test)]
            pub(crate) fn all_dyn() -> impl Iterator<Item = Enc> {
                [$($encoding::dyn_enc()),*].into_iter()
            }

            pub(crate) const fn max_discrim() -> usize {
                let mut a = 0;
                $( stringify!($encoding); a += 1; )*
                a
            }

            #[allow(unused)]
            pub(crate) fn replacement(self) -> char {
                match self {
                    $(Self::$encoding => $encoding::REPLACEMENT),*
                }
            }

            #[allow(unused)]
            pub(crate) fn shorthand(self) -> &'static str {
                match self {
                    $(Self::$encoding => $encoding::shorthand()),*
                }
            }

            /// Given a byte slice, determine whether it is valid for the current encoding.
            pub fn validate(self, bytes: &[u8]) -> Result<(), ValidateError> {
                match self {
                    $(Self::$encoding => $encoding::validate(bytes)),*
                }
            }

            /// Take a character and encode it directly into the provided buffer. If successful, returns the
            /// length of the buffer that was written.
            pub fn encode(self, char: char, out: &mut [u8]) -> Result<usize, EncodeError> {
                match self.encode_char(char) {
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
            pub fn recode(self, str: &crate::EncStr, out: &mut [u8]) -> Result<usize, RecodeError> {
                str.char_indices().try_fold(0, |out_pos, (idx, c)| {
                    match self.encode(c, &mut out[out_pos..]) {
                        Ok(len) => Ok(out_pos + len),
                        Err(e) => Err(RecodeError {
                            input_used: idx,
                            output_valid: out_pos,
                            cause: match e {
                                EncodeError::NeedSpace { len } => RecodeCause::NeedSpace { len },
                                EncodeError::InvalidChar => RecodeCause::InvalidChar {
                                    char: c,
                                    len: self.char_len(c),
                                },
                            },
                        }),
                    }
                })
            }

            pub(crate) fn encode_char(self, c: char) -> Option<ArrayVec<u8, 4>> {
                match self {
                    $(
                    Self::$encoding => $encoding::encode_char(c).map(IntoAV::into_av)
                    ),*
                }
            }

            pub(crate) unsafe fn decode_char_unchecked(self, str: &crate::EncStr) -> (char, &crate::EncStr) {
                match self {
                    $(
                    Self::$encoding => {
                        let (c, str) = $encoding::decode_char(str.downcast_unchecked());
                        (c, str.into())
                    }
                    ),*
                }
            }

            pub(crate) unsafe fn char_bound_unchecked(self, str: &crate::EncStr, idx: usize) -> bool {
                match self {
                    $(
                    Self::$encoding => $encoding::char_bound(str.downcast_unchecked(), idx)
                    ),*
                }
            }

            pub(crate) fn char_len(self, c: char) -> usize {
                match self {
                    $(
                    Self::$encoding => $encoding::char_len(c)
                    ),*
                }
            }
        }

        impl TryFrom<usize> for Enc {
            type Error = ();

            #[allow(non_upper_case_globals)]
            fn try_from(value: usize) -> Result<Self, Self::Error> {
                $(
                const $encoding: usize = Enc::$encoding as u8 as usize;
                )*

                Ok(match value {
                    $(
                    $encoding => Self::$encoding,
                    )*
                    _ => return Err(()),
                })
            }
        }
    }
}

// This should be kept up-to-date with new [`crate::Encoding`] impls
enc!(
    // Basic impls
    Ascii,
    ExtendedAscii,
    // ISO impls
    Iso8859_1,
    Iso8859_2,
    Iso8859_3,
    Iso8859_4,
    Iso8859_5,
    Iso8859_6,
    Iso8859_7,
    Iso8859_8,
    Iso8859_9,
    Iso8859_10,
    Iso8859_11,
    Iso8859_13,
    Iso8859_14,
    Iso8859_15,
    Iso8859_16,
    // JIS impls
    JisX0201,
    JisX0208,
    ShiftJIS,
    // Mac impls
    MacRoman,
    // Win impls
    Win1251,
    Win1252,
    Win1252Loose,
    // UTF impls
    Utf8,
    Utf16LE,
    Utf16BE,
    Utf32LE,
    Utf32BE,
);

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

impl fmt::Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while validating data")
    }
}

impl Error for ValidateError {}

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

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while encoding data: ")?;
        match self {
            EncodeError::NeedSpace { len } => {
                write!(f, "insufficient space, need {len} bytes for next character")
            }
            EncodeError::InvalidChar => write!(f, "invalid character for output encoding"),
        }
    }
}

impl Error for EncodeError {}

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

impl RecodeCause {
    pub(crate) fn write_cause(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecodeCause::NeedSpace { .. } => {
                write!(f, "insufficient space")
            }
            RecodeCause::InvalidChar { char, .. } => {
                write!(f, "invalid character for output encoding '{char}'")
            }
        }
    }
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

impl fmt::Display for RecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while recoding data: ")?;
        self.cause.write_cause(f)
    }
}

impl Error for RecodeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;

    #[test]
    fn unique_shorthands() {
        let mut set = BTreeSet::new();
        for e in Enc::all() {
            let short = e.shorthand();
            if !set.insert(short) {
                panic!("Duplicate encoding shorthand {short}. All encodings should have a unique shorthand.")
            }
        }
    }

    #[test]
    fn unique_dyn() {
        let mut set = BTreeSet::new();
        for e in Enc::all_dyn() {
            if !set.insert(e) {
                panic!("Variant for shorthand {} returned by multiple encoding implementations. All encodings should return a unique variant.", e.shorthand())
            }
        }
    }
}
