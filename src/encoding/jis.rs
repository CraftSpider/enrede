use crate::encoding::sealed::Sealed;
use crate::encoding::ValidateError;
use crate::{Encoding, Str};
use arrayvec::ArrayVec;
#[cfg(feature = "rand")]
use rand::{distributions::Distribution, Rng};

mod x0208_tables;

const DECODE_MAP_0201: [char; 63] = [
    '｡', '｢', '｣', '､', '･', 'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｰ', 'ｱ', 'ｲ', 'ｳ',
    'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ',
    'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ',
    'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ', 'ﾞ', 'ﾟ',
];

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct JisX0201;

impl Sealed for JisX0201 {}

impl Encoding for JisX0201 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 1;
    type Bytes = u8;

    fn shorthand() -> &'static str {
        "jisx0201"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        bytes.iter().enumerate().try_for_each(|(idx, c)| {
            if (..0x20).contains(c) || (0x80..0xA1).contains(c) || (0xE0..).contains(c) {
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
        if c == '¥' {
            Some(0x5C)
        } else if c == '‾' {
            Some(0x7E)
        } else if (0x20..0x80).contains(&(c as u32)) {
            Some(c as u8)
        } else {
            let pos = DECODE_MAP_0201.iter().position(|v| *v == c)? as u8;
            Some(pos + 0xA1)
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let b = str.as_bytes()[0];
        if b == 0x5C {
            ('¥', &str[1..])
        } else if b == 0x7E {
            ('‾', &str[1..])
        } else if (..0x80).contains(&b) {
            (b as char, &str[1..])
        } else {
            (DECODE_MAP_0201[b as usize - 0xA1], &str[1..])
        }
    }

    fn char_bound(_: &Str<Self>, _: usize) -> bool {
        true
    }

    fn char_len(c: char) -> usize {
        if (0x20..0x80).contains(&(c as u32)) || (0xA1..0xE0).contains(&(c as u32)) {
            1
        } else {
            0
        }
    }
}

#[cfg(feature = "rand")]
impl Distribution<char> for JisX0201 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        // Number of JIS 0201 characters
        let c = rng.gen_range(0..159);
        let c = if c < 0x60 { c + 0x20 } else { c + 0x41 };
        Self::decode_char(unsafe { Str::from_bytes_unchecked(&[c]) }).0
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct JisX0208;

impl Sealed for JisX0208 {}

impl Encoding for JisX0208 {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 2;
    type Bytes = ArrayVec<u8, 2>;

    fn shorthand() -> &'static str {
        "jisx0208"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        let mut row = 0;
        for (idx, b) in bytes.iter().enumerate() {
            if row == 0 && *b >= 0x80 {
                if (0x29..0x30).contains(b) {
                    return Err(ValidateError {
                        valid_up_to: idx,
                        error_len: Some(1),
                    });
                } else if (0x75..0x7F).contains(b) || (0x80..).contains(b) {
                    return Err(ValidateError {
                        valid_up_to: idx,
                        error_len: Some(2),
                    });
                } else if (0x21..0x7F).contains(b) {
                    row = *b - 0x20;
                }
                // Characters in range 0..0x20 are ASCII control codes
            } else if row != 0 {
                if !(0x21..0x7F).contains(b)
                    || x0208_tables::DECODE_MAP_0208[row as usize][(*b - 0x21) as usize] == '�'
                {
                    return Err(ValidateError {
                        valid_up_to: idx - 1,
                        error_len: Some(2),
                    });
                }
                row = 0;
            }
        }
        Ok(())
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        if c as u32 <= 0x20 || c as u32 == 0x7F {
            Some(ArrayVec::from_iter([c as u8]))
        } else {
            let (row, col) =
                x0208_tables::DECODE_MAP_0208
                    .iter()
                    .enumerate()
                    .find_map(|(row_idx, row)| {
                        let col = row.iter().position(|v| *v == c)? as u8;
                        Some((row_idx as u8, col))
                    })?;
            Some(ArrayVec::from([row + 0x21, col + 0x21]))
        }
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        let bytes = str.as_bytes();
        let first = bytes[0];
        if (..0x21).contains(&first) {
            (char::from(first), &str[1..])
        } else {
            let second = bytes[1];
            let (row, col) = (first - 0x21, second - 0x21);
            let c = x0208_tables::DECODE_MAP_0208[row as usize][col as usize];
            (c, &str[2..])
        }
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        let bytes = str.as_bytes();
        let first = bytes[0];
        // Control code bytes, space, and del - always single-byte, never used as a second byte
        if (..0x21).contains(&first) || first == 0x7F {
            true
        } else {
            // Otherwise, first and second bytes look the same - iterate to here
            for (idx2, _) in str.char_indices() {
                if idx == idx2 {
                    return true;
                } else if idx < idx2 {
                    return false;
                }
            }
            false
        }
    }

    fn char_len(c: char) -> usize {
        if (..0x21).contains(&(c as u32)) || c as u32 == 0x7F {
            1
        } else if x0208_tables::DECODE_MAP_0208
            .iter()
            .any(|row| row.iter().any(|v| *v == c))
        {
            2
        } else {
            0
        }
    }
}

#[cfg(feature = "rand")]
impl Distribution<char> for JisX0208 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        todo!()
    }
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ShiftJIS;

impl Sealed for ShiftJIS {}

impl Encoding for ShiftJIS {
    const REPLACEMENT: char = '?';
    const MAX_LEN: usize = 0;
    type Bytes = ArrayVec<u8, 2>;

    fn shorthand() -> &'static str {
        "shiftjis"
    }

    fn validate(bytes: &[u8]) -> Result<(), ValidateError> {
        todo!()
    }

    fn encode_char(c: char) -> Option<Self::Bytes> {
        todo!()
    }

    fn decode_char(str: &Str<Self>) -> (char, &Str<Self>) {
        todo!()
    }

    fn char_bound(str: &Str<Self>, idx: usize) -> bool {
        todo!()
    }

    fn char_len(c: char) -> usize {
        todo!()
    }
}
