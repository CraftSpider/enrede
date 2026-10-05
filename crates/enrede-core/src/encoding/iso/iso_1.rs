use crate::encoding::sealed::Sealed;
use crate::encoding::{Enc, NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const DECODE_MAP_8859_1: [char; 96] = [
    ' ', '¡', '¢', '£', '¤', '¥', '¦', '§', '¨', '©', 'ª', '«', '¬', '\u{AD}', '®', '¯', '°', '±',
    '²', '³', '´', 'µ', '¶', '·', '¸', '¹', 'º', '»', '¼', '½', '¾', '¿', 'À', 'Á', 'Â', 'Ã', 'Ä',
    'Å', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ð', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×',
    'Ø', 'Ù', 'Ú', 'Û', 'Ü', 'Ý', 'Þ', 'ß', 'à', 'á', 'â', 'ã', 'ä', 'å', 'æ', 'ç', 'è', 'é', 'ê',
    'ë', 'ì', 'í', 'î', 'ï', 'ð', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'ø', 'ù', 'ú', 'û', 'ü', 'ý',
    'þ', 'ÿ',
];

/// The [ISO/IEC 8859-1](https://en.wikipedia.org/wiki/ISO/IEC_8859-1) encoding.
///
/// All ISO encodings include the C0 control plane.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_1;

impl Sealed for Iso8859_1 {}

impl Encoding for Iso8859_1 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso8859_1"
    }

    fn dyn_enc() -> Enc {
        Enc::Iso8859_1
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
            let pos = DECODE_MAP_8859_1.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_1[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (..0x80).contains(&(c as u32)) || DECODE_MAP_8859_1.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_1 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_1 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.random_range(0u8..224);
        if c < 128 {
            char::from(c)
        } else {
            DECODE_MAP_8859_1[(c - 128) as usize]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(Iso8859_1::encode_char('¡'), Some(0xA1));
        assert_eq!(Iso8859_1::encode_char('£'), Some(0xA3));
        assert_eq!(Iso8859_1::encode_char('×'), Some(0xD7));
        assert_eq!(Iso8859_1::encode_char('÷'), Some(0xF7));
    }

    const HELLO_WORLD_ISO1: &[u8] = b"\xA1\xA36\xD7\xF7A";

    #[test]
    fn test_decode() {
        let s = unsafe { Str::from_bytes_unchecked(HELLO_WORLD_ISO1) };
        let (char, s) = Iso8859_1::decode_char(s);
        assert_eq!(char, '¡');
        let (char, s) = Iso8859_1::decode_char(s);
        assert_eq!(char, '£');
        let (char, s) = Iso8859_1::decode_char(s);
        assert_eq!(char, '6');
        let (char, s) = Iso8859_1::decode_char(s);
        assert_eq!(char, '×');
        let (char, s) = Iso8859_1::decode_char(s);
        assert_eq!(char, '÷');
        let (char, _) = Iso8859_1::decode_char(s);
        assert_eq!(char, 'A');
    }
}
