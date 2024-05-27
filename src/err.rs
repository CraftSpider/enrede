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
