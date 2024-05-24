use crate::encoding::sealed::Sealed;
use crate::encoding::{Encoding, ValidateError};
use crate::str::Str;
use arrayvec::ArrayVec;

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
pub struct Win1251;

impl Sealed for Win1251 {}

impl Encoding for Win1251 {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;

    fn shorthand() -> &'static str {
        "win1251"
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

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        if (..0x80).contains(&(c as u32)) {
            Some(arrvec![c as u8])
        } else {
            let pos = DECODE_MAP_1251.iter().position(|v| *v == c)? as u8;
            Some(arrvec![pos + 0x80])
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
        if c as u32 == 0x98 || (c as u32) > 255 {
            0
        } else {
            1
        }
    }
}

/// The [Windows-1252](https://en.wikipedia.org/wiki/Windows-1252) encoding.
#[non_exhaustive]
pub struct Win1252;

impl Sealed for Win1252 {}

impl Encoding for Win1252 {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;

    fn shorthand() -> &'static str {
        "win1252"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, b)| {
            if [0x82, 0x8D, 0x8F, 0x90, 0x9D].contains(b) {
                Err(ValidateError {
                    valid_up_to: idx,
                    error_len: Some(1),
                })
            } else {
                Ok(())
            }
        })
    }

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        if (..0x80).contains(&(c as u32)) || (0xA0..0x100).contains(&(c as u32)) {
            Some(arrvec![c as u8])
        } else {
            let pos = DECODE_MAP_1252.iter().position(|v| *v == c)? as u8;
            Some(arrvec![pos + 0x80])
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
        if [0x82, 0x8D, 0x8F, 0x90, 0x9D].contains(&(c as u32)) || c as u32 > 255 {
            0
        } else {
            1
        }
    }
}

/// The [Windows-1252](https://en.wikipedia.org/wiki/Windows-1252) encoding, with empty spots
/// replaced by the corresponding C1 control codes.
#[non_exhaustive]
pub struct Win1252Loose;

impl Sealed for Win1252Loose {}

impl Encoding for Win1252Loose {
    const REPLACEMENT: char = '\x1A';
    const MAX_LEN: usize = 1;

    fn shorthand() -> &'static str {
        "win1252_loose"
    }

    fn validate(_: &[u8]) -> Result<(), ValidateError> {
        // All bytes are valid in this variant of Win1252, we just leave the invalid bytes alone
        Ok(())
    }

    fn encode_char(c: char) -> Option<ArrayVec<u8, 4>> {
        if (..0x80).contains(&(c as u32)) || (0xA0..0x100).contains(&(c as u32)) {
            Some(arrvec![c as u8])
        } else {
            let pos = DECODE_MAP_1252.iter().position(|v| *v == c)? as u8;
            Some(arrvec![pos + 0x80])
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
