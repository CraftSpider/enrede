use crate::encoding::sealed::Sealed;
use crate::encoding::{AlwaysValid, NullTerminable, ValidateError};
use crate::{Encoding, Str};

/// The [ASCII](https://en.wikipedia.org/wiki/ASCII) encoding.
#[non_exhaustive]
pub struct Ascii;

impl Sealed for Ascii {}

impl Encoding for Ascii {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "ascii"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if *c > 127 {
                Err(ValidateError {
                    valid_up_to: idx,
                    error_len: Some(1),
                })
            } else {
                Ok(())
            }
        })
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if c as u32 > 127 {
            None
        } else {
            Some(c as u8)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        (str.as_bytes()[0] as char, &str[1..])
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (c as u32) < 128 {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Ascii {}

/// The [Extended ASCII](https://en.wikipedia.org/wiki/ASCII#8-bit_codes) encoding. This encoding is
/// not assign any particular meaning to values beyond 127 - it simply round-trips them as `char`s
/// of that exact codepoint value.
#[non_exhaustive]
pub struct ExtendedAscii;

impl Sealed for ExtendedAscii {}

impl Encoding for ExtendedAscii {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "ascii_ext"
    }

    fn validate(_: &[u8]) -> Result<(), ValidateError> {
        Ok(())
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if (c as u32) < 256 {
            Some(c as u8)
        } else {
            None
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        (str.as_bytes()[0] as char, &str[1..])
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (c as u32) < 256 {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for ExtendedAscii {}

impl AlwaysValid for ExtendedAscii {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ascii() {
        assert!(Ascii::validate(b"A simple ASCII string").is_ok());
        assert!(Ascii::validate(b"Contains odd chars \0 \x01 \x02 \r \x7F").is_ok());
        assert_eq!(
            Ascii::validate(b"\x80"),
            Err(ValidateError {
                valid_up_to: 0,
                error_len: Some(1),
            })
        );
        assert_eq!(
            Ascii::validate(b"Foo \xFF"),
            Err(ValidateError {
                valid_up_to: 4,
                error_len: Some(1),
            })
        )
    }

    #[test]
    fn test_encode_ascii() {
        assert_eq!(Ascii::encode_char('A'), Some(b'A'));
        assert_eq!(Ascii::encode_char('\u{80}'), None);
        assert_eq!(Ascii::encode_char('𐐷'), None);
    }

    #[test]
    fn test_decode_ascii() {
        // SAFETY: This test data is guaranteed valid
        let str = unsafe { Str::from_bytes_unchecked(b"A simple sentence") };
        let (c, str) = Ascii::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Ascii::decode_char(str);
        assert_eq!(c, ' ');
        let (c, _) = Ascii::decode_char(str);
        assert_eq!(c, 's');
    }
}
