use crate::encoding::sealed::Sealed;
use crate::encoding::{NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_8: [char; 95] = [
    ' ', '\u{FFFD}', '¢', '£', '¤', '¥', '¦', '§', '¨', '©', '×', '«', '¬', '\u{AD}', '®', '¯',
    '°', '±', '²', '³', '´', 'µ', '¶', '·', '¸', '¹', '÷', '»', '¼', '½', '¾', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '‗', 'א',
    'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח', 'ט', 'י', 'ך', 'כ', 'ל', 'ם', 'מ', 'ן', 'נ', 'ס', 'ע', 'ף',
    'פ', 'ץ', 'צ', 'ק', 'ר', 'ש', 'ת', '\u{FFFD}', '\u{FFFD}', '\u{200E}', '\u{200F}',
];

/// The [ISO/IEC 8859-8](https://en.wikipedia.org/wiki/ISO/IEC_8859-8) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_8;

impl Sealed for Iso8859_8 {}

impl Encoding for Iso8859_8 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_8"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x80).contains(c)
                || [0xA0].contains(c)
                || (0xA2..0xBF).contains(c)
                || (0xDF..0xFB).contains(c)
                || (0xFD..0xFF).contains(c)
            {
                Ok(())
            } else {
                Err(ValidateError {
                    valid_up_to: idx,
                    error_len: Some(1),
                })
            }
        })
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if (..0x80).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_8859_8.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_8[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_8.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_8 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_8 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..188);
        if c < 128 {
            char::from(c)
        } else {
            let offset = match c + 32 {
                ..=0xA0 => 0,
                ..=0xBD => 1,
                ..=0xD9 => 33,
                _ => 35,
            };
            DECODE_MAP_8859_8[(c - 128 + offset) as usize]
        }
    }
}
