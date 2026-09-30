use crate::encoding::sealed::Sealed;
use crate::encoding::{NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_3: [char; 96] = [
    ' ', 'Ħ', '˘', '£', '¤', '\u{FFFD}', 'Ĥ', '§', '¨', 'İ', 'Ş', 'Ğ', 'Ĵ', '\u{AD}', '\u{FFFD}',
    'Ż', '°', 'ħ', '²', '³', '´', 'µ', 'ĥ', '·', '¸', 'ı', 'ş', 'ğ', 'ĵ', '½', '\u{FFFD}', 'ż',
    'À', 'Á', 'Â', '\u{FFFD}', 'Ä', 'Ċ', 'Ĉ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï',
    '\u{FFFD}', 'Ñ', 'Ò', 'Ó', 'Ô', 'Ġ', 'Ö', '×', 'Ĝ', 'Ù', 'Ú', 'Û', 'Ü', 'Ŭ', 'Ŝ', 'ß', 'à',
    'á', 'â', '\u{FFFD}', 'ä', 'ċ', 'ĉ', 'ç', 'è', 'é', 'ê', 'ë', 'ì', 'í', 'î', 'ï', '\u{FFFD}',
    'ñ', 'ò', 'ó', 'ô', 'ġ', 'ö', '÷', 'ĝ', 'ù', 'ú', 'û', 'ü', 'ŭ', 'ŝ', '˙',
];

/// The [ISO/IEC 8859-3](https://en.wikipedia.org/wiki/ISO/IEC_8859-3) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_3;

impl Sealed for Iso8859_3 {}

impl Encoding for Iso8859_3 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_3"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x80).contains(c)
                || ((0xA0..).contains(c) && ![0xA5, 0xAE, 0xBE, 0xC3, 0xD0, 0xE3, 0xF0].contains(c))
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
            let pos = DECODE_MAP_8859_3.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_3[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_3.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_3 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_3 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..217);
        if c < 128 {
            char::from(c)
        } else {
            let offset = match c + 32 {
                ..=0xA4 => 0,
                ..=0xAC => 1,
                ..=0xBB => 2,
                ..=0xBF => 3,
                ..=0xCB => 4,
                ..=0xDD => 5,
                ..=0xE9 => 6,
                _ => 7,
            };
            DECODE_MAP_8859_3[(c - 128 + offset) as usize]
        }
    }
}
