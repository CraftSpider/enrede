//! Definitions for errors that are used across multiple modules

use crate::encoding;
use crate::encoding::RecodeCause;
use core::error::Error;
use core::fmt;

/// Error encountered while re-encoding a [`Str`](crate::Str) or [`CStr`](crate::CStr) into another
/// format
#[derive(Clone, Debug, PartialEq)]
pub struct RecodeError {
    pub(crate) valid_up_to: usize,
    pub(crate) char: char,
    pub(crate) char_len: u8,
}

impl RecodeError {
    /// The length of valid data in the input before the error was encountered. Calling
    /// [`recode`](Str::recode) again on the input sliced down to this length will succeed.
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// The character encountered that caused re-encoding to fail. This character most likely isn't
    /// supported by the new encoding.
    pub fn char(&self) -> char {
        self.char
    }

    /// The length of the character in the input encoding. Skipping this many bytes forwards from
    /// [`valid_up_to`](Self::valid_up_to) and trying again will avoid this particular error
    /// character (though recoding may fail again immediately due to another invalid character).
    pub fn char_len(&self) -> usize {
        self.char_len as usize
    }
}

impl fmt::Display for RecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while recoding `Str`: invalid character for output encoding '{}'",
            self.char
        )
    }
}

impl Error for RecodeError {}

/// Error encountered while re-encoding a [`Str`](Str) or [`CStr`](crate::CStr) into another
/// format in a pre-allocated buffer
#[derive(Clone, PartialEq)]
pub struct RecodeIntoError<'a, S: ?Sized> {
    input_used: usize,
    str: &'a S,
    cause: RecodeCause,
}

impl<'a, S: ?Sized> RecodeIntoError<'a, S> {
    pub(crate) fn from_recode(err: encoding::RecodeError, str: &'a S) -> Self {
        RecodeIntoError {
            input_used: err.input_used(),
            str,
            cause: err.cause().clone(),
        }
    }

    /// The length of valid data in the input before the error was encountered. Calling
    /// [`recode_into`](Str::recode_into) again on the input sliced down to this length will succeed.
    pub fn valid_up_to(&self) -> usize {
        self.input_used
    }

    /// The portion of the buffer with valid data written into it, as a [`Str`] in the desired
    /// encoding.
    pub fn output_valid(&self) -> &'a S {
        self.str
    }

    /// The reason encoding stopped. See [`RecodeCause`] for more details on possible reasons.
    pub fn cause(&self) -> &RecodeCause {
        &self.cause
    }
}

impl<S: ?Sized + fmt::Debug> fmt::Debug for RecodeIntoError<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecodeIntoError")
            .field("input_used", &self.input_used)
            .field("str", &self.str)
            .field("cause", &self.cause)
            .finish()
    }
}

impl<S: ?Sized> fmt::Display for RecodeIntoError<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while recoding `Str` into buffer: ")?;
        self.cause.write_cause(f)
    }
}

impl<S: ?Sized + fmt::Debug> Error for RecodeIntoError<'_, S> {}
