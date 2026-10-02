use crate::encoding::sealed::Sealed;
use crate::encoding::{AlwaysValid, Enc, NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_1252: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// The [Windows-1252](https://en.wikipedia.org/wiki/Windows-1252) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Win1252;

impl Sealed for Win1252 {}

impl Encoding for Win1252 {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "win1252"
    }

    fn dyn_enc() -> Enc {
        Enc::Win1252
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, b)| {
            if [0x81, 0x8D, 0x8F, 0x90, 0x9D].contains(b) {
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
        if (..0x80).contains(&(c as u32)) || (0xA0..0x100).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_1252.iter().position(|v| *v == c)? as u8;
            Some(pos + 0x80)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0x80..0xA0).contains(&b) {
            (DECODE_MAP_1252[b as usize - 0x80], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (c as u32) < 0x80 || DECODE_MAP_1252.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Win1252 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Win1252 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Number of characters
        let c = rng.random_range(0u8..251);
        if c <= 0x7F {
            char::from(c)
        } else if c >= (0xA0 - 5) {
            char::from(c + 5)
        } else {
            let offset = match c {
                0x80 => 0,
                ..=0x8B => 1,
                ..=0x8C => 2,
                ..=0x98 => 4,
                _ => 5,
            };
            DECODE_MAP_1252[(c + offset - 0x80) as usize]
        }
    }
}

/// The [Windows-1252](https://en.wikipedia.org/wiki/Windows-1252) encoding, with empty spots
/// replaced by the corresponding C1 control codes.
#[non_exhaustive]
#[derive(Default)]
pub struct Win1252Loose;

impl Sealed for Win1252Loose {}

impl Encoding for Win1252Loose {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "win1252_loose"
    }

    fn dyn_enc() -> Enc {
        Enc::Win1252Loose
    }

    fn validate(_: &[u8]) -> Result<(), ValidateError> {
        // All bytes are valid in this variant of Win1252, we just leave the invalid bytes alone
        Ok(())
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if (..0x80).contains(&(c as u32)) || (0xA0..0x100).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_1252.iter().position(|v| *v == c)? as u8;
            Some(pos + 0x80)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0x80..0xA0).contains(&b) {
            (DECODE_MAP_1252[b as usize - 0x80], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (c as u32) > 255 {
            0
        } else {
            1
        }
    }
}

impl NullTerminable for Win1252Loose {}

impl AlwaysValid for Win1252Loose {}

#[cfg(feature = "rand")]
impl Distribution<char> for Win1252Loose {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Number of characters
        let c = rng.random::<u8>();
        if c <= 0x7F || c >= 0xA0 {
            char::from(c)
        } else {
            DECODE_MAP_1252[(c - 0x80) as usize]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        assert!(Win1252::validate(b"01\xD5\xFF").is_ok());
        assert_eq!(
            Win1252::validate(b"0\xFF\x97\x9D\x99"),
            Err(ValidateError {
                valid_up_to: 3,
                error_len: Some(1),
            })
        );
    }

    #[test]
    fn test_encode() {
        assert_eq!(Win1252::encode_char('A'), Some(b'A'));
        assert_eq!(Win1252::encode_char('€'), Some(0x80));
        assert_eq!(Win1252::encode_char('ÿ'), Some(0xFF));
        assert_eq!(Win1252::encode_char('𐐷'), None,);
    }

    #[test]
    fn test_decode() {
        // SAFETY: This test data is guaranteed valid
        let str = unsafe { Str::from_bytes_unchecked(b"A\x80\xFF\0") };
        let (c, str) = Win1252::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Win1252::decode_char(str);
        assert_eq!(c, '€');
        let (c, str) = Win1252::decode_char(str);
        assert_eq!(c, 'ÿ');
        let (c, _) = Win1252::decode_char(str);
        assert_eq!(c, '\0');
    }

    #[test]
    fn test_char_len() {
        let c = '€';
        assert_eq!(Win1252::char_len(c), 1);
        let c = '𐐷';
        assert_eq!(Win1252::char_len(c), 0);
    }
}
