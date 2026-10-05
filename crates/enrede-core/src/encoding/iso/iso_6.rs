use crate::encoding::sealed::Sealed;
use crate::encoding::{Enc, NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_6: [char; 83] = [
    ' ', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '¤', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '،', '\u{AD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}',
    '\u{FFFD}', '\u{FFFD}', '؛', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '؟', '\u{FFFD}', 'ء', 'آ',
    'أ', 'ؤ', 'إ', 'ئ', 'ا', 'ب', 'ة', 'ت', 'ث', 'ج', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'س', 'ش', 'ص',
    'ض', 'ط', 'ظ', 'ع', 'غ', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', '\u{FFFD}', 'ـ', 'ف',
    'ق', 'ك', 'ل', 'م', 'ن', 'ه', 'و', 'ى', 'ي', 'ً', 'ٌ', 'ٍ', 'َ', 'ُ', 'ِ', 'ّ', 'ْ',
];

/// The [ISO/IEC 8859-6](https://en.wikipedia.org/wiki/ISO/IEC_8859-6) encoding.
///
/// All ISO encodings include the C0 control plane.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_6;

impl Sealed for Iso8859_6 {}

impl Encoding for Iso8859_6 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_6"
    }

    fn dyn_enc() -> Enc {
        Enc::Iso8859_6
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x80).contains(c)
                || [0xA0, 0xA4, 0xAC, 0xAD, 0xBB, 0xBF].contains(c)
                || (0xC1..0xDB).contains(c)
                || (0xE0..0xF3).contains(c)
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
            let pos = DECODE_MAP_8859_6.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_6[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_6.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_6 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_6 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..179);
        if c < 128 {
            char::from(c)
        } else {
            let offset = match c + 32 {
                ..=0xA0 => 0,
                ..=0xA1 => 3,
                ..=0xA3 => 10,
                ..=0xA4 => 23,
                ..=0xA5 => 26,
                ..=0xBF => 27,
                _ => 32,
            };
            DECODE_MAP_8859_6[(c - 128 + offset) as usize]
        }
    }
}
