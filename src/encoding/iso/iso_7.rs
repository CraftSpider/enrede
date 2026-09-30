use crate::encoding::sealed::Sealed;
use crate::encoding::{NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_7: [char; 96] = [
    ' ', '\u{2018}', '\u{2019}', '£', '€', '₯', '¦', '§', '¨', '©', 'ͺ', '«', '¬', '\u{AD}',
    '\u{FFFD}', '―', '°', '±', '²', '³', 'ʹ', '΅', 'Ά', '·', 'Έ', 'Ή', 'Ί', '»', 'Ό', '½', 'Ύ',
    'Ώ', 'ΐ', 'Α', 'Β', 'Γ', 'Δ', 'Ε', 'Ζ', 'Η', 'Θ', 'Ι', 'Κ', 'Λ', 'Μ', 'Ν', 'Ξ', 'Ο', 'Π', 'Ρ',
    '\u{FFFD}', 'Σ', 'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω', 'Ϊ', 'Ϋ', 'ά', 'έ', 'ή', 'ί', 'ΰ', 'α', 'β',
    'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'ς', 'σ', 'τ', 'υ',
    'φ', 'χ', 'ψ', 'ω', 'ϊ', 'ϋ', 'ό', 'ύ', 'ώ', '\u{FFFD}',
];

/// The [ISO/IEC 8859-3](https://en.wikipedia.org/wiki/ISO/IEC_8859-3) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_7;

impl Sealed for Iso8859_7 {}

impl Encoding for Iso8859_7 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_7"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x80).contains(c) || ((0xA0..).contains(c) && ![0xAE, 0xD2, 0xFF].contains(c)) {
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
            let pos = DECODE_MAP_8859_7.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_7[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_7.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_7 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_7 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..221);
        if c < 128 {
            char::from(c)
        } else {
            let offset = match c + 32 {
                ..=0xAD => 0,
                ..=0xD0 => 1,
                _ => 2,
            };
            DECODE_MAP_8859_7[(c - 128 + offset) as usize]
        }
    }
}
