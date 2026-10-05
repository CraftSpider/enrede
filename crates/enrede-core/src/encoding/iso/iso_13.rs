use crate::encoding::sealed::Sealed;
use crate::encoding::{Enc, NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_13: [char; 96] = [
    ' ', '”', '¢', '£', '¤', '„', '¦', '§', 'Ø', '©', 'Ŗ', '«', '¬', '\u{AD}', '®', 'Æ', '°', '±',
    '²', '³', '“', 'µ', '¶', '·', 'ø', '¹', 'ŗ', '»', '¼', '½', '¾', 'æ', 'Ą', 'Į', 'Ā', 'Ć', 'Ä',
    'Å', 'Ę', 'Ē', 'Č', 'É', 'Ź', 'Ė', 'Ģ', 'Ķ', 'Ī', 'Ļ', 'Š', 'Ń', 'Ņ', 'Ó', 'Ō', 'Õ', 'Ö', '×',
    'Ų', 'Ł', 'Ś', 'Ū', 'Ü', 'Ż', 'Ž', 'ß', 'ą', 'į', 'ā', 'ć', 'ä', 'å', 'ę', 'ē', 'č', 'é', 'ź',
    'ė', 'ģ', 'ķ', 'ī', 'ļ', 'š', 'ń', 'ņ', 'ó', 'ō', 'õ', 'ö', '÷', 'ų', 'ł', 'ś', 'ū', 'ü', 'ż',
    'ž', '’',
];

/// The [ISO/IEC 8859-13](https://en.wikipedia.org/wiki/ISO/IEC_8859-13) encoding.
///
/// All ISO encodings include the C0 control plane.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_13;

impl Sealed for Iso8859_13 {}

impl Encoding for Iso8859_13 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_13"
    }

    fn dyn_enc() -> Enc {
        Enc::Iso8859_13
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
            let pos = DECODE_MAP_8859_13.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_13[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_13.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_13 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_13 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..224);
        if c < 128 {
            char::from(c)
        } else {
            DECODE_MAP_8859_13[(c - 128) as usize]
        }
    }
}
