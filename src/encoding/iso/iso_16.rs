use crate::encoding::sealed::Sealed;
use crate::encoding::{NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_16: [char; 96] = [
    ' ', 'Ą', 'ą', 'Ł', '€', '„', 'Š', '§', 'š', '©', 'Ș', '«', 'Ź', '\u{AD}', 'ź', 'Ż', '°', '±',
    'Č', 'ł', 'Ž', '”', '¶', '·', 'ž', 'č', 'ș', '»', 'Œ', 'œ', 'Ÿ', 'ż', 'À', 'Á', 'Â', 'Ă', 'Ä',
    'Ć', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Đ', 'Ń', 'Ò', 'Ó', 'Ô', 'Ő', 'Ö', 'Ś',
    'Ű', 'Ù', 'Ú', 'Û', 'Ü', 'Ę', 'Ț', 'ß', 'à', 'á', 'â', 'ă', 'ä', 'ć', 'æ', 'ç', 'è', 'é', 'ê',
    'ë', 'ì', 'í', 'î', 'ï', 'đ', 'ń', 'ò', 'ó', 'ô', 'ő', 'ö', 'ś', 'ű', 'ù', 'ú', 'û', 'ü', 'ę',
    'ț', 'ÿ',
];

/// The [ISO/IEC 8859-16](https://en.wikipedia.org/wiki/ISO/IEC_8859-16) encoding.
///
/// All ISO encodings include the C0 control plane.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_16;

impl Sealed for Iso8859_16 {}

impl Encoding for Iso8859_16 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_16"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x80).contains(c) || (0xA0..).contains(c) {
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
            let pos = DECODE_MAP_8859_16.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_16[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_16.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_16 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_16 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..224);
        if c < 128 {
            char::from(c)
        } else {
            DECODE_MAP_8859_16[(c - 128) as usize]
        }
    }
}
