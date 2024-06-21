use crate::encoding::sealed::Sealed;
use crate::encoding::{NullTerminable, ValidateError};
use crate::{Encoding, Str};
#[cfg(feature = "rand")]
use rand::{distributions::Distribution, Rng};

const DECODE_MAP_8859_2: [char; 96] = [
    ' ', 'Ą', '˘', 'Ł', '¤', 'Ľ', 'Ś', '§', '¨', 'Š', 'Ş', 'Ť', 'Ź', '\u{AD}', 'Ž', 'Ż', '°', 'ą',
    '˛', 'ł', '´', 'ľ', 'ś', 'ˇ', '¸', 'š', 'ş', 'ť', 'ź', '˝', 'ž', 'ż', 'Ŕ', 'Á', 'Â', 'Ă', 'Ä',
    'Ĺ', 'Ć', 'Ç', 'Č', 'É', 'Ę', 'Ë', 'Ě', 'Í', 'Î', 'Ď', 'Đ', 'Ń', 'Ň', 'Ó', 'Ô', 'Ő', 'Ö', '×',
    'Ř', 'Ů', 'Ú', 'Ű', 'Ü', 'Ý', 'Ţ', 'ß', 'ŕ', 'á', 'â', 'ă', 'ä', 'ĺ', 'ć', 'ç', 'č', 'é', 'ę',
    'ë', 'ě', 'í', 'î', 'ď', 'đ', 'ń', 'ň', 'ó', 'ô', 'ő', 'ö', '÷', 'ř', 'ů', 'ú', 'ű', 'ü', 'ý',
    'ţ', '˙',
];

const DECODE_MAP_8859_15: [char; 96] = [
    ' ', '¡', '¢', '£', '€', '¥', 'Š', '§', 'š', '©', 'ª', '«', '¬', '\u{AD}', '®', '¯', '°', '±',
    '²', '³', 'Ž', 'µ', '¶', '·', 'ž', '¹', 'º', '»', 'Œ', 'œ', 'Ÿ', '¿', 'À', 'Á', 'Â', 'Ã', 'Ä',
    'Å', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ð', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×',
    'Ø', 'Ù', 'Ú', 'Û', 'Ü', 'Ý', 'Þ', 'ß', 'à', 'á', 'â', 'ã', 'ä', 'å', 'æ', 'ç', 'è', 'é', 'ê',
    'ë', 'ì', 'í', 'î', 'ï', 'ð', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'ø', 'ù', 'ú', 'û', 'ü', 'ý',
    'þ', 'ÿ',
];

/// The [ISO/IEC 8859-2](https://en.wikipedia.org/wiki/ISO/IEC_8859-2) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_2;

impl Sealed for Iso8859_2 {}

impl Encoding for Iso8859_2 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso5889_2"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (0x20..0x7F).contains(c) || (0xA0..).contains(c) {
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
        if (0x20..0x7F).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_8859_2.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_2[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (0x20..0x7F).contains(&(c as u32)) || DECODE_MAP_8859_2.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_2 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.gen_range(0u8..112);
        if c < 95 {
            char::from(c + 0x20)
        } else {
            DECODE_MAP_8859_2[(c - 95) as usize]
        }
    }
}

/// The [ISO/IEC 8859-15](https://en.wikipedia.org/wiki/ISO/IEC_8859-15) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Iso8859_15;

impl Sealed for Iso8859_15 {}

impl Encoding for Iso8859_15 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "iso5889_15"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (0x20..0x7F).contains(c) || (0xA0..).contains(c) {
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
        if (0x20..0x7F).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_8859_15.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA0)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (0xA0..).contains(&b) {
            (DECODE_MAP_8859_15[b as usize - 0xA0], &str[1..])
        } else {
            (b as char, &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (0x20..0x7F).contains(&(c as u32)) || DECODE_MAP_8859_15.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Iso8859_15 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Iso8859_15 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Total number of characters in encoding
        let c = rng.gen_range(0u8..112);
        if c < 95 {
            char::from(c + 0x20)
        } else {
            DECODE_MAP_8859_15[(c - 95) as usize]
        }
    }
}
