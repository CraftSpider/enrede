use crate::encoding::RecodeCause;
use crate::{encoding, Encoding, Str};

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
    /// [`recode`](crate::Str::recode) again on the input sliced down to this length will succeed.
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

/// Error encountered while re-encoding a [`Str`](crate::Str) or [`CStr`](crate::CStr) into another
/// format in a pre-allocated buffer
pub struct RecodeIntoError<'a, E: Encoding> {
    pub(crate) input_used: usize,
    pub(crate) str: &'a Str<E>,
    pub(crate) cause: RecodeCause,
}

impl<'a, E: Encoding> RecodeIntoError<'a, E> {
    pub(crate) fn from_recode(err: encoding::RecodeError, str: &'a Str<E>) -> Self {
        RecodeIntoError {
            input_used: err.input_used(),
            str,
            cause: err.cause().clone(),
        }
    }

    /// The length of valid data in the input before the error was encountered. Calling
    /// [`recode_into`](crate::Str::recode_into) again on the input sliced down to this length will succeed.
    pub fn valid_up_to(&self) -> usize {
        self.input_used
    }

    /// The portion of the buffer with valid data written into it, as a [`Str`] in the desired
    /// encoding.
    pub fn output_valid(&self) -> &'a Str<E> {
        self.str
    }

    /// The reason encoding stopped. See [`RecodeCause`] for more details on possible reasons.
    pub fn cause(&self) -> &RecodeCause {
        &self.cause
    }
}
