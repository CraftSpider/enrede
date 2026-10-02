use crate::encoding::sealed::Sealed;
use crate::encoding::{AlwaysValid, Enc, Encoding, NullTerminable, ValidateError};
use crate::str::Str;
#[cfg(feature = "rand")]
use rand::{distr::Distribution, Rng};

const ENCODE_MAP_1251: phf::Map<char, u8> = phf::phf_map! {
    'Ђ' => 0, 'Ѓ' => 1, '‚' => 2, 'ѓ' => 3, '„' => 4, '…' => 5, '†' => 6, '‡' => 7, '€' => 8,
    '‰' => 9, 'Љ' => 10, '‹' => 11, 'Њ' => 12, 'Ќ' => 13, 'Ћ' => 14, 'Џ' => 15, 'ђ' => 16,
    '‘' => 17, '’' => 18, '“' => 19, '”' => 20, '•' => 21, '–' => 22, '—' => 23, '␚' => 24,
    '™' => 25, 'љ' => 26, '›' => 27, 'њ' => 28, 'ќ' => 29, 'ћ' => 30, 'џ' => 31, ' ' => 32,
    'Ў' => 33, 'ў' => 34, 'Ј' => 35, '¤' => 36, 'Ґ' => 37, '¦' => 38, '§' => 39, 'Ё' => 40,
    '©' => 41, 'Є' => 42, '«' => 43, '¬' => 44, '\u{AD}' => 45, '®' => 46, 'Ї' => 47, '°' => 48,
    '±' => 49, 'І' => 50, 'і' => 51, 'ґ' => 52, 'µ' => 53, '¶' => 54, '·' => 55, 'ё' => 56,
    '№' => 57, 'є' => 58, '»' => 59, 'ј' => 60, 'Ѕ' => 61, 'ѕ' => 62, 'ї' => 63, 'А' => 64,
    'Б' => 65, 'В' => 66, 'Г' => 67, 'Д' => 68, 'Е' => 69, 'Ж' => 70, 'З' => 71, 'И' => 72,
    'Й' => 73, 'К' => 74, 'Л' => 75, 'М' => 76, 'Н' => 77, 'О' => 78, 'П' => 79, 'Р' => 80,
    'С' => 81, 'Т' => 82, 'У' => 83, 'Ф' => 84, 'Х' => 85, 'Ц' => 86, 'Ч' => 87, 'Ш' => 88,
    'Щ' => 89, 'Ъ' => 90, 'Ы' => 91, 'Ь' => 92, 'Э' => 93, 'Ю' => 94, 'Я' => 95, 'а' => 96,
    'б' => 97, 'в' => 98, 'г' => 99, 'д' => 100, 'е' => 101, 'ж' => 102, 'з' => 103, 'и' => 104,
    'й' => 105, 'к' => 106, 'л' => 107, 'м' => 108, 'н' => 109, 'о' => 110, 'п' => 111, 'р' => 112,
    'с' => 113, 'т' => 114, 'у' => 115, 'ф' => 116, 'х' => 117, 'ц' => 118, 'ч' => 119, 'ш' => 120,
    'щ' => 121, 'ъ' => 122, 'ы' => 123, 'ь' => 124, 'э' => 125, 'ю' => 126, 'я' => 127,
};

const DECODE_MAP_1251: [char; 128] = [
    'Ђ', 'Ѓ', '‚', 'ѓ', '„', '…', '†', '‡', '€', '‰', 'Љ', '‹', 'Њ', 'Ќ', 'Ћ', 'Џ', 'ђ', '‘', '’',
    '“', '”', '•', '–', '—', '␚', '™', 'љ', '›', 'њ', 'ќ', 'ћ', 'џ', ' ', 'Ў', 'ў', 'Ј', '¤', 'Ґ',
    '¦', '§', 'Ё', '©', 'Є', '«', '¬', '\u{AD}', '®', 'Ї', '°', '±', 'І', 'і', 'ґ', 'µ', '¶', '·',
    'ё', '№', 'є', '»', 'ј', 'Ѕ', 'ѕ', 'ї', 'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З', 'И', 'Й', 'К',
    'Л', 'М', 'Н', 'О', 'П', 'Р', 'С', 'Т', 'У', 'Ф', 'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э',
    'Ю', 'Я', 'а', 'б', 'в', 'г', 'д', 'е', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р',
    'с', 'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
];

const DECODE_MAP_1252: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// The [Windows-1251](https://en.wikipedia.org/wiki/Windows-1251) encoding.
#[non_exhaustive]
#[derive(Default)]
pub struct Win1251;

impl Sealed for Win1251 {}

impl Encoding for Win1251 {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "win1251"
    }

    fn dyn_enc() -> Enc {
        Enc::Win1251
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, b)| {
            if *b == 0x98 {
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
        if (..0x80).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            ENCODE_MAP_1251.get(&c).map(|b| b + 0x80)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if (..0x80).contains(&b) {
            (b as char, &str[1..])
        } else {
            (DECODE_MAP_1251[b as usize - 0x80], &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (c as u32) < 0x80 || DECODE_MAP_1251.contains(&c) {
            1
        } else {
            0
        }
    }
}

impl NullTerminable for Win1251 {}

#[cfg(feature = "rand")]
impl Distribution<char> for Win1251 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Number of characters
        let c = rng.random_range(0u8..255);
        if c <= 0x7F {
            char::from(c)
        } else if c < 0x98 {
            DECODE_MAP_1251[(c - 0x80) as usize]
        } else {
            DECODE_MAP_1251[(c + 1 - 0x80) as usize]
        }
    }
}

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
    fn test_validate_win1251() {
        assert!(Win1251::validate(b"01\xD5\xFF").is_ok());
        assert_eq!(
            Win1251::validate(b"0\xFF\x97\x98\x99"),
            Err(ValidateError {
                valid_up_to: 3,
                error_len: Some(1),
            })
        );
    }

    #[test]
    fn test_encode_win1251() {
        assert_eq!(Win1251::encode_char('A'), Some(b'A'));
        assert_eq!(Win1251::encode_char('Ђ'), Some(0x80));
        assert_eq!(Win1251::encode_char('я'), Some(0xFF));
        assert_eq!(Win1251::encode_char('𐐷'), None,);
    }

    #[test]
    fn test_decode_win1251() {
        // SAFETY: This test data is guaranteed valid
        let str = unsafe { Str::from_bytes_unchecked(b"A\x80\xFF\0") };
        let (c, str) = Win1251::decode_char(str);
        assert_eq!(c, 'A');
        let (c, str) = Win1251::decode_char(str);
        assert_eq!(c, 'Ђ');
        let (c, str) = Win1251::decode_char(str);
        assert_eq!(c, 'я');
        let (c, _) = Win1251::decode_char(str);
        assert_eq!(c, '\0');
    }

    #[test]
    fn test_char_len_win1251() {
        let c = 'ф';
        assert_eq!(Win1251::char_len(c), 1);
        let c = '𐐷';
        assert_eq!(Win1251::char_len(c), 0);
    }

    #[test]
    fn test_validate_win1252() {
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
    fn test_encode_win1252() {
        assert_eq!(Win1252::encode_char('A'), Some(b'A'));
        assert_eq!(Win1252::encode_char('€'), Some(0x80));
        assert_eq!(Win1252::encode_char('ÿ'), Some(0xFF));
        assert_eq!(Win1252::encode_char('𐐷'), None,);
    }

    #[test]
    fn test_decode_win1252() {
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
    fn test_char_len_win1252() {
        let c = '€';
        assert_eq!(Win1252::char_len(c), 1);
        let c = '𐐷';
        assert_eq!(Win1252::char_len(c), 0);
    }
}
